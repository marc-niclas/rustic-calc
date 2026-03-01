use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, Padding},
};

use crate::tui_app::{Focus, History};

pub fn render_history_block<'a>(history: &[History], focus: Focus) -> List<'a> {
    let results: Vec<ListItem> = history
        .iter()
        .enumerate()
        .rev()
        .map(|(i, m)| match (m.result, &m.error) {
            (Some(result), _) => {
                let content = Line::from(vec![
                    Span::styled(format!("{} ", i + 1), Style::default().dim()),
                    Span::styled(m.expression.clone(), Style::default().blue()),
                    Span::raw(" = "),
                    Span::styled(result.to_string(), Style::default().bold().green()),
                ]);
                ListItem::new(content)
            }
            (_, Some(_)) => {
                let content = Line::from(vec![
                    Span::raw(format!("{}: ", i + 1)),
                    Span::styled(format!("{m}"), Style::default().red().bold()),
                ]);
                ListItem::new(content)
            }
            (_, _) => {
                let content = Line::from(vec![
                    Span::raw(format!("{}: ", i + 1)),
                    Span::styled(format!("{m}"), Style::default().magenta().bold()),
                ]);
                ListItem::new(content)
            }
        })
        .collect();

    let history_focused = matches!(focus, Focus::History);
    let block = Block::bordered()
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
        .title("History");
    List::new(results)
        .highlight_style(Style::default().bg(Color::DarkGray).bold())
        .highlight_symbol("› ")
        .block(block)
}
