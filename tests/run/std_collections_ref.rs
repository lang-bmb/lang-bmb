// Rust translation of tests/run/std_collections.bmb (backlog 4B-02), for
// its expected output (`rustc --edition 2021`).
use std::collections::{BTreeMap, BTreeSet};

fn main() {
    let xs: [u64; 10] = [5, 3, 5, 1, 9, 3, 5, 7, 1, 5];
    let mut counts: BTreeMap<u64, u64> = BTreeMap::new();
    let mut seen: BTreeSet<u64> = BTreeSet::new();
    for x in xs {
        let c = counts.get(&x).copied().unwrap_or(0);
        if c < 100 {
            counts.insert(x, c + 1);
        }
        seen.insert(x);
    }
    for v in counts.values() {
        println!("{v}");
    }
    for k in counts.keys() {
        println!("{k}");
    }
    println!("{}", seen.len());
    if seen.contains(&7) {
        println!("has 7");
    }
    if !seen.contains(&2) {
        println!("no 2");
    }
    seen.remove(&5);
    for k in &seen {
        println!("{k}");
    }
}
