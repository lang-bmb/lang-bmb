// The stage0 side of the JSON round trip (port map P0, backlog 4D-15):
// stage0's own `json` and `diag` modules, compiled as they are
// (`rustc --edition 2021`). Each line of standard input is parsed and
// written back; a line stage0 does not accept prints `error`.
#![allow(dead_code)]
#[path = "../../compiler/src/diag.rs"]
mod diag;
#[path = "../../compiler/src/json.rs"]
mod json;
#[path = "../../compiler/src/source.rs"]
mod source;

use std::io::{BufRead, Write};

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        match json::parse(&line) {
            Ok(v) => writeln!(out, "{}", v.to_json()).unwrap(),
            Err(_) => writeln!(out, "error").unwrap(),
        }
    }
}
