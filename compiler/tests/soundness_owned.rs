//! Soundness fuzzing of owned data (backlog 2O-05, ADR-0047).
//!
//! Random functions of `x`, `y` run a sequence of operations on an owned
//! linked list `l: Option<Box<Node>>` (`struct Node { v: i64, next:
//! Option<Box<Node>> }`), a second list `m`, a boxed number `b: Box<i64>`
//! and a struct with a `Vec` field `s: Bag`: prepend a node (moving the
//! list into it), pop the head (`std::mem::replace` of the list and of the
//! popped node's `next`), write and read the head through a `&mut` and a
//! shared reference `match`, swap the two lists, replace the box, a
//! partial move of the bag's `Vec` and its reassignment, and the sum of a
//! list by structural recursion. A rose tree `t: Rose` (`struct Rose { v:
//! i64, kids: Vec<Rose> }`, a struct holding itself, backlog 2O-03d) gets
//! children and grandchildren, a write two levels down and reads of its
//! first child; a tree `tr: Tree` (`enum Tree { Leaf(i64), Many(Vec<Tree>)
//! }`, backlog 2O-03c) is wrapped in a new node with a leaf beside it; both
//! are summed by structural recursion through their `Vec`s. The generator
//! knows every list's length,
//! so the head reads it writes are those an interpreter (a Rust `VecDeque`)
//! can replay. As in the other fuzzers it checks every claim:
//!
//! - a verified function never faults (overflow) on an input its
//!   precondition admits, and meets its postcondition;
//! - every counterexample satisfies the precondition, and one from a
//!   function without a `sum` call really breaks it (`sum` has no contract:
//!   past it the solver picks any value);
//! - the program built with proved checks removed, and with every check
//!   kept, prints what the interpreter computes;
//! - a third run under an address checker (valgrind memcheck with leak
//!   checking, else AddressSanitizer) finds no invalid access, double free
//!   or leak: every owned value is freed exactly once at the point the
//!   move check fixes.
//!
//! It drives the `bmb` command line (HEALTH.md S5). `BMB_SOUND_ITERS` and
//! `BMB_SOUND_SEED` as in the other fuzzers.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs;
use std::process::Command;

mod common;

use common::scalar::*;

/// One statement.
enum Op {
    /// `l = Some(Box::new(Node { v: e, next: l }));`
    Prepend(E),
    /// Pops the head of `l` into `acc` (`acc = acc + v`).
    Pop,
    /// `match &mut l { None => {}, Some(n) => { n.v = e; } }`
    SetHead(E),
    /// `let h: i64 = match &l { None => d, Some(n) => n.v };`
    ReadHead(E),
    /// `std::mem::swap(&mut l, &mut m);`
    Swap,
    /// `let o: Box<i64> = std::mem::replace(&mut b, Box::new(e)); acc = acc + *o;`
    ReplaceBox(E),
    /// `let t: Vec<i64> = s.items; s.items = t; s.items.push(e);` (a partial
    /// move and its reassignment)
    BagPush(E),
    /// `let c: i64 = sum(&l);` (no contract)
    Sum,
    /// `if t.kids.len() < 50 { t.kids.push(Rose { v: e, kids: Vec::new() }); }`
    RoseAdopt(E),
    /// The same under the first child, if any.
    RoseNest(E),
    /// `if t.kids.len() > 0 { t.kids[0].v = e; }`
    RoseSetKid(E),
    /// `let k: i64 = if t.kids.len() > 0 { t.kids[0].v } else { d };`
    RoseRead(E),
    /// `let c: i64 = rose_total(&t);` (no contract)
    RoseTotal,
    /// `tr = Tree::Many([tr, Tree::Leaf(e)])`, through a `Vec`
    TreeWrap(E),
    /// `let f: i64 = match &tr { Tree::Leaf(v) => *v, Tree::Many(_) => d };`
    TreeFirst(E),
    /// `let c: i64 = tree_total(&tr);` (no contract)
    TreeTotal,
}

/// The interpreter's rose tree.
struct Rose {
    v: i64,
    kids: Vec<Rose>,
}

fn rose_total(t: &Rose) -> i64 {
    t.kids
        .iter()
        .fold(t.v, |a, k| a.wrapping_add(rose_total(k)))
}

