//! Soundness fuzzing of text (backlog 2U-04, ADR-0050).
//!
//! Random functions of `x, y: usize` build a `String` from a literal and run
//! a sequence of text operations on it: `push` of a constant character and
//! of `char::from_u32(x)` (surrogates and values past `0x10FFFF` included),
//! `push_str` of a literal, of a clone, of `&s[a..b]` of the string itself
//! (the buffer the growth moves), `from_utf8` of random bytes (valid and
//! not), slices `&s[a..b]` with and without a boundary guard, byte reads
//! `s.as_bytes()[i]`, `is_char_boundary`, the comparisons with a literal,
//! and the length. Bounds are `x`, `y`, constants or the length, so a slice
//! may be reversed, past the end or inside a character. As in the other
//! fuzzers an independent interpreter (Rust's `String`) checks every claim:
//!
//! - a verified function never faults (a slice off the character
//!   boundaries, an index past the end) on an input its precondition
//!   admits, and meets its postcondition;
//! - every counterexample satisfies the precondition, and one from a
//!   function whose text the prover knows exactly (no `from_utf8`, whose
//!   outcome it does not model, and no `push` of a character it does not
//!   know, whose bytes it knows by class only) really breaks it;
//! - the program built with proved checks removed, and with every check
//!   kept, prints what the interpreter computes, and the proved build runs
//!   clean under valgrind (or AddressSanitizer);
//! - a call the interpreter says faults runs as a program of its own
//!   (`common::trap`): both builds print the line before it and trap.
//!
//! It drives the `bmb` command line (HEALTH.md S5). `BMB_SOUND_ITERS` and
//! `BMB_SOUND_SEED` as in the other fuzzers.

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

mod common;

use common::scalar::{Fault, Rng, field, parse_cex};
use common::trap;

/// A position: `x`, `y`, a constant or the string's length.
#[derive(Clone, Copy, Debug)]
enum Idx {
    X,
    Y,
    K(usize),
    Len,
}

