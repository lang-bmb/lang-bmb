//! Soundness fuzzing of `Vec` (backlog 2G-05, ADR-0031 §8).
//!
//! Random functions of `x`, `y` run a sequence of `Vec` operations: `push`,
//! `pop` into an accumulator, `truncate`, element writes and reads, the
//! length, a clone that replaces the original, a `&mut Vec` callee with an
//! exact contract, a `push` loop, a temporary clone read for its length,
//! and a move into a by-value callee. Lengths do not depend on `x` and `y`,
//! so the generator knows them and writes the loop invariants. As in the
//! other fuzzers an independent interpreter (a Rust `Vec`) checks every
//! claim:
//!
//! - a verified function never faults (overflow, out-of-bounds index) on an
//!   input its precondition admits, and meets its postcondition;
//! - every counterexample satisfies the precondition, and one from a
//!   function without a loop or a `count` call really breaks it (past those
//!   the solver picks any state the invariant or the callee allows, as in
//!   `soundness_loops.rs` and `soundness_calls.rs`; `bump`'s contract fixes
//!   the state exactly, so it does not stop a replay);
//! - the program built with proved checks removed, and with every check
//!   kept, prints what the interpreter computes;
//! - a third run under an address checker (valgrind memcheck with leak
//!   checking, else AddressSanitizer) finds no invalid access, double free
//!   or leak. Without either the test fails: the free points are the point
//!   of this fuzzer (ADR-0031 adoption note 9).
//!
//! Backlog 2S-02 (ADR-0044 §6) adds a byte buffer `w: Vec<u8>` (pushes of
//! byte literals, reads of its elements) and byte strings passed to
//! `first_or`, whose contract fixes its result (escapes included: quote,
//! backslash, NUL, newline and bytes above 127 as `\xHH`).
//!
//! It drives the `bmb` command line (HEALTH.md S5). `BMB_SOUND_ITERS` and
//! `BMB_SOUND_SEED` as in the other fuzzers.

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

mod common;

use common::scalar::*;

/// One statement over the local `v: Vec<i64>` (`k` numbers its names).
enum Op {
    Push(E),
    /// `match v.pop() { None => {}, Some(t) => { acc = acc + t; } }`
    Pop,
    Truncate(usize),
    Set(usize, E),
    /// `let e: i64 = v[i];`
    Read(usize),
    /// `let n: i64 = i64::try_from(v.len()).unwrap();`
    Len,
    /// `let w: Vec<i64> = v.clone(); v = w;` (frees the replaced buffer)
    CloneSwap,
    /// `bump(&mut v, e);`: pushes, with an exact `ensures`.
    Bump(E),
    /// `n` pushes of `e` in a loop.
    Loop(usize, E),
    /// `let s: i64 = i64::try_from(v.clone().len()).unwrap();` (a temporary)
    TempLen,
    /// `let c: i64 = count(v.clone());` (a move into a by-value callee)
    Count,
    /// `w.push(b'..');` on the byte buffer.
    WPush(u8),
    /// `let b: i64 = i64::try_from(w[i]).unwrap();`
    WRead(usize),
    /// `let l: i64 = first_or(b"..", d);`
    Lit(Vec<u8>, u8),
}

/// A byte as a `b'..'` literal.
fn byte_lit(c: u8) -> String {
    match c {
        b'\'' => "b'\\''".into(),
        b'\\' => "b'\\\\'".into(),
        b'\n' => "b'\\n'".into(),
        0 => "b'\\0'".into(),
        0x20..=0x7e => format!("b'{}'", c as char),
        c => format!("b'\\x{c:02x}'"),
    }
}

/// Bytes as a `b"..."` literal.
fn bytes_lit(v: &[u8]) -> String {
    let mut s = String::from("b\"");
    for &c in v {
        match c {
            b'"' => s.push_str("\\\""),
            b'\\' => s.push_str("\\\\"),
            b'\n' => s.push_str("\\n"),
            0 => s.push_str("\\0"),
            0x20..=0x7e => s.push(c as char),
            c => s.push_str(&format!("\\x{c:02x}")),
        }
    }
    s.push('"');
    s
}

