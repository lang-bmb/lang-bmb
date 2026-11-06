//! The system interface (ADR-0091, backlog 4C-02), through the `bmb` command
//! line: what each built-in checks, proves and does when the program runs.
//! Slice (a): `std::process::exit` and `eprint_str`.

mod common;

use std::process::{Command, Output};

use common::cli;

fn one(text: &str) -> Vec<(String, String)> {
    vec![("s.bmb".to_string(), text.to_string())]
}

/// Builds `text` (fully proved) and runs it with `args` and `input`.
fn run(text: &str, args: &[&str], input: &[u8]) -> Output {
    let out = cli::emit(&one(text), false);
    assert!(out.ok, "{}", out.output);
    let ir = out.ir.expect("IR");
    let dir = cli::fresh_dir();
    let exe = dir.join("p");
    cli::build(&ir, &exe).unwrap();
    let mut child = Command::new(&exe)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(input).unwrap();
    let o = child.wait_with_output().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    o
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

const EXIT: &str = r#"#[summary("exit")]
mod app {
    spec fn small(n: i64) -> bool { 0 <= n && n <= 10 }

    #[effects(io)]
    #[uses(small)]
    fn check(n: i64) -> (r: i64)
        ensures small(r),
    {
        if n < 0 {
            eprint_str("negative input\n");
            std::process::exit(2);
        }
        if n > 10 {
            10
        } else {
            n
        }
    }

    #[effects(io)]
    fn main() -> () {
        print_i64(check(7));
        print_i64(check(42));
        print_i64(check(-1));
        print_i64(99);
    }
}
"#;

/// `exit` ends the process with its code; what was printed before it is
/// flushed, nothing after it runs, and `eprint_str` writes standard error
/// after flushing standard output (the order of the calls).
#[test]
fn exit_ends_the_process_with_its_code() {
    let o = run(EXIT, &[], b"");
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(text(&o.stdout), "7\n10\n");
    assert_eq!(text(&o.stderr), "negative input\n");
    // A program that returns from `main` exits with 0 and needs no runtime.
    let plain = "#[summary(\"p\")]\nmod app {\n    #[effects(io)]\n    fn main() -> () {\n        print_i64(1);\n    }\n}\n";
    let out = cli::emit(&one(plain), false);
    let ir = out.ir.unwrap();
    assert!(!ir.contains("@exit(") && !ir.contains("@stderr"), "{ir}");
    assert_eq!(run(plain, &[], b"").status.code(), Some(0));
}

/// The proof: a path that calls `exit` owes no postcondition and nothing
/// after the call is checked, but what comes before it is.
#[test]
fn exit_ends_the_path_of_the_proof() {
    let out = cli::emit(&one(EXIT), false);
    assert!(out.ok, "{}", out.output);
    assert_eq!(out.checks, 0, "{}", out.output);
    let before = EXIT.replace(
        "            eprint_str(\"negative input\\n\");\n",
        "            eprint_str(\"negative input\\n\");\n            let z: i64 = n - 9223372036854775807 - 9;\n",
    );
    let (ok, diags) = cli::run(&["verify"], &one(&before));
    assert!(!ok && diags.contains("E0803"), "{diags}");
    let after = EXIT.replace(
        "            std::process::exit(2);\n",
        "            std::process::exit(2);\n            let z: i64 = n - 9223372036854775807 - 9;\n",
    );
    let (ok, diags) = cli::run(&["verify"], &one(&after));
    assert!(ok, "code after `exit` is not reached:\n{diags}");
}

/// The surface: `exit` takes an `i32` and has the `io` effect, as Rust's.
#[test]
fn exit_and_eprint_are_io_builtins() {
    let pure = EXIT.replace(
        "    #[effects(io)]\n    #[uses(small)]",
        "    #[effects(pure)]\n    #[uses(small)]",
    );
    let (ok, diags) = cli::run(&["check"], &one(&pure));
    assert!(!ok && diags.contains("E0603"), "{diags}");
    let wide = EXIT.replace("std::process::exit(2);", "std::process::exit(2i64);");
    let (ok, diags) = cli::run(&["check"], &one(&wide));
    assert!(!ok, "an `i64` code is refused:\n{diags}");
}

const ARGS: &str = r#"#[summary("args")]
mod app {
    #[effects(alloc, io, nondet)]
    fn main() -> () {
        let a: Vec<String> = std::env::args();
        print_u64(u64::try_from(a.len()).unwrap());
        for i in 1..a.len() {
            print_str(a[i].as_str());
            print_str("\n");
        }
    }
}
"#;

/// `args()`: the program's name, then each argument as Rust gives it; an
/// argument that is not UTF-8 ends the process with code 101, as Rust's
/// panic in `env::args` does.
#[test]
fn args_are_the_command_line() {
    let o = run(ARGS, &["héllo", "two words", ""], b"");
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(text(&o.stdout), "4\nhéllo\ntwo words\n\n");
    let dir = cli::fresh_dir();
    let exe = dir.join("p");
    cli::build(&cli::emit(&one(ARGS), false).ir.unwrap(), &exe).unwrap();
    use std::os::unix::ffi::OsStrExt;
    let bad = Command::new(&exe)
        .arg(std::ffi::OsStr::from_bytes(b"\xff"))
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(bad.status.code(), Some(101));
    assert_eq!(
        text(&bad.stderr),
        "bmb: a command-line argument is not valid UTF-8\n"
    );
}

/// `nondet` (ADR-0030 §1, sourced by ADR-0091 §3): `args` needs it, the
/// caller must declare it, and it is a reason to test the definition.
#[test]
fn args_need_nondet_and_make_a_test_reason() {
    let missing = ARGS.replace("#[effects(alloc, io, nondet)]", "#[effects(alloc, io)]");
    let (ok, diags) = cli::run(&["check"], &one(&missing));
    assert!(
        !ok && diags.contains("E0606") && diags.contains("`nondet`"),
        "{diags}"
    );
    let (ok, status) = cli::run(&["query", "status", "--"], &one(ARGS));
    assert!(ok, "{status}");
    let row = status
        .lines()
        .find(|l| l.contains("\"path\":\"app::main\""))
        .unwrap();
    assert!(row.contains(r#""test_reasons":["io","nondet"]"#), "{row}");
    // Declared without a source, `nondet` is reported as unneeded (W0909).
    let wide = EXIT.replace(
        "    #[effects(io)]\n    fn main()",
        "    #[effects(io, nondet)]\n    fn main()",
    );
    let (_, lint) = cli::run(&["lint"], &one(&wide));
    assert!(lint.contains("W0909") && lint.contains("nondet"), "{lint}");
}

/// Reads up to five lines, printing each result, then the rest of the
/// input's length: the same program in Rust is `LINES_REF`.
const LINES: &str = r#"#[summary("lines")]
mod app {
    #[effects(alloc, io, nondet)]
    fn main() -> () {
        let mut line: String = String::new();
        for k in 0..5 {
            match std::io::read_line(&mut line) {
                Ok(n) => {
                    print_u64(u64::try_from(n).unwrap());
                    print_u64(u64::try_from(line.len()).unwrap());
                }
                Err(e) => {
                    print_i64(i64::try_from(e.code).unwrap());
                    print_str(e.message.as_str());
                    print_str("\n");
                    print_u64(u64::try_from(line.len()).unwrap());
                }
            }
        }
        let mut rest: Vec<u8> = Vec::new();
        match std::io::read_to_end(&mut rest) {
            Ok(got) => {
                print_u64(u64::try_from(got).unwrap());
            }
            Err(why) => {
                print_str(why.message.as_str());
                print_str("\n");
            }
        }
    }
}
"#;

const LINES_REF: &str = r#"use std::io::{BufRead, Read};
fn main() {
    let stdin = std::io::stdin();
    let mut h = stdin.lock();
    let mut line = String::new();
    for _ in 0..5 {
        match h.read_line(&mut line) {
            Ok(n) => {
                println!("{n}");
                println!("{}", line.len());
            }
            Err(e) => {
                println!("{}", e.raw_os_error().unwrap_or(0));
                println!("{e}");
                println!("{}", line.len());
            }
        }
    }
    let mut rest = Vec::new();
    match h.read_to_end(&mut rest) {
        Ok(n) => println!("{n}"),
        Err(e) => println!("{e}"),
    }
}
"#;

/// Standard input (ADR-0091 §2): line by line and to the end, with the
/// errors Rust gives (a line that is not UTF-8 is taken back out of the
/// buffer; a closed standard input is the OS error), byte for byte against
/// the Rust program built by rustc.
#[test]
fn input_reads_as_rust_reads() {
    let dir = cli::fresh_dir();
    let rs = dir.join("r.rs");
    std::fs::write(&rs, LINES_REF).unwrap();
    let rexe = dir.join("r");
    let built = Command::new("rustc")
        .args(["-O", "-o"])
        .arg(&rexe)
        .arg(&rs)
        .output()
        .unwrap();
    assert!(built.status.success(), "{}", text(&built.stderr));
    let bexe = dir.join("b");
    cli::build(&cli::emit(&one(LINES), false).ir.unwrap(), &bexe).unwrap();
    let feed = |exe: &std::path::Path, input: Option<&[u8]>| -> Output {
        let mut c = Command::new(exe);
        match input {
            Some(i) => {
                let mut child = c
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .unwrap();
                use std::io::Write;
                child.stdin.take().unwrap().write_all(i).unwrap();
                child.wait_with_output().unwrap()
            }
            // A closed standard input (EBADF, which Rust reads as empty)
            // and a directory (EISDIR, an OS error).
            None => {
                let sh = format!("{e} <&-; echo; {e} < /", e = exe.display());
                Command::new("sh").args(["-c", &sh]).output().unwrap()
            }
        }
    };
    for input in [
        Some(&b"ab\n\xff\ncd\nlast"[..]),
        Some(&b""[..]),
        Some(&b"h\xc3\xa9llo\n\n\nx\ny\nrest of it\nmore"[..]),
        None,
    ] {
        let r = feed(&rexe, input);
        let b = feed(&bexe, input);
        assert_eq!(text(&b.stdout), text(&r.stdout), "input {input:?}");
        assert_eq!(b.status.code(), r.status.code(), "input {input:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