#[derive(Clone, Copy, Debug)]
enum Cmp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Debug)]
enum Op {
    PushChar(char),
    PushStr(&'static str),
    /// `push` of `char::from_u32(x)`, or `acc + 7` for `None`, and its
    /// `len_utf8`.
    PushX,
    /// `acc + take(&s[a..b])`, under a boundary guard when set.
    Slice(bool, Idx, Idx),
    /// `s.push_str(&s[a..b])`, under a boundary guard when set.
    SelfAppend(bool, Idx, Idx),
    Byte(Idx),
    Cmp(Cmp, &'static str),
    Boundary(Idx),
    Len,
    /// `from_utf8` of the bytes: appended when valid, else `acc + len`.
    Utf8(Vec<u8>),
    Clone,
}

const CHARS: &[char] = &['a', 'é', '€', '😀', '\n', '"'];
const LITS: &[&str] = &["", "ab", "é€", "😀x", "a\\b"];
const STARTS: &[&str] = &["", "a", "héllo", "€😀", "ab\"c"];

/// `s` as a string literal.
fn str_lit(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn char_lit(c: char) -> String {
    match c {
        '\n' => "'\\n'".into(),
        '\'' => "'\\''".into(),
        c => format!("'{c}'"),
    }
}

struct TFunc {
    k: usize,
    start: &'static str,
    ops: Vec<Op>,
    /// `x <= xh`, `y <= yh`.
    xh: usize,
    yh: usize,
    post: (Cmp, i64),
}

fn gen_idx(rng: &mut Rng) -> Idx {
    match rng.below(5) {
        0 => Idx::X,
        1 => Idx::Y,
        2 => Idx::Len,
        _ => Idx::K(rng.below(7)),
    }
}

fn gen_cmp(rng: &mut Rng) -> Cmp {
    [Cmp::Eq, Cmp::Ne, Cmp::Lt, Cmp::Le, Cmp::Gt, Cmp::Ge][rng.below(6)]
}

fn gen_bytes(rng: &mut Rng) -> Vec<u8> {
    const SOME: &[&[u8]] = &[
        b"ok",
        &[0xc3, 0xa9],
        &[0xed, 0xa0, 0x80],
        &[0xc0, 0x80],
        &[0xf4, 0x90, 0x80, 0x80],
        &[0xe2, 0x82],
        &[0x80],
    ];
    match rng.below(3) {
        0 => (0..rng.below(4)).map(|_| rng.below(256) as u8).collect(),
        _ => SOME[rng.below(SOME.len())].to_vec(),
    }
}

fn gen_op(rng: &mut Rng, clones: &mut usize) -> Op {
    match rng.below(12) {
        0 => Op::PushChar(CHARS[rng.below(CHARS.len())]),
        1 => Op::PushStr(LITS[rng.below(LITS.len())]),
        2 => Op::PushX,
        3 | 4 => Op::Slice(rng.chance(50), gen_idx(rng), gen_idx(rng)),
        5 => Op::SelfAppend(rng.chance(60), gen_idx(rng), gen_idx(rng)),
        6 => Op::Byte(gen_idx(rng)),
        7 => Op::Cmp(
            gen_cmp(rng),
            [LITS, STARTS].concat()[rng.below(LITS.len() + STARTS.len())],
        ),
        8 => Op::Boundary(gen_idx(rng)),
        9 => Op::Len,
        10 => Op::Utf8(gen_bytes(rng)),
        _ if *clones == 0 => {
            *clones += 1;
            Op::Clone
        }
        _ => Op::Len,
    }
}

fn gen_func(rng: &mut Rng, k: usize) -> TFunc {
    let mut clones = 0;
    let ops = (0..1 + rng.below(6))
        .map(|_| gen_op(rng, &mut clones))
        .collect();
    let big = [0x10ffff + 2, 0xe001, 0xd801];
    let xh = if rng.chance(30) {
        big[rng.below(3)]
    } else {
        1 + rng.below(12)
    };
    let mut f = TFunc {
        k,
        start: STARTS[rng.below(STARTS.len())],
        ops,
        xh,
        yh: 1 + rng.below(12),
        post: (Cmp::Le, 0),
    };
    // The bound: the value at one admitted input, moved a little, so some
    // contracts hold everywhere and some do not.
    let (x, y) = (rng.below(f.xh + 1), rng.below(f.yh + 1));
    let c = match f.run(x, y) {
        Ok(r) => r + [-1, 0, 0, 1][rng.below(4)],
        Err(_) => rng.below(40) as i64,
    };
    f.post = ([Cmp::Eq, Cmp::Le, Cmp::Ge][rng.below(3)], c);
    f
}

fn idx_text(i: Idx) -> String {
    match i {
        Idx::X => "x".into(),
        Idx::Y => "y".into(),
        Idx::K(k) => k.to_string(),
        Idx::Len => "s.len()".into(),
    }
}

fn cmp_text(c: Cmp) -> &'static str {
    match c {
        Cmp::Eq => "==",
        Cmp::Ne => "!=",
        Cmp::Lt => "<",
        Cmp::Le => "<=",
        Cmp::Gt => ">",
        Cmp::Ge => ">=",
    }
}

fn cmp_eval<T: Ord>(c: Cmp, a: T, b: T) -> bool {
    match c {
        Cmp::Eq => a == b,
        Cmp::Ne => a != b,
        Cmp::Lt => a < b,
        Cmp::Le => a <= b,
        Cmp::Gt => a > b,
        Cmp::Ge => a >= b,
    }
}

const HELPERS: &str = "    spec fn is_len(r: usize, t: &str) -> bool {
        r == t.len()
    }

    #[effects(pure)]
    #[uses(is_len)]
    fn take(t: &str) -> (r: usize)
        ensures is_len(r, t),
    {
        t.len()
    }

";

impl TFunc {
    fn name(&self) -> String {
        format!("f{}", self.k)
    }

