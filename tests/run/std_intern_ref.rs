// Rust translation of tests/run/std_intern.bmb (backlog 4B-02 (g)), for its
// expected output (`rustc --edition 2021`): an interner as rustc keeps one,
// a map from text to number and the names by number.
use std::collections::BTreeMap;

#[derive(Default)]
struct Interner {
    ids: BTreeMap<String, usize>,
    names: Vec<String>,
}

impl Interner {
    fn intern(&mut self, s: &str) -> usize {
        if let Some(&id) = self.ids.get(s) {
            return id;
        }
        let id = self.names.len();
        self.ids.insert(s.to_string(), id);
        self.names.push(s.to_string());
        id
    }
}

fn number(s: &str) {
    let mut t = Interner::default();
    if !s.is_empty() {
        for w in s.split(' ') {
            println!("{}", t.intern(w));
        }
    }
    match t.ids.get("let") {
        None => println!("no let"),
        Some(i) => print!("let is {i}\n"),
    }
    match t.ids.get("while") {
        None => println!("no while"),
        Some(j) => println!("{j}"),
    }
    if !t.names.is_empty() && t.names[0] == "fn" {
        println!("0 is fn");
    }
    for n in &t.names {
        println!("{n}");
    }
}

fn main() {
    number("fn main let x let y fn é x");
    number("");
}
