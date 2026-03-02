use std::collections::HashMap;

use rustic_calc::types::{AppState, History, VariableEntry};

pub fn sample_state() -> AppState {
    let mut variables = HashMap::new();
    variables.insert(
        "x".to_string(),
        VariableEntry {
            expression: "2+3".to_string(),
            value: 5.0,
        },
    );

    AppState {
        history: vec![History {
            expression: "1+1".to_string(),
            result: Some(2.0),
            error: None,
        }],
        variables,
        plot_data: Some(vec![(0.0, 1.0), (1.0, 2.0)]),
    }
}
