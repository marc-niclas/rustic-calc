#[derive(Debug)]
pub struct VariableParseReturn<'a> {
    pub var_name: String,
    pub tokens: Vec<&'a str>,
}

pub fn parse_variables<'a>(tokens: Vec<&'a str>) -> Result<VariableParseReturn<'a>, String> {
    if !tokens.contains(&"=") {
        return Err("No assignment found".to_string());
    }

    let assignment_index = tokens
        .iter()
        .position(|&t| t == "=")
        .ok_or_else(|| "No assignment found".to_string())?;

    if assignment_index == 0 {
        return Err("Missing variable name before '='".to_string());
    }

    let var_name = tokens[assignment_index - 1].to_string();
    let value_tokens = tokens.into_iter().skip(assignment_index + 1).collect();

    Ok(VariableParseReturn {
        var_name,
        tokens: value_tokens,
    })
}
