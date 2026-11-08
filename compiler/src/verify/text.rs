//! Text in the prover (ADR-0050 §3, backlog 2U-02).
//!
//! A `String` or `&str` is its bytes, the pair of a byte array and a length
//! (`Tables::lowered`), and a `char` its scalar value. The UTF-8 invariant
//! holds by construction: every way to make text is a built-in that keeps
//! it (a literal, `String::new`, `push_str` of text, `push` of a `char`,
//! `from_utf8` after its check, a slice at character boundaries), so it is
//! no obligation at those points. It is a fact where a boundary is asked
//! (backlog 2U-06): a slice's obligation assumes what UTF-8 holds of the
//! bytes just before each bound, of the sliced text and of every text the
//! definition received (parameters, call results), and at the end of each
//! received text (`utf8_window`), without a quantifier. So a boundary
//! after an ASCII byte, after a whole character or before a first byte is
//! proved, and a refuted one has a counterexample of valid UTF-8. A
//! quantified fact over the whole text was tried first and left refuted
//! boundaries without a model, or with one at half the limit.
//!
//! What text adds to the bytes:
//! - `&s[a..b]` is an obligation (E0820): `a <= b` and both are character
//!   boundaries of `s` (Rust's `is_char_boundary`: `0`, the length, or a
//!   byte below the length that does not continue a character). The value
//!   is the shifted array `(bmb_shift a lo)`, whose one axiom (`text_decls`)
//!   a query holds only when it mentions it. (A `lambda` was tried first:
//!   Z3 gave up on queries with one, "incomplete (theory array)".)
//! - `s.push(c)` appends the UTF-8 bytes of `c` (one to four, by its
//!   value), `s.push_str(t)` the bytes of `t` (a fresh array with its
//!   quantified definition); both are obligations on the maximum length
//!   (E0816).
//! - `==` compares the bytes below the lengths (the slice equality of
//!   specifications); `<` and the other orders are Rust's: lexicographic on
//!   the bytes. In code they use the longest common prefix, a fresh constant
//!   with its defining facts; in specifications a quantified formula.
//! - `String::from_utf8(v)` is `Ok` or `Err` of the same bytes, by a fresh
//!   condition: no proof may rely on which. `char::from_u32(x)` is `Some(x)`
//!   exactly for the scalar values.

use super::*;

/// The declaration and axiom of `bmb_shift` when `text` mentions it (none
/// otherwise: a query without text slices is unchanged).
pub(super) fn text_decls(text: &str) -> &'static str {
    if text.contains("bmb_shift") {
        "(declare-fun bmb_shift ((Array Int Int) Int) (Array Int Int))\n(assert (forall ((a (Array Int Int)) (o Int) (k Int)) (! (= (select (bmb_shift a o) k) (select a (+ k o))) :pattern ((select (bmb_shift a o) k)))))\n"
    } else {
        ""
    }
}

/// Rust's `str::is_char_boundary(i)` of bytes `a` with length `n`.
pub fn boundary(a: &str, n: &str, i: &str) -> String {
    let b = format!("(select {a} {i})");
    format!("(or (= {i} 0) (= {i} {n}) (and (< 0 {i}) (< {i} {n}) (or (< {b} 128) (<= 192 {b}))))")
}

/// A Unicode scalar value: `0..=0xD7FF` or `0xE000..=0x10FFFF`.
pub fn scalar(c: &str) -> String {
    format!("(or (and (<= 0 {c}) (<= {c} 55295)) (and (<= 57344 {c}) (<= {c} 1114111)))")
}

/// `c.len_utf8()`.
pub fn utf8_len(c: &str) -> String {
    format!("(ite (< {c} 128) 1 (ite (< {c} 2048) 2 (ite (< {c} 65536) 3 4)))")
}

/// A continuation byte (`0x80..=0xBF`).
fn cont(x: &str) -> String {
    format!("(and (<= 128 {x}) (< {x} 192))")
}

