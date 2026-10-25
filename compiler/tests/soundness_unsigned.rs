//! Soundness fuzzing of unsigned arithmetic (backlog 1B-33): `u64` values up
//! to 2^64 - 1, subtraction underflow, unsigned `/` and `%`, `wrapping_*`
//! and unsigned comparisons, checked like `soundness.rs`: verified functions
//! never fault and meet their postcondition on admitted inputs, definite
//! counterexamples break the function, both builds print what the
//! interpreter computes.
//!
//! Backlog 2C-04 (ADR-0028 §9) adds `saturating_*`, `checked_*` (a `match`
//! with a default for `None`) and conversions through `i64` and `usize`: a
//! value converted to `i64` (out of range above `i64::MAX`), a signed
//! difference converted back to `u64` (out of range when negative) and the
//! same through `.ok()`. The interpreter computes them with Rust's own
//! methods; an out-of-range `.unwrap()` is a fault.
//!
//! Backlog 2S-02 (ADR-0044 §6) adds `u8`: a value through `u8` (out of
//! range above 255), the sum of two low bytes in `u8` (overflows above
//! 255), and their wrapping and saturating products.
//!
//! `BMB_SOUND_ITERS` / `BMB_SOUND_SEED` as in `soundness.rs`.

use std::fmt::Write as _;
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

mod common;

use common::scalar::{Rng, field};

#[derive(Clone, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    WrapAdd,
    WrapSub,
    WrapMul,
}

