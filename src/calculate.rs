use std::collections::HashMap;

pub fn calculate(tokens: Vec<&str>, variables: &HashMap<String, f64>) -> Result<f64, String> {
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
                if !precedence_map.keys().any(|k| k == t) {
                    if let Some(var) = variables.get(*t) {
                        values.push(*var);
                    } else {
                        return Err(format!("Unknown variable: {}", t));
                    }
                } else {
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
