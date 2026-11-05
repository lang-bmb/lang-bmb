// Rust translation of tests/run/escape.bmb (backlog 2U-05), for its
// expected output.
fn hex(d: u32) -> char {
    char::from_digit(d, 16).unwrap()
}

fn escape(t: &str) -> String {
    let mut out = String::from("\"");
    let mut a = 0;
    for (i, &b) in t.as_bytes().iter().enumerate() {
        if b == b'"' || b == b'\\' || b < 32 {
            out.push_str(&t[a..i]);
            match b {
                b'"' => out.push_str("\\\""),
                b'\\' => out.push_str("\\\\"),
                b'\n' => out.push_str("\\n"),
                _ => {
                    out.push_str("\\u00");
                    out.push(hex(u32::from(b / 16)));
                    out.push(hex(u32::from(b % 16)));
                }
            }
            a = i + 1;
        }
    }
    out.push_str(&t[a..]);
    out.push('"');
    out
}

fn show(t: &str) {
    let e = escape(t);
    println!("{e}");
    println!("{}", e.len());
}

fn main() {
    show("plain");
    show("say \"hi\"\n");
    show("tab\tbell\u{7}back\\slash");
    show("naïve café — ok ✓");
    show("");
}