enum U {
    Var(&'static str),
    Lit(u64),
    Bin(Op, Box<U>, Box<U>),
    /// `if a < b { t } else { e }`
    IfLt(Box<U>, Box<U>, Box<U>, Box<U>),
    /// `a.saturating_op(b)` (`Add`, `Sub` or `Mul`).
    Sat(Op, Box<U>, Box<U>),
    /// `match a.checked_op(b) { None => d, Some(v) => v }` (`v` is the
    /// binding's name).
    Checked(Op, Box<U>, Box<U>, Box<U>, String),
    /// `u64::try_from(i64::try_from(a).unwrap()).unwrap()`
    ViaI64(Box<U>),
    /// `u64::try_from(usize::try_from(a).unwrap()).unwrap()`
    ViaUsize(Box<U>),
    /// `u64::try_from(i64::try_from(a).unwrap() - i64::try_from(b).unwrap()).unwrap()`
    Diff(Box<U>, Box<U>),
    /// The same difference through `.ok()`, `d` for `None`.
    DiffOk(Box<U>, Box<U>, Box<U>, String),
    /// `u64::try_from(u8::try_from(a).unwrap()).unwrap()`
    ViaU8(Box<U>),
    /// The low bytes of `a` and `b` (`% 256`) combined in `u8`: `+` (may
    /// overflow), `wrapping_mul` or `saturating_mul`, back as `u64`.
    Byte(Op, Box<U>, Box<U>),
}

/// Binding names are unique in the program, so in each function (E0303).
static BINDINGS: AtomicUsize = AtomicUsize::new(0);

fn binding() -> String {
    format!("v{}", BINDINGS.fetch_add(1, Ordering::Relaxed))
}

struct Fault;

const INTERESTING: &[u64] = &[
    0,
    1,
    2,
    3,
    7,
    10,
    255,
    1 << 32,
    (1 << 63) - 1,
    1 << 63,
    (1 << 63) + 1,
    u64::MAX - 1,
    u64::MAX,
];

fn value(rng: &mut Rng) -> u64 {
    match rng.below(3) {
        0 => INTERESTING[rng.below(INTERESTING.len())],
        1 => rng.below(100) as u64,
        _ => rng.next(),
    }
}

fn gen_u(rng: &mut Rng, depth: usize) -> U {
    if depth == 0 || rng.chance(30) {
        return if rng.chance(65) {
            U::Var(["x", "y"][rng.below(2)])
        } else {
            U::Lit(value(rng))
        };
    }
    if rng.chance(15) {
        let mut a = gen_u(rng, depth - 1);
        let b = gen_u(rng, depth - 1);
        // Two literal-only sides have no type to compare at.
        if !has_var(&a) && !has_var(&b) {
            a = U::Var(["x", "y"][rng.below(2)]);
        }
        return U::IfLt(
            Box::new(a),
            Box::new(b),
            Box::new(gen_u(rng, depth - 1)),
            Box::new(gen_u(rng, depth - 1)),
        );
    }
    if rng.chance(30) {
        return gen_adr28(rng, depth);
    }
    let op = [
        Op::Add,
        Op::Sub,
        Op::Mul,
        Op::Div,
        Op::Rem,
        Op::WrapAdd,
        Op::WrapSub,
        Op::WrapMul,
    ][rng.below(8)];
    let mut lhs = gen_u(rng, depth - 1);
    // As in Rust, an expression of literals alone has no type to call a
    // method on.
    if matches!(op, Op::WrapAdd | Op::WrapSub | Op::WrapMul) && !has_var(&lhs) {
        lhs = U::Var(["x", "y"][rng.below(2)]);
    }
    U::Bin(op, Box::new(lhs), Box::new(gen_u(rng, depth - 1)))
}

/// An operand with a type of its own: a method receiver or a conversion
/// source (a literal alone would be `i64`).
fn typed(rng: &mut Rng, depth: usize) -> U {
    let u = gen_u(rng, depth);
    if has_var(&u) {
        u
    } else {
        U::Var(["x", "y"][rng.below(2)])
    }
}

/// The ADR-0028 forms (backlog 2C-04).
fn gen_adr28(rng: &mut Rng, depth: usize) -> U {
    let b = |rng: &mut Rng| Box::new(gen_u(rng, depth - 1));
    let t = |rng: &mut Rng| Box::new(typed(rng, depth - 1));
    match rng.below(8) {
        6 => U::ViaU8(t(rng)),
        7 => {
            let op = [Op::Add, Op::WrapMul, Op::Mul][rng.below(3)];
            U::Byte(op, t(rng), t(rng))
        }
        0 => {
            let op = [Op::Add, Op::Sub, Op::Mul][rng.below(3)];
            U::Sat(op, t(rng), b(rng))
        }
        1 => {
            let op = [Op::Add, Op::Sub, Op::Mul, Op::Div, Op::Rem][rng.below(5)];
            U::Checked(op, t(rng), b(rng), b(rng), binding())
        }
        2 => U::ViaI64(t(rng)),
        3 => U::ViaUsize(t(rng)),
        4 => U::Diff(t(rng), t(rng)),
        _ => U::DiffOk(t(rng), t(rng), b(rng), binding()),
    }
}

/// Whether a `u8` form (ADR-0044) occurs in the value.
fn bytes(u: &U) -> bool {
    match u {
        U::ViaU8(_) | U::Byte(..) => true,
        U::Var(_) | U::Lit(_) => false,
        U::Bin(_, a, b) | U::Sat(_, a, b) | U::Diff(a, b) => bytes(a) || bytes(b),
        U::IfLt(a, b, t, e) => bytes(a) || bytes(b) || bytes(t) || bytes(e),
        U::Checked(_, a, b, d, _) | U::DiffOk(a, b, d, _) => bytes(a) || bytes(b) || bytes(d),
        U::ViaI64(a) | U::ViaUsize(a) => bytes(a),
    }
}

/// Whether an ADR-0028 or ADR-0044 form occurs in the value.
fn adr28(u: &U) -> bool {
    match u {
        U::Var(_) | U::Lit(_) => false,
        U::Bin(_, a, b) => adr28(a) || adr28(b),
        U::IfLt(a, b, t, e) => adr28(a) || adr28(b) || adr28(t) || adr28(e),
        _ => true,
    }
}

/// Whether the value has a type of its own (some typed variable reaches it).
fn has_var(u: &U) -> bool {
    match u {
        U::Var(_) => true,
        U::Lit(_) => false,
        U::Bin(_, a, b) => has_var(a) || has_var(b),
        // The value's type comes from the branches, not the condition.
        U::IfLt(_, _, t, e) => has_var(t) || has_var(e),
        // `u64` by the method or the conversion.
        U::Sat(..) | U::Checked(..) | U::ViaI64(_) | U::ViaUsize(_) | U::Diff(..) => true,
        U::ViaU8(_) | U::Byte(..) => true,
        U::DiffOk(..) => true,
    }
}

fn text(u: &U) -> String {
    match u {
        U::Var(v) => v.to_string(),
        U::Lit(v) => v.to_string(),
        U::Bin(op, a, b) => {
            let (a, b) = (text(a), text(b));
            match op {
                Op::Add => format!("({a} + {b})"),
                Op::Sub => format!("({a} - {b})"),
                Op::Mul => format!("({a} * {b})"),
                Op::Div => format!("({a} / {b})"),
                Op::Rem => format!("({a} % {b})"),
                Op::WrapAdd => format!("{a}.wrapping_add({b})"),
                Op::WrapSub => format!("{a}.wrapping_sub({b})"),
                Op::WrapMul => format!("{a}.wrapping_mul({b})"),
            }
        }
        U::IfLt(a, b, t, e) => format!(
            "(if {} < {} {{ {} }} else {{ {} }})",
            text(a),
            text(b),
            text(t),
            text(e)
        ),
        U::Sat(op, a, b) => format!("{}.saturating_{}({})", text(a), op_name(*op), text(b)),
        U::Checked(op, a, b, d, v) => format!(
            "(match {}.checked_{}({}) {{ None => {}, Some({v}) => {v} }})",
            text(a),
            op_name(*op),
            text(b),
            text(d)
        ),
        U::ViaI64(a) => format!(
            "u64::try_from(i64::try_from({}).unwrap()).unwrap()",
            text(a)
        ),
        U::ViaUsize(a) => format!(
            "u64::try_from(usize::try_from({}).unwrap()).unwrap()",
            text(a)
        ),
        U::Diff(a, b) => format!("u64::try_from({}).unwrap()", diff_text(a, b)),
        U::DiffOk(a, b, d, v) => format!(
            "(match u64::try_from({}).ok() {{ None => {}, Some({v}) => {v} }})",
            diff_text(a, b),
            text(d)
        ),
        U::ViaU8(a) => format!("u64::try_from(u8::try_from({}).unwrap()).unwrap()", text(a)),
        U::Byte(op, a, b) => {
            let low = |x: &U| format!("u8::try_from({} % 256).unwrap()", text(x));
            let (a, b) = (low(a), low(b));
            let e = match op {
                Op::Add => format!("({a} + {b})"),
                Op::WrapMul => format!("{a}.wrapping_mul({b})"),
                _ => format!("{a}.saturating_mul({b})"),
            };
            format!("u64::try_from({e}).unwrap()")
        }
    }
}

fn op_name(op: Op) -> &'static str {
    match op {
        Op::Add => "add",
        Op::Sub => "sub",
        Op::Mul => "mul",
        Op::Div => "div",
        Op::Rem => "rem",
        _ => unreachable!("not a checked or saturating operation"),
    }
}

