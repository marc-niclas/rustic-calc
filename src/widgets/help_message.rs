use ratatui::{
    style::Style,
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use crate::{input_editor::InputEditMode, types::Focus};

pub fn render_help_message<'a>(focus: Focus, input_edit_mode: InputEditMode) -> Paragraph<'a> {
    let mode_label = match focus {
        Focus::Input => match input_edit_mode {
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
            match input_edit_mode {
                InputEditMode::Insert => Style::default().bold(),
                InputEditMode::Normal | InputEditMode::Visual => Style::default().bold().blue(),
            },
        ),
        Span::raw(match focus {
            Focus::Input => {
                "Enter: submit/select • Esc: mode/focus • i: input • v: visual • y: yank • d/x: delete • p/P: paste"
            }
            Focus::History => "Enter: select • Esc: mode/focus • d/x: delete",
            Focus::Variables => "Enter: select • Esc: mode/focus • d/x: delete",
        }),
    ]);

    Paragraph::new(Text::from(help_line))
}
