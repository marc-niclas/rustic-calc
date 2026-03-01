use rustic_calc::variables::parse_variables;

#[test]
fn test_parse_variables() {
    let res = parse_variables(vec!["x", "=", "2"]).unwrap();
    assert_eq!(res.var_name, "x".to_string());
    assert_eq!(res.tokens, vec!["2"]);
}

#[test]
fn test_parse_variables_formula() {
    let res = parse_variables(vec!["x", "=", "2", "+", "3"]).unwrap();
    assert_eq!(res.var_name, "x".to_string());
    assert_eq!(res.tokens, vec!["2", "+", "3"]);
}