fn diff_text(a: &U, b: &U) -> String {
    format!(
        "i64::try_from({}).unwrap() - i64::try_from({}).unwrap()",
        text(a),
        text(b)
    )
}

/// `a - b` as `i64`: a fault when either is above `i64::MAX` (the
/// difference of two non-negative `i64` cannot overflow).
fn diff(a: u64, b: u64) -> Result<i64, Fault> {
    let a = i64::try_from(a).map_err(|_| Fault)?;
    let b = i64::try_from(b).map_err(|_| Fault)?;
    Ok(a - b)
}

fn eval(u: &U, x: u64, y: u64) -> Result<u64, Fault> {
    Ok(match u {
        U::Var("x") => x,
        U::Var(_) => y,
        U::Lit(v) => *v,
        U::Bin(op, a, b) => {
            let (a, b) = (eval(a, x, y)?, eval(b, x, y)?);
            match op {
                Op::Add => a.checked_add(b).ok_or(Fault)?,
                Op::Sub => a.checked_sub(b).ok_or(Fault)?,
                Op::Mul => a.checked_mul(b).ok_or(Fault)?,
                Op::Div => a.checked_div(b).ok_or(Fault)?,
                Op::Rem => a.checked_rem(b).ok_or(Fault)?,
                Op::WrapAdd => a.wrapping_add(b),
                Op::WrapSub => a.wrapping_sub(b),
                Op::WrapMul => a.wrapping_mul(b),
            }
        }
        U::IfLt(a, b, t, e) => {
            if eval(a, x, y)? < eval(b, x, y)? {
                eval(t, x, y)?
            } else {
                eval(e, x, y)?
            }
        }
        U::Sat(op, a, b) => {
            let (a, b) = (eval(a, x, y)?, eval(b, x, y)?);
            match op {
                Op::Add => a.saturating_add(b),
                Op::Sub => a.saturating_sub(b),
                _ => a.saturating_mul(b),
            }
        }
        U::Checked(op, a, b, d, _) => {
            let (a, b) = (eval(a, x, y)?, eval(b, x, y)?);
            let r = match op {
                Op::Add => a.checked_add(b),
                Op::Sub => a.checked_sub(b),
                Op::Mul => a.checked_mul(b),
                Op::Div => a.checked_div(b),
                _ => a.checked_rem(b),
            };
            match r {
                Some(v) => v,
                None => eval(d, x, y)?,
            }
        }
        U::ViaI64(a) => {
            let v = i64::try_from(eval(a, x, y)?).map_err(|_| Fault)?;
            u64::try_from(v).map_err(|_| Fault)?
        }
        U::ViaUsize(a) => eval(a, x, y)?,
        U::Diff(a, b) => {
            let d = diff(eval(a, x, y)?, eval(b, x, y)?)?;
            u64::try_from(d).map_err(|_| Fault)?
        }
        U::DiffOk(a, b, d, _) => match u64::try_from(diff(eval(a, x, y)?, eval(b, x, y)?)?) {
            Ok(v) => v,
            Err(_) => eval(d, x, y)?,
        },
        U::ViaU8(a) => u64::from(u8::try_from(eval(a, x, y)?).map_err(|_| Fault)?),
        U::Byte(op, a, b) => {
            let (a, b) = ((eval(a, x, y)? % 256) as u8, (eval(b, x, y)? % 256) as u8);
            u64::from(match op {
                Op::Add => a.checked_add(b).ok_or(Fault)?,
                Op::WrapMul => a.wrapping_mul(b),
                _ => a.saturating_mul(b),
            })
        }
    })
}