/// What valid UTF-8 bytes `a` with length `n` hold at position `k`
/// (backlog 2U-06), when it is below the length: the byte is one Rust's
/// `str` can hold (not `0xC0`, `0xC1`, above `0xF4`); an ASCII byte or a
/// whole character ends there or before a byte that is not a
/// continuation; a first byte has its continuation bytes after it; a
/// continuation byte is not the first of four in a row.
pub fn utf8_at(a: &str, n: &str, k: &str) -> String {
    let at = |o: i64| match o {
        0 => format!("(select {a} {k})"),
        o => format!("(select {a} (+ {k} {o}))"),
    };
    let b = at(0);
    let ends = |m: i64| format!("(or (= (+ {k} {m}) {n}) (not {}))", cont(&at(m)));
    let run = |m: i64| {
        let mut parts = vec![format!("(< (+ {k} {}) {n})", m - 1)];
        for j in 1..m {
            parts.push(cont(&at(j)));
        }
        parts.push(ends(m));
        format!("(and {})", parts.join(" "))
    };
    let run4 = format!(
        "(=> (< (+ {k} 3) {n}) (not (and {} {} {})))",
        cont(&at(1)),
        cont(&at(2)),
        cont(&at(3))
    );
    format!(
        "(=> (and (<= 0 {k}) (< {k} {n})) (and (not (and (<= 192 {b}) (<= {b} 193))) (<= {b} 244) (=> (< {b} 128) {}) (=> (and (<= 194 {b}) (< {b} 224)) {}) (=> (and (<= 224 {b}) (< {b} 240)) {}) (=> (<= 240 {b}) {}) (=> {} {run4})))",
        ends(1),
        run(2),
        run(3),
        run(4),
        cont(&b)
    )
}

/// What valid UTF-8 bytes `a` with length `n` hold just before position
/// `i` (backlog 2U-06): the first byte begins a character, a continuation
/// byte at `i` has its first byte at most three bytes before it, and the
/// four bytes before `i` are as `utf8_at` has them. Enough to decide
/// whether `i` is a character boundary from the bytes before it: after an
/// ASCII byte, after a whole character, before a first byte. No
/// quantifier: a quantified form over the whole text left refuted
/// boundaries with no model or one at half the limit (the solver's
/// instances chain from byte to byte).
pub fn utf8_before(a: &str, n: &str, i: &str) -> String {
    let at = |o: i64| format!("(select {a} (- {i} {o}))");
    let mut parts = vec![
        format!("(=> (< 0 {n}) (not {}))", cont(&format!("(select {a} 0)"))),
        format!(
            "(=> (and (< 0 {i}) (< {i} {n}) {}) (or (and (<= 1 {i}) (<= 194 {})) (and (<= 2 {i}) (<= 194 {})) (and (<= 3 {i}) (<= 194 {}))))",
            cont(&format!("(select {a} {i})")),
            at(1),
            at(2),
            at(3)
        ),
    ];
    for j in 1..=4 {
        parts.push(utf8_at(a, n, &format!("(- {i} {j})")));
    }
    format!("(and {})", parts.join(" "))
}

fn parts(v: &Val) -> (String, String) {
    match v {
        Val::A { elems, len } => (elems.clone(), len.clone()),
        other => panic!("text value expected, found {other:?}"),
    }
}

impl Enc<'_, '_> {
    /// Records text value `v` as valid UTF-8 where it enters the
    /// definition: a parameter, a call's result, a `&mut` argument after
    /// the call (backlog 2U-06). No obligation goes with it: every way to
    /// make text keeps the invariant (ADR-0050 §3). What it holds is stated
    /// where a boundary is asked (`utf8_window`).
    pub(super) fn assume_text(&mut self, v: &Val) {
        if let Val::A { elems, len } = v
            && !self.texts.contains(&(elems.clone(), len.clone()))
        {
            self.texts.push((elems.clone(), len.clone()));
        }
    }

