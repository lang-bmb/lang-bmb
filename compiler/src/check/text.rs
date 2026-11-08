//! Strings in the checker (ADR-0050, backlog 2U-01): `String`, `&str`
//! and `char`, their literals, the operations of ADR-0050 §2 with Rust's
//! names and signatures, and the Rust habits around them.
//!
//! The prover and the code generator see a `String` as a `Vec<u8>`, a
//! `&str` as a `&[u8]` and a `char` as a `u32` (`lower`); `Tables::text`
//! marks the expressions whose value is text, for the operations whose
//! byte meaning differs (`push` of a `char`, `==` and `<` on text).

use super::*;

/// The text functions of the standard library, which are methods of text
/// (ADR-0092 §2, `lib/std/str.bmb`).
const STD_STR_METHODS: &str = "the text methods are the built-in `len`, `as_bytes`, `as_str`, `is_char_boundary`, `push`, `push_str`, `clone`, and `std::str`'s `starts_with`, `ends_with`, `find`, `strip_prefix`, `strip_suffix`, `trim_ascii_start`, `trim_ascii_end`, `decode_utf8`";

/// Whether `t` is text: `String`, `str`, or a reference to one.
pub fn is_text(t: &TypeKind) -> bool {
    match t {
        TypeKind::String | TypeKind::Str => true,
        TypeKind::Ref { inner, .. } => matches!(inner.kind, TypeKind::String | TypeKind::Str),
        _ => false,
    }
}

/// Whether `t` is an owned `String`, or a reference to one.
fn is_owned_text(t: &TypeKind) -> bool {
    match t {
        TypeKind::String => true,
        TypeKind::Ref { inner, .. } => inner.kind == TypeKind::String,
        _ => false,
    }
}

/// `&str`.
pub fn str_ref() -> TypeKind {
    TypeKind::Ref {
        mutable: false,
        inner: mk(TypeKind::Str),
    }
}

/// `&[u8]`.
fn bytes_ref() -> TypeKind {
    TypeKind::Ref {
        mutable: false,
        inner: mk(TypeKind::Slice(mk(TypeKind::Int(IntTy::U8)))),
    }
}

/// `t` as the prover and the code generator see it: text is bytes and a
/// `char` its scalar value (ADR-0050 §3, §4).
pub fn lower(t: &TypeKind) -> TypeKind {
    let sub = |x: &Type| Type {
        kind: lower(&x.kind),
        span: x.span,
    };
    match t {
        TypeKind::Char => TypeKind::Int(IntTy::U32),
        TypeKind::String => TypeKind::Vec(mk(TypeKind::Int(IntTy::U8))),
        TypeKind::Str => TypeKind::Slice(mk(TypeKind::Int(IntTy::U8))),
        TypeKind::Named { path, args } => TypeKind::Named {
            path: path.clone(),
            args: args.iter().map(sub).collect(),
        },
        TypeKind::Array(e, n) => TypeKind::Array(Box::new(sub(e)), *n),
        TypeKind::Slice(e) => TypeKind::Slice(Box::new(sub(e))),
        TypeKind::Vec(e) => TypeKind::Vec(Box::new(sub(e))),
        TypeKind::Box(e) => TypeKind::Box(Box::new(sub(e))),
        TypeKind::Ref { mutable, inner } => TypeKind::Ref {
            mutable: *mutable,
            inner: Box::new(sub(inner)),
        },
        _ => t.clone(),
    }
}

/// Whether `t` mentions text or `char` anywhere.
pub fn mentions_text(t: &TypeKind) -> bool {
    match t {
        TypeKind::Char | TypeKind::String | TypeKind::Str => true,
        TypeKind::Named { args, .. } => args.iter().any(|a| mentions_text(&a.kind)),
        TypeKind::Array(e, _) | TypeKind::Slice(e) | TypeKind::Vec(e) | TypeKind::Box(e) => {
            mentions_text(&e.kind)
        }
        TypeKind::Ref { inner, .. } => mentions_text(&inner.kind),
        _ => false,
    }
}

