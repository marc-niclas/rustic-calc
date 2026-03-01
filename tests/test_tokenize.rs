use rustic_calc::tokenize::tokenize;

#[test]
fn test_tokenize() {
    let res = tokenize("2*2");
    assert_eq!(res, vec!["2", "*", "2"]);

    let res = tokenize("2.5*2");
    assert_eq!(res, vec!["2.5", "*", "2"]);
}

#[test]
fn test_tokenize_with_negative() {
    let res = tokenize("-2*2");
    assert_eq!(res, vec!["-", "2", "*", "2"]);
}

#[test]
fn save_variable_assignment_tokenized() {
    let res = tokenize("x=2");
    assert_eq!(res, vec!["x", "=", "2"]);

    let res = tokenize("x=abc");
    assert_eq!(res, vec!["x", "=", "a", "*", "b", "*", "c"]);
}

#[test]
fn save_variables_tokenized() {
    let res = tokenize("abc");
    assert_eq!(res, vec!["a", "*", "b", "*", "c"]);

    let res = tokenize("x=ab");
    assert_eq!(res, vec!["x", "=", "a", "*", "b"]);
}

#[test]
fn coefficients_tokenized() {
    let res = tokenize("7x");
    assert_eq!(res, vec!["7", "*", "x"]);
}
