use std::collections::HashMap;

use crate::types::VariableEntry;

const OPERATORS: &[&str] = &["+", "-", "*", "/", "^"];

const PHRASE_LIMITERS: &[&str] = &["(", ")"];

pub fn inspect_unknown_variables(
    tokens: &Vec<&str>,
    variables: &HashMap<String, VariableEntry>,
) -> Vec<String> {
    let mut unknown_variables: Vec<String> = Vec::new();

    for t in tokens {
        if t.parse::<f64>().is_ok() {
            continue;
        }
        if OPERATORS.contains(t) | PHRASE_LIMITERS.contains(t) {
            continue;
        }
        if variables.get(*t).is_some() {
            continue;
        }
        if !unknown_variables.contains(&t.to_string()) {
            unknown_variables.push(t.to_string());
        }
    }

    unknown_variables
}