/// The interpreter's tree.
enum Tree {
    Leaf(i64),
    Many(Vec<Tree>),
}

fn tree_total(t: &Tree) -> i64 {
    match t {
        Tree::Leaf(v) => *v,
        Tree::Many(ks) => ks.iter().fold(0, |a, k| a.wrapping_add(tree_total(k))),
    }
}

struct OFunc {
    base: Func,
    ops: Vec<Op>,
    tail: E,
    post: P,
}

fn gen_ofunc(rng: &mut Rng, k: usize) -> OFunc {
    let base = gen_func(rng, k);
    let mut vars: Vec<String> = vec!["x".into(), "y".into(), "acc".into()];
    let mut ops = Vec::new();
    for j in 0..2 + rng.below(7) {
        let op = match rng.below(18) {
            0..=2 => Op::Prepend(gen_e(rng, &vars, 2)),
            3 => Op::Pop,
            4 => Op::SetHead(gen_e(rng, &vars, 1)),
            5 => {
                let d = gen_e(rng, &vars, 1);
                vars.push(format!("h{j}"));
                Op::ReadHead(d)
            }
            6 => Op::Swap,
            7 => Op::ReplaceBox(gen_e(rng, &vars, 1)),
            8 => Op::BagPush(gen_e(rng, &vars, 1)),
            9 => {
                vars.push(format!("c{j}"));
                Op::Sum
            }
            10 | 11 => Op::RoseAdopt(gen_e(rng, &vars, 1)),
            12 => Op::RoseNest(gen_e(rng, &vars, 1)),
            13 => Op::RoseSetKid(gen_e(rng, &vars, 1)),
            14 => {
                let d = gen_e(rng, &vars, 1);
                vars.push(format!("k{j}"));
                Op::RoseRead(d)
            }
            15 => Op::TreeWrap(gen_e(rng, &vars, 1)),
            16 => {
                let d = gen_e(rng, &vars, 1);
                vars.push(format!("f{j}"));
                Op::TreeFirst(d)
            }
            _ => {
                vars.push(format!("c{j}"));
                if rng.below(2) == 0 {
                    Op::RoseTotal
                } else {
                    Op::TreeTotal
                }
            }
        };
        ops.push(op);
    }
    OFunc {
        tail: gen_e(rng, &vars, 2),
        post: gen_p(rng, &["x", "y", "r"], 1),
        ops,
        base,
    }
}

