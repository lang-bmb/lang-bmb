// Rust translation of tests/run/std_text_keys.bmb (backlog 4B-04), for its
// expected output (`rustc --edition 2021`).
use std::collections::{BTreeMap, BTreeSet};

fn count(s: &str, drop: &str) {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    if !s.is_empty() {
        for w in s.split(' ') {
            let c = counts.get(w).copied().unwrap_or(0);
            if c < 1000 {
                counts.insert(w.to_string(), c + 1);
            }
            seen.insert(w.to_string());
        }
    }
    for v in counts.values() {
        println!("{v}");
    }
    for k in counts.keys() {
        println!("{k}");
    }
    if seen.contains("ab") {
        println!("has ab");
    }
    seen.remove(drop);
    println!("{}", seen.len());
    for k in &seen {
        println!("{k}");
    }
}

fn main() {
    count("a b a ab é B a ab", "a");
    count("", "x");
    count("x", "x");
}
