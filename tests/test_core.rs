use approx::assert_relative_eq;
use rustic_calc::{calculate, tokenize};

#[test]
fn test_tokenize() {
    let res = tokenize("2x2");
    assert_eq!(res, vec!["2", "*", "2"]);

    let res = tokenize("2.5x2");
    println!("{res:?}");
    assert_eq!(res, vec!["2.5", "*", "2"]);
}

#[test]
fn test_tokenize_with_negative() {
    let res = tokenize("-2x2");
    assert_eq!(res, vec!["-", "2", "*", "2"]);
}

#[test]
fn test_multiply() {
    let tokens = vec!["2", "*", "2"];
    let res = calculate(tokens);
    assert_relative_eq!(res, 4.0);

    let tokens = vec!["2.5", "*", "2"];
    let res = calculate(tokens);
    println!("{res:?}");
    assert_relative_eq!(res, 5.0);
}

#[test]
fn test_sum() {
    let tokens = vec!["2", "+", "2"];
    let res = calculate(tokens);
    assert_relative_eq!(res, 4.0);

    let tokens = vec!["1.5", "+", "1", "+", "0.5"];
    let res = calculate(tokens);
    println!("{res:?}");
    assert_relative_eq!(res, 3.0);
}

#[test]
fn test_subtract() {
    let tokens = vec!["2", "-", "2"];
    let res = calculate(tokens);
    assert_relative_eq!(res, 0.0);

    let tokens = vec!["2.5", "-", "1", "-", "0.5"];
    let res = calculate(tokens);
    println!("{res:?}");
    assert_relative_eq!(res, 1.0);
}

#[test]
fn test_divide() {
    let tokens = vec!["2", "/", "2"];
    let res = calculate(tokens);
    assert_relative_eq!(res, 1.);

    let tokens = vec!["2.5", "/", "0.5", "/", "0.5"];
    let res = calculate(tokens);
    println!("{res:?}");
    assert_relative_eq!(res, 10.0);
}

#[test]
fn test_order_of_operations() {
    let tokens = vec!["2", "+", "3", "*", "2"];
    let res = calculate(tokens);
    assert_relative_eq!(res, 8.);
}

#[test]
fn test_start_w_negative() {
    let tokens = vec!["-", "2", "+", "2", "*", "2"];
    let res = calculate(tokens);
    assert_relative_eq!(res, 2.);
}