    /// Whether the prover knows the text exactly, so a counterexample is a
    /// real input.
    fn replayable(&self) -> bool {
        !self
            .ops
            .iter()
            .any(|o| matches!(o, Op::Utf8(_) | Op::PushX))
    }

    fn pre(&self, x: usize, y: usize) -> bool {
        x <= self.xh && y <= self.yh
    }

    fn post_ok(&self, r: i64) -> bool {
        cmp_eval(self.post.0, r, self.post.1)
    }

    fn text(&self) -> String {
        let k = self.k;
        let mut b = String::new();
        let _ = writeln!(
            b,
            "        let mut s: String = String::from({});",
            str_lit(self.start)
        );
        b.push_str("        let mut acc: i64 = 0;\n");
        for (n, op) in self.ops.iter().enumerate() {
            let guard = |a: Idx, z: Idx| {
                let (a, z) = (idx_text(a), idx_text(z));
                format!("s.is_char_boundary({a}) && s.is_char_boundary({z}) && {a} <= {z}")
            };
            match op {
                Op::PushChar(c) => {
                    let _ = writeln!(b, "        s.push({});", char_lit(*c));
                }
                Op::PushStr(t) => {
                    let _ = writeln!(b, "        s.push_str({});", str_lit(t));
                }
                Op::PushX => {
                    let _ = writeln!(
                        b,
                        "        match char::from_u32(u32::try_from(x).unwrap()) {{\n            None => {{\n                acc = acc + 7;\n            }}\n            Some(c{n}) => {{\n                s.push(c{n});\n                acc = acc + i64::try_from(c{n}.len_utf8()).unwrap();\n            }}\n        }}"
                    );
                }
                Op::Slice(g, a, z) => {
                    let line = format!(
                        "acc = acc + i64::try_from(take(&s[{}..{}])).unwrap();",
                        idx_text(*a),
                        idx_text(*z)
                    );
                    if *g {
                        let _ = writeln!(
                            b,
                            "        if {} {{\n            {line}\n        }}",
                            guard(*a, *z)
                        );
                    } else {
                        let _ = writeln!(b, "        {line}");
                    }
                }
                Op::SelfAppend(g, a, z) => {
                    let line = format!("s.push_str(&s[{}..{}]);", idx_text(*a), idx_text(*z));
                    if *g {
                        let _ = writeln!(
                            b,
                            "        if {} {{\n            {line}\n        }}",
                            guard(*a, *z)
                        );
                    } else {
                        let _ = writeln!(b, "        {line}");
                    }
                }
                Op::Byte(i) => {
                    let _ = writeln!(
                        b,
                        "        acc = acc + i64::try_from(s.as_bytes()[{}]).unwrap();",
                        idx_text(*i)
                    );
                }
                Op::Cmp(c, t) => {
                    let _ = writeln!(
                        b,
                        "        if s {} {} {{\n            acc = acc + 1;\n        }}",
                        cmp_text(*c),
                        str_lit(t)
                    );
                }
                Op::Boundary(i) => {
                    let _ = writeln!(
                        b,
                        "        if s.is_char_boundary({}) {{\n            acc = acc + 2;\n        }}",
                        idx_text(*i)
                    );
                }
                Op::Len => {
                    b.push_str("        acc = acc + i64::try_from(s.len()).unwrap();\n");
                }
                Op::Utf8(bytes) => {
                    let _ = writeln!(b, "        let mut v{n}: Vec<u8> = Vec::new();");
                    for x in bytes {
                        let _ = writeln!(b, "        v{n}.push({x});");
                    }
                    let _ = writeln!(
                        b,
                        "        match String::from_utf8(v{n}) {{\n            Ok(u{n}) => {{\n                s.push_str(u{n}.as_str());\n            }}\n            Err(e{n}) => {{\n                acc = acc + i64::try_from(e{n}.len()).unwrap();\n            }}\n        }}"
                    );
                }
                Op::Clone => {
                    let _ = writeln!(b, "        let c{n}: String = s.clone();");
                    let _ = writeln!(b, "        s.push_str(c{n}.as_str());");
                }
            }
        }
        b.push_str("        acc\n");
        format!(
            "    spec fn pre{k}(x: usize, y: usize) -> bool {{\n        x <= {} && y <= {}\n    }}\n\n    spec fn post{k}(r: i64) -> bool {{\n        r {} {}\n    }}\n\n    #[effects(alloc)]\n    #[uses(is_len, post{k}, pre{k})]\n    fn f{k}(x: usize, y: usize) -> (r: i64)\n        requires pre{k}(x, y),\n        ensures post{k}(r),\n    {{\n{b}    }}\n\n",
            self.xh,
            self.yh,
            cmp_text(self.post.0),
            if self.post.1 < 0 {
                format!("-{}", -self.post.1)
            } else {
                self.post.1.to_string()
            }
        )
    }

