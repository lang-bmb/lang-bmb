// Differential test (backlog 4D-19): the ranges of the code points outside
// ASCII in each of the classes the lexer asks of Rust's `char`. The port's
// tables (`selfhost/unicode.bmb`) print the same through
// `unicode_classes.bmb`.
fn ranges(f: impl Fn(char) -> bool) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for c in 0x80u32..=0x10FFFF {
        let Some(ch) = char::from_u32(c) else {
            continue;
        };
        if f(ch) {
            match out.last_mut() {
                Some(l) if l.1 + 1 == c => l.1 = c,
                _ => out.push((c, c)),
            }
        }
    }
    out
}

fn main() {
    for (name, r) in [
        ("alphabetic", ranges(char::is_alphabetic)),
        ("numeric", ranges(char::is_numeric)),
        ("whitespace", ranges(char::is_whitespace)),
    ] {
        let s: String = r.iter().map(|(a, b)| format!("{a:06x}{b:06x}")).collect();
        println!("{name} {}\n{s}", r.len());
    }
}
