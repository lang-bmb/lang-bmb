// The stage0 side of the JSON value tree: stage0's own `json` and `diag`
// modules, compiled as they are (`rustc --edition 2021`). Each line of
// standard input is parsed: a line stage0 does not accept prints `error: `
// and stage0's message; another prints the value written back, then, when
// there is one, the value as an `i64`, its text as a string, and its
// member `a`. A line ends at a line feed only: a carriage return before it
// belongs to the line, as it does for the program this one is compared with.
#![allow(dead_code)]
#[path = "../../compiler/src/diag.rs"]
mod diag;
#[path = "../../compiler/src/json.rs"]
mod json;
#[path = "../../compiler/src/source.rs"]
mod source;

use std::io::{Read, Write};

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let mut text = String::new();
    stdin.lock().read_to_string(&mut text).unwrap();
    let text = text.strip_suffix('\n').unwrap_or(&text);
    let lines = if text.is_empty() {
        Vec::new()
    } else {
        text.split('\n').collect()
    };
    for line in lines {
        match json::parse(line) {
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