/// A built-in call of ADR-0050 §2 by path: `String::new`,
/// `String::with_capacity`, `String::from`, `String::from_utf8`,
/// `char::from_u32`.
pub fn text_builtin(path: &[Ident]) -> bool {
    let p: Vec<&str> = path.iter().map(|i| i.name.as_str()).collect();
    matches!(
        p.as_slice(),
        ["String", "new" | "with_capacity" | "from" | "from_utf8"] | ["char", "from_u32"]
    )
}

/// `s.as_bytes()` (no arguments).
pub fn as_bytes_of(e: &Expr) -> Option<&Expr> {
    match &e.kind {
        ExprKind::Method { recv, name, args } if name.name == "as_bytes" && args.is_empty() => {
            Some(recv)
        }
        _ => None,
    }
}

/// `&s[a..b]`: the string, and the bounds.
pub fn str_slice_of(e: &Expr) -> Option<(&Expr, &Expr, &Expr)> {
    let ExprKind::Ref {
        mutable: false,
        expr,
    } = &e.kind
    else {
        return None;
    };
    let ExprKind::Index { base, index } = &expr.kind else {
        return None;
    };
    let ExprKind::Range { lo, hi } = &index.kind else {
        return None;
    };
    Some((base, lo, hi))
}

impl Checker<'_> {
    pub(super) fn key_of(e: &Expr) -> (u32, u32, u32) {
        (e.span.file, e.span.lo, e.span.hi)
    }

    /// A text operand of a comparison: a string literal is a `&str` there
    /// (the Rust habit `s == "abc"`, ADR-0050 §2).
    pub(super) fn text_operand(
        &mut self,
        cx: &mut FnCtx,
        e: &Expr,
        hint: Option<&TypeKind>,
        mode: Mode,
    ) -> TypeKind {
        if let ExprKind::StrLit(_) = &e.kind {
            let t = str_ref();
            self.types.insert(Self::key_of(e), t.clone());
            return t;
        }
        // `s.as_str()` and `&s[a..b]` as operands (`t < s.as_str()`).
        if matches!(&e.kind, ExprKind::Method { name, .. } if name.name == "as_str")
            || str_slice_of(e).is_some()
        {
            self.text_arg(cx, e, mode);
            return self
                .types
                .get(&Self::key_of(e))
                .cloned()
                .unwrap_or(TypeKind::Error);
        }
        self.expr(cx, e, hint, mode)
    }

    /// `String::new()`, `String::with_capacity(n)`, `String::from(t)`,
    /// `String::from_utf8(v)` and `char::from_u32(x)` (ADR-0050 §2).
    pub(super) fn text_call(
        &mut self,
        cx: &mut FnCtx,
        path: &[Ident],
        args: &[Expr],
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let what = Self::path_text(path);
        // `char::from_u32` is a pure function of its argument; the others
        // make (or give back) owned text, which specifications do not.
        if mode != Mode::Exec && path[0].name == "String" {
            self.err(
                "E0503",
                span,
                format!("`{what}` is not allowed in a specification"),
            );
        }
        let last = path[1].name.as_str();
        let arity = usize::from(last != "new");
        if args.len() != arity {
            self.diags.push(
                Diag::new(
                    "E0405",
                    span,
                    format!("`{what}` takes {arity} argument(s), found {}", args.len()),
                )
                .fix(
                    "Rust's signatures: `String::new()`, `String::with_capacity(n)`, `String::from(t)`, `String::from_utf8(v)`, `char::from_u32(x)`",
                    None,
                ),
            );
            for a in args {
                self.expr(cx, a, None, mode);
            }
            return TypeKind::Error;
        }
        if mode == Mode::Exec && matches!(last, "new" | "with_capacity" | "from") {
            cx.need_own(Effects::ALLOC, || format!("`{what}`"));
        }
        match (path[0].name.as_str(), last) {
            ("String", "new") => TypeKind::String,
            ("String", "with_capacity") => {
                let u = TypeKind::Int(IntTy::Usize);
                let t = self.expr(cx, &args[0], Some(&u), mode);
                self.expect_ty(&t, &u, args[0].span);
                TypeKind::String
            }
            ("String", "from") => {
                self.text_arg(cx, &args[0], mode);
                TypeKind::String
            }
            ("String", "from_utf8") => {
                let v = TypeKind::Vec(mk(TypeKind::Int(IntTy::U8)));
                let t = self.expr(cx, &args[0], Some(&v), mode);
                self.expect_ty(&t, &v, args[0].span);
                // Rust's error is `FromUtf8Error`, which gives the bytes
                // back: the same information (ADR-0050 §2).
                TypeKind::Named {
                    path: vec!["Result".into()],
                    args: vec![
                        Type {
                            kind: TypeKind::String,
                            span: Span::default(),
                        },
                        Type {
                            kind: v,
                            span: Span::default(),
                        },
                    ],
                }
            }
            _ => {
                let u = TypeKind::Int(IntTy::U32);
                let t = self.expr(cx, &args[0], Some(&u), mode);
                self.expect_ty(&t, &u, args[0].span);
                option_of(TypeKind::Char)
            }
        }
    }

    /// An argument for a `&str` parameter (ADR-0050 §2): a string literal,
    /// `s.as_str()` of a `String`, a `&str` passed on, or `&s[a..b]` of
    /// either. The Rust habits `&s` and `s` of a `String` are E0218.
    pub(super) fn text_arg(&mut self, cx: &mut FnCtx, a: &Expr, mode: Mode) {
        let key = Self::key_of(a);
        match &a.kind {
            ExprKind::StrLit(_) => {
                self.types.insert(key, str_ref());
            }
            ExprKind::Method { recv, name, args } if name.name == "as_str" => {
                let rt = self.expr(cx, recv, None, mode);
                if !args.is_empty() {
                    self.err("E0405", name.span, "`as_str` takes no arguments");
                }
                if !is_owned_text(&rt) {
                    if rt != TypeKind::Error {
                        self.err(
                            "E0404",
                            name.span,
                            format!("`as_str` is not defined for `{}`", ty_str(&rt)),
                        );
                    }
                    return;
                }
                self.types.insert(key, str_ref());
            }
            _ if str_slice_of(a).is_some() => {
                let (base, lo, hi) = str_slice_of(a).expect("a string slice");
                let bt = self.text_operand(cx, base, None, mode);
                let u = TypeKind::Int(IntTy::Usize);
                for x in [lo, hi] {
                    let t = self.expr(cx, x, Some(&u), mode);
                    self.expect_ty(&t, &u, x.span);
                }
                if !is_text(&bt) {
                    if bt != TypeKind::Error {
                        self.diags.push(
                            Diag::new(
                                "E0401",
                                a.span,
                                format!("expected `&str`, found a range of `{}`", ty_str(&bt)),
                            )
                            .fix(
                                "a range argument `&s[a..b]` is of a `String` or a `&str`",
                                None,
                            ),
                        );
                    }
                    return;
                }
                self.types.insert(key, str_ref());
            }
            ExprKind::Ref { expr, .. } if matches!(expr.kind, ExprKind::Var(_)) => {
                let t = self.expr(cx, expr, None, mode);
                if is_owned_text(&t) {
                    let r = format!("{}.as_str()", crate::fmt::expr_text(expr));
                    self.diags.push(
                        Diag::new("E0218", a.span, "a `String` is not a `&str`")
                            .fix(format!("pass `{r}`"), Some(r)),
                    );
                } else if t != TypeKind::Error {
                    self.err(
                        "E0401",
                        a.span,
                        format!("expected `&str`, found `&{}`", ty_str(&t)),
                    );
                }
            }
            _ => {
                let t = self.expr(cx, a, None, mode);
                if t == str_ref() || t == TypeKind::Error || t == TypeKind::Never {
                    return;
                }
                let mut d = Diag::new(
                    "E0401",
                    a.span,
                    format!("expected `&str`, found `{}`", ty_str(&t)),
                );
                if is_owned_text(&t) {
                    let r = format!("{}.as_str()", crate::fmt::expr_text(a));
                    d = Diag::new("E0218", a.span, "a `String` is not a `&str`")
                        .fix(format!("pass `{r}`"), Some(r));
                }
                self.diags.push(d);
            }
        }
    }

    /// `s.as_bytes()` given for a `&[u8]` parameter or indexed (ADR-0050
    /// §2): the bytes of a text value, read-only.
    pub(super) fn as_bytes_arg(
        &mut self,
        cx: &mut FnCtx,
        a: &Expr,
        recv: &Expr,
        mode: Mode,
    ) -> bool {
        let rt = self.expr(cx, recv, None, mode);
        if !is_text(&rt) {
            return false;
        }
        self.types.insert(Self::key_of(a), bytes_ref());
        true
    }

    /// A method of text (ADR-0050 §2): `len`, `is_char_boundary`, and of a
    /// `String` `push_str`, `push` and `clone`; `as_str` and `as_bytes`
    /// are arguments (or indexed, `s.as_bytes()[i]`).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn text_method(
        &mut self,
        cx: &mut FnCtx,
        recv: &Expr,
        rt: &TypeKind,
        name: &Ident,
        args: &[Expr],
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let m = name.name.as_str();
        let owned = is_owned_text(rt);
        let (arity, mutates, ret) = match m {
            "len" => (0, false, TypeKind::Int(IntTy::Usize)),
            "is_char_boundary" => (1, false, TypeKind::Bool),
            "push_str" | "push" if owned => (1, true, TypeKind::Unit),
            "clone" if owned => (0, false, TypeKind::String),
            "as_str" | "as_bytes" => {
                for a in args {
                    self.expr(cx, a, None, mode);
                }
                let what = if m == "as_str" { "&str" } else { "&[u8]" };
                self.diags.push(
                    Diag::new(
                        "E0409",
                        span,
                        format!("`.{m}()` may only appear directly as a call argument"),
                    )
                    .fix(
                        if m == "as_bytes" {
                            format!(
                                "pass it to a `{what}` parameter, or read a byte: `s.as_bytes()[i]`"
                            )
                        } else {
                            format!("pass it to a `{what}` parameter")
                        },
                        None,
                    ),
                );
                return TypeKind::Error;
            }
            _ => {
                for a in args {
                    self.text_operand(cx, a, None, mode);
                }
                let fix = match m {
                    "push_str" | "push" | "clone" => {
                        Some("a `&str` is read-only: build a `String` (`String::from(t)`)".into())
                    }
                    "to_string" | "to_owned" | "into" => {
                        Some("own it: `String::from(t)` (one form, ADR-0050 §2)".to_string())
                    }
                    "chars" | "bytes" | "char_indices" | "iter" => Some(
                        "read the bytes with `while` and `s.as_bytes()[i]` (no iterators, ADR-0049)"
                            .into(),
                    ),
                    "is_empty" => Some("compare `s.len() == 0`".into()),
                    "trim" | "trim_start" | "trim_end" => Some(format!(
                        "`std::str` trims ASCII whitespace and returns the position: `s.trim_ascii_start()`, `s.trim_ascii_end()`, then `&s[a..b]`; {}",
                        STD_STR_METHODS
                    )),
                    "contains" => Some(format!(
                        "`s.find(p)` is `Some` where `s` contains `p` (`std::str`); {}",
                        STD_STR_METHODS
                    )),
                    "split_once" | "split" | "lines" => Some(format!(
                        "find the separator with `s.find(p)` and slice at the position (`std::str`); {}",
                        STD_STR_METHODS
                    )),
                    "parse" | "to_uppercase" | "to_lowercase" | "replace" => Some(format!(
                        "not in the standard library yet: write it over the bytes; {}",
                        STD_STR_METHODS
                    )),
                    _ => Some(STD_STR_METHODS.to_string()),
                };
                let mut d = Diag::new(
                    "E0404",
                    name.span,
                    format!("unknown method `{m}` of `{}`", ty_str(rt)),
                );
                if let Some(f) = fix {
                    d = d.fix(f, None);
                }
                self.diags.push(d);
                return TypeKind::Error;
            }
        };
        if args.len() != arity {
            self.err(
                "E0405",
                name.span,
                format!("`{m}` takes {arity} argument(s), found {}", args.len()),
            );
            for a in args {
                self.expr(cx, a, None, mode);
            }
            return TypeKind::Error;
        }
        if mode != Mode::Exec && mutates {
            if cx.ghost {
                self.ghost_violation(
                    span,
                    &format!("`.{m}()`"),
                    "change the value outside the `proof` block",
                );
            } else {
                self.err(
                    "E0503",
                    span,
                    format!("`.{m}()` is not allowed in a specification"),
                );
            }
        }
        if mode != Mode::Exec && m == "clone" {
            self.diags.push(
                Diag::new(
                    "E0503",
                    span,
                    "`.clone()` is not allowed in a specification",
                )
                .fix(
                    "a specification reads the value itself: drop `.clone()`",
                    Some(crate::fmt::expr_text(recv)),
                ),
            );
        }
        if mode == Mode::Exec && matches!(m, "push_str" | "push" | "clone") {
            cx.need_own(Effects::ALLOC, || format!("`String::{m}`"));
        }
        match m {
            "is_char_boundary" => {
                let u = TypeKind::Int(IntTy::Usize);
                let t = self.expr(cx, &args[0], Some(&u), mode);
                self.expect_ty(&t, &u, args[0].span);
            }
            "push_str" => self.text_arg(cx, &args[0], mode),
            "push" => {
                let t = self.expr(cx, &args[0], Some(&TypeKind::Char), mode);
                if t == TypeKind::Int(IntTy::U8) {
                    self.diags.push(
                        Diag::new(
                            "E0401",
                            args[0].span,
                            "expected `char`, found `u8`: a `String` holds characters",
                        )
                        .fix("push a character literal `'a'`, or the byte's character `char::from_u32(u32::from(b))`", None),
                    );
                } else {
                    self.expect_ty(&t, &TypeKind::Char, args[0].span);
                }
            }
            _ => {}
        }
        if mutates && mode == Mode::Exec && !Self::writable(cx, recv) && *rt != TypeKind::Error {
            let fix = if matches!(rt, TypeKind::Ref { .. }) {
                "take the parameter as `&mut String`"
            } else {
                "declare it with `let mut`"
            };
            self.diags.push(
                Diag::new(
                    "E0408",
                    recv.span,
                    format!("`.{m}()` needs a mutable `String`"),
                )
                .fix(fix, None),
            );
        }
        ret
    }

    /// A method of `char` (ADR-0050 §2): `len_utf8`.
    pub(super) fn char_method(
        &mut self,
        cx: &mut FnCtx,
        name: &Ident,
        args: &[Expr],
        mode: Mode,
    ) -> TypeKind {
        for a in args {
            self.expr(cx, a, None, mode);
        }
        match name.name.as_str() {
            "len_utf8" if args.is_empty() => TypeKind::Int(IntTy::Usize),
            "len_utf8" => {
                self.err("E0405", name.span, "`len_utf8` takes no arguments");
                TypeKind::Error
            }
            m => {
                let fix = match m {
                    "to_digit" | "is_digit" | "is_ascii_digit" | "is_alphabetic"
                    | "is_whitespace" => Some("compare the scalar value: `'0' <= c && c <= '9'`"),
                    "to_string" => Some("build a `String` and `push(c)`"),
                    _ => None,
                };
                let mut d = Diag::new(
                    "E0404",
                    name.span,
                    format!("unknown method `{m}` of `char`"),
                );
                if let Some(f) = fix {
                    d = d.fix(f, None);
                }
                self.diags.push(d);
                TypeKind::Error
            }
        }
    }
}