fn gen_byte(rng: &mut Rng) -> u8 {
    const SOME: &[u8] = b"az09 \"\\\n";
    match rng.below(4) {
        0 => 0,
        1 => 255,
        2 => SOME[rng.below(SOME.len())],
        _ => rng.below(256) as u8,
    }
}

struct VFunc {
    /// Name, precondition bounds and `extra`; its body and post are unused.
    base: Func,
    cap: Option<usize>,
    ops: Vec<Op>,
    tail: E,
    post: P,
}

fn gen_vfunc(rng: &mut Rng, k: usize) -> VFunc {
    let base = gen_func(rng, k);
    let mut vars: Vec<String> = vec!["x".into(), "y".into(), "acc".into()];
    let mut len = 0usize;
    let mut wlen = 0usize;
    let mut ops = Vec::new();
    for j in 0..2 + rng.below(6) {
        let op = match rng.below(14) {
            11 => {
                wlen += 1;
                Op::WPush(gen_byte(rng))
            }
            12 => {
                vars.push(format!("b{j}"));
                Op::WRead(rng.below(wlen + 2))
            }
            13 => {
                vars.push(format!("l{j}"));
                let n = rng.below(5);
                Op::Lit((0..n).map(|_| gen_byte(rng)).collect(), gen_byte(rng))
            }
            0 | 1 => {
                len += 1;
                Op::Push(gen_e(rng, &vars, 2))
            }
            2 => {
                len = len.saturating_sub(1);
                Op::Pop
            }
            3 => {
                let n = rng.below(4);
                len = len.min(n);
                Op::Truncate(n)
            }
            // Indices sometimes one past the end: the bounds obligation fails.
            4 => Op::Set(rng.below(len + 2), gen_e(rng, &vars, 1)),
            5 => {
                vars.push(format!("e{j}"));
                Op::Read(rng.below(len + 2))
            }
            6 => {
                vars.push(format!("n{j}"));
                Op::Len
            }
            7 => Op::CloneSwap,
            8 => {
                len += 1;
                Op::Bump(gen_e(rng, &vars, 1))
            }
            9 => {
                let n = rng.below(4);
                len += n;
                Op::Loop(n, gen_e(rng, &["x".into(), "y".into()], 1))
            }
            _ => {
                if rng.chance(50) {
                    vars.push(format!("s{j}"));
                    Op::TempLen
                } else {
                    vars.push(format!("c{j}"));
                    Op::Count
                }
            }
        };
        ops.push(op);
    }
    VFunc {
        cap: rng.chance(40).then(|| rng.below(6)),
        tail: gen_e(rng, &vars, 2),
        post: gen_p(rng, &["x", "y", "r"], 1),
        ops,
        base,
    }
}

impl VFunc {
    fn name(&self) -> &str {
        &self.base.name
    }

    /// Whether a counterexample is a real run: no loop and no `count` call
    /// leave a state only partly known.
    fn replayable(&self) -> bool {
        !self
            .ops
            .iter()
            .any(|o| matches!(o, Op::Loop(..) | Op::Count))
    }

