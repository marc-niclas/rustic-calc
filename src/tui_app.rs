use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::calculate::calculate;
pub use crate::input_editor::InputEditMode;
use crate::input_editor::{EditorCommand, InputEditor, Motion};
use crate::tokenize::tokenize;
use crate::types::VariableEntry;
use crate::variables::parse_variables;
use color_eyre::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::BorderType;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Padding, Paragraph},
};

pub struct History {
    pub expression: String,
    pub result: Option<f64>,
    pub error: Option<String>,
}

impl std::fmt::Display for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.result, self.error.clone()) {
            (Some(result), _) => write!(f, "{} = {}", self.expression, result),
            (_, Some(error)) => write!(f, "'{}' resulted in error: {}", self.expression, error),
            _ => panic!("Either provide an error or a result"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Input,
    History,
    Variables,
}

struct YankFlash {
    start: usize,
    end: usize,
    expires_at: Instant,
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub character_index: usize,
    /// History of recorded messages
    pub history: Vec<History>,
    /// Variables stored in the calculator
    pub variables: HashMap<String, VariableEntry>,
    pub input_mode: bool,
    pub focus: Focus,
    pub input_edit_mode: InputEditMode,
    pub history_state: ListState,
    pub variables_state: ListState,
    editor: InputEditor,
    editor_needs_sync: bool,
    yank_flash: Option<YankFlash>,
}

impl App {
    pub fn new() -> Self {
        let editor = InputEditor::new();
        Self {
            input: editor.input().to_string(),
            history: Vec::new(),
            character_index: editor.cursor(),
            variables: HashMap::new(),
            input_mode: true,
            focus: Focus::Input,
            input_edit_mode: editor.mode(),
            history_state: ListState::default(),
            variables_state: ListState::default(),
            editor,
            editor_needs_sync: false,
            yank_flash: None,
        }
    }

    fn sync_public_from_editor(&mut self) {
        self.input = self.editor.input().to_string();
        self.character_index = self.editor.cursor();
        self.input_edit_mode = self.editor.mode();
        self.editor_needs_sync = false;
    }

    fn mark_editor_dirty_if_public_changed(&mut self) {
        if self.input != self.editor.input()
            || self.character_index != self.editor.cursor()
            || self.input_edit_mode != self.editor.mode()
        {
            self.editor_needs_sync = true;
        }
    }

    fn ensure_editor_synced_from_public(&mut self) {
        if !self.editor_needs_sync {
            return;
        }

        let public_len = self.input.chars().count();
        let target_cursor = match self.input_edit_mode {
            InputEditMode::Insert => self.character_index.min(public_len),
            InputEditMode::Normal | InputEditMode::Visual => {
                if public_len == 0 {
                    0
                } else {
                    self.character_index.min(public_len - 1)
                }
            }
        };

        self.editor.set_input(self.input.clone());
        match self.input_edit_mode {
            InputEditMode::Insert => self.editor.switch_to_insert_mode(),
            InputEditMode::Normal => self.editor.switch_to_normal_mode(),
            InputEditMode::Visual => self.editor.switch_to_visual_mode(),
        }

        let mut current = self.editor.cursor();
        while current > target_cursor {
            match self.input_edit_mode {
                InputEditMode::Insert => self.editor.move_insert_left(),
                InputEditMode::Normal | InputEditMode::Visual => {
                    self.editor.apply_motion(Motion::Left)
                }
            }
            current = self.editor.cursor();
        }
        while current < target_cursor {
            match self.input_edit_mode {
                InputEditMode::Insert => self.editor.move_insert_right(),
                InputEditMode::Normal | InputEditMode::Visual => {
                    self.editor.apply_motion(Motion::Right)
                }
            }
            let next = self.editor.cursor();
            if next == current {
                break;
            }
            current = next;
        }

        self.sync_public_from_editor();
    }

    pub fn move_cursor_left(&mut self) {
        self.mark_editor_dirty_if_public_changed();
        self.ensure_editor_synced_from_public();
        self.editor.move_insert_left();
        self.sync_public_from_editor();
    }

