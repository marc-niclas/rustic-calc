use std::collections::HashMap;

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, Padding},
};

use crate::{tui_app::Focus, types::VariableEntry};

pub fn render_variable_block<'a>(
    variables: &HashMap<String, VariableEntry>,
    focus: Focus,
) -> List<'a> {
    let mut sorted_variables: Vec<(&String, &VariableEntry)> = variables.iter().collect();
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

    let variables_focused = matches!(focus, Focus::Variables);
    let block = Block::bordered()
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
        .title("Variables");
    List::new(variable_items)
        .highlight_style(Style::default().bg(Color::DarkGray).bold())
        .highlight_symbol("› ")
        .block(block)
}
