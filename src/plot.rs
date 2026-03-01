use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    symbols::Marker,
    widgets::{Axis, Block, BorderType, Chart, Dataset, GraphType, Padding},
};

pub fn render_scatter(frame: &mut Frame, area: Rect, data: &[(f64, f64)], name: String) {
    let datasets = vec![
        Dataset::default()
            .name(name)
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::new().yellow())
            .data(data),
    ];

    let (x_min, x_max, y_min, y_max) = min_max_xy(data).unwrap_or((0., 10., 0., 100.));
    let x_labels = generate_labels(x_min, x_max);
    let y_labels = generate_labels(y_min, y_max);
    let chart = Chart::new(datasets)
        .block(
            Block::bordered()
                .title("Scatter Chart")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Magenta))
                .padding(Padding::uniform(1)),
        )
        .x_axis(
            Axis::default()
                .title("x")
                .bounds([x_min, x_max])
                .style(Style::default().fg(Color::Gray))
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("y")
                .bounds([y_min, y_max])
                .style(Style::default().fg(Color::Gray))
                .labels(y_labels),
        )
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));

    frame.render_widget(chart, area);
}

fn min_max_xy(data: &[(f64, f64)]) -> Option<(f64, f64, f64, f64)> {
    let &(x0, y0) = data.first()?;

    Some(
        data.iter()
            .copied()
            .fold((x0, x0, y0, y0), |(min_x, max_x, min_y, max_y), (x, y)| {
                (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
            }),
    )
}

fn generate_labels(min: f64, max: f64) -> Vec<String> {
    let delta = max - min;
    let step = delta / 5.0;
    let mut labels = Vec::new();
    for i in 0..6 {
        let x = min + step * i as f64;
        labels.push(format!("{:.1}", x));
    }
    labels
}
