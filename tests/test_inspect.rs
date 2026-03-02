use std::collections::HashMap;

use rustic_calc::{inspect::inspect_unknown_variables, types::VariableEntry};

#[test]
fn inspect_zero_unknown_variables() {
    let unknown_variables = inspect_unknown_variables(&vec!["2", "+", "2"], &HashMap::new());
    assert!(unknown_variables.is_empty());

    let unknown_variables = inspect_unknown_variables(
        &vec!["2", "+", "a"],
        &HashMap::from([(
            "a".to_string(),
            VariableEntry {
                expression: "a=1".to_string(),
                value: 1.0,
            },
        )]),
    );
    assert!(unknown_variables.is_empty());
}

#[test]
fn inspect_one_variable() {
    let unknown_variables = inspect_unknown_variables(&vec!["2", "+", "a"], &HashMap::new());
    assert_eq!(unknown_variables.len(), 1);
    assert_eq!(unknown_variables[0], "a".to_string());
}

#[test]
fn inspect_duplicate_variables() {
    let unknown_variables =
        inspect_unknown_variables(&vec!["2", "+", "a", "*", "a"], &HashMap::new());
    assert_eq!(unknown_variables.len(), 1);
    assert_eq!(unknown_variables[0], "a".to_string());
}

#[test]
fn inspect_phrase() {
    let unknown_variables =
        inspect_unknown_variables(&vec!["3", "*", "(", "2", "-", "5", ")"], &HashMap::new());
    assert!(unknown_variables.is_empty());
}
