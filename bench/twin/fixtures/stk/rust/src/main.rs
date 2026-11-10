//! main: runs the commands of standard input, line by line, on one stack

mod lex;
mod vm;

use std::io::{Read, Write};

/// One token: a command word or an integer to push.
fn step(s: &[u8], lo: usize, hi: usize, st: &mut Vec<i64>) -> u8 {
    if lex::is(s, lo, hi, "add") {
        vm::binary(st, 1)
    } else if lex::is(s, lo, hi, "sub") {
        vm::binary(st, 2)
    } else if lex::is(s, lo, hi, "mul") {
        vm::binary(st, 3)
    } else if lex::is(s, lo, hi, "div") {
        vm::binary(st, 4)
    } else if lex::is(s, lo, hi, "dup") {
        vm::dup(st)
    } else if lex::is(s, lo, hi, "swap") {
        vm::swap(st)
    } else if lex::is(s, lo, hi, "drop") {
        vm::drop(st)
    } else if lex::is(s, lo, hi, "print") {
        vm::print(st)
    } else {
        match std::str::from_utf8(&s[lo..hi]).ok().and_then(|t| t.parse::<i64>().ok()) {
            Some(v) => vm::push(st, v),
            None => 4,
        }
    }
}

/// `error LINE: WHAT` for error `code` at token `s[lo..hi]`.
fn report(line: u64, code: u8, s: &[u8], lo: usize, hi: usize) {
    let what = match code {
        1 => "stack underflow",
        2 => "overflow",
        3 => "division by zero",
        5 => "stack full",
        _ => "",
    };
    let mut out = std::io::stdout().lock();
    if code == 4 {
        write!(out, "error {line}: unknown token ").unwrap();
        out.write_all(&s[lo..hi]).unwrap();
        writeln!(out).unwrap();
    } else {
        writeln!(out, "error {line}: {what}").unwrap();
    }
}

/// Runs the tokens of line `s[lo..hi]`; the first error ends the line.
fn run_line(s: &[u8], lo: usize, hi: usize, st: &mut Vec<i64>, line: u64) {
    let mut i = lo;
    while i < hi {
        if lex::space(s[i]) {
            i += 1;
        } else {
            let e = lex::token_end(s, i, hi);
            let code = step(s, i, e, st);
            if code != 0 {
                report(line, code, s, i, e);
                return;
            }
            i = e;
        }
    }
}

fn main() {
    let mut input = Vec::new();
    if let Err(e) = std::io::stdin().read_to_end(&mut input) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    let mut st = Vec::new();
    let mut line: u64 = 1;
    let mut start = 0;
    for i in 0..input.len() {
        if input[i] == b'\n' {
            run_line(&input, start, i, &mut st, line);
            line = line.saturating_add(1);
            start = i + 1;
        }
    }
    if start < input.len() {
        run_line(&input, start, input.len(), &mut st, line);
    }
    println!("depth {}", st.len());
}