impl OFunc {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn replayable(&self) -> bool {
        !self
            .ops
            .iter()
            .any(|o| matches!(o, Op::Sum | Op::RoseTotal | Op::TreeTotal))
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
            "    #[effects(pure)]\n    #[uses({n}_pre, {n}_post)]\n    fn {n}(x: i64, y: i64) -> (r: i64)\n        requires {n}_pre(x, y),\n        ensures {n}_post(x, y, r),\n    {{"
        );
        let _ = writeln!(t, "        let mut l: Option<Box<Node>> = None;");
        let _ = writeln!(t, "        let mut m: Option<Box<Node>> = None;");
        let _ = writeln!(t, "        let mut b: Box<i64> = Box::new(0);");
        let _ = writeln!(
            t,
            "        let mut s: Bag = Bag {{ items: Vec::new(), n: 0 }};"
        );
        let _ = writeln!(t, "        let mut acc: i64 = 0;");
        let _ = writeln!(
            t,
            "        let mut t: Rose = Rose {{ v: 0, kids: Vec::new() }};"
        );
        let _ = writeln!(t, "        let mut tr: Tree = Tree::Leaf(0);");
        for (j, op) in self.ops.iter().enumerate() {
            let line = match op {
                Op::Prepend(e) => {
                    format!("l = Some(Box::new(Node {{ v: {}, next: l }}));", e_text(e))
                }
                Op::Pop => format!(
                    "match std::mem::replace(&mut l, None) {{\n            None => {{}}\n            Some(p{j}) => {{\n                let mut q{j}: Box<Node> = p{j};\n                acc = acc + q{j}.v;\n                l = std::mem::replace(&mut q{j}.next, None);\n            }}\n        }}"
                ),
                Op::SetHead(e) => format!(
                    "match &mut l {{\n            None => {{}}\n            Some(w{j}) => {{\n                w{j}.v = {};\n            }}\n        }}",
                    e_text(e)
                ),
                Op::ReadHead(d) => format!(
                    "let h{j}: i64 = match &l {{\n            None => {},\n            Some(g{j}) => g{j}.v,\n        }};",
                    e_text(d)
                ),
                Op::Swap => "std::mem::swap(&mut l, &mut m);".into(),
                Op::ReplaceBox(e) => format!(
                    "let o{j}: Box<i64> = std::mem::replace(&mut b, Box::new({}));\n        acc = acc + *o{j};",
                    e_text(e)
                ),
                Op::BagPush(e) => format!(
                    "let t{j}: Vec<i64> = s.items;\n        s.items = t{j};\n        if s.items.len() < 100 {{\n            s.items.push({});\n        }}",
                    e_text(e)
                ),
                Op::Sum => format!("let c{j}: i64 = sum(&l);"),
                Op::RoseAdopt(e) => format!(
                    "if t.kids.len() < 50 {{\n            t.kids.push(Rose {{ v: {}, kids: Vec::new() }});\n        }}",
                    e_text(e)
                ),
                Op::RoseNest(e) => format!(
                    "if t.kids.len() > 0 {{\n            if t.kids[0].kids.len() < 50 {{\n                t.kids[0].kids.push(Rose {{ v: {}, kids: Vec::new() }});\n            }}\n        }}",
                    e_text(e)
                ),
                Op::RoseSetKid(e) => format!(
                    "if t.kids.len() > 0 {{\n            t.kids[0].v = {};\n        }}",
                    e_text(e)
                ),
                Op::RoseRead(d) => format!(
                    "let k{j}: i64 = if t.kids.len() > 0 {{ t.kids[0].v }} else {{ {} }};",
                    e_text(d)
                ),
                Op::RoseTotal => format!("let c{j}: i64 = rose_total(&t);"),
                Op::TreeWrap(e) => format!(
                    "let mut ks{j}: Vec<Tree> = Vec::new();\n        ks{j}.push(tr);\n        ks{j}.push(Tree::Leaf({}));\n        tr = Tree::Many(ks{j});",
                    e_text(e)
                ),
                Op::TreeFirst(d) => format!(
                    "let f{j}: i64 = match &tr {{\n            Tree::Leaf(lv{j}) => *lv{j},\n            Tree::Many(_) => {},\n        }};",
                    e_text(d)
                ),
                Op::TreeTotal => format!("let c{j}: i64 = tree_total(&tr);"),
            };
            let _ = writeln!(t, "        {line}");
        }
        let _ = writeln!(t, "        {}\n    }}\n", e_text(&self.tail));
        t
    }

    fn run(&self, x: i64, y: i64) -> Result<i64, Fault> {
        let mut env: Env = vec![("x".into(), x), ("y".into(), y), ("acc".into(), 0)];
        let mut l: VecDeque<i64> = VecDeque::new();
        let mut m: VecDeque<i64> = VecDeque::new();
        let mut b: i64 = 0;
        let mut t = Rose {
            v: 0,
            kids: Vec::new(),
        };
        let mut tr = Tree::Leaf(0);
        let add = |env: &mut Env, t: i64| -> Result<(), Fault> {
            let acc = lookup(env, "acc").checked_add(t).ok_or(Fault)?;
            env.push(("acc".into(), acc));
            Ok(())
        };
        for (j, op) in self.ops.iter().enumerate() {
            match op {
                Op::Prepend(e) => l.push_front(e_eval(e, &env)?),
                Op::Pop => {
                    if let Some(v) = l.pop_front() {
                        add(&mut env, v)?;
                    }
                }
                // The value is computed in the `Some` arm only.
                Op::SetHead(e) => {
                    if !l.is_empty() {
                        let v = e_eval(e, &env)?;
                        l[0] = v;
                    }
                }
                Op::ReadHead(d) => {
                    let v = match l.front() {
                        None => e_eval(d, &env)?,
                        Some(h) => *h,
                    };
                    env.push((format!("h{j}"), v));
                }
                Op::Swap => std::mem::swap(&mut l, &mut m),
                Op::ReplaceBox(e) => {
                    let v = e_eval(e, &env)?;
                    let old = std::mem::replace(&mut b, v);
                    add(&mut env, old)?;
                }
                Op::BagPush(e) => {
                    e_eval(e, &env)?;
                }
                Op::Sum => {
                    let s = l.iter().fold(0i64, |a, v| a.wrapping_add(*v));
                    env.push((format!("c{j}"), s));
                }
                // The value is computed in the guarded branch only.
                Op::RoseAdopt(e) => {
                    if t.kids.len() < 50 {
                        let v = e_eval(e, &env)?;
                        t.kids.push(Rose {
                            v,
                            kids: Vec::new(),
                        });
                    }
                }
                Op::RoseNest(e) => {
                    if !t.kids.is_empty() && t.kids[0].kids.len() < 50 {
                        let v = e_eval(e, &env)?;
                        t.kids[0].kids.push(Rose {
                            v,
                            kids: Vec::new(),
                        });
                    }
                }
                Op::RoseSetKid(e) => {
                    if !t.kids.is_empty() {
                        t.kids[0].v = e_eval(e, &env)?;
                    }
                }
                Op::RoseRead(d) => {
                    let v = match t.kids.first() {
                        Some(k) => k.v,
                        None => e_eval(d, &env)?,
                    };
                    env.push((format!("k{j}"), v));
                }
                Op::RoseTotal => env.push((format!("c{j}"), rose_total(&t))),
                Op::TreeWrap(e) => {
                    let v = e_eval(e, &env)?;
                    let old = std::mem::replace(&mut tr, Tree::Leaf(0));
                    tr = Tree::Many(vec![old, Tree::Leaf(v)]);
                }
                Op::TreeFirst(d) => {
                    let v = match &tr {
                        Tree::Leaf(v) => *v,
                        Tree::Many(_) => e_eval(d, &env)?,
                    };
                    env.push((format!("f{j}"), v));
                }
                Op::TreeTotal => env.push((format!("c{j}"), tree_total(&tr))),
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

const HELPERS: &str = "    struct Node { v: i64, next: Option<Box<Node>> }

    struct Bag { items: Vec<i64>, n: i64 }

    struct Rose { v: i64, kids: Vec<Rose> }

    enum Tree { Leaf(i64), Many(Vec<Tree>) }

    #[effects(pure)]
    fn rose_total(t: &Rose) -> i64
        decreases t,
    {
        let mut s: i64 = t.v;
        let mut i: usize = 0;
        while i < t.kids.len()
            invariant i <= t.kids.len(),
            decreases t.kids.len() - i,
        {
            s = s.wrapping_add(rose_total(&t.kids[i]));
            i = i + 1;
        }
        s
    }

    #[effects(pure)]
    fn tree_total(t: &Tree) -> i64
        decreases t,
    {
        match t {
            Tree::Leaf(v) => *v,
            Tree::Many(ks) => {
                let mut s: i64 = 0;
                let mut i: usize = 0;
                while i < ks.len()
                    invariant i <= ks.len(),
                    decreases ks.len() - i,
                {
                    s = s.wrapping_add(tree_total(&ks[i]));
                    i = i + 1;
                }
                s
            }
        }
    }

    #[effects(pure)]
    fn sum(l: &Option<Box<Node>>) -> i64
        decreases l,
    {
        match l {
            None => 0,
            Some(n) => n.v.wrapping_add(sum(&n.next)),
        }
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
        "the owned-data fuzzer needs an address checker: install valgrind or clang's AddressSanitizer runtime (scripts/setup.sh)"
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
    let funcs: Vec<OFunc> = (0..6).map(|k| gen_ofunc(&mut rng, k)).collect();
    let mut src = String::from("#[summary(\"소유 자료 건전성 퍼징\")]\nmod m {\n");
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
fn owned_proofs_counterexamples_builds_and_frees_agree_with_the_interpreter() {
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
    let dir = std::env::temp_dir().join(format!("bmb-sound-owned-{}", std::process::id()));
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
        "soundness (owned, {checker:?}): {} functions, {} verified ({} runs), {} counterexamples ({} replayed), {} compiled runs, {} address-checked runs",
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
