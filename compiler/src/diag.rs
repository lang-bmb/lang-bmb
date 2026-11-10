//! Structured diagnostics with stable codes. Output is JSONL, one object per line,
//! sorted so the same input always produces the same bytes.

use crate::source::{SourceMap, Span};

/// Every diagnostic code the compiler can emit. Codes are stable: never renumber,
/// never reuse a retired code.
pub const CODES: &[(&str, &str)] = &[
    // E01xx: lexing and syntax
    ("E0101", "unexpected character"),
    ("E0102", "unterminated string literal"),
    ("E0103", "syntax error: unexpected token"),
    (
        "E0104",
        "integer literal suffix not allowed; the type comes from context",
    ),
    ("E0105", "integer literal too large"),
    ("E0106", "block comments are not allowed; use `//`"),
    ("E0107", "nesting too deep"),
    (
        "E0108",
        "malformed integer literal: a digit outside its base, or a base prefix without digits",
    ),
    (
        "E0109",
        "malformed float literal: digits on both sides of the point (`1.0`, `0.5`) and after an exponent `e` are required",
    ),
    // E02xx: Rust habits rejected by BMB (each carries a fix when mechanical)
    ("E0201", "`let` requires an explicit type"),
    ("E0202", "macros are not allowed"),
    (
        "E0205",
        "`as` converts between integer types only (to `f64`: `f64::from(x)`)",
    ),
    ("E0207", "construct not supported in milestone 1"),
    ("E0208", "lifetime annotations are not allowed"),
    (
        "E0209",
        "function return type is required (write `-> ()` for unit)",
    ),
    ("E0210", "`mut` parameters are not allowed"),
    ("E0211", "comparison operators cannot be chained"),
    (
        "E0212",
        "`T::from(x)` / `x.into()` / `x.try_into()`: convert with `x as T`",
    ),
    (
        "E0213",
        "`T::try_from(x).expect(..)` is written `.unwrap()`",
    ),
    ("E0214", "`use` is not used: names are written in full"),
    // E0218-E0219: Rust habits around `Vec` (ADR-0031 §7) and `String`
    (
        "E0218",
        "a `Vec` or `String` is passed where a slice or `&str` is expected: write `v.as_slice()` or `s.as_str()`",
    ),
    ("E0219", "`drop(v)` and `mem::take` are not used"),
    (
        "E0215",
        "`unsafe` is not used: a body that cannot be proved is `#[external_body]`",
    ),
    (
        "E0216",
        "Verus's `#[verifier::external_body]`: BMB writes `#[external_body]`",
    ),
    (
        "E0217",
        "Verus's `admit()`: BMB writes `assume(p(..))` with a rationale",
    ),
    ("E0235", "a blanket impl `impl<T> Trait for T` is not used"),
    (
        "E0240",
        "a `where` clause is not used: bounds go next to the type parameter",
    ),
    ("E0236", "trait objects `dyn Trait` are not used"),
    (
        "E0241",
        "`?` whose `None`/`Err` the function cannot return (another error type, or not an `Option`/`Result` function)",
    ),
    (
        "E0242",
        "`crate::`, `super::` or `self::` path: a path names a unit from the root",
    ),
    ("E0250", "`break` or `continue` outside a loop"),
    ("E0251", "`break` with a value outside a `loop`"),
    ("E0252", "`for` over something other than an integer range"),
    (
        "E0253",
        "`decreases` on a `for` loop (its measure is automatic)",
    ),
    // E0254-E0255: type invariants (ADR-0060 §4)
    (
        "E0254",
        "`#[invariant(p)]`: `p` is not a spec function `(x: &T) -> bool` over the type's parameters, or states nothing",
    ),
    (
        "E0255",
        "more than one `#[invariant(..)]` on a type: join the conditions with `&&` in one predicate",
    ),
    // E0256-E0258: proof functions and ghost code (ADR-0062 §4)
    (
        "E0256",
        "a `proof fn` takes no effect attribute and no return type",
    ),
    (
        "E0257",
        "ghost code (a `proof fn` body or a `proof { .. }` block) cannot run executable code: no executable call, assignment, `&mut` or loop",
    ),
    (
        "E0258",
        "a `proof fn` is called only from ghost code: a `proof { .. }` block or another `proof fn`",
    ),
    (
        "E0259",
        "a `const`'s value is not a constant expression: literals, other `const`s, operators, `if`, `match`, the wrapping and saturating methods, `as`, arrays, structs, variants, fields and elements (ADR-0052 §3)",
    ),
    (
        "E0260",
        "a `const`'s evaluation failed: overflow, division by zero, a shift of the width or more, a conversion out of range, an index past the end (ADR-0052 §4)",
    ),
    (
        "E0262",
        "a closure that captures a local is written `move`: captures move into it (ADR-0053 §2)",
    ),
    (
        "E0263",
        "a closure's parameters and result have their types written (ADR-0053 §2)",
    ),
    (
        "E0264",
        "a function pointer type, a returned closure, or a closure that is not the argument of an `Fn`-bounded parameter (ADR-0053 §1, §3)",
    ),
    (
        "E0265",
        "an argument-position `impl Fn(..)`: the form is a type parameter `F: Fn(..)` (ADR-0053 §1)",
    ),
    (
        "E0261",
        "a `static` item: an immutable one is a `const`, global mutable state is passed as an argument (ADR-0052 §7)",
    ),
    (
        "E0237",
        "a call of a generic function names its type arguments: `f::<T>(..)`",
    ),
    (
        "E0238",
        "`Rc`, `RefCell`, `Cell` and `Arc` are not used: values have one owner",
    ),
    (
        "E0239",
        "an owned value is moved out from behind a `&mut`: use `std::mem::replace`",
    ),
    // E0220-E0233: Rust habits around data types (ADR-0026 §5)
    ("E0220", "`#[derive(..)]` is not used"),
    (
        "E0221",
        "built-in variants are written bare (`Some`, not `Option::Some`)",
    ),
    ("E0222", "field shorthand is not allowed (`x: x`)"),
    ("E0223", "struct update syntax `..` is not allowed"),
    ("E0224", "a wildcard arm is not allowed"),
    ("E0225", "`if let` is not allowed; use `match`"),
    (
        "E0226",
        "`while let`, `let .. else` and pattern `let` are not allowed",
    ),
    ("E0228", "the receiver needs its type (`self: &T`)"),
    (
        "E0229",
        "`.unwrap()` / `.expect()` are not allowed; `match` both cases",
    ),
    ("E0230", "destructuring `let` is not allowed"),
    ("E0231", "`(*p).x` is written `p.x`"),
    (
        "E0232",
        "pattern form not allowed (guard, or, literal, nested, struct)",
    ),
    (
        "E0233",
        "tuples, tuple structs and struct-like variants are not allowed",
    ),
    // E03xx: names
    ("E0301", "unknown name"),
    ("E0302", "duplicate definition"),
    (
        "E0303",
        "a local name hides a visible one (a parameter, an outer local, the result), or is bound elsewhere in the function with another type (ADR-0095 A3)",
    ),
    ("E0304", "unknown function"),
    ("E0305", "unknown type or trait"),
    ("E0306", "unknown field"),
    ("E0307", "unknown variant"),
    (
        "E0308",
        "`impl` block not in the unit of its type, or a second one with the same bounds (one block per list of bounds, ADR-0097)",
    ),
    ("E0309", "a built-in name cannot be defined"),
    ("E0310", "`self` names exactly the receiver"),
    (
        "E0311",
        "two impls of a trait for types that can be the same",
    ),
    (
        "E0312",
        "a trait impl in neither the unit of the trait nor the unit of the type",
    ),
    (
        "E0313",
        "a method call names a method of more than one trait; call by path",
    ),
    (
        "E0314",
        "a method call names a function that has no receiver or type parameters of its own; call by path",
    ),
    (
        "E0315",
        "a trait impl misses a method of the trait, or has one the trait lacks",
    ),
    (
        "E0316",
        "a trait impl's function does not match the trait's signature",
    ),
    (
        "E0317",
        "a `break` or `continue` names a label no loop around it has",
    ),
    ("E0318", "`const`s that refer to each other (ADR-0052 §3)"),
    // E04xx: types and ownership
    ("E0401", "type mismatch"),
    ("E0402", "unsupported type"),
    ("E0403", "references are only allowed in parameter position"),
    (
        "E0404",
        "unknown method (no method lookup; only built-in methods exist)",
    ),
    ("E0405", "wrong number of arguments"),
    ("E0406", "integer literal out of range for its type"),
    ("E0407", "value is not indexable"),
    ("E0408", "assignment to an immutable place"),
    (
        "E0409",
        "`&` / `&mut` may only appear directly as a call argument",
    ),
    (
        "E0410",
        "a `&mut` argument aliases another argument of the same call",
    ),
    ("E0411", "cannot negate an unsigned value"),
    (
        "E0412",
        "`T::try_from(x)` takes an integer and is used as `.unwrap()` or `.ok()`",
    ),
    ("E0413", "use of a moved value, or a move out of a place"),
    (
        "E0414",
        "a value that moves is moved on some paths and not on others",
    ),
    (
        "E0420",
        "malformed data type: wrong type arguments, or no field or variant",
    ),
    ("E0421", "a type contains itself"),
    (
        "E0422",
        "struct literal fields: not a struct, a field given twice, or a field missing",
    ),
    ("E0423", "`match` needs an enum value"),
    (
        "E0424",
        "the type of `None` / `Ok` / `Err` is not known here",
    ),
    (
        "E0425",
        "`==` on data types is not executable (compare the fields)",
    ),
    (
        "E0426",
        "`match` arms: one per variant, in declaration order",
    ),
    (
        "E0427",
        "a pattern names the wrong number of payload values",
    ),
    (
        "E0428",
        "a type argument does not satisfy a bound of its parameter",
    ),
    ("E0429", "wrong number of type arguments in a call"),
    (
        "E0430",
        "a recursive generic call wraps a type parameter in a larger type",
    ),
    (
        "E0431",
        "`==` on a recursive type that holds a `Vec` (in a specification)",
    ),
    (
        "E0432",
        "a value of a recursive struct is built in a specification",
    ),
    (
        "E0433",
        "a value is used whole after one of its fields was moved",
    ),
    (
        "E0434",
        "struct literal fields must be given in declaration order (the order of evaluation)",
    ),
    (
        "E0435",
        "a range `lo..hi` outside the source argument of `copy_within` and the bounds of a `&str` argument `&s[lo..hi]`",
    ),
    (
        "E0436",
        "a `std::process::Child` built or read by its fields (it comes from `std::process::spawn`)",
    ),
    (
        "E0437",
        "a type a `const` cannot have: a `const` holds a value type, an integer, `bool` or `char`, or an array, struct or enum of them (ADR-0052 §2)",
    ),
    // E05xx: specifications
    (
        "E0501",
        "a contract clause with a quantifier must be a call to a named predicate (`spec fn ... -> bool`); one without is an expression (ADR-0095 A2)",
    ),
    ("E0502", "vacuous predicate used in a contract"),
    (
        "E0503",
        "specification-only construct used in executable code",
    ),
    ("E0504", "executable function called in a specification"),
    ("E0505", "spec function called in executable code"),
    ("E0506", "`while` requires `decreases`"),
    ("E0507", "recursive function requires `decreases`"),
    (
        "E0513",
        "conversions are not used in specifications (integer types mix there)",
    ),
    ("E0514", "`==` is not defined for `Vec`"),
    (
        "E0515",
        "`assert(..) by (bit_vector)` takes integer, `f64` and `bool` terms only (no quantifier, `old` or data value; an element, field or length that names a `let` of the assertion)",
    ),
    (
        "E0516",
        "unknown prover in `assert(..) by (..)` (the one prover is `bit_vector`)",
    ),
    (
        "E0508",
        "`old(..)` is only allowed in `ensures`, loop invariants and `assert` on a `&mut` parameter",
    ),
    ("E0509", "`#[uses(..)]` entry is not a spec function"),
    (
        "E0510",
        "`#[trigger]` is only allowed inside `forall` or `exists`",
    ),
    (
        "E0511",
        "`forall` and `exists` require at least one `#[trigger]`",
    ),
    (
        "E0512",
        "the triggers of a `forall` or `exists` must together mention every bound variable",
    ),
    // E06xx: effects and attributes
    ("E0601", "function requires an `#[effects(..)]` attribute"),
    ("E0602", "unknown effect"),
    ("E0603", "`pure` function calls a function with effect `io`"),
    ("E0604", "spec functions take no `#[effects(..)]` attribute"),
    ("E0605", "unknown or misplaced attribute"),
    (
        "E0606",
        "the body needs an effect the function does not declare",
    ),
    (
        "E0607",
        "`pure` listed with another effect, or an effect listed twice",
    ),
    (
        "E0608",
        "a function value whose effects the generic function that calls it does not declare (ADR-0053 §5)",
    ),
    // E07xx: program structure
    ("E0701", "`main` must be `#[effects(io)] fn main() -> ()`"),
    ("E0702", "more than one `main`"),
    ("E0703", "items must be declared inside a `mod`"),
    ("E0704", "`mod` requires a `#[summary(\"..\")]` attribute"),
    // E08xx: proof gate (a failed or undecided proof obligation)
    ("E0801", "precondition of a call may not hold"),
    ("E0802", "postcondition may not hold"),
    ("E0803", "arithmetic may overflow"),
    ("E0804", "divisor may be zero"),
    ("E0805", "index may be out of bounds"),
    ("E0806", "loop invariant may not hold on entry"),
    ("E0807", "loop invariant may not be preserved"),
    (
        "E0808",
        "loop `decreases` may not decrease or may be negative",
    ),
    ("E0809", "recursive call may not decrease the measure"),
    ("E0810", "assertion may not hold"),
    (
        "E0811",
        "solver could not decide (resource limit or incompleteness)",
    ),
    (
        "E0812",
        "proof gate cannot run (solver missing, wrong version or failed)",
    ),
    ("E0813", "recursive spec function may not terminate"),
    (
        "E0815",
        "integer conversion may be out of range (to `f64`: a 64-bit value may be above 2^53 and inexact)",
    ),
    ("E0816", "a `Vec` may exceed its maximum length"),
    (
        "E0817",
        "a trait impl's function does not refine the trait's contract",
    ),
    (
        "E0818",
        "a `by (bit_vector)` sub-proof may not hold, or its assertion has no bit-vector translation",
    ),
    (
        "E0819",
        "an argument of a `proof fn` call may be outside its parameter's type (ghost arithmetic is mathematical, ADR-0062 §2)",
    ),
    (
        "E0820",
        "a string position may not be a character boundary (`&s[a..b]` needs `s.is_char_boundary(a)`, `s.is_char_boundary(b)` and `a <= b`, ADR-0050 §2)",
    ),
    (
        "E0823",
        "a type invariant may not hold where a value leaves the function's hands: a call argument, a return, a `&mut` parameter at exit, a loop head (ADR-0060 §2)",
    ),
    // E10xx: escapes and rationales (ADR-0029)
    (
        "E1001",
        "an escape (`assume`, `#[external_body]`) without a `#[rationale(..)]` before it",
    ),
    (
        "E1002",
        "a malformed `#[rationale(..)]`: `kind` outside the list, no explanation, or not before an escape",
    ),
    (
        "E1003",
        "`assume` takes one named predicate call, in executable code",
    ),
    (
        "E1004",
        "`#[external_body]` on a spec fn, a proof fn or `main`",
    ),
    // W10xx: escapes (`bmb lint` only)
    (
        "W1001",
        "an unconfirmed rationale: what it is bound to changed, or it carries no hash (`bmb fix --confirm`)",
    ),
    (
        "W1002",
        "an `assume` the proof does not need: the definition is proved without it",
    ),
    // W09xx: contract adjustment (`bmb lint` and `bmb weaken` only; severity "warning", never from `check`/`verify`)
    (
        "W0901",
        "preconditions are unsatisfiable together: the function can never be called and its proofs are vacuous",
    ),
    (
        "W0902",
        "precondition is not needed by this function's proof",
    ),
    ("W0903", "postcondition is not used by any caller's proof"),
    (
        "W0904",
        "the specification does not distinguish this change to the body (the mutant still verifies)",
    ),
    (
        "W0905",
        "a point of a verified function is unreachable in its proof: what follows is proved vacuously",
    ),
    (
        "W0906",
        "a quantifier's only trigger matches a term of its own body: instances feed each other (matching loop)",
    ),
    (
        "W0907",
        "a counting `while` whose canonical form is a range `for` (the fix rewrites it; `bmb fix` applies it)",
    ),
    (
        "W0909",
        "an executable function declares an effect its body does not need (callers must declare it too)",
    ),
    (
        "W0912",
        "a clause keyword repeated (`invariant a, invariant b,`): the canonical form lists the clauses after one keyword (an input alias, ADR-0095; `bmb fmt` and `bmb fix` rewrite it)",
    ),
    (
        "W0913",
        "loop clauses at the start of the body (`while c { invariant a, .. }`): the canonical form puts them before the body (an input alias, ADR-0095; `bmb fmt` and `bmb fix` rewrite it)",
    ),
    (
        "W0914",
        "a compound assignment `t op= e`: the canonical form is `t = t op e` (an input alias, backlog 2N-07, ADR-0095; `bmb fmt` and `bmb fix` rewrite it)",
    ),
    (
        "W0915",
        "an integer conversion written `T::try_from(x).unwrap()`: the canonical form is `x as T` (an input alias, ADR-0095 A4; `bmb fmt` and `bmb fix` rewrite it)",
    ),
    (
        "W0916",
        "a `#[uses(..)]` entry the default proof context reveals already (the spec fns the definition's own clauses, invariants, assertions and proof code and its callees' contracts name, ADR-0095 A1; `bmb fix` removes it)",
    ),
    (
        "W0917",
        "a widening integer conversion `x as T` in a specification: every value of `x` is one of `T`, so it is `x` (an input alias, ADR-0099; `bmb fix` removes it)",
    ),
    (
        "W0918",
        "the postconditions allow two different results for one input the preconditions admit (with the two results as a witness): the contract does not fix the result",
    ),
    (
        "W0911",
        "a call by path of a method whose normal form is the method call `x.f(..)` (the fix rewrites it; `bmb fmt` and `bmb fix` apply it)",
    ),
    (
        "W0910",
        "a loop of an `io` function prints and carries state whose every update is pure: the update can be a pure function, proved instead of tested",
    ),
    (
        "W0908",
        "the contract is weaker than in the old version: a stronger `requires`, a weaker `ensures` or `pure` become `io` (`bmb weaken`, with the counterexample)",
    ),
];

