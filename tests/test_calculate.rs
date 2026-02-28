use std::collections::HashMap;

use approx::assert_relative_eq;
use rustic_calc::calculate::calculate;
use rustic_calc::tokenize::tokenize;

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
