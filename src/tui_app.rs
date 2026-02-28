use std::collections::HashMap;

use crate::calculate::calculate;
use crate::tokenize::tokenize;
use crate::variables::parse_variables;
use color_eyre::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::BorderType;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Padding, Paragraph},
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

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub character_index: usize,
    /// History of recorded messages
    pub history: Vec<History>,
    /// Variables stored in the calculator
    pub variables: HashMap<String, f64>,
    pub input_mode: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            history: Vec::new(),
            character_index: 0,
            variables: HashMap::new(),
            input_mode: true,
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

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
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
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn submit_message(&mut self) {
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
                    self.variables.insert(var_name.to_string(), result);
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
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => true,
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
                    self.character_index = self.input.len();
                }
                false
            }
            KeyCode::Esc => {
                self.input_mode = false;
                false
            }
            _ => false,
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

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = vertical.areas(frame.area());

        let (msg, style) = (
            vec!["Enter".bold(), " to calculate".into()],
            Style::default(),
        );
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let caret = if self.input_mode { "❯" } else { "❮" };
        let input = Paragraph::new(format!("{} {}", caret, self.input))
            .style(Style::new().bg(Color::DarkGray))
            .block(Block::new().padding(Padding::vertical(1)));
        frame.render_widget(input, input_area);
        frame.set_cursor_position(Position::new(
            input_area.x + self.character_index as u16 + 2,
            input_area.y + 1,
        ));

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
                        Span::raw(format!("{}: ", i + 1)),
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
        let results = List::new(results).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title_style(Style::default().fg(Color::Cyan).bold())
                .title("History"),
        );
        frame.render_widget(results, layout[0]);

        let variable_items: Vec<ListItem> = self
            .variables
            .iter()
            .map(|(k, v)| {
                let content = Line::from(vec![
                    Span::styled(format!("{} = ", k), Style::default().bold()),
                    Span::styled(v.to_string(), Style::default().bold().green()),
                ]);
                ListItem::new(content)
            })
            .collect();
        let variables = List::new(variable_items).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title_style(Style::default().fg(Color::Yellow).bold())
                .title("Variables"),
        );
        frame.render_widget(variables, layout[1]);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