/// Conjunctions of `lo <= r <= hi`, `r <= x`, `r <= y`, `r >= x`.
#[derive(Clone, Copy)]
enum Post {
    In(u64, u64),
    LeX,
    LeY,
    GeX,
}

fn post_text(p: &[Post]) -> String {
    p.iter()
        .map(|c| match c {
            Post::In(lo, hi) => format!("{lo} <= r && r <= {hi}"),
            Post::LeX => "r <= x".into(),
            Post::LeY => "r <= y".into(),
            Post::GeX => "r >= x".into(),
        })
        .collect::<Vec<_>>()
        .join(" && ")
}

fn post_holds(p: &[Post], x: u64, y: u64, r: u64) -> bool {
    p.iter().all(|c| match c {
        Post::In(lo, hi) => *lo <= r && r <= *hi,
        Post::LeX => r <= x,
        Post::LeY => r <= y,
        Post::GeX => r >= x,
    })
}

struct Func {
    name: String,
    lo: (u64, u64),
    hi: (u64, u64),
    body: U,
    post: Vec<Post>,
}

impl Func {
    fn pre(&self, x: u64, y: u64) -> bool {
        self.lo.0 <= x && x <= self.hi.0 && self.lo.1 <= y && y <= self.hi.1
    }

    fn text(&self) -> String {
        let n = &self.name;
        format!(
            "    spec fn {n}_pre(x: u64, y: u64) -> bool {{ {} <= x && x <= {} && {} <= y && y <= {} }}\n\n    spec fn {n}_post(x: u64, y: u64, r: u64) -> bool {{ {} }}\n\n    #[effects(pure)]\n    #[uses({n}_pre, {n}_post)]\n    fn {n}(x: u64, y: u64) -> (r: u64)\n        requires {n}_pre(x, y),\n        ensures {n}_post(x, y, r),\n    {{\n        {}\n    }}\n\n",
            self.lo.0,
            self.hi.0,
            self.lo.1,
            self.hi.1,
            post_text(&self.post),
            text(&self.body)
        )
    }

