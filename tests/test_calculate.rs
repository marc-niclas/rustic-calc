use std::collections::HashMap;

use approx::assert_relative_eq;
use rustic_calc::calculate::calculate;
use rustic_calc::tokenize::tokenize;
use rustic_calc::types::VariableEntry;

#[test]
fn test_multiply() {
    let tokens = vec!["2", "*", "2"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 4.0);

    let tokens = vec!["2.5", "*", "2"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 5.0);
}

#[test]
fn test_sum() {
    let tokens = vec!["2", "+", "2"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 4.0);

    let tokens = vec!["1.5", "+", "1", "+", "0.5"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 3.0);
}

#[test]
fn test_subtract() {
    let tokens = vec!["2", "-", "2"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 0.0);

    let tokens = vec!["2.5", "-", "1", "-", "0.5"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 1.0);
}

#[test]
fn test_divide() {
    let tokens = vec!["2", "/", "2"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 1.);

    let tokens = vec!["2.5", "/", "0.5", "/", "0.5"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 10.0);
}

#[test]
fn test_powers() {
    let tokens = vec!["2", "^", "2"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 4.);

    let tokens = vec!["4", "^", "0.5"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 2.);
}

#[test]
fn test_order_of_operations() {
    let tokens = vec!["2", "+", "3", "*", "2"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 8.);

    let tokens = vec!["2", "*", "3", "^", "2"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 18.);
}

#[test]
fn test_parenthesized_expression_with_power() {
    let tokens = tokenize("(2+2)^2");
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 16.0);
}

#[test]
fn test_parenthesized_expression_with_variable() {
    let tokens = tokenize("(a+5)/2");
    let res = calculate(
        tokens,
        &HashMap::from([(
            "a".to_string(),
            VariableEntry {
                expression: "a=5".to_string(),
                value: 10.0,
            },
        )]),
    )
    .unwrap();
    assert_relative_eq!(res, 7.5);
}

#[test]
fn test_double_nested_parenthesized_expression_with_power() {
    let tokens = tokenize("((2+2)/5)^2");
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 0.64);

    let tokens = tokenize("3((2+2)/5)^2");
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 1.92, epsilon = 1e-12);
}

#[test]
fn test_start_w_negative() {
    let tokens = vec!["-", "2", "+", "2", "*", "2"];
    let res = calculate(tokens, &HashMap::new()).unwrap();
    assert_relative_eq!(res, 2.);
}

#[test]
fn test_error_handling() {
    let tokens = tokenize("asdf");
    let res = calculate(tokens, &HashMap::new());
    match res {
        Ok(_) => panic!("no way"),
        Err(err) => {
            assert_eq!(err, "Unknown variable: a")
        }
    }
}
