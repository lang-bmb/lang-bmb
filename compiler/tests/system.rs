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

/// Reads the file named by the first argument as bytes and as text, reads
/// a path with a NUL byte, writes the second argument's file and reads it
/// back: the same program in Rust is `FILES_REF`.
const FILES: &str = r#"#[summary("files")]
mod app {
    #[effects(alloc, io, nondet)]
    fn main() -> () {
        let a: Vec<String> = std::env::args();
        if a.len() < 3 {
            std::process::exit(2);
        }
        match std::fs::read(a[1].as_str()) {
            Ok(b) => {
                print_u64(u64::try_from(b.len()).unwrap());
            }
            Err(e) => {
                print_i64(i64::try_from(e.code).unwrap());
                print_str(e.message.as_str());
                print_str("\n");
            }
        }
        match std::fs::read_to_string(a[1].as_str()) {
            Ok(t) => {
                print_str(t.as_str());
                print_str("|\n");
            }
            Err(e2) => {
                print_i64(i64::try_from(e2.code).unwrap());
                print_str(e2.message.as_str());
                print_str("\n");
            }
        }
        match std::fs::read("a\0b") {
            Ok(b2) => {
                print_str("read a NUL path\n");
            }
            Err(e3) => {
                print_i64(i64::try_from(e3.code).unwrap());
                print_str(e3.message.as_str());
                print_str("\n");
            }
        }
        match std::fs::write(a[2].as_str(), b"one\ntwo\n") {
            Ok(u) => {
                print_str("written\n");
            }
            Err(e4) => {
                print_i64(i64::try_from(e4.code).unwrap());
                print_str(e4.message.as_str());
                print_str("\n");
            }
        }
        match std::fs::read_to_string(a[2].as_str()) {
            Ok(t2) => {
                print_str(t2.as_str());
            }
            Err(e5) => {
                print_str(e5.message.as_str());
                print_str("\n");
            }
        }
    }
}
"#;

const FILES_REF: &str = r#"fn err(e: std::io::Error) {
    println!("{}", e.raw_os_error().unwrap_or(0));
    println!("{e}");
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    match std::fs::read(&a[1]) {
        Ok(b) => println!("{}", b.len()),
        Err(e) => err(e),
    }
    match std::fs::read_to_string(&a[1]) {
        Ok(t) => println!("{t}|"),
        Err(e) => err(e),
    }
    match std::fs::read("a\0b") {
        Ok(_) => println!("read a NUL path"),
        Err(e) => err(e),
    }
    match std::fs::write(&a[2], b"one\ntwo\n") {
        Ok(()) => println!("written"),
        Err(e) => err(e),
    }
    match std::fs::read_to_string(&a[2]) {
        Ok(t) => print!("{t}"),
        Err(e) => println!("{e}"),
    }
}
"#;