    fn text(&self) -> String {
        let n = self.name();
        let mut t = format!(
            "    spec fn {n}_pre(x: i64, y: i64) -> bool {{ {} }}\n\n    spec fn {n}_post(x: i64, y: i64, r: i64) -> bool {{ {} }}\n\n",
            self.base.pre_text(),
            p_text(&self.post)
        );
        let _ = writeln!(
            t,
            "    #[effects(pure)]\n    #[uses({n}_pre, {n}_post, room, pushed, is_first)]\n    fn {n}(x: i64, y: i64) -> (r: i64)\n        requires {n}_pre(x, y),\n        ensures {n}_post(x, y, r),\n    {{"
        );
        let new = match self.cap {
            Some(c) => format!("Vec::with_capacity({c})"),
            None => "Vec::new()".into(),
        };
        let _ = writeln!(t, "        let mut v: Vec<i64> = {new};");
        let _ = writeln!(t, "        let mut acc: i64 = 0;");
        let _ = writeln!(t, "        let mut w: Vec<u8> = Vec::new();");
        let mut len = 0usize;
        for (j, op) in self.ops.iter().enumerate() {
            let line = match op {
                Op::Push(e) => {
                    len += 1;
                    format!("v.push({});", e_text(e))
                }
                Op::Pop => {
                    len = len.saturating_sub(1);
                    format!(
                        "match v.pop() {{ None => {{}}, Some(t{j}) => {{ acc = acc + t{j}; }} }}"
                    )
                }
                Op::Truncate(m) => {
                    len = len.min(*m);
                    format!("v.truncate({m});")
                }
                Op::Set(i, e) => format!("v[{i}] = {};", e_text(e)),
                Op::Read(i) => format!("let e{j}: i64 = v[{i}];"),
                Op::Len => format!("let n{j}: i64 = i64::try_from(v.len()).unwrap();"),
                Op::CloneSwap => format!("let w{j}: Vec<i64> = v.clone();\n        v = w{j};"),
                Op::Bump(e) => {
                    len += 1;
                    format!("bump(&mut v, {});", e_text(e))
                }
                Op::Loop(m, e) => {
                    let s = format!(
                        "let mut i{j}: usize = 0;\n        while i{j} < {m}\n            invariant i{j} <= {m}, v.len() == {len} + i{j},\n            decreases {m} - i{j},\n        {{\n            v.push({});\n            i{j} = i{j} + 1;\n        }}",
                        e_text(e)
                    );
                    len += m;
                    s
                }
                Op::TempLen => format!("let s{j}: i64 = i64::try_from(v.clone().len()).unwrap();"),
                Op::Count => format!("let c{j}: i64 = count(v.clone());"),
                Op::WPush(c) => format!("w.push({});", byte_lit(*c)),
                Op::WRead(i) => format!("let b{j}: i64 = i64::try_from(w[{i}]).unwrap();"),
                Op::Lit(s, d) => format!(
                    "let l{j}: i64 = first_or({}, {});",
                    bytes_lit(s),
                    byte_lit(*d)
                ),
            };
            let _ = writeln!(t, "        {line}");
        }
        let _ = writeln!(t, "        {}\n    }}\n", e_text(&self.tail));
        t
    }

    fn run(&self, x: i64, y: i64) -> Result<i64, Fault> {
        let mut env: Env = vec![("x".into(), x), ("y".into(), y), ("acc".into(), 0)];
        let mut v: Vec<i64> = Vec::new();
        let mut w: Vec<u8> = Vec::new();
        for (j, op) in self.ops.iter().enumerate() {
            match op {
                Op::Push(e) | Op::Bump(e) => v.push(e_eval(e, &env)?),
                Op::Pop => {
                    if let Some(t) = v.pop() {
                        let acc = lookup(&env, "acc").checked_add(t).ok_or(Fault)?;
                        env.push(("acc".into(), acc));
                    }
                }
                Op::Truncate(m) => v.truncate(*m),
                Op::Set(i, e) => {
                    let val = e_eval(e, &env)?;
                    *v.get_mut(*i).ok_or(Fault)? = val;
                }
                Op::Read(i) => env.push((format!("e{j}"), *v.get(*i).ok_or(Fault)?)),
                Op::Len => env.push((format!("n{j}"), v.len() as i64)),
                Op::CloneSwap => {}
                Op::Loop(m, e) => {
                    for _ in 0..*m {
                        v.push(e_eval(e, &env)?);
                    }
                }
                Op::TempLen => env.push((format!("s{j}"), v.len() as i64)),
                Op::Count => env.push((format!("c{j}"), v.len() as i64)),
                Op::WPush(c) => w.push(*c),
                Op::WRead(i) => env.push((format!("b{j}"), i64::from(*w.get(*i).ok_or(Fault)?))),
                Op::Lit(s, d) => env.push((format!("l{j}"), i64::from(*s.first().unwrap_or(d)))),
            }
        }
        e_eval(&self.tail, &env)
    }