    /// Rust's meaning of the function at `(x, y)`.
    fn run(&self, x: usize, y: usize) -> Result<i64, Fault> {
        let mut s = String::from(self.start);
        let mut acc: i64 = 0;
        let at = |s: &String, i: Idx| match i {
            Idx::X => x,
            Idx::Y => y,
            Idx::K(k) => k,
            Idx::Len => s.len(),
        };
        let guard = |s: &String, a: usize, z: usize| {
            s.is_char_boundary(a) && s.is_char_boundary(z) && a <= z
        };
        for op in &self.ops {
            match op {
                Op::PushChar(c) => s.push(*c),
                Op::PushStr(t) => s.push_str(t),
                Op::PushX => {
                    let v = u32::try_from(x).map_err(|_| Fault)?;
                    match char::from_u32(v) {
                        None => acc += 7,
                        Some(c) => {
                            s.push(c);
                            acc += c.len_utf8() as i64;
                        }
                    }
                }
                Op::Slice(g, a, z) => {
                    let (a, z) = (at(&s, *a), at(&s, *z));
                    if *g && !guard(&s, a, z) {
                        continue;
                    }
                    acc += s.get(a..z).ok_or(Fault)?.len() as i64;
                }
                Op::SelfAppend(g, a, z) => {
                    let (a, z) = (at(&s, *a), at(&s, *z));
                    if *g && !guard(&s, a, z) {
                        continue;
                    }
                    let part = s.get(a..z).ok_or(Fault)?.to_string();
                    s.push_str(&part);
                }
                Op::Byte(i) => acc += *s.as_bytes().get(at(&s, *i)).ok_or(Fault)? as i64,
                Op::Cmp(c, t) => {
                    if cmp_eval(*c, s.as_str(), t) {
                        acc += 1;
                    }
                }
                Op::Boundary(i) => {
                    if s.is_char_boundary(at(&s, *i)) {
                        acc += 2;
                    }
                }
                Op::Len => acc += s.len() as i64,
                Op::Utf8(bytes) => match String::from_utf8(bytes.clone()) {
                    Ok(u) => s.push_str(&u),
                    Err(e) => acc += e.into_bytes().len() as i64,
                },
                Op::Clone => {
                    let c = s.clone();
                    s.push_str(&c);
                }
            }
        }
        Ok(acc)
    }

