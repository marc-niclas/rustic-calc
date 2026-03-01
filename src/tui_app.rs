use std::collections::HashMap;

use crate::calculate::calculate;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEditMode {
    Insert,
    Normal,
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
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            history: Vec::new(),
            character_index: 0,
            variables: HashMap::new(),
            input_mode: true,
            focus: Focus::Input,
            input_edit_mode: InputEditMode::Insert,
            history_state: ListState::default(),
            variables_state: ListState::default(),
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn move_cursor_left_normal(&mut self) {
        if self.input.is_empty() {
            self.character_index = 0;
            return;
        }
        self.character_index = self.character_index.saturating_sub(1);
    }

    fn move_cursor_right_normal(&mut self) {
        let len = self.input.chars().count();
        if len == 0 {
            self.character_index = 0;
            return;
        }
        self.character_index = (self.character_index + 1).min(len - 1);
    }

    fn move_cursor_to_line_start(&mut self) {
        self.character_index = 0;
    }

    fn move_cursor_to_line_end_insert(&mut self) {
        self.character_index = self.input.chars().count();
    }

    fn move_cursor_to_line_end_normal(&mut self) {
        let len = self.input.chars().count();
        self.character_index = len.saturating_sub(1);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn delete_char_under_cursor(&mut self) {
        let len = self.input.chars().count();
        if len == 0 || self.character_index >= len {
            return;
        }

        let idx = self.character_index;
        let before = self.input.chars().take(idx);
        let after = self.input.chars().skip(idx + 1);
        self.input = before.chain(after).collect();

        let new_len = self.input.chars().count();
        if new_len == 0 {
            self.character_index = 0;
        } else if self.character_index >= new_len {
            self.character_index = new_len - 1;
        }
    }

    fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    fn move_cursor_word_forward(&mut self) {
        let chars: Vec<char> = self.input.chars().collect();
        let len = chars.len();
        if len == 0 {
            self.character_index = 0;
            return;
        }

        let mut i = self.character_index.min(len - 1);

        if Self::is_word_char(chars[i]) {
            while i < len && Self::is_word_char(chars[i]) {
                i += 1;
            }
        }

        while i < len && !Self::is_word_char(chars[i]) {
            i += 1;
        }

        self.character_index = if i >= len { len - 1 } else { i };
    }

    fn move_cursor_word_backward(&mut self) {
        let chars: Vec<char> = self.input.chars().collect();
        let len = chars.len();
        if len == 0 {
            self.character_index = 0;
            return;
        }

        let mut i = self.character_index.min(len - 1);
        if i == 0 {
            return;
        }

        i -= 1;

        while i > 0 && !Self::is_word_char(chars[i]) {
            i -= 1;
        }

        while i > 0 && Self::is_word_char(chars[i - 1]) {
            i -= 1;
        }

        self.character_index = i;
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
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
    }

    fn switch_to_normal_mode(&mut self) {
        self.input_edit_mode = InputEditMode::Normal;
        let len = self.input.chars().count();
        if len == 0 {
            self.character_index = 0;
            return;
        }
        self.character_index = self.character_index.saturating_sub(1).min(len - 1);
    }

    fn switch_to_insert_mode(&mut self) {
        self.input_edit_mode = InputEditMode::Insert;
        self.character_index = self.clamp_cursor(self.character_index);
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
            self.input = self.history[history_idx].expression.clone();
            self.character_index = self.input.chars().count();
            self.set_focus(Focus::Input);
            self.set_input_edit_mode(InputEditMode::Insert);
        }
    }

    fn populate_input_from_variable(&mut self) {
        let keys = self.sorted_variable_keys();
        if let Some(selected_idx) = self.variables_state.selected()
            && let Some(key) = keys.get(selected_idx)
            && let Some(entry) = self.variables.get(key)
        {
            self.input = entry.expression.clone();
            self.character_index = self.input.chars().count();
            self.set_focus(Focus::Input);
            self.set_input_edit_mode(InputEditMode::Insert);
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
    }

    fn handle_input_key_event(&mut self, code: KeyCode) -> bool {
        match self.input_edit_mode {
            InputEditMode::Insert => match code {
                KeyCode::Esc => {
                    self.switch_to_normal_mode();
                    false
                }
                KeyCode::Enter => {
                    self.submit_message();
                    false
                }
                KeyCode::Char(to_insert) => {
                    self.enter_char(to_insert);
                    false
                }
                KeyCode::Backspace => {
                    self.delete_char();
                    false
                }
                KeyCode::Left => {
                    self.move_cursor_left();
                    false
                }
                KeyCode::Right => {
                    self.move_cursor_right();
                    false
                }
                KeyCode::Up => {
                    if let Some(last) = self.history.last() {
                        self.input = last.expression.clone();
                        self.character_index = self.input.chars().count();
                    }
                    false
                }
                _ => false,
            },
            InputEditMode::Normal => match code {
                KeyCode::Esc => {
                    self.set_focus(Focus::Variables);
                    false
                }
                KeyCode::Enter => {
                    self.submit_message();
                    false
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.move_cursor_left_normal();
                    false
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.move_cursor_right_normal();
                    false
                }
                KeyCode::Down => {
                    self.focus = Focus::History;
                    false
                }
                KeyCode::Char('0') => {
                    self.move_cursor_to_line_start();
                    false
                }
                KeyCode::Char('$') => {
                    self.move_cursor_to_line_end_normal();
                    false
                }
                KeyCode::Char('x') => {
                    self.delete_char_under_cursor();
                    false
                }
                KeyCode::Char('w') => {
                    self.move_cursor_word_forward();
                    false
                }
                KeyCode::Char('b') => {
                    self.move_cursor_word_backward();
                    false
                }
                KeyCode::Char('i') => {
                    self.switch_to_insert_mode();
                    false
                }
                KeyCode::Char('a') => {
                    let len = self.input.chars().count();
                    if len == 0 {
                        self.character_index = 0;
                    } else {
                        self.character_index = (self.character_index + 1).min(len);
                    }
                    self.switch_to_insert_mode();
                    false
                }
                KeyCode::Char('A') => {
                    self.move_cursor_to_line_end_insert();
                    self.switch_to_insert_mode();
                    false
                }
                KeyCode::Char('I') => {
                    self.move_cursor_to_line_start();
                    self.switch_to_insert_mode();
                    false
                }
                _ => false,
            },
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
            Focus::Input => self.handle_input_key_event(key.code),
            Focus::History | Focus::Variables => self.handle_list_key_event(key.code),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()?
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
            },
            Focus::History => "HISTORY",
            Focus::Variables => "VARIABLES",
        };

        let help_line = Line::from(vec![
            Span::styled(format!("[{}] ", mode_label), Style::default().bold()),
            Span::raw("Enter: submit/select • Esc: mode/focus • i: input mode"),
        ]);
        let help_message = Paragraph::new(Text::from(help_line));
        frame.render_widget(help_message, help_area);

        let caret = if matches!(self.focus, Focus::Input) {
            match self.input_edit_mode {
                InputEditMode::Insert => "❯",
                InputEditMode::Normal => "▮",
            }
        } else {
            "❮"
        };

        let input = Paragraph::new(format!("{} {}", caret, self.input))
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
