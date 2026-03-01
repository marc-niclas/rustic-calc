use std::time::Instant;

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

use crate::{
    tui_app::{Focus, InputEditMode},
    types::YankFlash,
};

pub fn render_input<'a>(
    focus: Focus,
    input_edit_mode: InputEditMode,
    input: &str,
    yank_flash: Option<&YankFlash>,
    visual_selection_range: impl Fn() -> Option<(usize, usize)>,
) -> Paragraph<'a> {
    let caret = if matches!(focus, Focus::Input) {
        match input_edit_mode {
            InputEditMode::Insert => "❯",
            InputEditMode::Normal | InputEditMode::Visual => "❮",
        }
    } else {
        "❮"
    };

    let visual_range =
        if matches!(focus, Focus::Input) && matches!(input_edit_mode, InputEditMode::Visual) {
            visual_selection_range()
        } else {
            None
        };

    let now = Instant::now();
    let flash_range = yank_flash.as_ref().and_then(|flash| {
        if now < flash.expires_at {
            Some((flash.start, flash.end))
        } else {
            None
        }
    });

    let mut spans = vec![Span::raw(format!("{} ", caret))];
    for (idx, ch) in input.chars().enumerate() {
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

    Paragraph::new(Line::from(spans))
        .style(Style::new().bg(Color::DarkGray))
        .block(Block::new().padding(Padding::vertical(1)))
}
