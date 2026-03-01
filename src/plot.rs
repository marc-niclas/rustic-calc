use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Chart, Dataset, GraphType},
};

pub fn render_scatter(frame: &mut Frame, area: Rect, data: &[(f64, f64)]) {
    let datasets = vec![
        Dataset::default()
            .name("Heavy")
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::new().yellow())
            .data(data),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::bordered().title(
                Line::from(Span::styled(
                    "Scatter chart",
                    Style::default().cyan().bold(),
                ))
                .centered(),
            ),
        )
        .x_axis(
            Axis::default()
                .title("Year")
                .bounds([1960., 2020.])
                .style(Style::default().fg(Color::Gray))
                .labels(["1960", "1990", "2020"]),
        )
        .y_axis(
            Axis::default()
                .title("Cost")
                .bounds([0., 75000.])
                .style(Style::default().fg(Color::Gray))
                .labels(["0", "37 500", "75 000"]),
        )
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));

    frame.render_widget(chart, area);
}