    fn post(&self, x: i64, y: i64, r: i64) -> bool {
        p_eval(
            &self.post,
            &[("x", x as i128), ("y", y as i128), ("r", r as i128)],
        )
    }
}

/// The helpers every batch shares: an exact `push` contract and a callee
/// that takes a `Vec` by value.
const HELPERS: &str = "    spec fn room(a: &[i64]) -> bool { a.len() < 1000 }

    spec fn pushed(a: &[i64], b: &[i64], x: i64) -> bool {
        a.len() == b.len() + 1 && a[b.len()] == x && forall|k: usize| k < b.len() ==> #[trigger] a[k] == b[k]
    }

    #[effects(pure)]
    #[uses(room, pushed)]
    fn bump(v: &mut Vec<i64>, x: i64) -> ()
        requires room(v.as_slice()),
        ensures pushed(v.as_slice(), old(v).as_slice(), x),
    {
        v.push(x);
    }

    #[effects(pure)]
    fn count(w: Vec<i64>) -> i64 {
        i64::try_from(w.len()).unwrap()
    }

    spec fn is_first(r: i64, s: &[u8], d: u8) -> bool { (s.len() > 0 && r == s[0]) || (s.len() == 0 && r == d) }

    #[effects(pure)]
    #[uses(is_first)]
    fn first_or(s: &[u8], d: u8) -> (r: i64)
        ensures is_first(r, s, d),
    {
        if s.len() > 0 { i64::try_from(s[0]).unwrap() } else { i64::try_from(d).unwrap() }
    }

";

#[derive(Clone, Copy, Debug)]
enum AddrCheck {
    Valgrind,
    Asan,
}