    pub fn move_cursor_right(&mut self) {
        self.mark_editor_dirty_if_public_changed();
        self.ensure_editor_synced_from_public();
        self.editor.move_insert_right();
        self.sync_public_from_editor();
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.mark_editor_dirty_if_public_changed();
        self.ensure_editor_synced_from_public();
        self.editor.enter_char(new_char);
        self.sync_public_from_editor();
    }

    pub fn delete_char(&mut self) {
        self.mark_editor_dirty_if_public_changed();
        self.ensure_editor_synced_from_public();
        self.editor.backspace();
        self.sync_public_from_editor();
    }

    fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
        self.input_mode = matches!(self.focus, Focus::Input);

        match self.focus {
            Focus::Input => {}
            Focus::History => self.select_first_history_if_available(),
            Focus::Variables => self.select_first_variable_if_available(),
        }
    }

    fn set_input_edit_mode(&mut self, mode: InputEditMode) {
        self.input_edit_mode = mode;
        self.editor_needs_sync = true;
        self.ensure_editor_synced_from_public();
    }

    fn set_input_text(&mut self, text: String) {
        self.input = text;
        self.character_index = self.input.chars().count();
        self.input_edit_mode = InputEditMode::Insert;
        self.editor_needs_sync = true;
        self.ensure_editor_synced_from_public();
        self.yank_flash = None;
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
        self.editor_needs_sync = true;
        self.ensure_editor_synced_from_public();
    }

    fn select_first_history_if_available(&mut self) {
        if self.history.is_empty() {
            self.history_state.select(None);
        } else if self.history_state.selected().is_none() {
            self.history_state.select(Some(0));
        }
    }

    fn select_first_variable_if_available(&mut self) {
        if self.variables.is_empty() {
            self.variables_state.select(None);
        } else if self.variables_state.selected().is_none() {
            self.variables_state.select(Some(0));
        }
    }

    fn move_history_selection_up(&mut self) {
        let len = self.history.len();
        if len == 0 {
            self.history_state.select(None);
            return;
        }

        let next = match self.history_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.history_state.select(Some(next));
    }

    fn move_history_selection_down(&mut self) {
        let len = self.history.len();
        if len == 0 {
            self.history_state.select(None);
            return;
        }

        let next = match self.history_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.history_state.select(Some(next));
    }

