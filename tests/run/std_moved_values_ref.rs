// Rust translation of tests/run/std_moved_values.bmb (backlog 2T-11), for
// its expected output (`rustc --edition 2021`).
use std::collections::BTreeMap;

fn main() {
    let mut m: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for i in 0..4u64 {
        let mut v: Vec<u64> = Vec::new();
        for j in 0..i {
            v.push(j);
        }
        match m.insert(10 - i, v) {
            None => println!("new"),
            Some(w) => println!("{}", w.len()),
        }
    }
    match m.insert(9, Vec::new()) {
        None => println!("new"),
        Some(w) => println!("{}", w.len()),
    }
    match m.remove(&8) {
        None => println!("none"),
        Some(w) => println!("{}", w.len()),
    }
    println!("{}", m.len());
    for v in m.into_values() {
        println!("{}", v.len());
    }
}