    /// What UTF-8 holds where boundary `i` of text `(a, n)` is asked
    /// (backlog 2U-06): the bytes before `i` (`utf8_before`) of that text
    /// and of every text known valid here (another text's bytes may equal
    /// these), and the end of each known text (its last character is
    /// whole: a prefix that matched it ends at a boundary).
    pub(super) fn utf8_window(&self, a: &str, n: &str, i: &str) -> String {
        let mut parts = vec![utf8_before(a, n, i)];
        for (b, m) in &self.texts {
            if (b.as_str(), m.as_str()) != (a, n) {
                parts.push(utf8_before(b, m, i));
            }
            parts.push(utf8_before(b, m, m));
        }
        format!("(and {})", parts.join(" "))
    }

    /// Whether parameter `i` (or the result, `None`) of function `path` is
    /// text (`Tables::text_sigs`).
    pub(super) fn text_at(&self, path: &str, i: Option<usize>) -> bool {
        self.prog
            .t
            .text_sigs
            .get(path)
            .is_some_and(|(ps, r)| match i {
                Some(i) => ps.get(i).copied().unwrap_or(false),
                None => *r,
            })
    }

    /// Whether `e`'s value is text (the checker's `Tables::text`).
    pub(super) fn is_text_expr(&self, e: &Expr) -> bool {
        self.prog
            .t
            .text
            .contains(&(e.span.file, e.span.lo, e.span.hi))
    }

    fn bound_name(&mut self, base: &str) -> String {
        let n = sym(&format!("{base}!{}", self.bound));
        self.bound += 1;
        n
    }

    /// The bytes of `v` from `lo` to `hi`: `&s[lo..hi]`.
    pub(super) fn sub_text(&mut self, v: &Val, lo: &str, hi: &str) -> Val {
        let (a, _) = parts(v);
        Val::A {
            elems: format!("(bmb_shift {a} {lo})"),
            len: format!("(- {hi} {lo})"),
        }
    }

    /// `&s[lo..hi]` in code: the boundary obligation (E0820) and the value.
    pub(super) fn str_slice(&mut self, e: &Expr, base: &Expr, lo: &Expr, hi: &Expr) -> Val {
        let v = self.eval(base);
        let (lo, hi) = (self.eval(lo).s(), self.eval(hi).s());
        let (a, n) = parts(&v);
        // The bytes before each bound, as UTF-8 has them (`utf8_window`),
        // for this obligation alone.
        let given = format!(
            "(and {} {})",
            self.utf8_window(&a, &n, &lo),
            self.utf8_window(&a, &n, &hi)
        );
        self.oblige_given(
            "E0820",
            e.span,
            "the range may not lie on character boundaries of the string",
            Some(given),
            format!(
                "(and (<= {lo} {hi}) {} {})",
                boundary(&a, &n, &lo),
                boundary(&a, &n, &hi)
            ),
        );
        self.sub_text(&v, &lo, &hi)
    }

    /// `&v[lo..hi]` of an array, `Vec` or slice of integers (backlog
    /// 4C-05): the bounds obligation (E0805, Rust's `slice index starts at
    /// .. but ends at ..` and `range end index .. out of range` panics)
    /// and the elements from `lo`, the same shifted array as text's.
    pub(super) fn elem_slice(&mut self, e: &Expr, base: &Expr, lo: &Expr, hi: &Expr) -> Val {
        let v = self.eval(base);
        let (lo, hi) = (self.eval(lo).s(), self.eval(hi).s());
        let (a, n) = parts(&v);
        self.oblige(
            "E0805",
            e.span,
            "the range may be out of bounds",
            format!("(and (<= {lo} {hi}) (<= {hi} {n}))"),
        );
        Val::A {
            elems: format!("(bmb_shift {a} {lo})"),
            len: format!("(- {hi} {lo})"),
        }
    }

