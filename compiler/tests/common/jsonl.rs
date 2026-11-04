//! Fields of one JSONL row as the CLI writes them (flat keys, `fix` nested
//! one level), for tests that run the CLI instead of the library (backlog
//! 4A-04). A key is found by `"key":`, so `line` does not match `end_line`.

/// The string value of `key`, unescaped.
pub fn str_field(row: &str, key: &str) -> Option<String> {
    let start = row.find(&format!("\"{key}\":\""))? + key.len() + 4;
    let mut out = String::new();
    let mut chars = row[start..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    out.push(char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?);
                }
                other => out.push(other),
            },
            c => out.push(c),
        }
    }
    None
}

/// The integer value of `key`.
pub fn num_field(row: &str, key: &str) -> Option<i64> {
    let start = row.find(&format!("\"{key}\":"))? + key.len() + 3;
    let digits: String = row[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .collect();
    digits.parse().ok()
}
