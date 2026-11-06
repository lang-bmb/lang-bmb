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