    /// The bytes of `push(c)` for a `c` that is not a constant: an ASCII
    /// value is its own byte, and otherwise the bytes are fresh, a lead
    /// byte and continuation bytes. The exact encoding's `div` and `mod`
    /// of `c` made Z3 give up on queries that also hold the quantified
    /// facts of text ("incomplete (theory array)", `tests/run/escape.bmb`).
    fn utf8_classes(&mut self, c: &str) -> [String; 4] {
        let lead = self.fresh("utf8.lead", "Int");
        self.pc
            .push(format!("(and (<= 192 {lead}) (<= {lead} 244))"));
        let mut bs: [String; 4] = Default::default();
        bs[0] = format!("(ite (< {c} 128) {c} {lead})");
        for b in bs.iter_mut().skip(1) {
            let x = self.fresh("utf8.cont", "Int");
            self.pc.push(format!("(and (<= 128 {x}) (<= {x} 191))"));
            *b = x;
        }
        bs
    }

    /// `s.push_str(t)` and `s.push(c)` on a `String` place `recv`.
    pub(super) fn text_change(
        &mut self,
        e: &Expr,
        recv: &Expr,
        rv: Val,
        m: &str,
        args: &[Expr],
    ) -> Val {
        let (a, n) = parts(&rv);
        // The length obligation before the new bytes' facts: a query that
        // can be satisfied without them (a counterexample) stays without
        // their quantifier.
        let grown = |s: &mut Self, new_len: &str| {
            s.oblige(
                "E0816",
                e.span,
                format!("`{m}` may exceed the maximum length of a `String`"),
                format!("(<= {new_len} {LEN_MAX_BYTE})"),
            );
        };
        let (elems, new_len) = if m == "push_str" {
            let t = self.eval(&args[0]);
            let (b, tn) = parts(&t);
            grown(self, &format!("(+ {n} {tn})"));
            let new = self.fresh("push_str", "(Array Int Int)");
            let k = self.bound_name("cat");
            self.pc.push(format!(
                "(forall (({k} Int)) (! (= (select {new} {k}) (ite (< {k} {n}) (select {a} {k}) (select {b} (- {k} {n})))) :pattern ((select {new} {k}))))"
            ));
            (new, format!("(+ {n} {tn})"))
        } else {
            let c = self.eval(&args[0]).s();
            self.pc.push(scalar(&c));
            grown(self, &format!("(+ {n} {})", utf8_len(&c)));
            let bs = match numeral(&c)
                .and_then(|v| u32::try_from(v).ok())
                .and_then(char::from_u32)
            {
                Some(ch) => {
                    let mut buf = [0u8; 4];
                    let enc = ch.encode_utf8(&mut buf).as_bytes();
                    let mut bs: [String; 4] = Default::default();
                    for (k, x) in bs.iter_mut().enumerate() {
                        *x = enc.get(k).map_or("0".into(), u8::to_string);
                    }
                    bs
                }
                None => self.utf8_classes(&c),
            };
            let mut arr = a.clone();
            for (k, b) in bs.iter().enumerate() {
                arr = format!("(store {arr} (+ {n} {k}) {b})");
            }
            (arr, format!("(+ {n} {})", utf8_len(&c)))
        };
        let new = Val::A {
            elems,
            len: new_len,
        };
        let mut steps = Vec::new();
        let root = self.place(recv, &mut steps, false);
        let cur = self.vars[&root].clone();
        let nv = self.set_at(cur, &steps, new);
        self.bind_var(root, nv);
        Val::Unit
    }