    /// Inputs the precondition admits: the edges and some others.
    fn inputs(&self, rng: &mut Rng) -> Vec<(usize, usize)> {
        let mut xs = vec![0, 1, 2, 3, self.xh, self.xh.saturating_sub(1)];
        for v in [0xd7ff, 0xd800, 0xdfff, 0xe000, 0x10ffff, 0x110000] {
            if v <= self.xh {
                xs.push(v);
            }
        }
        xs.push(rng.below(self.xh + 1));
        let mut out = Vec::new();
        for x in xs {
            for y in [0, 1, self.yh, rng.below(self.yh + 1)] {
                if self.pre(x, y) && !out.contains(&(x, y)) {
                    out.push((x, y));
                }
            }
        }
        out
    }
}

#[derive(Default)]
struct Stats {
    funcs: usize,
    verified: usize,
    verified_runs: usize,
    cexes: usize,
    replayed: usize,
    compiled_runs: usize,
    checked_runs: usize,
    traps: usize,
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let funcs: Vec<TFunc> = (0..6).map(|k| gen_func(&mut rng, k)).collect();
    let mut src = String::from("#[summary(\"문자열 건전성 퍼징\")]\nmod m {\n");
    src.push_str(HELPERS);
    for f in &funcs {
        src.push_str(&f.text());
    }
    src.push_str("}\n\n");
    let module = src.clone();
    let main_head =
        "#[summary(\"실행\")]\nmod app {\n    #[effects(alloc, io)]\n    fn main() -> () {\n";
    src.push_str(main_head);
    let mut expected = String::new();
    let mut faults: Vec<(usize, String)> = Vec::new();
    for (fi, f) in funcs.iter().enumerate() {
        for (x, y) in f.inputs(&mut rng) {
            let line = format!("        print_i64(m::{}({x}, {y}));\n", f.name());
            match f.run(x, y) {
                Ok(r) => {
                    src.push_str(&line);
                    let _ = writeln!(expected, "{r}");
                }
                Err(Fault) => faults.push((fi, line)),
            }
        }
    }
    src.push_str("    }\n}\n");
    let file = dir.join(format!("t{seed}.bmb"));
    fs::write(&file, &src).unwrap();
    let cache = dir.join(format!("cache{seed}"));
    let emit = |all_checks: bool| -> (String, Option<String>) {
        let ll = dir.join(format!("t{seed}-{all_checks}.ll"));
        let _ = fs::remove_file(&ll);
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
        cmd.arg("emit-llvm");
        if all_checks {
            cmd.arg("--all-checks");
        } else {
            cmd.arg("--cache").arg(&cache);
        }
        let out = cmd.arg(&file).arg("-o").arg(&ll).output().unwrap();
        let ir = fs::read_to_string(&ll).ok();
        let _ = fs::remove_file(&ll);
        (String::from_utf8_lossy(&out.stdout).into_owned(), ir)
    };
    let (output, ir) = emit(false);
    let ctx = |what: &str| format!("seed {seed}: {what}\n{src}\n{output}");
    assert!(ir.is_some(), "{}", ctx("generated program must check"));

    for line in output.lines() {
        let Some(name) = field(line, "def").and_then(|d| d.strip_prefix("m::")) else {
            continue;
        };
        let Some(f) = funcs.iter().find(|f| f.name() == name) else {
            continue;
        };
        stats.funcs += 1;
        if field(line, "status") != Some("verified") {
            continue;
        }
        stats.verified += 1;
        for (x, y) in f.inputs(&mut rng) {
            stats.verified_runs += 1;
            match f.run(x, y) {
                Ok(r) => assert!(
                    f.post_ok(r),
                    "{}",
                    ctx(&format!(
                        "verified {name} breaks its postcondition at x = {x}, y = {y}, r = {r}"
                    ))
                ),
                Err(Fault) => panic!(
                    "{}",
                    ctx(&format!("verified {name} faults at x = {x}, y = {y}"))
                ),
            }
        }
    }