    fn sorted_variable_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.variables.keys().cloned().collect();
        keys.sort();
        keys
    }

    fn move_variables_selection_up(&mut self) {
        let len = self.sorted_variable_keys().len();
        if len == 0 {
            self.variables_state.select(None);
            return;
        }

        let next = match self.variables_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.variables_state.select(Some(next));
    }

    fn move_variables_selection_down(&mut self) {
        let len = self.sorted_variable_keys().len();
        if len == 0 {
            self.variables_state.select(None);
            return;
        }

        let next = match self.variables_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.variables_state.select(Some(next));
    }

    fn populate_input_from_history(&mut self) {
        let len = self.history.len();
        if len == 0 {
            return;
        }

        if let Some(selected_visual_idx) = self.history_state.selected()
            && selected_visual_idx < len
        {
            let history_idx = len - 1 - selected_visual_idx;
            self.set_input_text(self.history[history_idx].expression.clone());
            self.set_focus(Focus::Input);
        }
    }

    fn populate_input_from_variable(&mut self) {
        let keys = self.sorted_variable_keys();
        if let Some(selected_idx) = self.variables_state.selected()
            && let Some(key) = keys.get(selected_idx)
            && let Some(entry) = self.variables.get(key)
        {
            self.set_input_text(entry.expression.clone());
            self.set_focus(Focus::Input);
        }
    }

    pub fn submit_message(&mut self) {
        if self.input.is_empty() {
            return;
        }

        let mut tokenized = tokenize(&self.input);
        let mut var_name: Option<String> = None;
        if tokenized.contains(&"=") {
            let parsed_variables = parse_variables(tokenized);
            match parsed_variables {
                Ok(result) => {
                    tokenized = result.tokens;
                    var_name = Some(result.var_name);
                }
                Err(err) => {
                    self.history.push(History {
                        expression: self.input.clone(),
                        result: None,
                        error: Some(err),
                    });
                    return;
                }
            }
        }

        let res = calculate(tokenized, &self.variables);
        match res {
            Ok(result) => {
                if let Some(var_name) = var_name {
                    self.variables.insert(
                        var_name.to_string(),
                        VariableEntry {
                            expression: self.input.clone(),
                            value: result,
                        },
                    );
                } else {
                    self.history.push(History {
                        expression: self.input.clone(),
                        result: Some(result),
                        error: None,
                    });
                }
            }
            Err(err) => {
                self.history.push(History {
                    expression: self.input.clone(),
                    result: None,
                    error: Some(err),
                });
            }
        }

        self.input.clear();
        self.reset_cursor();
        self.set_focus(Focus::Input);
        self.set_input_edit_mode(InputEditMode::Insert);
        self.yank_flash = None;
    }

    fn handle_input_key_event(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Up && matches!(self.input_edit_mode, InputEditMode::Insert) {
            if let Some(last) = self.history.last() {
                self.set_input_text(last.expression.clone());
            }
            return false;
        }

        self.mark_editor_dirty_if_public_changed();
        self.ensure_editor_synced_from_public();
        match self.editor.handle_key_event(key) {
            EditorCommand::None => {
                self.sync_public_from_editor();
                false
            }
            EditorCommand::Submit => {
                self.sync_public_from_editor();
                self.submit_message();
                false
            }
            EditorCommand::ExitInputMode => {
                self.sync_public_from_editor();
                self.set_focus(Focus::History);
                false
            }
            EditorCommand::Yanked { start, end } => {
                self.sync_public_from_editor();
                self.yank_flash = Some(YankFlash {
                    start,
                    end,
                    expires_at: Instant::now() + Duration::from_millis(250),
                });
                false
            }
        }
    }

    fn handle_list_key_event(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Enter => {
                match self.focus {
                    Focus::History => self.populate_input_from_history(),
                    Focus::Variables => self.populate_input_from_variable(),
                    Focus::Input => {}
                }
                false
            }
            KeyCode::Char('i') => {
                self.set_focus(Focus::Input);
                self.set_input_edit_mode(InputEditMode::Insert);
                false
            }
            KeyCode::Left => {
                self.set_focus(Focus::History);
                false
            }
            KeyCode::Right => {
                self.set_focus(Focus::Variables);
                false
            }
            KeyCode::Up => {
                match self.focus {
                    Focus::History => self.move_history_selection_up(),
                    Focus::Variables => self.move_variables_selection_up(),
                    Focus::Input => {}
                }
                false
            }
            KeyCode::Down => {
                match self.focus {
                    Focus::History => self.move_history_selection_down(),
                    Focus::Variables => self.move_variables_selection_down(),
                    Focus::Input => {}
                }
                false
            }
            _ => false,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return true;
        }

        match self.focus {
            Focus::Input => self.handle_input_key_event(key),
            Focus::History | Focus::Variables => self.handle_list_key_event(key.code),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            if let Some(flash) = &self.yank_flash
                && Instant::now() >= flash.expires_at
            {
                self.yank_flash = None;
            }

            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(16))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && self.handle_key_event(key)
            {
                return Ok(());
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = vertical.areas(frame.area());

        let mode_label = match self.focus {
            Focus::Input => match self.input_edit_mode {
                InputEditMode::Insert => "INSERT",
                InputEditMode::Normal => "NORMAL",
                InputEditMode::Visual => "VISUAL",
            },
            Focus::History => "HISTORY",
            Focus::Variables => "VARIABLES",
        };

        let help_line = Line::from(vec![
            Span::styled(
                format!("[{}] ", mode_label),
                match self.input_edit_mode {
                    InputEditMode::Insert => Style::default().bold(),
                    InputEditMode::Normal | InputEditMode::Visual => Style::default().bold().blue(),
                },
            ),
            Span::raw(
                "Enter: submit/select • Esc: mode/focus • i: input • v: visual • y: yank • d/x: delete • p/P: paste",
            ),
        ]);
        let help_message = Paragraph::new(Text::from(help_line));
        frame.render_widget(help_message, help_area);

        let caret = if matches!(self.focus, Focus::Input) {
            match self.input_edit_mode {
                InputEditMode::Insert => "❯",
                InputEditMode::Normal | InputEditMode::Visual => "❮",
            }
        } else {
            "❮"
        };

        let visual_range = if matches!(self.focus, Focus::Input)
            && matches!(self.input_edit_mode, InputEditMode::Visual)
        {
            self.editor.visual_selection_range()
        } else {
            None
        };

        let now = Instant::now();
        let flash_range = self.yank_flash.as_ref().and_then(|flash| {
            if now < flash.expires_at {
                Some((flash.start, flash.end))
            } else {
                None
            }
        });

        let mut spans = vec![Span::raw(format!("{} ", caret))];
        for (idx, ch) in self.input.chars().enumerate() {
            let ch_text = ch.to_string();
            if let Some((start, end)) = flash_range
                && idx >= start
                && idx <= end
            {
                spans.push(Span::styled(
                    ch_text,
                    Style::default()
                        .bg(Color::Rgb(255, 165, 0))
                        .fg(Color::Black)
                        .bold(),
                ));
                continue;
            }

            if let Some((start, end)) = visual_range
                && idx >= start
                && idx <= end
            {
                spans.push(Span::styled(
                    ch_text,
                    Style::default().bg(Color::Cyan).fg(Color::Black),
                ));
                continue;
            }

            spans.push(Span::raw(ch_text));
        }

        let input = Paragraph::new(Line::from(spans))
            .style(Style::new().bg(Color::DarkGray))
            .block(Block::new().padding(Padding::vertical(1)));
        frame.render_widget(input, input_area);

        if matches!(self.focus, Focus::Input) {
            frame.set_cursor_position(Position::new(
                input_area.x + self.character_index as u16 + 2,
                input_area.y + 1,
            ));
        }

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(messages_area);

        let results: Vec<ListItem> = self
            .history
            .iter()
            .enumerate()
            .rev()
            .map(|(i, m)| match m.result {
                Some(result) => {
                    let content = Line::from(vec![
                        Span::styled(format!("{} ", i + 1), Style::default().dim()),
                        Span::styled(m.expression.clone(), Style::default().blue()),
                        Span::raw(" = "),
                        Span::styled(result.to_string(), Style::default().bold().green()),
                    ]);
                    ListItem::new(content)
                }
                _ => {
                    let content = Line::from(vec![
                        Span::raw(format!("{}: ", i + 1)),
                        Span::styled(format!("{m}"), Style::default().red().bold()),
                    ]);
                    ListItem::new(content)
                }
            })
            .collect();

        let history_focused = matches!(self.focus, Focus::History);
        let results = List::new(results)
            .highlight_style(Style::default().bg(Color::DarkGray).bold())
            .highlight_symbol("› ")
            .block(
                Block::bordered()
                    .border_type(if history_focused {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .border_style(Style::default().fg(if history_focused {
                        Color::LightCyan
                    } else {
                        Color::Cyan
                    }))
                    .padding(Padding::new(1, 1, 0, 0))
                    .title_style(Style::default().fg(Color::Cyan).bold())
                    .title("History"),
            );
        frame.render_stateful_widget(results, layout[0], &mut self.history_state);

        let mut sorted_variables: Vec<(&String, &VariableEntry)> = self.variables.iter().collect();
        sorted_variables.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));

        let variable_items: Vec<ListItem> = sorted_variables
            .into_iter()
            .map(|(k, v)| {
                let content = Line::from(vec![
                    Span::styled(format!("{} = ", k), Style::default().bold()),
                    Span::styled(v.value.to_string(), Style::default().bold().green()),
                ]);
                ListItem::new(content)
            })
            .collect();

        let variables_focused = matches!(self.focus, Focus::Variables);
        let variables = List::new(variable_items)
            .highlight_style(Style::default().bg(Color::DarkGray).bold())
            .highlight_symbol("› ")
            .block(
                Block::bordered()
                    .border_type(if variables_focused {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .border_style(Style::default().fg(if variables_focused {
                        Color::LightYellow
                    } else {
                        Color::Yellow
                    }))
                    .padding(Padding::new(1, 1, 0, 0))
                    .title_style(Style::default().fg(Color::Yellow).bold())
                    .title("Variables"),
            );
        frame.render_stateful_widget(variables, layout[1], &mut self.variables_state);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