    /// `l op r` on text in code (Rust's byte-lexicographic `Ord`): the
    /// longest common prefix `p` is a fresh constant with its defining
    /// facts, and every order follows from it.
    pub(super) fn text_cmp_code(&mut self, op: BinOp, l: &Val, r: &Val) -> String {
        let (a, la) = parts(l);
        let (b, lb) = parts(r);
        let p = self.fresh("lcp", "Int");
        let j = self.bound_name("lcp");
        let (ap, bp) = (format!("(select {a} {p})"), format!("(select {b} {p})"));
        self.pc.push(format!("(<= 0 {p})"));
        self.pc.push(format!("(<= {p} {la})"));
        self.pc.push(format!("(<= {p} {lb})"));
        self.pc.push(format!(
            "(forall (({j} Int)) (! (=> (and (<= 0 {j}) (< {j} {p})) (= (select {a} {j}) (select {b} {j}))) :pattern ((select {a} {j})) :pattern ((select {b} {j}))))"
        ));
        self.pc.push(format!(
            "(or (= {p} {la}) (= {p} {lb}) (not (= {ap} {bp})))"
        ));
        let eq = format!("(and (= {la} {lb}) (= {p} {la}))");
        let lt = format!(
            "(or (and (= {p} {la}) (< {la} {lb})) (and (< {p} {la}) (< {p} {lb}) (< {ap} {bp})))"
        );
        let gt = format!(
            "(or (and (= {p} {lb}) (< {lb} {la})) (and (< {p} {la}) (< {p} {lb}) (< {bp} {ap})))"
        );
        match op {
            BinOp::Eq => eq,
            BinOp::Ne => format!("(not {eq})"),
            BinOp::Lt => lt,
            BinOp::Gt => gt,
            BinOp::Le => format!("(not {gt})"),
            BinOp::Ge => format!("(not {lt})"),
            _ => unreachable!("a comparison"),
        }
    }

    /// `l op r` on text in a specification: `==` is the slice equality,
    /// the orders a quantified formula on the common prefix.
    pub(super) fn text_cmp_spec(&mut self, op: BinOp, l: &Val, r: &Val) -> String {
        let bytes = TypeKind::Ref {
            mutable: false,
            inner: Box::new(Type {
                kind: TypeKind::Slice(Box::new(Type {
                    kind: TypeKind::Int(IntTy::U8),
                    span: Span::default(),
                })),
                span: Span::default(),
            }),
        };
        let lt = |s: &mut Self, x: &Val, y: &Val| {
            let (a, la) = parts(x);
            let (b, lb) = parts(y);
            let p = s.bound_name("lcp");
            let j = s.bound_name("lcp");
            format!(
                "(exists (({p} Int)) (and (<= 0 {p}) (<= {p} {la}) (<= {p} {lb}) (forall (({j} Int)) (=> (and (<= 0 {j}) (< {j} {p})) (= (select {a} {j}) (select {b} {j})))) (or (and (= {p} {la}) (< {la} {lb})) (and (< {p} {la}) (< {p} {lb}) (< (select {a} {p}) (select {b} {p}))))))"
            )
        };
        match op {
            BinOp::Eq => self.ext_eq(l, r, &bytes),
            BinOp::Ne => format!("(not {})", self.ext_eq(l, r, &bytes)),
            BinOp::Lt => lt(self, l, r),
            BinOp::Gt => lt(self, r, l),
            BinOp::Le => format!("(not {})", lt(self, r, l)),
            BinOp::Ge => format!("(not {})", lt(self, l, r)),
            _ => unreachable!("a comparison"),
        }
    }

    /// `String::from_utf8(v)`: `Ok` or `Err` of the same bytes, by a fresh
    /// condition (the prover does not model UTF-8 validity, ADR-0050 §3).
    pub(super) fn string_from_utf8(&mut self, e: &Expr, v: Val) -> Val {
        let ok = self.fresh("utf8", "Bool");
        let t = self.ty(e);
        let o = self.variant_value(&t, "Ok", std::slice::from_ref(&v)).s();
        let r = self.variant_value(&t, "Err", &[v]).s();
        Val::S(ite(&ok, &o, &r))
    }

    /// `char::from_u32(x)`: `Some(x)` exactly for a scalar value.
    pub(super) fn char_from_u32(&mut self, e: &Expr, x: String) -> Val {
        let t = self.ty(e);
        let some = self.variant_value(&t, "Some", &[Val::S(x.clone())]).s();
        let none = self.variant_value(&t, "None", &[]).s();
        Val::S(ite(&scalar(&x), &some, &none))
    }
}
