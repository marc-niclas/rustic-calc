use std::collections::HashMap;

use crate::types::VariableEntry;

pub fn calculate(
    tokens: Vec<&str>,
    variables: &HashMap<String, VariableEntry>,
) -> Result<f64, String> {
    if tokens.is_empty() {
        return Err("Expression could not be parsed".to_string());
    }

    let mut parser = Parser::new(&tokens, variables);
    let value = parser.parse_expr()?;

    if let Some(tok) = parser.peek() {
        return Err(format!("Unexpected token: {}", tok));
    }

    Ok(value)
}

struct Parser<'a> {
    tokens: &'a [&'a str],
    pos: usize,
    variables: &'a HashMap<String, VariableEntry>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [&'a str], variables: &'a HashMap<String, VariableEntry>) -> Self {
        Self {
            tokens,
            pos: 0,
            variables,
        }
    }

    fn peek(&self) -> Option<&'a str> {
        self.tokens.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<&'a str> {
        let tok = self.peek();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn consume(&mut self, expected: &str) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    // expr := add_sub
    fn parse_expr(&mut self) -> Result<f64, String> {
        self.parse_add_sub()
    }

    // add_sub := mul_div (("+" | "-") mul_div)*
    fn parse_add_sub(&mut self) -> Result<f64, String> {
        let mut lhs = self.parse_mul_div()?;

        loop {
            if self.consume("+") {
                let rhs = self.parse_mul_div()?;
                lhs += rhs;
            } else if self.consume("-") {
                let rhs = self.parse_mul_div()?;
                lhs -= rhs;
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    // mul_div := unary (("*" | "/") unary)*
    fn parse_mul_div(&mut self) -> Result<f64, String> {
        let mut lhs = self.parse_unary()?;

        loop {
            if self.consume("*") {
                let rhs = self.parse_unary()?;
                lhs *= rhs;
            } else if self.consume("/") {
                let rhs = self.parse_unary()?;
                lhs /= rhs;
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    // unary := ("+" | "-") unary | power
    fn parse_unary(&mut self) -> Result<f64, String> {
        if self.consume("+") {
            return self.parse_unary();
        }

        if self.consume("-") {
            return Ok(-self.parse_unary()?);
        }

        self.parse_power()
    }

    // power := primary ("^" unary)?
    // Right-associative because exponent is parsed via unary -> power.
    fn parse_power(&mut self) -> Result<f64, String> {
        let base = self.parse_primary()?;

        if self.consume("^") {
            let exponent = self.parse_unary()?;
            Ok(base.powf(exponent))
        } else {
            Ok(base)
        }
    }

    // primary := NUMBER | IDENT | "(" expr ")"
    fn parse_primary(&mut self) -> Result<f64, String> {
        let Some(tok) = self.next() else {
            return Err("Expression could not be parsed".to_string());
        };

        if tok == "(" {
            let value = self.parse_expr()?;
            if !self.consume(")") {
                return Err("Missing closing ')'".to_string());
            }
            return Ok(value);
        }

        if tok == ")" {
            return Err("Unexpected token: )".to_string());
        }

        if let Ok(num) = tok.parse::<f64>() {
            return Ok(num);
        }

        if let Some(var) = self.variables.get(tok) {
            return Ok(var.value);
        }

        Err(format!("Unknown variable: {}", tok))
    }
}
