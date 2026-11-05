// Rust translation of tests/run/strings.bmb (backlog 2U-05), for its
// expected output.
fn word_end(t: &str, i: usize) -> usize {
    let mut j = i;
    while j < t.len() && t.as_bytes()[j] != b' ' {
        j += 1;
    }
    j
}

fn words(t: &str) -> Vec<String> {
    let mut v = Vec::new();
    let mut i = 0;
    while i < t.len() {
        if t.as_bytes()[i] == b' ' {
            i += 1;
        } else {
            let e = word_end(t, i);
            if t.is_char_boundary(i) && t.is_char_boundary(e) {
                v.push(String::from(&t[i..e]));
            }
            i = if e < t.len() { e + 1 } else { e };
        }
    }
    v
}

fn longest(v: &[String]) -> usize {
    let mut best = 0;
    for i in 1..v.len() {
        if v[i].len() > v[best].len() {
            best = i;
        }
    }
    best
}

fn first(v: &[String]) -> usize {
    let mut best = 0;
    for i in 1..v.len() {
        if v[i] < v[best] {
            best = i;
        }
    }
    best
}

fn join(v: &[String]) -> String {
    let mut out = String::new();
    for (i, w) in v.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(w);
    }
    out
}

fn show(t: &str) {
    let v = words(t);
    println!("{}", v.len());
    if !v.is_empty() {
        let (l, f) = (longest(&v), first(&v));
        println!("{} {}", v[l], v[f]);
    }
    println!("{}", join(&v));
}

fn main() {
    show("the quick brown fox jumps over the lazy dog");
    show("  café crème  brûlée ");
    show("");
    show("Ελληνικά und Deutsch: Übermut");
    let mut s = String::from("a");
    s.push('é');
    s.push('€');
    s.push('😀');
    println!("{}", s.len());
    if s.as_str() == "aé€😀" && s.as_str() < "b" {
        print!("same\n");
    }
}