    fn inputs(&self, rng: &mut Rng) -> Vec<(u64, u64)> {
        let near = |lo: u64, hi: u64, rng: &mut Rng| match rng.below(4) {
            0 => lo,
            1 => hi,
            _ => lo + rng.next() % (hi - lo).saturating_add(1).max(1),
        };
        let mut out = Vec::new();
        for _ in 0..30 {
            let p = (
                near(self.lo.0, self.hi.0, rng),
                near(self.lo.1, self.hi.1, rng),
            );
            if self.pre(p.0, p.1) && !out.contains(&p) {
                out.push(p);
            }
            if out.len() == 6 {
                break;
            }
        }
        out
    }
}

fn gen_func(rng: &mut Rng, k: usize) -> Func {
    let range = |rng: &mut Rng| {
        let (a, b) = (value(rng), value(rng));
        (a.min(b), a.max(b))
    };
    let (lx, hx) = range(rng);
    let (ly, hy) = range(rng);
    let body = gen_u(rng, 3);
    let mut post = Vec::new();
    for c in [Post::LeX, Post::LeY, Post::GeX] {
        if rng.chance(20) {
            post.push(c);
        }
    }
    if post.is_empty() || rng.chance(40) {
        let (a, b) = range(rng);
        post.push(Post::In(a, b));
    }
    Func {
        name: format!("u{k}"),
        lo: (lx, ly),
        hi: (hx, hy),
        body,
        post,
    }
}

/// `name = value` pairs after "counterexample: ".
fn parse_cex(msg: &str) -> Option<(u64, u64)> {
    let rest = msg.split("; counterexample: ").nth(1)?;
    let mut x = None;
    let mut y = None;
    for kv in rest.split(", ") {
        let (k, v) = kv.split_once(" = ")?;
        let v: u64 = v.trim().parse().ok()?;
        match k {
            "x" => x = Some(v),
            "y" => y = Some(v),
            _ => {}
        }
    }
    Some((x?, y?))
}

#[derive(Default)]
struct Stats {
    funcs: usize,
    verified: usize,
    /// Verified functions with an ADR-0028 or ADR-0044 form, and with a `u8` one.
    verified_adr28: usize,
    verified_u8: usize,
    runs: usize,
    cexes: usize,
    compiled_runs: usize,
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let mut funcs: Vec<Func> = (0..8).map(|k| gen_func(&mut rng, k)).collect();
    // One function per batch is a bare byte operation on `x` and `y` with a
    // postcondition that always holds, so the `u8` range rule is checked
    // directly (ADR-0044 §1).
    let op = [Op::Add, Op::WrapMul, Op::Mul][rng.below(3)];
    funcs[0].body = U::Byte(op, Box::new(U::Var("x")), Box::new(U::Var("y")));
    // Its verdict rests on the obligations alone.
    funcs[0].post = vec![Post::In(0, u64::MAX)];
    let mut src = String::from("#[summary(\"부호 없는 산술 건전성 퍼징\")]\nmod m {\n");
    for f in &funcs {
        src.push_str(&f.text());
    }
    // `main` prints each result (`print_u64`, ADR-0028 §4).
    src.push_str(
        "}\n\n#[summary(\"실행\")]\nmod app {\n    #[effects(io)]\n    fn main() -> () {\n",
    );
    let mut expected = String::new();
    for f in &funcs {
        for (x, y) in f.inputs(&mut rng) {
            if let Ok(r) = eval(&f.body, x, y) {
                let _ = writeln!(src, "        print_u64(m::{}({x}, {y}));", f.name);
                let _ = writeln!(expected, "{r}");
            }
        }
    }
    src.push_str("    }\n}\n");
    let files = vec![("u.bmb".to_string(), src.clone())];
    let built = bmb::emit_llvm_sources(&files);
    let out = &built.verify;
    let ctx = |what: &str| format!("seed {seed}: {what}\n{src}\n{}", out.output);
    assert!(
        built.ir.is_some(),
        "{}",
        ctx("generated program must check")
    );

