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

            if needs_implicit_mul_before_number(&tokens) {
                tokens.push("*");
            }

            tokens.push(&phrase[start..i]);
            continue;
        }

        if b.is_ascii_alphabetic() {
            // Split alphabetic runs into single-letter variables:
            // "abc" -> ["a", "*", "b", "*", "c"]
            if needs_implicit_mul_before_ident(&tokens) {
                tokens.push("*");
            }

            let start = i;
            i += 1;
            tokens.push(&phrase[start..i]);

            while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                tokens.push("*");
                let s = i;
                i += 1;
                tokens.push(&phrase[s..i]);
            }

            continue;
        }

        match b {
            b'+' => tokens.push("+"),
            b'-' => tokens.push("-"),
            b'*' => tokens.push("*"),
            b'/' => tokens.push("/"),
            b'^' => tokens.push("^"),
            b'=' => tokens.push("="),
            b'(' => tokens.push("("),
            b')' => tokens.push(")"),
            _ => {}
        }

        i += 1;
    }

    tokens
}

fn needs_implicit_mul_before_ident(tokens: &[&str]) -> bool {
    matches!(
        tokens.last().copied(),
        Some(tok) if is_number_token(tok) || is_identifier_token(tok) || tok == ")"
    )
}

fn needs_implicit_mul_before_number(tokens: &[&str]) -> bool {
    matches!(
        tokens.last().copied(),
        Some(tok) if is_identifier_token(tok) || tok == ")"
    )
}

fn is_identifier_token(tok: &str) -> bool {
    tok.len() == 1 && tok.as_bytes()[0].is_ascii_alphabetic()
}

fn is_number_token(tok: &str) -> bool {
    let mut saw_digit = false;
    let mut saw_dot = false;

    for b in tok.bytes() {
        if b.is_ascii_digit() {
            saw_digit = true;
            continue;
        }

        if b == b'.' && !saw_dot {
            saw_dot = true;
            continue;
        }

        return false;
    }

    saw_digit
}
