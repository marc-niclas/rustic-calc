use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub use crate::input_editor::InputEditMode;
use crate::{
    calculate::calculate, inspect::inspect_unknown_variables, types::YankFlash,
    widgets::input_area::render_input,
};
use crate::{
    input_editor::{EditorCommand, InputEditor, Motion},
    widgets::plot_block::render_scatter,
};
use crate::{tokenize::tokenize, widgets::variable_block::render_variable_block};
use crate::{types::VariableEntry, widgets::history_block::render_history_block};
use crate::{variables::parse_variables, widgets::help_message::render_help_message};
use color_eyre::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEventKind},
    layout::{Constraint, Direction, Layout, Position},
    widgets::ListState,
};

#[derive(Debug, Clone)]
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
            (_, _) => write!(f, "{} 📈", self.expression),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Input,
    History,
    Variables,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Focus::Input => Focus::History,
            Focus::History => Focus::Variables,
            Focus::Variables => Focus::Input, // wrap
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Focus::Input => Focus::Variables, // wrap
            Focus::History => Focus::Input,
            Focus::Variables => Focus::History,
        }
    }
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
    pub plot_data: Option<Vec<(f64, f64)>>,
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
            plot_data: None,
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

        let unknown_variables = inspect_unknown_variables(&tokenized, &self.variables);
        if !unknown_variables.is_empty() {
            if unknown_variables.len() == 1 {
                let mut plot_data: Vec<(f64, f64)> = Vec::new();
                let mut cloned_variables = self.variables.clone();
                for i in -10..11 {
                    cloned_variables.insert(
                        unknown_variables[0].to_string(),
                        VariableEntry {
                            expression: "".to_string(),
                            value: i as f64,
                        },
                    );
                    let value = calculate(tokenized.clone(), &cloned_variables).unwrap_or_default();
                    plot_data.push((i as f64, value));
                }
                self.plot_data = Some(plot_data);
                self.history.push(History {
                    expression: self.input.clone(),
                    result: None,
                    error: None,
                });
                self.input.clear();
                self.reset_cursor();
                return;
            }

            self.history.push(History {
                expression: self.input.clone(),
                result: None,
                error: Some(format!(
                    "Unknown variables: {}",
                    unknown_variables.join(", ")
                )),
            });
            self.input.clear();
            self.reset_cursor();
            return;
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
            EditorCommand::IncrementFocus => {
                self.set_focus(self.focus.next());
                false
            }
            EditorCommand::DecrementFocus => {
                self.set_focus(self.focus.prev());
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
            KeyCode::Tab => {
                self.set_focus(self.focus.next());
                false
            }
            KeyCode::BackTab => {
                self.set_focus(self.focus.prev());
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

        let help_message = render_help_message(self.focus, self.input_edit_mode);
        frame.render_widget(help_message, help_area);

        let get_visual_range = || self.editor.visual_range();

        let input = render_input(
            self.focus,
            self.input_edit_mode,
            &self.input,
            self.yank_flash.as_ref(),
            get_visual_range,
        );
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
        let right_pane = layout[0];
        let left_pane = layout[1];
        let mut right_layout_constraints = vec![Constraint::Percentage(100)];
        if let Some(plot_data) = self.plot_data.as_ref()
            && !plot_data.is_empty()
        {
            right_layout_constraints = vec![Constraint::Percentage(50), Constraint::Percentage(50)];
        }
        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(right_layout_constraints)
            .split(right_pane);

        let history_block = render_history_block(&self.history, self.focus);
        frame.render_stateful_widget(history_block, right_layout[0], &mut self.history_state);

        let variable_list = render_variable_block(&self.variables, self.focus);
        frame.render_stateful_widget(variable_list, left_pane, &mut self.variables_state);

        if let Some(plot_data) = &self.plot_data
            && let Some(pane) = right_layout.get(1)
            && let Some(last) = self.history.last()
        {
            let chart = render_scatter(plot_data, last.expression.clone());
            frame.render_widget(chart, *pane);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