/// Files (ADR-0091 §2, slice (b)), byte for byte against the Rust program
/// built by rustc: a file, text that is not UTF-8, a missing file, a
/// directory, a path with a NUL byte, writing into a missing directory.
#[test]
fn files_read_and_write_as_rust_does() {
    let dir = cli::fresh_dir();
    let rs = dir.join("r.rs");
    std::fs::write(&rs, FILES_REF).unwrap();
    let rexe = dir.join("r");
    let built = Command::new("rustc")
        .args(["-O", "-o"])
        .arg(&rexe)
        .arg(&rs)
        .output()
        .unwrap();
    assert!(built.status.success(), "{}", text(&built.stderr));
    let bexe = dir.join("b");
    let out = cli::emit(&one(FILES), false);
    assert!(out.ok, "{}", out.output);
    cli::build(&out.ir.unwrap(), &bexe).unwrap();
    std::fs::write(dir.join("text"), "héllo\nworld").unwrap();
    std::fs::write(dir.join("bytes"), b"ok\xff\xfe").unwrap();
    let d = |n: &str| dir.join(n).display().to_string();
    for (read, write) in [
        (d("text"), d("w1")),
        (d("bytes"), d("w2")),
        (d("missing"), d("nodir/w3")),
        (dir.display().to_string(), d("w4")),
    ] {
        let r = Command::new(&rexe).args([&read, &write]).output().unwrap();
        let b = Command::new(&bexe).args([&read, &write]).output().unwrap();
        assert_eq!(text(&b.stdout), text(&r.stdout), "{read} {write}");
        assert_eq!(b.status.code(), r.status.code());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// In the directory named by the first argument: makes a nested tree, makes
/// it again, lists it, replaces a file atomically (a temporary, then a
/// rename), removes files, and tries what fails. The same program in Rust
/// is `DIRS_REF`.
const DIRS: &str = r#"#[summary("dirs")]
mod app {
    #[effects(alloc, io)]
    fn show(r: Result<(), std::io::Error>) -> () {
        match r {
            Ok(u) => {
                print_str("ok\n");
            }
            Err(e) => {
                print_i64(i64::try_from(e.code).unwrap());
                print_str(e.message.as_str());
                print_str("\n");
            }
        }
    }

    #[effects(alloc, io, nondet)]
    fn list(at: &str) -> () {
        match std::fs::read_dir(at) {
            Ok(names) => {
                for i in 0..names.len() {
                    print_str(names[i].as_str());
                    print_str(" ");
                }
                print_str("\n");
            }
            Err(e) => {
                print_i64(i64::try_from(e.code).unwrap());
                print_str(e.message.as_str());
                print_str("\n");
            }
        }
    }

    #[effects(alloc, io, nondet)]
    fn main() -> () {
        let a: Vec<String> = std::env::args();
        if a.len() < 6 {
            std::process::exit(2);
        }
        show(std::fs::create_dir_all(a[2].as_str()));
        show(std::fs::create_dir_all(a[2].as_str()));
        show(std::fs::write(a[3].as_str(), b"first\n"));
        show(std::fs::write(a[4].as_str(), b"second\n"));
        show(std::fs::rename(a[4].as_str(), a[3].as_str()));
        list(a[1].as_str());
        match std::fs::read_to_string(a[3].as_str()) {
            Ok(t) => {
                print_str(t.as_str());
            }
            Err(e) => {
                print_str(e.message.as_str());
            }
        }
        show(std::fs::create_dir_all(a[5].as_str()));
        show(std::fs::remove_file(a[3].as_str()));
        show(std::fs::remove_file(a[3].as_str()));
        show(std::fs::rename(a[3].as_str(), a[4].as_str()));
        list(a[3].as_str());
        list(a[2].as_str());
    }
}
"#;

const DIRS_REF: &str = r#"fn show(r: std::io::Result<()>) {
    match r {
        Ok(()) => println!("ok"),
        Err(e) => {
            println!("{}", e.raw_os_error().unwrap_or(0));
            println!("{e}");
        }
    }
}
fn list(at: &str) {
    match std::fs::read_dir(at) {
        Ok(rd) => {
            let mut names: Vec<String> =
                rd.map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
            names.sort();
            for n in names {
                print!("{n} ");
            }
            println!();
        }
        Err(e) => {
            println!("{}", e.raw_os_error().unwrap_or(0));
            println!("{e}");
        }
    }
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    show(std::fs::create_dir_all(&a[2]));
    show(std::fs::create_dir_all(&a[2]));
    show(std::fs::write(&a[3], b"first\n"));
    show(std::fs::write(&a[4], b"second\n"));
    show(std::fs::rename(&a[4], &a[3]));
    list(&a[1]);
    match std::fs::read_to_string(&a[3]) {
        Ok(t) => print!("{t}"),
        Err(e) => print!("{e}"),
    }
    show(std::fs::create_dir_all(&a[5]));
    show(std::fs::remove_file(&a[3]));
    show(std::fs::remove_file(&a[3]));
    show(std::fs::rename(&a[3], &a[4]));
    list(&a[3]);
    list(&a[2]);
}
"#;

/// Directories and an atomic replace (ADR-0091 §2, slice (c)), byte for
/// byte against the Rust program built by rustc: nested and repeated
/// `create_dir_all`, a file in the way, sorted listings, `rename` over a
/// file, removing a missing file, listing a missing directory.
#[test]
fn directories_behave_as_rust() {
    let dir = cli::fresh_dir();
    let rs = dir.join("r.rs");
    std::fs::write(&rs, DIRS_REF).unwrap();
    let rexe = dir.join("r");
    let built = Command::new("rustc")
        .args(["-O", "-o"])
        .arg(&rexe)
        .arg(&rs)
        .output()
        .unwrap();
    assert!(built.status.success(), "{}", text(&built.stderr));
    let bexe = dir.join("b");
    let out = cli::emit(&one(DIRS), false);
    assert!(out.ok, "{}", out.output);
    cli::build(&out.ir.unwrap(), &bexe).unwrap();
    for (k, exe) in [(0, &rexe), (1, &bexe)] {
        let root = dir.join(format!("t{k}"));
        std::fs::create_dir_all(&root).unwrap();
        let p = |n: &str| root.join(n).display().to_string();
        let o = Command::new(exe)
            .args([
                root.display().to_string(),
                p("a/b//c/"),
                p("f.txt"),
                p("f.tmp"),
                p("f.txt/inside"),
            ])
            .output()
            .unwrap();
        std::fs::write(dir.join(format!("out{k}")), &o.stdout).unwrap();
    }
    let r = std::fs::read(dir.join("out0")).unwrap();
    let b = std::fs::read(dir.join("out1")).unwrap();
    assert_eq!(text(&b), text(&r));
    let _ = std::fs::remove_dir_all(&dir);
}

const ENV: &str = r#"#[summary("env")]
mod app {
    #[effects(alloc, io, nondet)]
    fn show(name: &str) -> () {
        match std::env::var(name) {
            None => {
                print_str("none\n");
            }
            Some(v) => {
                print_str("some ");
                print_str(v.as_str());
                print_str("\n");
            }
        }
    }

    #[effects(alloc, io, nondet)]
    fn main() -> () {
        show("BMB_TEST_SET");
        show("BMB_TEST_EMPTY");
        show("BMB_TEST_UNSET");
        show("BMB_TEST_BYTES");
        show("A\0B");
        let t0: u64 = std::time::monotonic_ms();
        let t1: u64 = std::time::monotonic_ms();
        if t1 >= t0 {
            print_str("monotonic\n");
        }
    }
}
"#;

/// The environment and the clock (ADR-0091 §2, slice (d)): a set, empty,
/// unset and non-UTF-8 variable, a name with a NUL byte, and a clock that
/// never goes back.
#[test]
fn environment_and_clock() {
    let out = cli::emit(&one(ENV), false);
    assert!(out.ok, "{}", out.output);
    let dir = cli::fresh_dir();
    let exe = dir.join("p");
    cli::build(&out.ir.unwrap(), &exe).unwrap();
    use std::os::unix::ffi::OsStrExt;
    let o = Command::new(&exe)
        .env("BMB_TEST_SET", "héllo")
        .env("BMB_TEST_EMPTY", "")
        .env_remove("BMB_TEST_UNSET")
        .env("BMB_TEST_BYTES", std::ffi::OsStr::from_bytes(b"\xff"))
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        text(&o.stdout),
        "some héllo\nsome \nnone\nnone\nnone\nmonotonic\n"
    );
    // The values are from outside: a definition reading one needs `nondet`.
    let missing = ENV.replacen("#[effects(alloc, io, nondet)]", "#[effects(alloc, io)]", 1);
    let (ok, diags) = cli::run(&["check"], &one(&missing));
    assert!(!ok && diags.contains("E0606"), "{diags}");
}