/// Severity of a code: `W` codes are warnings (reported only by `bmb lint` and `bmb weaken`),
/// every other code is an error.
pub fn severity(code: &str) -> &'static str {
    if code.starts_with('W') {
        "warning"
    } else {
        "error"
    }
}

pub fn describe(code: &str) -> &'static str {
    CODES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, d)| *d)
        .expect("unregistered diagnostic code")
}

#[derive(Debug, Clone)]
pub struct Fix {
    pub message: String,
    /// Replacement text for the diagnostic span, when the fix is mechanical.
    pub replacement: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Diag {
    pub code: &'static str,
    pub span: Span,
    pub message: String,
    pub fix: Option<Fix>,
    /// For E0811 (undecided): the code of the obligation the solver could
    /// not decide (`E0802` a postcondition, `E0807` an invariant kept, ..),
    /// as a field of its own (backlog 2D-32).
    pub obligation: Option<&'static str>,
    /// For E0811: the inputs' bounds under which a search found no
    /// counterexample (`|integer| <= b`, unbounded lengths `<= l`, ADR-0108
    /// §1). Not a proof.
    pub bounded: Option<(i64, usize)>,
}

impl Diag {
    pub fn new(code: &'static str, span: Span, message: impl Into<String>) -> Self {
        let _ = describe(code);
        Diag {
            code,
            span,
            message: message.into(),
            fix: None,
            obligation: None,
            bounded: None,
        }
    }
    /// The undecided obligation's own code (E0811's `obligation`).
    pub fn of_obligation(mut self, code: &'static str) -> Self {
        self.obligation = Some(code);
        self
    }
    pub fn fix(mut self, message: impl Into<String>, replacement: Option<String>) -> Self {
        self.fix = Some(Fix {
            message: message.into(),
            replacement,
        });
        self
    }
}

#[derive(Default, Clone)]
pub struct Diags {
    pub items: Vec<Diag>,
    /// Input aliases the parser took (ADR-0095, "the meaning of an input
    /// alias"): habit forms read as the canonical syntax tree, each with
    /// the replacement that writes the canonical form. They are not
    /// errors: `bmb lint` reports them and `bmb fix` applies them.
    pub aliases: Vec<Diag>,
}

impl Diags {
    pub fn push(&mut self, d: Diag) {
        self.items.push(d);
    }
    pub fn has_errors(&self) -> bool {
        !self.items.is_empty()
    }
    /// Deterministic JSONL rendering: sorted by position then code, duplicates removed.
    pub fn to_jsonl(&self, sm: &SourceMap) -> String {
        let mut rows: Vec<(u32, u32, &'static str, String)> = self
            .items
            .iter()
            .map(|d| (d.span.file, d.span.lo, d.code, render(d, sm)))
            .collect();
        rows.sort();
        rows.dedup_by(|a, b| a.3 == b.3);
        let mut out = String::new();
        for r in rows {
            out.push_str(&r.3);
            out.push('\n');
        }
        out
    }
}

fn render(d: &Diag, sm: &SourceMap) -> String {
    let (line, col) = sm.line_col(d.span.file, d.span.lo);
    let (eline, ecol) = sm.line_col(d.span.file, d.span.hi);
    let mut s = String::from("{");
    s.push_str(&format!("\"code\":{},", json_str(d.code)));
    s.push_str(&format!("\"severity\":\"{}\",", severity(d.code)));
    s.push_str(&format!("\"message\":{},", json_str(&d.message)));
    s.push_str(&format!("\"file\":{},", json_str(sm.name(d.span.file))));
    s.push_str(&format!(
        "\"line\":{line},\"col\":{col},\"end_line\":{eline},\"end_col\":{ecol}"
    ));
    if let Some(o) = d.obligation {
        s.push_str(&format!(",\"obligation\":{}", json_str(o)));
    }
    if let Some((b, l)) = d.bounded {
        s.push_str(&format!(",\"bounded\":{{\"b\":{b},\"l\":{l}}}"));
    }
    if let Some(f) = &d.fix {
        s.push_str(&format!(",\"fix\":{{\"message\":{}", json_str(&f.message)));
        if let Some(r) = &f.replacement {
            s.push_str(&format!(",\"replacement\":{}", json_str(r)));
        }
        s.push('}');
    }
    s.push('}');
    s
}

pub fn json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Codes no longer emitted. They are listed so they are never reused.
pub const RETIRED: &[(&str, &str)] = &[
    (
        "E0814",
        "construct not supported by the proof gate (retired by backlog 2V-05: every construct is encoded)",
    ),
    (
        "E0203",
        "`return` is not allowed (retired by backlog 2K-01a: `return` is part of the language, ADR-0049)",
    ),
    (
        "E0204",
        "`for` is not allowed (retired by backlog 2K-01d: the range `for` is part of the language, ADR-0049)",
    ),
    (
        "E0206",
        "the `?` operator is not allowed (retired by backlog 2K-01e: `?` is part of the language, ADR-0049)",
    ),
    (
        "E0234",
        "a string literal: text is bytes, write `b\"...\"` (retired by backlog 2U-01: `\"...\"` is a `&str`, ADR-0050)",
    ),
    (
        "E0227",
        "method calls are not allowed; call by path (retired by backlog 2M-01: `x.f(..)` calls a method, ADR-0051)",
    ),
];

pub fn codes_jsonl() -> String {
    let mut out = String::new();
    for (c, d) in CODES {
        out.push_str(&format!(
            "{{\"code\":{},\"severity\":\"{}\",\"description\":{}}}\n",
            json_str(c),
            severity(c),
            json_str(d)
        ));
    }
    out
}

/// The diagnostics reference page (`docs/diagnostics.md`), generated from
/// `CODES` so it cannot drift (backlog 1A-08).
pub fn codes_markdown() -> String {
    let groups = [
        ("E01", "어휘·구문"),
        ("E02", "Rust 습관(수정 제안 포함)"),
        ("E03", "이름"),
        ("E04", "타입·소유권"),
        ("E05", "명세"),
        ("E06", "효과·속성"),
        ("E07", "프로그램 구조"),
        ("E08", "증명 게이트"),
        ("W09", "계약 조정(경고, `bmb lint`·`bmb weaken`만)"),
        ("E10", "탈출구와 근거(ADR-0029)"),
        ("W10", "탈출구와 근거(경고, `bmb lint`만)"),
    ];
    let mut out = String::from(
        "# 진단 코드\n\n> 생성 파일: `bmb codes --md`가 `compiler/src/diag.rs`의 `CODES`에서 만든다. 손으로 고치지 않는다.\n> 코드는 바꾸거나 재사용하지 않는다. 진단 형식은 `compiler/README.md`.\n",
    );
    for (prefix, title) in groups {
        out.push_str(&format!(
            "\n## {prefix}xx {title}\n\n| 코드 | 뜻 |\n| --- | --- |\n"
        ));
        for (c, d) in CODES.iter().filter(|(c, _)| c.starts_with(prefix)) {
            out.push_str(&format!("| {c} | {} |\n", d.replace('|', "\\|")));
        }
    }
    out.push_str("\n## 은퇴한 코드(다시 쓰지 않는다)\n\n| 코드 | 뜻 |\n| --- | --- |\n");
    for (c, d) in RETIRED {
        out.push_str(&format!("| {c} | {} |\n", d.replace('|', "\\|")));
    }
    out
}
