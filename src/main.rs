use rustic_calc::{calculate, tokenize};

fn main() {
    println!("Hello, world!");
    let expr = "2 + 2";
    let tokens = tokenize(expr);
    let res = calculate(tokens);
    println!("{} = {}", expr, res);
}