    for line in out.output.lines() {
        let Some(name) = field(line, "def").and_then(|d| d.strip_prefix("m::")) else {
            continue;
        };
        let Some(f) = funcs.iter().find(|f| f.name == name) else {
            continue;
        };
        stats.funcs += 1;
        if field(line, "status") != Some("verified") {
            continue;
        }
        stats.verified += 1;
        stats.verified_adr28 += usize::from(adr28(&f.body));
        stats.verified_u8 += usize::from(bytes(&f.body));
        for (x, y) in f.inputs(&mut rng) {
            stats.runs += 1;
            match eval(&f.body, x, y) {
                Err(Fault) => panic!(
                    "{}",
                    ctx(&format!("verified {name} faults at x = {x}, y = {y}"))
                ),
                Ok(r) => assert!(
                    post_holds(&f.post, x, y, r),
                    "{}",
                    ctx(&format!(
                        "verified {name}({x}, {y}) = {r} breaks its postcondition"
                    ))
                ),
            }
        }
    }

    for line in out.output.lines() {
        let Some(msg) = field(line, "message") else {
            continue;
        };
        let Some(f) = funcs
            .iter()
            .find(|f| msg.contains(&format!("in `m::{}`", f.name)))
        else {
            continue;
        };
        let Some((x, y)) = parse_cex(msg) else {
            continue;
        };
        stats.cexes += 1;
        assert!(
            f.pre(x, y),
            "{}",
            ctx(&format!("counterexample breaks the precondition: {msg}"))
        );
        let broken = match eval(&f.body, x, y) {
            Err(Fault) => true,
            Ok(r) => !post_holds(&f.post, x, y, r),
        };
        assert!(
            broken,
            "{}",
            ctx(&format!(
                "counterexample does not break {name}: {msg}",
                name = f.name
            ))
        );
    }
    let n = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            bmb::emit_llvm_sources_with(&files, true).ir.unwrap()
        } else {
            built.ir.clone().unwrap()
        };
        let exe = dir.join(format!("u{seed}-{all_checks}-{}", std::process::id()));
        bmb::toolchain::build_executable(&ir, exe.to_str().unwrap())
            .unwrap_or_else(|e| panic!("{}", ctx(&e)));
        let res = Command::new(&exe).output().unwrap();
        let _ = fs::remove_file(&exe);
        assert!(
            res.status.success(),
            "{}",
            ctx(&format!(
                "build (all_checks = {all_checks}) exited with {}",
                res.status
            ))
        );
        assert_eq!(
            String::from_utf8_lossy(&res.stdout),
            expected,
            "{}",
            ctx(&format!(
                "build (all_checks = {all_checks}) disagrees with the interpreter"
            ))
        );
        stats.compiled_runs += n;
    }
}

#[test]
fn unsigned_proofs_counterexamples_and_builds_agree_with_the_interpreter() {
    if std::env::var("BMB_Z3_RLIMIT").is_err() {
        // SAFETY: set before any thread of this test binary reads the
        // environment; this file holds a single test.
        unsafe { std::env::set_var("BMB_Z3_RLIMIT", "500000") };
    }
    let iters: u64 = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let start: u64 = std::env::var("BMB_SOUND_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let dir = std::env::temp_dir();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        batch(seed, &mut stats, &dir);
    }
    eprintln!(
        "soundness (unsigned): {} functions, {} verified ({} with ADR-0028/0044 forms, {} with u8, {} runs), {} counterexamples replayed, {} compiled runs",
        stats.funcs,
        stats.verified,
        stats.verified_adr28,
        stats.verified_u8,
        stats.runs,
        stats.cexes,
        stats.compiled_runs
    );
    assert!(stats.verified > 0 && stats.cexes > 0);
}