/// Runs `sh -c <first argument>` with this program's own standard input as
/// the child's input, then prints the status, the child's output length
/// and bytes, and its error output.
const RUN: &str = r#"#[summary("run")]
mod app {
    #[effects(alloc, io, nondet)]
    fn main() -> () {
        let a: Vec<String> = std::env::args();
        if a.len() < 3 {
            std::process::exit(2);
        }
        let mut input: Vec<u8> = Vec::new();
        match std::io::read_to_end(&mut input) {
            Ok(n) => {}
            Err(e) => {
                std::process::exit(3);
            }
        }
        let mut sh: Vec<String> = Vec::new();
        sh.push(String::from("-c"));
        sh.push(String::from(a[2].as_str()));
        match std::process::run(a[1].as_str(), sh.as_slice(), input.as_slice()) {
            Ok(o) => {
                print_i64(i64::try_from(o.status).unwrap());
                print_u64(u64::try_from(o.stdout.len()).unwrap());
                eprint_str("[");
                print_bytes(o.stderr.as_slice());
                eprint_str("]");
                std::process::exit(0);
            }
            Err(e2) => {
                print_i64(i64::try_from(e2.code).unwrap());
                print_str(e2.message.as_str());
                print_str("\n");
            }
        }
    }
}
"#;

/// Processes (ADR-0091 §2, slice (e)): a megabyte through `cat` both ways
/// (the input is written while the output is read, so neither blocks), a
/// child that stops reading early, one ended by a signal and a program that
/// does not exist (a NUL byte in an argument is the path rule, tested with
/// the files: a process argument cannot carry one).
#[test]
fn processes_run_with_pipes() {
    let out = cli::emit(&one(RUN), false);
    assert!(out.ok, "{}", out.output);
    let dir = cli::fresh_dir();
    let exe = dir.join("p");
    cli::build(&out.ir.unwrap(), &exe).unwrap();
    let feed = |prog: &str, script: &str, input: &[u8]| -> String {
        let mut child = Command::new(&exe)
            .args([prog, script])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        child.stdin.take().unwrap().write_all(input).unwrap();
        let o = child.wait_with_output().unwrap();
        text(&o.stdout)
    };
    let big: Vec<u8> = (0..1u32 << 20).map(|i| (i % 251) as u8).collect();
    assert_eq!(
        feed("sh", "cat; cat >&2 </dev/null", &big),
        format!("0\n{}\n", big.len())
    );
    assert_eq!(feed("sh", "head -c 10 >/dev/null; exit 4", &big), "4\n0\n");
    assert_eq!(feed("sh", "kill -9 $$", b""), "137\n0\n");
    assert_eq!(
        feed("no-such-program-bmb", "x", b""),
        "2\nNo such file or directory (os error 2)\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

const SOLVER: &str = r#"#[summary("ask z3")]
mod app {
    #[effects(alloc, io, nondet)]
    fn main() -> () {
        let mut args: Vec<String> = Vec::new();
        args.push(String::from("-in"));
        match std::process::run("z3", args.as_slice(), b"(declare-const x Int)\n(assert (> (* x x) 10))\n(check-sat)\n(assert (< x 0))\n(assert (> x 0))\n(check-sat)\n") {
            Ok(o) => {
                print_i64(i64::try_from(o.status).unwrap());
                print_bytes(o.stdout.as_slice());
            }
            Err(e) => {
                print_str(e.message.as_str());
                print_str("\n");
            }
        }
        let mut v: Vec<String> = Vec::new();
        v.push(String::from("--version"));
        match std::process::run("clang", v.as_slice(), b"") {
            Ok(c) => {
                print_i64(i64::try_from(c.status).unwrap());
            }
            Err(e2) => {
                print_str(e2.message.as_str());
                print_str("\n");
            }
        }
    }
}
"#;

/// The compiler's own outside processes (ADR-0091, 4C-02's criterion): a BMB
/// program starts the pinned Z3, sends it a script and reads its answers,
/// and runs `clang --version`.
#[test]
fn a_program_asks_z3_and_clang() {
    let o = run(SOLVER, &[], b"");
    assert_eq!(text(&o.stdout), "0\nsat\nunsat\n0\n", "{}", text(&o.stderr));
}

const DEEP: &str = r#"#[summary("deep recursion")]
mod app {
    spec fn bounded(n: u64, r: u64) -> bool { r <= n }

    #[effects(pure)]
    #[uses(bounded)]
    fn depth(n: u64) -> (r: u64)
        ensures bounded(n, r),
        decreases n,
    {
        if n == 0 {
            0
        } else {
            depth(n - 1) + 1
        }
    }

    #[effects(io)]
    fn main() -> () {
        print_u64(depth(1000000));
    }
}
"#;

/// The stack (ADR-0091 §5, slice (f)): a program with recursion runs its
/// `main` on a 256 MiB thread. A million nested calls (unoptimized, so no
/// call becomes a jump) need more than the default 8 MiB; a program
/// without recursion keeps the plain `main`.
#[test]
fn recursion_runs_on_a_large_stack() {
    let out = cli::emit(&one(DEEP), false);
    assert!(out.ok, "{}", out.output);
    let ir = out.ir.unwrap();
    assert!(ir.contains("call void @bmb_big_stack()"), "{ir}");
    let dir = cli::fresh_dir();
    let ll = dir.join("p.ll");
    std::fs::write(&ll, &ir).unwrap();
    let exe = dir.join("p");
    let built = Command::new("clang")
        .args(["-O0", "-Wno-override-module", "-o"])
        .arg(&exe)
        .arg(&ll)
        .output()
        .unwrap();
    assert!(built.status.success(), "{}", text(&built.stderr));
    let o = Command::new(&exe).output().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(text(&o.stdout), "1000000\n");
    let flat = cli::emit(&one(EXIT), false).ir.unwrap();
    assert!(!flat.contains("pthread"), "{flat}");
}

/// Builds a tool of `tests/tools` (fully proved, no check left).
fn tool(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/tools")
        .join(format!("{name}.bmb"));
    let out = cli::emit(&one(&std::fs::read_to_string(src).unwrap()), false);
    assert!(out.ok, "{}", out.output);
    assert_eq!(out.checks, 0, "{name}: {}", out.output);
    let dir = cli::fresh_dir();
    let exe = dir.join(name);
    cli::build(&out.ir.unwrap(), &exe).unwrap();
    (dir, exe)
}

/// A tool with input and output (HEALTH.md §5 after 4C): `wc` against
/// GNU `wc` on standard input.
#[test]
fn the_wc_tool_counts_as_wc_does() {
    let (dir, exe) = tool("wc");
    let big: Vec<u8> = (0..1u32 << 20).map(|i| (i % 256) as u8).collect();
    for input in [
        &b""[..],
        &b"a b\n"[..],
        &b"  h\xc3\xa9llo\tworld\n\nlast line without newline"[..],
        &b"\x0b\x0c\r \n\n\n"[..],
        &big[..],
    ] {
        let ours = Command::new(&exe)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut c| {
                use std::io::Write;
                c.stdin.take().unwrap().write_all(input)?;
                c.wait_with_output()
            })
            .unwrap();
        let theirs = Command::new("sh")
            .args(["-c", "LC_ALL=C wc -l -w -c"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut c| {
                use std::io::Write;
                c.stdin.take().unwrap().write_all(input)?;
                c.wait_with_output()
            })
            .unwrap();
        let ours: Vec<String> = text(&ours.stdout)
            .split_whitespace()
            .map(String::from)
            .collect();
        let theirs: Vec<String> = text(&theirs.stdout)
            .split_whitespace()
            .map(String::from)
            .collect();
        assert_eq!(ours, theirs, "input of {} bytes", input.len());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// The second tool: `fgrep PATTERN FILE` against `grep -F` (output and
/// exit codes: 0 found, 1 none, 2 an error).
#[test]
fn the_fgrep_tool_finds_as_grep_does() {
    let (dir, exe) = tool("fgrep");
    let file = dir.join("in.txt");
    std::fs::write(
        &file,
        "alpha beta\n\ngamma alphabet\nbeta\nno newline alpha",
    )
    .unwrap();
    let f = file.display().to_string();
    let missing = dir.join("missing").display().to_string();
    for (pat, path) in [
        ("alpha", f.as_str()),
        ("beta", f.as_str()),
        ("zeta", f.as_str()),
        ("", f.as_str()),
        ("a much longer pattern than any line in it", f.as_str()),
        ("alpha", missing.as_str()),
    ] {
        let ours = Command::new(&exe).args([pat, path]).output().unwrap();
        let theirs = Command::new("grep")
            .args(["-F", "--", pat, path])
            .output()
            .unwrap();
        assert_eq!(text(&ours.stdout), text(&theirs.stdout), "{pat:?}");
        assert_eq!(ours.status.code(), theirs.status.code(), "{pat:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

const LOOP: &str = r#"#[summary("number lines")]
mod app {
    #[effects(alloc, io, nondet)]
    fn number() -> () {
        let mut count: u64 = 0;
        loop {
            let mut line: String = String::new();
            match std::io::read_line(&mut line) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    count = count.saturating_add(1);
                    print_u64(count);
                    print_str(line.as_str());
                }
                Err(e) => {
                    std::process::exit(1);
                }
            }
        }
    }

    #[effects(alloc, io, nondet)]
    fn main() -> () {
        number();
    }
}
"#;

/// A loop that runs until the input ends (ADR-0094): allowed without
/// `decreases` in a `nondet` function, proved without termination, and
/// marked (`termination: unproved`, the `nonterminating` test reason) on it
/// and on its callers; elsewhere `loop` still needs a measure (E0506).
#[test]
fn input_loops_run_until_the_end() {
    let o = run(LOOP, &[], b"a\nb\nlast");
    assert_eq!(text(&o.stdout), "1\na\n2\nb\n3\nlast");
    let (ok, status) = cli::run(&["query", "status", "--"], &one(LOOP));
    assert!(ok, "{status}");
    for def in ["app::number", "app::main"] {
        let row = status
            .lines()
            .find(|l| l.contains(&format!("\"path\":\"{def}\"")))
            .unwrap();
        assert!(row.contains(r#""termination":"unproved""#), "{row}");
        assert!(row.contains("\"nonterminating\""), "{row}");
    }
    let plain = cli::run(&["query", "status", "--"], &one(EXIT)).1;
    assert!(!plain.contains("termination"), "{plain}");
    let not_nondet = LOOP.replacen(
        "#[effects(alloc, io, nondet)]\n    fn number()",
        "#[effects(alloc, io)]\n    fn number()",
        1,
    );
    let (ok, diags) = cli::run(&["check"], &one(&not_nondet));
    assert!(!ok && diags.contains("E0506"), "{diags}");
}

const DIALOG: &str = r#"#[summary("talk to z3")]
mod app {
    // The line Z3 answers `q` with.
    #[effects(alloc, io, nondet)]
    fn ask(c: &mut std::process::Child, q: &str) -> String {
        let mut line: String = String::new();
        match c.write_all(q.as_bytes()) {
            Ok(u) => {}
            Err(e) => {
                eprint_str(e.message.as_str());
                return line;
            }
        }
        match c.read_line(&mut line) {
            Ok(n) => {}
            Err(f) => eprint_str(f.message.as_str()),
        }
        line
    }

    #[effects(alloc, io, nondet)]
    fn main() -> () {
        let mut args: Vec<String> = Vec::new();
        args.push(String::from("-in"));
        match std::process::spawn("z3", args.as_slice(), std::process::Stdio::Inherit) {
            Ok(child) => {
                let mut c: std::process::Child = child;
                let a: String = ask(&mut c, "(declare-const x Int)\n(assert (> (* x x) 10))\n(check-sat)\n");
                print_str(a.as_str());
                let b: String = ask(&mut c, "(assert (< x 0))\n(assert (> x 0))\n(check-sat)\n");
                print_str(b.as_str());
                let d: String = ask(&mut c, "(reset)\n(declare-const y Int)\n(assert (= (* 2 y) 7))\n(check-sat)\n");
                print_str(d.as_str());
                match c.write_all(b"(exit)\n") {
                    Ok(u) => {}
                    Err(e) => {}
                }
                match c.wait() {
                    Ok(s) => print_i64(i64::try_from(s).unwrap()),
                    Err(f) => eprint_str(f.message.as_str()),
                }
            }
            Err(g) => {
                print_str(g.message.as_str());
                print_str("\n");
            }
        }
    }
}
"#;

/// The rest of slice (e) (ADR-0091 §2): a BMB program keeps the pinned Z3
/// running and asks it three queries in turn, reading each answer before
/// it writes the next (what stage0's solver connection does), then waits.
#[test]
fn a_program_talks_to_z3() {
    let o = run(DIALOG, &[], b"");
    assert_eq!(
        text(&o.stdout),
        "sat\nunsat\nunsat\n0\n",
        "{}",
        text(&o.stderr)
    );
}

const CHILDREN: &str = r#"#[summary("children")]
mod app {
    #[effects(alloc)]
    fn sh(script: &str) -> Vec<String> {
        let mut a: Vec<String> = Vec::new();
        a.push(String::from("-c"));
        a.push(String::from(script));
        a
    }

    #[effects(alloc, io, nondet)]
    fn line(c: &mut std::process::Child) -> () {
        let mut l: String = String::new();
        match c.read_line(&mut l) {
            Ok(n) => {
                print_u64(u64::try_from(n).unwrap());
                print_str(l.as_str());
            }
            Err(e) => {
                print_str(e.message.as_str());
                print_str("\n");
            }
        }
    }

    #[effects(alloc, io, nondet)]
    fn status(c: std::process::Child) -> () {
        match c.wait() {
            Ok(s) => print_i64(i64::try_from(s).unwrap()),
            Err(e) => {
                print_str(e.message.as_str());
                print_str("\n");
            }
        }
    }

    #[effects(alloc, io, nondet)]
    fn say(c: &mut std::process::Child, data: &[u8]) -> () {
        match c.write_all(data) {
            Ok(u) => print_str("written\n"),
            Err(e) => {
                print_str(e.message.as_str());
                print_str("\n");
            }
        }
    }

    // Each case starts its own child; a failed start prints its error.
    #[effects(alloc, io, nondet)]
    fn case(k: u64, prog: &str, args: &[String]) -> () {
        match std::process::spawn(prog, args, std::process::Stdio::Inherit) {
            Ok(child) => {
                let mut c: std::process::Child = child;
                if k == 0 {
                    // An echo: each line comes back before the next is sent.
                    say(&mut c, b"one\n");
                    line(&mut c);
                    say(&mut c, b"two\n");
                    line(&mut c);
                    status(c);
                } else if k == 1 {
                    // The end of its output, then `Ok(0)` again.
                    line(&mut c);
                    line(&mut c);
                    line(&mut c);
                    status(c);
                } else if k == 2 {
                    // Dropped without `wait`: its input closes, so `cat`
                    // ends; this process does not wait for it.
                    if c.id() > 0 {
                        print_str("started\n");
                    }
                } else if k == 3 {
                    status(c);
                } else {
                    // More than a pipe holds, to a child that reads nothing:
                    // the write fails when it is gone.
                    let mut big: Vec<u8> = Vec::new();
                    for i in 0..1048576
                        invariant big.len() == i,
                    {
                        big.push(120);
                    }
                    say(&mut c, big.as_slice());
                    status(c);
                }
            }
            Err(e) => {
                print_str(e.message.as_str());
                print_str("\n");
            }
        }
    }

    #[effects(alloc, io, nondet)]
    fn main() -> () {
        let none: Vec<String> = Vec::new();
        case(0, "cat", none.as_slice());
        let s1: Vec<String> = sh("printf 'a\\nb'");
        case(1, "sh", s1.as_slice());
        case(2, "cat", none.as_slice());
        let s3: Vec<String> = sh("kill -9 $$");
        case(3, "sh", s3.as_slice());
        let s4: Vec<String> = sh("exit 3");
        case(4, "sh", s4.as_slice());
        case(5, "no-such-program-bmb", none.as_slice());
    }
}
"#;

/// A `Child`'s methods (ADR-0091 §2) behave as Rust's do with piped
/// `stdin` and `stdout`: an echo line by line, the end of the output
/// (`Ok(0)`, the last line without its newline), a child dropped without
/// `wait` (its pipes close and nothing waits: the program ends), the
/// status of a child killed by a signal (128 + 9), a write to a child that
/// has exited (`EPIPE`, Rust's message; `SIGPIPE` is ignored as in Rust)
/// and a program that does not exist (`ENOENT`). The program is fully
/// proved and, under valgrind where it is installed, leaks nothing.
#[test]
fn children_talk_through_pipes() {
    let o = run(CHILDREN, &[], b"");
    let want = "written\n4\none\nwritten\n4\ntwo\n0\n2\na\n1\nb0\n0\nstarted\n137\nBroken pipe (os error 32)\n3\nNo such file or directory (os error 2)\n";
    // Under valgrind: every `Vec` and both `FILE`s of each child freed,
    // the dropped one's too (definite and indirect leaks are errors).
    if Command::new("valgrind").arg("--version").output().is_ok() {
        let out = cli::emit(&one(CHILDREN), false);
        let dir = cli::fresh_dir();
        let exe = dir.join("p");
        cli::build(&out.ir.expect("IR"), &exe).unwrap();
        let v = Command::new("valgrind")
            .args([
                "-q",
                "--leak-check=full",
                "--errors-for-leak-kinds=definite,indirect",
                "--error-exitcode=9",
            ])
            .arg(&exe)
            .output()
            .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(v.status.success(), "{}", text(&v.stderr));
        // Valgrind runs `posix_spawnp` as a fork that cannot report the
        // failed `exec` back: the missing program starts and exits 127
        // there. The cases before it are compared.
        let before = want.trim_end_matches("No such file or directory (os error 2)\n");
        assert!(text(&v.stdout).starts_with(before), "{}", text(&v.stdout));
    }
    assert_eq!(
        text(&o.stdout),
        "written\n4\none\nwritten\n4\ntwo\n0\n2\na\n1\nb0\n0\nstarted\n137\nBroken pipe (os error 32)\n3\nNo such file or directory (os error 2)\n",
        "{}",
        text(&o.stderr)
    );
}

const META: &str = r#"#[summary("metadata")]
mod app {
    #[effects(alloc)]
    fn at(dir: &str, name: &str) -> String {
        let mut p: String = String::from(dir);
        if dir.len() < 4096 && name.len() < 4096 {
            p.push_str("/");
            p.push_str(name);
        }
        p
    }

    #[effects(io)]
    fn flag(b: bool) -> () {
        if b {
            print_str("1");
        } else {
            print_str("0");
        }
    }

    #[effects(alloc, io)]
    fn show(r: Result<std::fs::Metadata, std::io::Error>) -> () {
        match r {
            Ok(m) => {
                flag(m.is_dir);
                flag(m.is_file);
                flag(m.is_symlink);
                print_str(" ");
                print_u64(m.len);
            }
            Err(e) => {
                print_str(e.message.as_str());
                print_str("\n");
            }
        }
    }

    #[effects(alloc, io, nondet)]
    fn both(dir: &str, name: &str) -> () {
        let p: String = at(dir, name);
        show(std::fs::metadata(p.as_str()));
        show(std::fs::symlink_metadata(p.as_str()));
    }

    #[effects(alloc, io)]
    fn wrote(r: Result<(), std::io::Error>) -> () {
        match r {
            Ok(u) => print_str("ok\n"),
            Err(e) => {
                print_str(e.message.as_str());
                print_str("\n");
            }
        }
    }

    #[effects(alloc, io, nondet)]
    fn main() -> () {
        let a: Vec<String> = std::env::args();
        if a.len() < 2 {
            return;
        }
        both(a[1].as_str(), "f");
        both(a[1].as_str(), "l");
        both(a[1].as_str(), "d");
        both(a[1].as_str(), "missing");
        let n: String = at(a[1].as_str(), "new");
        wrote(std::fs::write_new(n.as_str(), b"fresh"));
        wrote(std::fs::write_new(n.as_str(), b"again"));
        let l: String = at(a[1].as_str(), "l");
        wrote(std::fs::write_new(l.as_str(), b"through the link"));
    }
}
"#;

/// The rest of stage0's outside contacts (backlog 4C-06, `dev-docs/health/4C.md`):
/// `metadata` (follows a link) and `symlink_metadata` (does not) tell the
/// same as Rust's `std::fs` on a file, a link to it, a directory and a
/// missing path; `write_new` writes a new file, refuses an existing one and
/// does not follow a planted link (Rust's `File::create_new`).
#[test]
fn metadata_and_new_files_are_rusts() {
    let dir = cli::fresh_dir();
    std::fs::write(dir.join("f"), b"hello").unwrap();
    std::os::unix::fs::symlink(dir.join("f"), dir.join("l")).unwrap();
    std::fs::create_dir(dir.join("d")).unwrap();
    let mut want = String::new();
    for name in ["f", "l", "d", "missing"] {
        let p = dir.join(name);
        for m in [std::fs::metadata(&p), std::fs::symlink_metadata(&p)] {
            match m {
                Ok(m) => {
                    let b = |x: bool| if x { "1" } else { "0" };
                    want.push_str(&format!(
                        "{}{}{} {}\n",
                        b(m.is_dir()),
                        b(m.is_file()),
                        b(m.file_type().is_symlink()),
                        m.len()
                    ));
                }
                Err(e) => want.push_str(&format!("{e}\n")),
            }
        }
    }
    // Rust's `File::create_new` on the same three paths, in a copy.
    let twin = cli::fresh_dir();
    std::fs::write(twin.join("f"), b"hello").unwrap();
    std::os::unix::fs::symlink(twin.join("f"), twin.join("l")).unwrap();
    for (name, data) in [
        ("new", "fresh"),
        ("new", "again"),
        ("l", "through the link"),
    ] {
        use std::io::Write;
        match std::fs::File::create_new(twin.join(name)) {
            Ok(mut f) => {
                f.write_all(data.as_bytes()).unwrap();
                want.push_str("ok\n");
            }
            Err(e) => want.push_str(&format!("{e}\n")),
        }
    }
    let o = run(META, &[dir.to_str().unwrap()], b"");
    assert_eq!(text(&o.stdout), want, "{}", text(&o.stderr));
    assert_eq!(std::fs::read(dir.join("new")).unwrap(), b"fresh");
    assert_eq!(std::fs::read(dir.join("f")).unwrap(), b"hello");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&twin);
}

const QUIET: &str = r#"#[summary("stderr and SIGPIPE of children")]
mod app {
    #[effects(alloc)]
    fn sh(script: &str) -> Vec<String> {
        let mut a: Vec<String> = Vec::new();
        a.push(String::from("-c"));
        a.push(String::from(script));
        a
    }

    #[effects(alloc, io, nondet)]
    fn first_line(script: &str, err: std::process::Stdio) -> () {
        let s: Vec<String> = sh(script);
        match std::process::spawn("sh", s.as_slice(), err) {
            Ok(child) => {
                let mut c: std::process::Child = child;
                let mut l: String = String::new();
                match c.read_line(&mut l) {
                    Ok(n) => print_str(l.as_str()),
                    Err(e) => print_str(e.message.as_str()),
                }
                match c.wait() {
                    Ok(st) => print_i64(i64::try_from(st).unwrap()),
                    Err(f) => print_str(f.message.as_str()),
                }
            }
            Err(g) => print_str(g.message.as_str()),
        }
    }

    #[effects(alloc, io, nondet)]
    fn main() -> () {
        first_line("echo out; echo err-shown >&2", std::process::Stdio::Inherit);
        first_line("echo out; echo err-hidden >&2", std::process::Stdio::Null);
        // A child starts with `SIGPIPE` at its default (Rust's `Command`):
        // `yes` ends quietly when `head` is done.
        first_line("yes | head -n 1", std::process::Stdio::Inherit);
        let a: Vec<String> = std::env::args();
        if a.len() > 1 {
            for i in 0..1000000 {
                print_i64(i);
            }
        }
    }
}
"#;

/// A child's standard error is this process's (`Stdio::Inherit`) or
/// discarded (`Stdio::Null`, stage0's solver, `smt.rs`); children start with
/// `SIGPIPE` at its default though this process ignores it; and output to
/// a closed standard output ends the program with code 141 after it has
/// run children too (stage0's `CLOSED_STDOUT`, `tests/cli.rs`).
#[test]
fn children_inherit_or_drop_stderr_and_a_closed_stdout_ends_with_141() {
    let o = run(QUIET, &[], b"");
    assert_eq!(text(&o.stdout), "out\n0\nout\n0\ny\n0\n");
    let err = text(&o.stderr);
    assert!(
        err.contains("err-shown") && !err.contains("err-hidden"),
        "{err}"
    );
    assert!(!err.contains("Broken pipe"), "{err}");
    let out = cli::emit(&one(QUIET), false);
    let dir = cli::fresh_dir();
    let exe = dir.join("p");
    cli::build(&out.ir.expect("IR"), &exe).unwrap();
    let mut child = Command::new(&exe)
        .arg("many")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut first = [0u8; 10];
    use std::io::Read;
    child
        .stdout
        .as_mut()
        .unwrap()
        .read_exact(&mut first)
        .unwrap();
    drop(child.stdout.take());
    assert_eq!(child.wait().unwrap().code(), Some(141));
    let _ = std::fs::remove_dir_all(&dir);
}
