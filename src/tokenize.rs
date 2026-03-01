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

        if b.is_ascii_alphabetic() {
            while i < bytes.len() {
                let c = bytes[i];
                if c.is_ascii_alphabetic() {
                    tokens.push(&phrase[i..i + 1]);
                    if i > 0 && i < bytes.len() - 1 {
                        tokens.push("*");
                    }
                    i += 1;
                    continue;
                }
                break;
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
            _ => (),
        }

        i += 1;
    }

    tokens
}
