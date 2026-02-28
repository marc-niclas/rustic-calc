use std::collections::HashMap;

pub fn tokenize(phrase: &str) -> Vec<&str> {
    let mut tokens: Vec<&str> = Vec::new();
    let bytes = phrase.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }

        if b.is_ascii_digit() || b == b'.' {
            let start = i;
            let mut saw_dot = b == b'.';
            i += 1;

            while i < bytes.len() {
                let c = bytes[i];
                if c.is_ascii_digit() {
                    i += 1;
                    continue;
                }
                if c == b'.' && !saw_dot {
                    saw_dot = true;
                    i += 1;
                    continue;
                }
                break;
            }

            tokens.push(&phrase[start..i]);
            continue;
        }

        match b {
            b'+' => tokens.push("+"),
            b'-' => tokens.push("-"),
            b'*' => tokens.push("*"),
            b'x' => tokens.push("*"),
            b'/' => tokens.push("/"),
            b'^' => tokens.push("^"),
            _ => (),
        }

        i += 1;
    }

    tokens
}

pub fn calculate(tokens: Vec<&str>) -> Result<f64, String> {
    let precedence_map: HashMap<String, i32> = HashMap::from([
        ("^".to_string(), 3),
        ("*".to_string(), 2),
        ("/".to_string(), 2),
        ("+".to_string(), 1),
        ("-".to_string(), 1),
    ]);

    let mut values: Vec<f64> = Vec::new();
    let mut ops: Vec<String> = Vec::new();

    for (i, t) in tokens.iter().enumerate() {
        match t.parse::<f64>() {
            Ok(num) => values.push(num),
            Err(_) => {
                if *t == "-" && (i == 0 || precedence_map.contains_key(tokens[i - 1])) {
                    values.push(0.);
                }

                while !ops.is_empty()
                    && precedence_map.get(&ops[ops.len() - 1]) >= precedence_map.get(*t)
                {
                    apply_top_operator(&mut values, &mut ops)?;
                }
                ops.push(t.to_string());
            }
        }
    }

    while !ops.is_empty() {
        apply_top_operator(&mut values, &mut ops)?;
    }

    if values.is_empty() {
        Err("Expression could not be parsed".to_string())
    } else {
        Ok(values[values.len() - 1])
    }
}

fn apply_top_operator(values: &mut Vec<f64>, ops: &mut Vec<String>) -> Result<(), String> {
    let b = values.pop();
    let a = values.pop();

    if let Some(op) = ops.pop() {
        match (a, b) {
            (Some(a), Some(b)) => match op.as_str() {
                "+" => {
                    values.push(a + b);
                    Ok(())
                }
                "-" => {
                    values.push(a - b);
                    Ok(())
                }
                "*" => {
                    values.push(a * b);
                    Ok(())
                }
                "/" => {
                    values.push(a / b);
                    Ok(())
                }
                "^" => {
                    values.push(a.powf(b));
                    Ok(())
                }
                _ => Err("Missing operator".to_string()),
            },
            _ => Err("Missing operand".to_string()),
        }
    } else {
        Err("Missing operator".to_string())
    }
}
