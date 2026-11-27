// The stage0 side of the JSON value tree: stage0's own `json` and `diag`
// modules, compiled as they are (`rustc --edition 2021`). Each line of
// standard input is parsed: a line stage0 does not accept prints `error: `
// and stage0's message; another prints the value written back, then, when
// there is one, the value as an `i64`, its text as a string, and its
// member `a`.
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
            Ok(v) => {
                writeln!(out, "{}", v.to_json()).unwrap();
                if let Some(n) = v.as_i64() {
                    writeln!(out, "  i64 {n}").unwrap();
                }
                if let Some(s) = v.as_str() {
                    writeln!(out, "  str {}", diag::json_str(s)).unwrap();
                }
                if let Some(a) = v.get("a") {
                    writeln!(out, "  a {}", a.to_json()).unwrap();
                }
            }
            Err(e) => writeln!(out, "error: {e}").unwrap(),
        }
    }
}