/// The address checker this machine has: valgrind memcheck (checks every
/// access), else AddressSanitizer (clang's runtime); neither fails the test.
fn addr_check(dir: &std::path::Path) -> AddrCheck {
    let ok = |c: &mut Command| c.output().is_ok_and(|o| o.status.success());
    if ok(Command::new("valgrind").arg("--version")) {
        return AddrCheck::Valgrind;
    }
    let c = dir.join("probe.c");
    let exe = dir.join("probe");
    fs::write(&c, "int main(void) { return 0; }\n").unwrap();
    if ok(Command::new("clang")
        .arg("-fsanitize=address")
        .arg("-o")
        .arg(&exe)
        .arg(&c))
        && ok(&mut Command::new(&exe))
    {
        return AddrCheck::Asan;
    }
    panic!(
        "the Vec fuzzer needs an address checker: install valgrind or clang's AddressSanitizer runtime (scripts/setup.sh)"
    );
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
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path, checker: AddrCheck) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let funcs: Vec<VFunc> = (0..6).map(|k| gen_vfunc(&mut rng, k)).collect();
    let mut src = String::from("#[summary(\"Vec 건전성 퍼징\")]\nmod m {\n");
    src.push_str(HELPERS);
    for f in &funcs {
        src.push_str(&f.text());
    }
    src.push_str(
        "}\n\n#[summary(\"실행\")]\nmod app {\n    #[effects(io)]\n    fn main() -> () {\n",
    );
    let mut expected = String::new();
    for f in &funcs {
        for (x, y) in f.base.inputs(&mut rng) {
            if let Ok(r) = f.run(x, y) {
                let _ = writeln!(
                    src,
                    "        print_i64(m::{}({}, {}));",
                    f.name(),
                    lit(x),
                    lit(y)
                );
                let _ = writeln!(expected, "{r}");
            }
        }
    }
    src.push_str("    }\n}\n");
    let file = dir.join(format!("v{seed}.bmb"));
    fs::write(&file, &src).unwrap();
    let emit = |all_checks: bool| -> (String, Option<String>) {
        let ll = dir.join(format!("v{seed}-{all_checks}.ll"));
        let _ = fs::remove_file(&ll);
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
        cmd.arg("emit-llvm");
        if all_checks {
            cmd.arg("--all-checks");
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
        for (x, y) in f.base.inputs(&mut rng) {
            stats.verified_runs += 1;
            match f.run(x, y) {
                Ok(r) => assert!(
                    f.post(x, y, r),
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
        assert!(
            f.base.pre(x, y),
            "{}",
            ctx(&format!("counterexample breaks the precondition: {msg}"))
        );
        if !f.replayable() {
            continue;
        }
        stats.replayed += 1;
        let broken = match f.run(x, y) {
            Err(Fault) => true,
            Ok(r) => !f.post(x, y, r),
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
        let ll = dir.join(format!("v{seed}-{all_checks}.build.ll"));
        fs::write(&ll, &ir).unwrap();
        // The address check builds at -O0: optimization may delete an
        // allocation that does not escape, and with it a missing free.
        let build = |exe: &std::path::Path, addr: bool| {
            let mut cc = Command::new("clang");
            cc.args([if addr { "-O0" } else { "-O2" }, "-Wno-override-module"]);
            if addr && matches!(checker, AddrCheck::Asan) {
                cc.arg("-fsanitize=address");
            }
            let out = cc.arg("-o").arg(exe).arg(&ll).output().unwrap();
            assert!(
                out.status.success(),
                "{}",
                ctx(&String::from_utf8_lossy(&out.stderr))
            );
        };
        let exe = dir.join(format!("v{seed}-{all_checks}"));
        build(&exe, false);
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
        // The address check, on the proved IR (its frees are the same in
        // both; the proved one has no checks to stop a bad access early).
        if !all_checks {
            let a = dir.join(format!("v{seed}-addr"));
            build(&a, true);
            let checked = match checker {
                AddrCheck::Valgrind => Command::new("valgrind")
                    .args([
                        "-q",
                        "--leak-check=full",
                        "--show-leak-kinds=definite,indirect,possible",
                        "--errors-for-leak-kinds=definite,indirect,possible",
                        "--error-exitcode=99",
                    ])
                    .arg(&a)
                    .output()
                    .unwrap(),
                AddrCheck::Asan => Command::new(&a)
                    .env("ASAN_OPTIONS", "detect_leaks=1:exitcode=99")
                    .output()
                    .unwrap(),
            };
            let _ = fs::remove_file(&a);
            assert!(
                checked.status.success(),
                "{}",
                ctx(&format!(
                    "{checker:?} reports a memory error:\n{}",
                    String::from_utf8_lossy(&checked.stderr)
                ))
            );
            assert_eq!(String::from_utf8_lossy(&checked.stdout), expected);
            stats.checked_runs += n;
        }
        let _ = fs::remove_file(&exe);
        let _ = fs::remove_file(&ll);
    }
}

#[test]
fn vec_proofs_counterexamples_builds_and_frees_agree_with_the_interpreter() {
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
    let dir = std::env::temp_dir().join(format!("bmb-sound-vec-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let checker = addr_check(&dir);
    let mut stats = Stats::default();
    for seed in start..start + iters {
        let t = std::time::Instant::now();
        batch(seed, &mut stats, &dir, checker);
        eprintln!("seed {seed}: {:?}", t.elapsed());
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness (vec, {checker:?}): {} functions, {} verified ({} runs), {} counterexamples ({} replayed), {} compiled runs, {} address-checked runs",
        stats.funcs,
        stats.verified,
        stats.verified_runs,
        stats.cexes,
        stats.replayed,
        stats.compiled_runs,
        stats.checked_runs
    );
    assert!(stats.verified > 0 && stats.replayed > 0);
}