    for line in output.lines() {
        let Some(msg) = field(line, "message") else {
            continue;
        };
        let Some(cex) = parse_cex(msg) else {
            continue;
        };
        let Some(f) = funcs
            .iter()
            .find(|f| msg.contains(&format!("in `m::{}`", f.name())))
        else {
            continue;
        };
        let get = |v: &str| cex.iter().find(|(k, _)| k == v).map(|(_, n)| *n);
        let (Some(x), Some(y)) = (get("x"), get("y")) else {
            panic!("{}", ctx(&format!("counterexample without inputs: {msg}")));
        };
        stats.cexes += 1;
        let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
            panic!("{}", ctx(&format!("counterexample outside `usize`: {msg}")));
        };
        assert!(
            f.pre(x, y),
            "{}",
            ctx(&format!("counterexample breaks the precondition: {msg}"))
        );
        if !f.replayable() {
            continue;
        }
        stats.replayed += 1;
        let broken = match f.run(x, y) {
            Err(Fault) => true,
            Ok(r) => !f.post_ok(r),
        };
        assert!(
            broken,
            "{}",
            ctx(&format!(
                "counterexample does not break {}: {msg}",
                f.name()
            ))
        );
    }

    let n = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            emit(true).1.unwrap()
        } else {
            ir.clone().unwrap()
        };
        let ll = dir.join(format!("t{seed}-{all_checks}.build.ll"));
        fs::write(&ll, &ir).unwrap();
        let exe = dir.join(format!("t{seed}-{all_checks}"));
        let out = Command::new("clang")
            .args(["-O2", "-Wno-override-module", "-o"])
            .arg(&exe)
            .arg(&ll)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            ctx(&String::from_utf8_lossy(&out.stderr))
        );
        let run = Command::new(&exe).output().unwrap();
        assert!(
            run.status.success(),
            "{}",
            ctx(&format!(
                "build (all_checks = {all_checks}) exited with {}",
                run.status
            ))
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{}",
            ctx(&format!("build (all_checks = {all_checks}) output differs"))
        );
        stats.compiled_runs += n;
        if !all_checks && Command::new("valgrind").arg("--version").output().is_ok() {
            let checked = Command::new("valgrind")
                .args([
                    "-q",
                    "--leak-check=full",
                    "--errors-for-leak-kinds=definite,indirect,possible",
                    "--error-exitcode=99",
                ])
                .arg(&exe)
                .output()
                .unwrap();
            assert!(
                checked.status.success(),
                "{}",
                ctx(&format!(
                    "valgrind reports a memory error:\n{}",
                    String::from_utf8_lossy(&checked.stderr)
                ))
            );
            stats.checked_runs += n;
        }
        let _ = fs::remove_file(&exe);
        let _ = fs::remove_file(&ll);
    }

    for (j, (fi, call)) in trap::choose(&faults, |_| ()).into_iter().enumerate() {
        let src = format!("{module}{main_head}{}    }}\n}}\n", trap::body(&call));
        let files = vec![(format!("t{seed}.bmb"), src.clone())];
        let exe = dir.join(format!("t{seed}-trap{j}"));
        if let Some(e) = trap::check(&files, &cache, &exe) {
            panic!("seed {seed}: {}: {e}\n{src}", funcs[fi].name());
        }
        stats.traps += 1;
    }
    let _ = fs::remove_dir_all(&cache);
}

#[test]
fn text_proofs_counterexamples_and_builds_agree_with_the_interpreter() {
    let iters: u64 = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    let start: u64 = std::env::var("BMB_SOUND_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    if std::env::var("BMB_Z3_RLIMIT").is_err() {
        // SAFETY: set before any thread of this test binary reads the
        // environment; this file holds a single test.
        unsafe { std::env::set_var("BMB_Z3_RLIMIT", "500000") };
    }
    let dir = std::env::temp_dir().join(format!("bmb-sound-text-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        let t = std::time::Instant::now();
        batch(seed, &mut stats, &dir);
        eprintln!("seed {seed}: {:?}", t.elapsed());
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness (text): {} functions, {} verified ({} runs), {} counterexamples ({} replayed), {} compiled runs, {} valgrind runs, {} traps confirmed",
        stats.funcs,
        stats.verified,
        stats.verified_runs,
        stats.cexes,
        stats.replayed,
        stats.compiled_runs,
        stats.checked_runs,
        stats.traps
    );
    assert!(stats.verified > 0 && stats.replayed > 0 && stats.traps > 0);
}
