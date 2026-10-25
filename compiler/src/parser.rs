//! Recursive-descent parser for the milestone-1 subset. Recovers at item and
//! statement boundaries so that one run reports every syntax error it can.

use crate::ast::*;
use crate::diag::{Diag, Diags};
use crate::lexer::{Tok, Token};
use crate::source::Span;

type PResult<T> = Result<T, ()>;

pub struct Parser<'a> {
    toks: Vec<Token>,
    pos: usize,
    diags: &'a mut Diags,
    /// Source text, for fixes that copy part of it (e.g. `x as T` -> `T::try_from(x).unwrap()`).
    text: &'a str,
    /// Current syntactic nesting (see `MAX_NESTING`).
    depth: usize,
    /// Inside an `if`/`while` condition or a `match` scrutinee, where `T {`
    /// opens the body, not a struct literal (as in Rust).
    no_struct: bool,
    /// The type of the `impl` block being read (for the receiver fix).
    impl_ty: Option<String>,
}

/// Maximum syntactic nesting (parenthesised and unary expressions, blocks,
/// `else if` chains, `==>` chains, `while` loops, types, units). Deeper input
/// is rejected with E0107 instead of overflowing the stack (backlog 1A-11).
pub const MAX_NESTING: usize = 256;

/// Rust constructs recognised only to be rejected with a precise message.
const UNSUPPORTED_ITEMS: &[&str] = &[
    "trait",
    "use",
    "const",
    "static",
    "type",
    "extern",
    "unsafe",
    "macro_rules",
    "union",
];

const RESERVED: &[&str] = &[
    "fn",
    "spec",
    "mod",
    "let",
    "mut",
    "while",
    "if",
    "else",
    "true",
    "false",
    "requires",
    "ensures",
    "decreases",
    "invariant",
    "forall",
    "old",
    "assert",
    "return",
    "for",
    "loop",
    "match",
    "break",
    "continue",
    "as",
    "pub",
    "struct",
    "enum",
    "impl",
    "trait",
    "use",
    "const",
    "static",
    "type",
    "unsafe",
    "self",
    "Self",
    "crate",
    "super",
    "where",
    "ref",
    "move",
    "dyn",
    "in",
    "async",
    "await",
];

/// Whether `name` is a name the parser accepts for a unit or definition: the
/// lexer's identifier rule (`[A-Za-z_][A-Za-z0-9_]*`) minus reserved words.
pub fn is_ident(name: &str) -> bool {
    let mut cs = name.chars();
    cs.next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !RESERVED.contains(&name)
}

impl<'a> Parser<'a> {
    pub fn new(toks: Vec<Token>, diags: &'a mut Diags, text: &'a str) -> Self {
        Parser {
            text,
            toks,
            pos: 0,
            diags,
            depth: 0,
            no_struct: false,
            impl_ty: None,
        }
    }

    /// Enters one nesting level, or reports E0107 at the limit.
    fn enter(&mut self) -> PResult<()> {
        if self.depth >= MAX_NESTING {
            self.diags.push(
                Diag::new(
                    "E0107",
                    self.span(),
                    format!("nesting deeper than {MAX_NESTING} levels"),
                )
                .fix(
                    "split the expression with `let` bindings or helper functions",
                    None,
                ),
            );
            return Err(());
        }
        self.depth += 1;
        Ok(())
    }

    /// Runs `f`, which may enter one level per link of a left-nested chain
    /// (`a + b + c`, `x[i][j]`), and restores the depth afterwards.
    fn chain<T>(&mut self, f: impl FnOnce(&mut Self) -> PResult<T>) -> PResult<T> {
        let d = self.depth;
        let r = f(self);
        self.depth = d;
        r
    }

    /// Runs `f` one nesting level deeper.
    fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> PResult<T>) -> PResult<T> {
        self.enter()?;
        let r = f(self);
        self.depth -= 1;
        r
    }

    // ---------- token helpers ----------

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }
    fn peek_at(&self, n: usize) -> &Tok {
        let i = (self.pos + n).min(self.toks.len() - 1);
        &self.toks[i].tok
    }
    fn span(&self) -> Span {
        self.toks[self.pos].span
    }
    fn prev_span(&self) -> Span {
        self.toks[self.pos.saturating_sub(1)].span
    }
    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn is_p(&self, p: &str) -> bool {
        matches!(self.peek(), Tok::P(q) if *q == p)
    }
    fn is_kw(&self, k: &str) -> bool {
        matches!(self.peek(), Tok::Ident(s) if s == k)
    }
    fn eat_p(&mut self, p: &str) -> bool {
        if self.is_p(p) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_kw(&mut self, k: &str) -> bool {
        if self.is_kw(k) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn describe(t: &Tok) -> String {
        match t {
            Tok::Ident(s) => format!("`{s}`"),
            Tok::Int(v) => format!("`{v}`"),
            Tok::Str(_) => "string literal".into(),
            Tok::Byte(_) => "byte literal".into(),
            Tok::Bytes(_) => "byte string literal".into(),
            Tok::P(p) => format!("`{p}`"),
            Tok::Eof => "end of file".into(),
        }
    }
    fn err_expected(&mut self, what: &str) {
        let found = Self::describe(self.peek());
        let mut d = Diag::new(
            "E0103",
            self.span(),
            format!("expected {what}, found {found}"),
        );
        // A single expected token (`;`, `)`, ..) can be inserted (backlog 2D-18).
        if what.starts_with('`') && what.ends_with('`') && !what.contains(' ') {
            d = d.fix(format!("insert {what} before {found}"), None);
        }
        self.diags.push(d);
    }
    fn expect_p(&mut self, p: &str) -> PResult<Span> {
        if self.is_p(p) {
            Ok(self.bump().span)
        } else {
            self.err_expected(&format!("`{p}`"));
            Err(())
        }
    }
    fn ident(&mut self, what: &str) -> PResult<Ident> {
        match self.peek().clone() {
            Tok::Ident(name) if !RESERVED.contains(&name.as_str()) => {
                let span = self.bump().span;
                Ok(Ident { name, span })
            }
            _ => {
                self.err_expected(what);
                Err(())
            }
        }
    }

    /// Skip a balanced delimited group starting at the current token, if any.
    fn skip_group(&mut self) {
        let (open, close) = match self.peek() {
            Tok::P("(") => ("(", ")"),
            Tok::P("[") => ("[", "]"),
            Tok::P("{") => ("{", "}"),
            _ => return,
        };
        let mut depth = 0i32;
        loop {
            if self.is_p(open) {
                depth += 1;
            } else if self.is_p(close) {
                depth -= 1;
                if depth == 0 {
                    self.bump();
                    return;
                }
            } else if matches!(self.peek(), Tok::Eof) {
                return;
            }
            self.bump();
        }
    }

    /// Recover inside a block: stop after `;` or before `}` at the current depth.
    fn recover_stmt(&mut self) {
        let mut depth = 0i32;
        loop {
            match self.peek() {
                Tok::Eof => return,
                Tok::P("{") | Tok::P("(") | Tok::P("[") => depth += 1,
                Tok::P("}") => {
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                }
                // Unbalanced `)` / `]` left over from the failed statement: skip.
                Tok::P(")") | Tok::P("]") => depth = (depth - 1).max(0),
                Tok::P(";") if depth == 0 => {
                    self.bump();
                    return;
                }
                _ => {}
            }
            self.bump();
        }
    }

    /// Recover inside a mod: stop before the next item start or the closing `}`.
    fn recover_item(&mut self) {
        let mut depth = 0i32;
        loop {
            match self.peek() {
                Tok::Eof => return,
                Tok::P("{") => depth += 1,
                Tok::P("}") => {
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                    if depth == 0 {
                        self.bump();
                        if self.starts_item() || self.is_p("}") {
                            return;
                        }
                        continue;
                    }
                }
                _ if depth == 0 && self.starts_item() => return,
                _ => {}
            }
            self.bump();
        }
    }

    fn starts_item(&self) -> bool {
        match self.peek() {
            Tok::P("#") => true,
            Tok::Ident(s) => {
                matches!(
                    s.as_str(),
                    "fn" | "spec" | "mod" | "pub" | "struct" | "enum" | "impl"
                ) || UNSUPPORTED_ITEMS.contains(&s.as_str())
            }
            _ => false,
        }
    }

    // ---------- items ----------

    pub fn parse_program(&mut self) -> Program {
        let mut mods = Vec::new();
        while !matches!(self.peek(), Tok::Eof) {
            let start = self.pos;
            let attrs = self.attrs();
            if self.is_kw("mod") {
                if let Ok(m) = self.parse_mod(attrs) {
                    mods.push(m);
                }
            } else {
                if self.is_kw("fn") || self.is_kw("spec") {
                    self.diags.push(
                        Diag::new(
                            "E0703",
                            self.span(),
                            "items must be declared inside a `mod`",
                        )
                        .fix(
                            "wrap the item in `#[summary(\"..\")] mod name { .. }`",
                            None,
                        ),
                    );
                } else {
                    self.err_expected("`mod`");
                }
                self.bump();
                self.recover_item();
            }
            if self.pos == start {
                self.bump();
            }
        }
        Program { mods }
    }

    fn attrs(&mut self) -> Vec<Attr> {
        let mut out = Vec::new();
        while self.is_p("#") && matches!(self.peek_at(1), Tok::P("[")) {
            let lo = self.bump().span;
            self.bump();
            let name = match self.ident("attribute name") {
                Ok(n) => n,
                Err(_) => {
                    self.recover_attr();
                    continue;
                }
            };
            let mut args = Vec::new();
            if self.eat_p("(") {
                while !self.is_p(")") && !matches!(self.peek(), Tok::Eof) {
                    match self.peek().clone() {
                        Tok::Ident(s) => {
                            // A path argument `unit::name` is kept as one name.
                            let mut name = s;
                            let mut sp = self.bump().span;
                            while self.is_p("::") && matches!(self.peek_at(1), Tok::Ident(_)) {
                                self.bump();
                                if let Tok::Ident(seg) = self.peek().clone() {
                                    sp = sp.to(self.bump().span);
                                    name = format!("{name}::{seg}");
                                }
                            }
                            args.push(AttrArg::Ident(Ident { name, span: sp }));
                        }
                        Tok::Str(s) => {
                            let sp = self.bump().span;
                            args.push(AttrArg::Str(s, sp));
                        }
                        _ => {
                            self.err_expected("attribute argument");
                            self.bump();
                        }
                    }
                    if !self.eat_p(",") {
                        break;
                    }
                }
                if self.expect_p(")").is_err() {
                    self.recover_attr();
                    continue;
                }
            }
            let hi = match self.expect_p("]") {
                Ok(s) => s,
                Err(_) => {
                    self.recover_attr();
                    continue;
                }
            };
            out.push(Attr {
                name,
                args,
                span: lo.to(hi),
            });
        }
        out
    }

    fn recover_attr(&mut self) {
        while !self.is_p("]") && !matches!(self.peek(), Tok::Eof) {
            self.bump();
        }
        self.eat_p("]");
    }

    fn parse_mod(&mut self, attrs: Vec<Attr>) -> PResult<Mod> {
        self.nested(|p| p.parse_mod_inner(attrs))
    }

    fn parse_mod_inner(&mut self, attrs: Vec<Attr>) -> PResult<Mod> {
        let lo = self.bump().span; // `mod`
        let name = self.ident("module name")?;
        self.expect_p("{")?;
        let mut fns = Vec::new();
        let mut mods = Vec::new();
        let mut types = Vec::new();
        let mut impls = Vec::new();
        while !self.is_p("}") && !matches!(self.peek(), Tok::Eof) {
            let start = self.pos;
            let attrs = self.attrs();
            if self.is_kw("pub") {
                let sp = self.bump().span;
                self.diags.push(
                    Diag::new(
                        "E0207",
                        sp,
                        "`pub` is not used; every definition of a unit is part of its surface",
                    )
                    .fix("remove `pub`", Some(String::new())),
                );
            }
            if self.is_kw("mod") {
                if let Ok(m) = self.parse_mod(attrs) {
                    mods.push(m);
                } else {
                    self.recover_item();
                }
            } else if self.is_kw("fn")
                || (self.is_kw("spec") && matches!(self.peek_at(1), Tok::Ident(s) if s == "fn"))
            {
                match self.parse_fn(attrs) {
                    Ok(f) => fns.push(f),
                    Err(_) => self.recover_item(),
                }
            } else if self.is_kw("struct") || self.is_kw("enum") {
                match self.parse_type_def(attrs) {
                    Ok(t) => types.push(t),
                    Err(_) => self.recover_item(),
                }
            } else if self.is_kw("impl") {
                match self.parse_impl() {
                    Ok(i) => impls.push(i),
                    Err(_) => self.recover_item(),
                }
            } else if let Tok::Ident(s) = self.peek().clone()
                && UNSUPPORTED_ITEMS.contains(&s.as_str())
            {
                let sp = self.span();
                let fix = match s.as_str() {
                    "use" => "refer to other units by full path, e.g. `unit::f(..)`",
                    "const" | "static" => "use a `spec fn` returning the value, or a literal",
                    _ => "units hold `fn`, `spec fn`, `struct`, `enum` and `impl T { .. }`",
                };
                self.diags.push(
                    Diag::new(
                        "E0207",
                        sp,
                        format!("`{s}` items are not supported in milestone 1"),
                    )
                    .fix(fix, None),
                );
                self.bump();
                self.recover_item();
            } else {
                self.err_expected("`fn`, `spec fn` or `mod`");
                self.bump();
                self.recover_item();
            }
            if self.pos == start {
                self.bump();
            }
        }
        let hi = self.expect_p("}")?;
        Ok(Mod {
            name,
            attrs,
            fns,
            mods,
            types,
            impls,
            span: lo.to(hi),
        })
    }

    /// Generic parameters are outside the language (ADR-0026 §1).
    fn reject_generics(&mut self) {
        if self.is_p("<") {
            self.diags.push(Diag::new(
                "E0207",
                self.span(),
                "generic parameters are not supported in milestone 1",
            ));
            self.skip_angle();
        }
    }

    /// `struct T { f: U, .. }` or `enum T { A, B(U, V), .. }` (ADR-0026 §1).
    fn parse_type_def(&mut self, attrs: Vec<Attr>) -> PResult<TypeDef> {
        let lo = self.span();
        let is_struct = matches!(self.bump().tok, Tok::Ident(ref k) if k == "struct");
        let name = self.ident("type name")?;
        self.reject_generics();
        if is_struct && self.is_p("(") {
            let sp = self.span();
            self.skip_group();
            self.eat_p(";");
            self.diags.push(
                Diag::new(
                    "E0233",
                    sp.to(self.prev_span()),
                    "tuple structs are not allowed",
                )
                .fix("name the fields: `struct P { a: T, b: U }`", None),
            );
            return Err(());
        }
        self.expect_p("{")?;
        let kind = if is_struct {
            let mut fields = Vec::new();
            while !self.is_p("}") {
                let fname = self.ident("field name")?;
                self.expect_p(":")?;
                let ty = self.parse_type();
                fields.push(Field { name: fname, ty });
                if !self.eat_p(",") {
                    break;
                }
            }
            TypeDefKind::Struct(fields)
        } else {
            let mut variants = Vec::new();
            while !self.is_p("}") {
                let vname = self.ident("variant name")?;
                let mut payload = Vec::new();
                if self.is_p("{") {
                    self.diags.push(
                        Diag::new("E0233", self.span(), "struct-like variants are not allowed")
                            .fix("wrap a struct with named fields: `V(Data)`", None),
                    );
                    self.skip_group();
                } else if self.eat_p("(") {
                    while !self.is_p(")") {
                        payload.push(self.parse_type());
                        if !self.eat_p(",") {
                            break;
                        }
                    }
                    self.expect_p(")")?;
                }
                let span = vname.span.to(self.prev_span());
                variants.push(Variant {
                    name: vname,
                    payload,
                    span,
                });
                if !self.eat_p(",") {
                    break;
                }
            }
            TypeDefKind::Enum(variants)
        };
        let hi = self.expect_p("}")?;
        Ok(TypeDef {
            name,
            attrs,
            kind,
            span: lo.to(hi),
        })
    }

    /// `impl T { fn .. }` (ADR-0026 §3).
    fn parse_impl(&mut self) -> PResult<Impl> {
        let lo = self.bump().span; // `impl`
        self.reject_generics();
        let ty = self.ident("type name")?;
        if self.is_kw("for") {
            self.diags.push(Diag::new(
                "E0207",
                self.span(),
                "`impl Trait for T` is not supported; use `impl T { .. }`",
            ));
            return Err(());
        }
        self.expect_p("{")?;
        let mut fns = Vec::new();
        let outer = self.impl_ty.replace(ty.name.clone());
        while !self.is_p("}") && !matches!(self.peek(), Tok::Eof) {
            let start = self.pos;
            let attrs = self.attrs();
            if self.is_kw("pub") {
                let sp = self.bump().span;
                self.diags.push(
                    Diag::new(
                        "E0207",
                        sp,
                        "`pub` is not used; every definition of a unit is part of its surface",
                    )
                    .fix("remove `pub`", Some(String::new())),
                );
            }
            if self.is_kw("fn")
                || (self.is_kw("spec") && matches!(self.peek_at(1), Tok::Ident(s) if s == "fn"))
            {
                match self.parse_fn(attrs) {
                    Ok(f) => fns.push(f),
                    Err(_) => self.recover_item(),
                }
            } else {
                self.err_expected("`fn` or `spec fn`");
                self.bump();
                self.recover_item();
            }
            if self.pos == start {
                self.bump();
            }
        }
        self.impl_ty = outer;
        let hi = self.expect_p("}")?;
        Ok(Impl {
            ty,
            fns,
            span: lo.to(hi),
        })
    }

    fn parse_fn(&mut self, attrs: Vec<Attr>) -> PResult<FnDef> {
        let lo = self.span();
        let kind = if self.eat_kw("spec") {
            FnKind::Spec
        } else {
            FnKind::Exec
        };
        self.bump(); // `fn`
        let name = self.ident("function name")?;
        self.reject_generics();
        self.expect_p("(")?;
        let mut params = Vec::new();
        while !self.is_p(")") {
            if self.is_kw("mut") {
                let sp = self.bump().span;
                self.diags.push(
                    Diag::new("E0210", sp, "`mut` parameters are not allowed").fix(
                        "copy into a `let mut` local inside the body",
                        Some(String::new()),
                    ),
                );
            }
            // `self: &T` names the receiver (ADR-0026); `&self`, `&mut self` and
            // a bare `self` leave the type out.
            let short = match (self.peek(), self.peek_at(1), self.peek_at(2)) {
                (Tok::P("&"), Tok::Ident(s), _) if s == "self" => true,
                (Tok::P("&"), Tok::Ident(m), Tok::Ident(s)) if m == "mut" && s == "self" => true,
                (Tok::Ident(s), next, _) => s == "self" && !matches!(next, Tok::P(":")),
                _ => false,
            };
            if short {
                let lo = self.span();
                let form = match (self.peek(), self.peek_at(1)) {
                    (Tok::P("&"), Tok::Ident(m)) if m == "mut" => "&mut ",
                    (Tok::P("&"), _) => "&",
                    _ => "",
                };
                while !self.is_kw("self") {
                    self.bump();
                }
                let sp = lo.to(self.bump().span);
                let mut d = Diag::new("E0228", sp, "the receiver needs its type");
                let mut kind = TypeKind::Error;
                d = match &self.impl_ty {
                    Some(t) => {
                        let named = Type {
                            kind: TypeKind::Named {
                                path: vec![t.clone()],
                                args: Vec::new(),
                            },
                            span: sp,
                        };
                        kind = match form {
                            "" => named.kind,
                            f => TypeKind::Ref {
                                mutable: f == "&mut ",
                                inner: Box::new(named),
                            },
                        };
                        let r = format!("self: {form}{t}");
                        d.fix(format!("write `{r}`"), Some(r))
                    }
                    None => d.fix(
                        "write `self: &T`, `self: &mut T` or `self: T` with the `impl` type",
                        None,
                    ),
                };
                self.diags.push(d);
                params.push(Param {
                    name: Ident {
                        name: "self".into(),
                        span: sp,
                    },
                    ty: Type { kind, span: sp },
                });
                if !self.eat_p(",") {
                    break;
                }
                continue;
            }
            let pname = if self.is_kw("self") {
                let span = self.bump().span;
                Ident {
                    name: "self".into(),
                    span,
                }
            } else {
                self.ident("parameter name")?
            };
            self.expect_p(":")?;
            let ty = self.parse_type();
            params.push(Param { name: pname, ty });
            if !self.eat_p(",") {
                break;
            }
        }
        self.expect_p(")")?;
        let ret = if self.eat_p("->") {
            self.parse_ret()?
        } else {
            let sp = self.prev_span();
            self.diags.push(
                Diag::new("E0209", sp, "function return type is required")
                    .fix("add `-> ()`", Some(") -> ()".into())),
            );
            Ret {
                name: None,
                ty: Type {
                    kind: TypeKind::Unit,
                    span: sp,
                },
            }
        };
        let (requires, ensures, decreases) = self.parse_clauses()?;
        let body = self.parse_block()?;
        let span = lo.to(body.span);
        Ok(FnDef {
            kind,
            name,
            attrs,
            params,
            ret,
            requires,
            ensures,
            decreases,
            body,
            span,
        })
    }

    fn skip_angle(&mut self) {
        let mut depth = 0;
        loop {
            match self.peek() {
                Tok::P("<") => depth += 1,
                Tok::P(">") => {
                    depth -= 1;
                    if depth == 0 {
                        self.bump();
                        return;
                    }
                }
                Tok::Eof | Tok::P("{") | Tok::P(";") => return,
                _ => {}
            }
            self.bump();
        }
    }

    fn parse_ret(&mut self) -> PResult<Ret> {
        if self.is_p("(") {
            if matches!(self.peek_at(1), Tok::P(")")) {
                let lo = self.bump().span;
                let hi = self.bump().span;
                return Ok(Ret {
                    name: None,
                    ty: Type {
                        kind: TypeKind::Unit,
                        span: lo.to(hi),
                    },
                });
            }
            if matches!(self.peek_at(1), Tok::Ident(_)) && matches!(self.peek_at(2), Tok::P(":")) {
                self.bump();
                let name = self.ident("result name")?;
                self.bump(); // `:`
                let ty = self.parse_type();
                self.expect_p(")")?;
                return Ok(Ret {
                    name: Some(name),
                    ty,
                });
            }
        }
        Ok(Ret {
            name: None,
            ty: self.parse_type(),
        })
    }

    fn parse_clauses(&mut self) -> PResult<(Vec<Expr>, Vec<Expr>, Option<Expr>)> {
        let order = ["requires", "ensures", "decreases"];
        let mut stage = 0usize;
        let mut requires = Vec::new();
        let mut ensures = Vec::new();
        let mut decreases = None;
        while let Some(k) = order.iter().position(|k| self.is_kw(k)) {
            if k < stage {
                self.diags.push(Diag::new(
                    "E0103",
                    self.span(),
                    "clauses must appear in the order `requires`, `ensures`, `decreases`",
                ));
            }
            stage = k + 1;
            self.bump();
            match k {
                0 => requires.extend(self.clause_list()?),
                1 => ensures.extend(self.clause_list()?),
                _ => {
                    let e = self.parse_expr()?;
                    self.eat_p(",");
                    decreases = Some(e);
                }
            }
        }
        Ok((requires, ensures, decreases))
    }

    fn clause_list(&mut self) -> PResult<Vec<Expr>> {
        let mut out = Vec::new();
        loop {
            out.push(self.parse_expr()?);
            if !self.eat_p(",") {
                break;
            }
            if self.is_p("{")
                || ["requires", "ensures", "decreases", "invariant"]
                    .iter()
                    .any(|k| self.is_kw(k))
            {
                break;
            }
        }
        Ok(out)
    }

    // ---------- types ----------

    fn parse_type(&mut self) -> Type {
        if self.enter().is_err() {
            return Type {
                kind: TypeKind::Error,
                span: self.span(),
            };
        }
        let t = self.parse_type_inner();
        self.depth -= 1;
        t
    }

    fn parse_type_inner(&mut self) -> Type {
        let lo = self.span();
        match self.peek().clone() {
            Tok::P("&") => {
                self.bump();
                let mutable = self.eat_kw("mut");
                let inner = self.parse_type();
                let span = lo.to(inner.span);
                Type {
                    kind: TypeKind::Ref {
                        mutable,
                        inner: Box::new(inner),
                    },
                    span,
                }
            }
            Tok::P("(") if matches!(self.peek_at(1), Tok::P(")")) => {
                self.bump();
                let hi = self.bump().span;
                Type {
                    kind: TypeKind::Unit,
                    span: lo.to(hi),
                }
            }
            Tok::P("[") => {
                self.bump();
                let elem = self.parse_type();
                if self.eat_p(";") {
                    let n = match self.peek().clone() {
                        Tok::Int(v) => {
                            let sp = self.bump().span;
                            // Same limit as `[v; N]` literals (backlog 1A-10).
                            if v > u32::MAX as u128 {
                                self.diags.push(Diag::new(
                                    "E0406",
                                    sp,
                                    "array length is too large",
                                ));
                                u64::MAX
                            } else {
                                v as u64
                            }
                        }
                        _ => {
                            self.err_expected("array length");
                            0
                        }
                    };
                    let hi = self.span();
                    let _ = self.expect_p("]");
                    return Type {
                        // Too large (reported above): no follow-on type errors.
                        kind: if n == u64::MAX {
                            TypeKind::Error
                        } else {
                            TypeKind::Array(Box::new(elem), n)
                        },
                        span: lo.to(hi),
                    };
                }
                let hi = self.span();
                let _ = self.expect_p("]");
                Type {
                    kind: TypeKind::Slice(Box::new(elem)),
                    span: lo.to(hi),
                }
            }
            Tok::Ident(s)
                if !matches!(
                    s.as_str(),
                    "i64"
                        | "u64"
                        | "usize"
                        | "bool"
                        | "i8"
                        | "i16"
                        | "i32"
                        | "i128"
                        | "isize"
                        | "u8"
                        | "u16"
                        | "u32"
                        | "u128"
                        | "f32"
                        | "f64"
                        | "char"
                        | "str"
                ) && !RESERVED.contains(&s.as_str()) =>
            {
                // A named type (ADR-0026): `Point`, `geo::Point`, `Option<T>`.
                let mut path = vec![s];
                self.bump();
                while self.is_p("::") && matches!(self.peek_at(1), Tok::Ident(_)) {
                    self.bump();
                    if let Tok::Ident(seg) = self.bump().tok {
                        path.push(seg);
                    }
                }
                let mut args = Vec::new();
                if self.eat_p("<") {
                    while !self.is_p(">") {
                        args.push(self.parse_type());
                        if !self.eat_p(",") {
                            break;
                        }
                    }
                    let _ = self.expect_p(">");
                }
                Type {
                    kind: TypeKind::Named { path, args },
                    span: lo.to(self.prev_span()),
                }
            }
            Tok::Ident(s) => {
                self.bump();
                let kind = match s.as_str() {
                    "i64" => TypeKind::I64,
                    "u64" => TypeKind::U64,
                    "u8" => TypeKind::U8,
                    "usize" => TypeKind::Usize,
                    "bool" => TypeKind::Bool,
                    _ => {
                        let fix = match s.as_str() {
                            "i8" | "i16" | "i32" | "i128" | "isize" => Some("i64"),
                            "u16" | "u32" | "u128" => Some("u64"),
                            // Text is bytes (ADR-0044 §5): `&str` is `&[u8]`.
                            "char" => Some("u8"),
                            "str" => Some("[u8]"),
                            _ => None,
                        };
                        let mut d = Diag::new(
                            "E0402",
                            lo,
                            format!("type `{s}` is not supported in milestone 1"),
                        );
                        d = match fix {
                            Some(f) => d.fix(format!("use `{f}`"), Some(f.to_string())),
                            None => d.fix("milestone 1 types: i64, u64, usize, bool, (), [T; N], &[T], &mut [T]", None),
                        };
                        self.diags.push(d);
                        if self.is_p("<") {
                            self.skip_angle();
                        }
                        TypeKind::Error
                    }
                };
                Type {
                    kind,
                    span: lo.to(self.prev_span()),
                }
            }
            _ => {
                self.err_expected("type");
                Type {
                    kind: TypeKind::Error,
                    span: lo,
                }
            }
        }
    }

    // ---------- blocks and statements ----------

    fn parse_block(&mut self) -> PResult<Block> {
        let lo = self.expect_p("{")?;
        let mut stmts = Vec::new();
        let mut tail: Option<Box<Expr>> = None;
        let mut had_err = false;
        while !self.is_p("}") && !matches!(self.peek(), Tok::Eof) {
            if let Some(t) = tail.take() {
                // A block-like expression followed by more statements is a statement.
                stmts.push(Stmt::Expr(*t));
            }
            let start = self.pos;
            match self.parse_stmt() {
                Ok(StmtOrTail::Stmt(s)) => stmts.push(s),
                Ok(StmtOrTail::Tail(e)) => tail = Some(Box::new(e)),
                Err(_) => {
                    had_err = true;
                    self.recover_stmt();
                }
            }
            if self.pos == start {
                self.bump();
            }
        }
        let hi = self.expect_p("}")?;
        if had_err && tail.is_none() {
            // The block's value is unknown after a syntax error; avoid a cascade.
            tail = Some(Box::new(Expr {
                kind: ExprKind::Error,
                span: hi,
            }));
        }
        Ok(Block {
            stmts,
            tail,
            span: lo.to(hi),
        })
    }

    fn parse_stmt(&mut self) -> PResult<StmtOrTail> {
        let lo = self.span();
        if let Tok::Ident(s) = self.peek().clone() {
            match s.as_str() {
                "let" => return self.parse_let().map(StmtOrTail::Stmt),
                "while" => return self.parse_while().map(StmtOrTail::Stmt),
                "assert" if matches!(self.peek_at(1), Tok::P("(")) => {
                    self.bump();
                    self.bump();
                    let cond = self.parse_expr()?;
                    self.expect_p(")")?;
                    let hi = self.expect_p(";")?;
                    return Ok(StmtOrTail::Stmt(Stmt::Assert {
                        cond,
                        span: lo.to(hi),
                    }));
                }
                "return" => {
                    self.bump();
                    self.diags
                        .push(Diag::new("E0203", lo, "`return` is not allowed").fix(
                            "restructure so the value is the block's tail expression",
                            None,
                        ));
                    return Err(());
                }
                "for" | "loop" => {
                    self.bump();
                    self.diags.push(
                        Diag::new("E0204", lo, format!("`{s}` is not allowed"))
                            .fix("use `while cond invariant .., decreases .., { .. }`", None),
                    );
                    self.recover_loop_body();
                    return Err(());
                }
                "break" | "continue" => {
                    self.bump();
                    self.diags.push(
                        Diag::new(
                            "E0207",
                            lo,
                            format!("`{s}` is not supported in milestone 1"),
                        )
                        .fix("move the exit condition into the `while` condition", None),
                    );
                    return Err(());
                }
                _ => {}
            }
        }
        let e = self.parse_expr()?;
        if self.eat_p("=") {
            let value = self.parse_expr()?;
            let hi = self.expect_p(";")?;
            return Ok(StmtOrTail::Stmt(Stmt::Assign {
                target: e,
                value,
                span: lo.to(hi),
            }));
        }
        if self.eat_p(";") {
            return Ok(StmtOrTail::Stmt(Stmt::Expr(e)));
        }
        if self.is_p("}")
            || matches!(
                e.kind,
                ExprKind::If { .. } | ExprKind::Block(_) | ExprKind::Match { .. }
            )
        {
            return Ok(StmtOrTail::Tail(e));
        }
        self.err_expected("`;`");
        Err(())
    }

    /// After `for`/`loop`/`match`, skip up to and including the body block.
    fn recover_loop_body(&mut self) {
        while !self.is_p("{") && !self.is_p("}") && !matches!(self.peek(), Tok::Eof) {
            self.bump();
        }
        if self.is_p("{") {
            self.skip_group();
        }
    }

    fn parse_let(&mut self) -> PResult<Stmt> {
        let lo = self.bump().span;
        let mutable = self.eat_kw("mut");
        // Rust habit: `let Point { x, y } = p;` (the checker writes the fix).
        if matches!(self.peek(), Tok::Ident(_))
            && matches!(self.peek_at(1), Tok::P("{") | Tok::P("::"))
        {
            let mut path = vec![self.ident("type name")?];
            while self.eat_p("::") {
                path.push(self.ident("type name")?);
            }
            self.expect_p("{")?;
            let mut fields = Vec::new();
            while !self.is_p("}") && !self.is_p("..") {
                fields.push(self.ident("field name")?);
                if !self.eat_p(",") {
                    break;
                }
            }
            self.eat_p("..");
            self.expect_p("}")?;
            self.expect_p("=")?;
            let init = self.parse_expr()?;
            let hi = self.expect_p(";")?;
            return Ok(Stmt::LetFields {
                path,
                fields,
                init,
                span: lo.to(hi),
            });
        }
        // Rust habit: a refutable pattern (`let Some(x) = o else { .. };`).
        if matches!(self.peek(), Tok::Ident(_)) && matches!(self.peek_at(1), Tok::P("(")) {
            while !self.is_p(";") && !self.is_p("}") && !matches!(self.peek(), Tok::Eof) {
                if self.is_p("{") || self.is_p("(") || self.is_p("[") {
                    self.skip_group();
                } else {
                    self.bump();
                }
            }
            self.diags.push(
                Diag::new(
                    "E0226",
                    lo.to(self.prev_span()),
                    "a pattern `let` is not allowed",
                )
                .fix("`match` the value, one arm per variant", None),
            );
            self.eat_p(";");
            return Err(());
        }
        let name = self.ident("variable name")?;
        // A missing type is reported by the checker, which can infer it for the fix.
        let ty_missing = !self.is_p(":");
        let ty = if self.eat_p(":") {
            self.parse_type()
        } else {
            Type {
                kind: TypeKind::Error,
                span: name.span,
            }
        };
        self.expect_p("=")?;
        let init = self.parse_expr()?;
        if self.is_kw("else") {
            let sp = self.bump().span;
            self.skip_group();
            self.diags.push(
                Diag::new(
                    "E0226",
                    sp.to(self.prev_span()),
                    "`let .. else` is not allowed",
                )
                .fix("`match` the value, one arm per variant", None),
            );
        }
        let hi = self.expect_p(";")?;
        Ok(Stmt::Let {
            mutable,
            name,
            ty,
            ty_missing,
            init,
            span: lo.to(hi),
        })
    }

    fn parse_while(&mut self) -> PResult<Stmt> {
        self.nested(Self::parse_while_inner)
    }

    fn parse_while_inner(&mut self) -> PResult<Stmt> {
        let lo = self.bump().span;
        if self.is_kw("let") {
            self.diags.push(
                Diag::new("E0226", lo.to(self.span()), "`while let` is not allowed").fix(
                    "loop with `while` on a condition and `match` the value inside",
                    None,
                ),
            );
            self.recover_loop_body();
            return Err(());
        }
        let cond = self.cond_expr()?;
        let mut invariants = Vec::new();
        let mut decreases = None;
        if self.eat_kw("invariant") {
            invariants = self.clause_list()?;
        }
        if self.eat_kw("decreases") {
            decreases = Some(self.parse_expr()?);
            self.eat_p(",");
        }
        let body = self.parse_block()?;
        let span = lo.to(body.span);
        Ok(Stmt::While {
            cond,
            invariants,
            decreases,
            body,
            span,
        })
    }

    // ---------- expressions ----------

    fn parse_expr(&mut self) -> PResult<Expr> {
        let lhs = self.parse_or()?;
        if self.eat_p("==>") {
            let rhs = self.nested(Self::parse_expr)?;
            let span = lhs.span.to(rhs.span);
            return Ok(Expr {
                kind: ExprKind::Binary(BinOp::Implies, Box::new(lhs), Box::new(rhs)),
                span,
            });
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        self.chain(Self::parse_or_inner)
    }

    fn parse_or_inner(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_and()?;
        while self.eat_p("||") {
            self.enter()?;
            let rhs = self.parse_and()?;
            lhs = bin(BinOp::Or, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        self.chain(Self::parse_and_inner)
    }

    fn parse_and_inner(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_cmp()?;
        while self.eat_p("&&") {
            self.enter()?;
            let rhs = self.parse_cmp()?;
            lhs = bin(BinOp::And, lhs, rhs);
        }
        Ok(lhs)
    }

    fn cmp_op(&self) -> Option<BinOp> {
        Some(match self.peek() {
            Tok::P("==") => BinOp::Eq,
            Tok::P("!=") => BinOp::Ne,
            Tok::P("<") => BinOp::Lt,
            Tok::P("<=") => BinOp::Le,
            Tok::P(">") => BinOp::Gt,
            Tok::P(">=") => BinOp::Ge,
            _ => return None,
        })
    }

    fn parse_cmp(&mut self) -> PResult<Expr> {
        let lhs = self.parse_add()?;
        let Some(op) = self.cmp_op() else {
            return Ok(lhs);
        };
        self.bump();
        let rhs = self.parse_add()?;
        let e = bin(op, lhs, rhs);
        if self.cmp_op().is_some() {
            let sp = self.span();
            self.diags.push(
                Diag::new("E0211", sp, "comparison operators cannot be chained")
                    .fix("split into two comparisons joined by `&&`", None),
            );
            self.bump();
            let _ = self.parse_add()?;
        }
        Ok(e)
    }

    fn parse_add(&mut self) -> PResult<Expr> {
        self.chain(Self::parse_add_inner)
    }

    fn parse_add_inner(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_mul()?;
        loop {
            let op = if self.is_p("+") {
                BinOp::Add
            } else if self.is_p("-") {
                BinOp::Sub
            } else {
                break;
            };
            self.bump();
            self.enter()?;
            let rhs = self.parse_mul()?;
            lhs = bin(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> PResult<Expr> {
        self.chain(Self::parse_mul_inner)
    }

    fn parse_mul_inner(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = if self.is_p("*") {
                BinOp::Mul
            } else if self.is_p("/") {
                BinOp::Div
            } else if self.is_p("%") {
                BinOp::Rem
            } else {
                break;
            };
            self.bump();
            self.enter()?;
            let rhs = self.parse_unary()?;
            lhs = bin(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        self.nested(Self::parse_unary_inner)
    }

    fn parse_unary_inner(&mut self) -> PResult<Expr> {
        let lo = self.span();
        if self.eat_p("-") {
            let e = self.parse_unary()?;
            let span = lo.to(e.span);
            return Ok(Expr {
                kind: ExprKind::Unary(UnOp::Neg, Box::new(e)),
                span,
            });
        }
        if self.eat_p("!") {
            let e = self.parse_unary()?;
            let span = lo.to(e.span);
            return Ok(Expr {
                kind: ExprKind::Unary(UnOp::Not, Box::new(e)),
                span,
            });
        }
        if self.eat_p("*") {
            let e = self.parse_unary()?;
            let span = lo.to(e.span);
            return Ok(Expr {
                kind: ExprKind::Deref(Box::new(e)),
                span,
            });
        }
        if self.eat_p("&") {
            let mutable = self.eat_kw("mut");
            let e = self.parse_unary()?;
            let span = lo.to(e.span);
            return Ok(Expr {
                kind: ExprKind::Ref {
                    mutable,
                    expr: Box::new(e),
                },
                span,
            });
        }
        if self.is_p("#")
            && matches!(self.peek_at(1), Tok::P("["))
            && matches!(self.peek_at(2), Tok::Ident(s) if s == "trigger")
        {
            self.bump();
            self.bump();
            self.bump();
            self.expect_p("]")?;
            let e = self.parse_unary()?;
            let span = lo.to(e.span);
            return Ok(Expr {
                kind: ExprKind::Trigger(Box::new(e)),
                span,
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> PResult<Expr> {
        self.chain(Self::parse_postfix_inner)
    }

    fn parse_postfix_inner(&mut self) -> PResult<Expr> {
        let mut e = self.parse_primary()?;
        loop {
            if self.is_p(".") || self.is_p("[") || self.is_kw("as") {
                self.enter()?;
            }
            if self.is_p(".") {
                self.bump();
                let name = self.ident("field or method name")?;
                // `.expect("..")`: the message is dropped; the checker
                // rejects or rewrites `expect` (E0213, E0229).
                if name.name == "expect"
                    && self.is_p("(")
                    && matches!(self.peek_at(1), Tok::Str(_))
                    && matches!(self.peek_at(2), Tok::P(")"))
                {
                    self.bump();
                    self.bump();
                    let hi = self.bump().span;
                    let span = e.span.to(hi);
                    e = Expr {
                        kind: ExprKind::Method {
                            recv: Box::new(e),
                            name,
                            args: Vec::new(),
                        },
                        span,
                    };
                    continue;
                }
                if !self.is_p("(") {
                    let span = e.span.to(name.span);
                    e = Expr {
                        kind: ExprKind::Field {
                            base: Box::new(e),
                            name,
                        },
                        span,
                    };
                    continue;
                }
                let args = self.parse_args()?;
                let span = e.span.to(self.prev_span());
                e = Expr {
                    kind: ExprKind::Method {
                        recv: Box::new(e),
                        name,
                        args,
                    },
                    span,
                };
            } else if self.is_p("[") {
                self.bump();
                let index = self.parse_expr()?;
                let hi = self.expect_p("]")?;
                let span = e.span.to(hi);
                e = Expr {
                    kind: ExprKind::Index {
                        base: Box::new(e),
                        index: Box::new(index),
                    },
                    span,
                };
            } else if self.is_p("?") {
                let sp = self.bump().span;
                self.diags.push(
                    Diag::new("E0206", sp, "the `?` operator is not allowed").fix(
                        "handle the error case explicitly with `if` / `else`",
                        Some(String::new()),
                    ),
                );
            } else if self.is_kw("as") {
                self.bump();
                let ty = self.parse_type();
                let span = e.span.to(ty.span);
                let target = match ty.kind {
                    TypeKind::I64 => Some("i64"),
                    TypeKind::U64 => Some("u64"),
                    TypeKind::U8 => Some("u8"),
                    TypeKind::Usize => Some("usize"),
                    _ => None,
                };
                let d = Diag::new("E0205", span, "`as` casts are not allowed");
                match target {
                    // An integer cast is a checked conversion (ADR-0028 §8):
                    // the value is kept, its range is a proof obligation.
                    Some(t) => {
                        let x = self
                            .text
                            .get(e.span.lo as usize..e.span.hi as usize)
                            .unwrap_or_default()
                            .to_string();
                        let r = format!("{t}::try_from({}).unwrap()", unparen(&x));
                        self.diags
                            .push(d.fix("convert with a range obligation (ADR-0028)", Some(r)));
                        let ident = |name: &str| Ident {
                            name: name.into(),
                            span,
                        };
                        e = Expr {
                            kind: ExprKind::Method {
                                recv: Box::new(Expr {
                                    kind: ExprKind::Call {
                                        path: vec![ident(t), ident("try_from")],
                                        args: vec![e],
                                    },
                                    span,
                                }),
                                name: ident("unwrap"),
                                args: Vec::new(),
                            },
                            span,
                        };
                    }
                    None => {
                        self.diags.push(d.fix(
                            "only integer types convert, with `T::try_from(x).unwrap()`",
                            None,
                        ));
                        e = Expr {
                            kind: ExprKind::Error,
                            span,
                        };
                    }
                }
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn parse_args(&mut self) -> PResult<Vec<Expr>> {
        self.expect_p("(")?;
        let mut args = Vec::new();
        while !self.is_p(")") {
            args.push(self.parse_expr()?);
            if !self.eat_p(",") {
                break;
            }
        }
        self.expect_p(")")?;
        Ok(args)
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let lo = self.span();
        match self.peek().clone() {
            Tok::Int(v) => {
                self.bump();
                Ok(Expr {
                    kind: ExprKind::Int(v),
                    span: lo,
                })
            }
            Tok::Byte(v) => {
                self.bump();
                Ok(Expr {
                    kind: ExprKind::ByteLit(v),
                    span: lo,
                })
            }
            Tok::Bytes(v) => {
                self.bump();
                Ok(Expr {
                    kind: ExprKind::ByteStr(v),
                    span: lo,
                })
            }
            // Rust habit: a `&str`. Text is bytes (ADR-0044 §5).
            Tok::Str(s) => {
                self.bump();
                let r = crate::lexer::byte_text(s.as_bytes(), '"');
                self.diags.push(
                    Diag::new("E0234", lo, "string literals are byte strings")
                        .fix("write `b\"...\"` (`&[u8]`)", Some(r)),
                );
                Ok(Expr {
                    kind: ExprKind::ByteStr(s.into_bytes()),
                    span: lo,
                })
            }
            Tok::P("(") => {
                self.bump();
                if self.eat_p(")") {
                    return Ok(Expr {
                        kind: ExprKind::Unit,
                        span: lo.to(self.prev_span()),
                    });
                }
                let e = self.parse_expr()?;
                if self.is_p(",") {
                    self.diags.push(
                        Diag::new("E0233", self.span(), "tuples are not allowed")
                            .fix("use a struct with named fields", None),
                    );
                    // Skip to the closing `)`: the tuple stands for an error value.
                    while !self.is_p(")") && !matches!(self.peek(), Tok::Eof) {
                        if self.is_p("(") || self.is_p("[") || self.is_p("{") {
                            self.skip_group();
                        } else {
                            self.bump();
                        }
                    }
                    let hi = self.expect_p(")")?;
                    return Ok(Expr {
                        kind: ExprKind::Error,
                        span: lo.to(hi),
                    });
                }
                let hi = self.expect_p(")")?;
                Ok(Expr {
                    kind: e.kind,
                    span: lo.to(hi),
                })
            }
            Tok::P("[") => {
                self.bump();
                if self.eat_p("]") {
                    return Ok(Expr {
                        kind: ExprKind::Array(Vec::new()),
                        span: lo.to(self.prev_span()),
                    });
                }
                let first = self.parse_expr()?;
                if self.eat_p(";") {
                    let (count, count_span) = match self.peek().clone() {
                        Tok::Int(v) => (v, self.bump().span),
                        _ => {
                            self.err_expected("array length literal");
                            return Err(());
                        }
                    };
                    let hi = self.expect_p("]")?;
                    return Ok(Expr {
                        kind: ExprKind::Repeat {
                            value: Box::new(first),
                            count,
                            count_span,
                        },
                        span: lo.to(hi),
                    });
                }
                let mut items = vec![first];
                while self.eat_p(",") {
                    if self.is_p("]") {
                        break;
                    }
                    items.push(self.parse_expr()?);
                }
                let hi = self.expect_p("]")?;
                Ok(Expr {
                    kind: ExprKind::Array(items),
                    span: lo.to(hi),
                })
            }
            Tok::P("{") => {
                let b = self.parse_block()?;
                let span = b.span;
                Ok(Expr {
                    kind: ExprKind::Block(b),
                    span,
                })
            }
            Tok::P("|") | Tok::P("||") => {
                self.diags.push(Diag::new(
                    "E0207",
                    lo,
                    "closures are not supported in milestone 1",
                ));
                Err(())
            }
            Tok::Ident(s) => match s.as_str() {
                "true" | "false" => {
                    self.bump();
                    Ok(Expr {
                        kind: ExprKind::Bool(s == "true"),
                        span: lo,
                    })
                }
                "if" => self.parse_if(),
                "old" if matches!(self.peek_at(1), Tok::P("(")) => {
                    self.bump();
                    self.bump();
                    let e = self.parse_expr()?;
                    let hi = self.expect_p(")")?;
                    Ok(Expr {
                        kind: ExprKind::Old(Box::new(e)),
                        span: lo.to(hi),
                    })
                }
                "forall" => self.parse_forall(),
                "match" => self.parse_match(),
                "self" => {
                    let span = self.bump().span;
                    Ok(Expr {
                        kind: ExprKind::Var(Ident {
                            name: "self".into(),
                            span,
                        }),
                        span,
                    })
                }
                _ => self.parse_path_expr(),
            },
            _ => {
                self.err_expected("expression");
                Err(())
            }
        }
    }

    /// An `if`/`while` condition or `match` scrutinee: `T {` opens the body.
    fn cond_expr(&mut self) -> PResult<Expr> {
        let outer = std::mem::replace(&mut self.no_struct, true);
        let r = self.parse_expr();
        self.no_struct = outer;
        r
    }

    fn parse_if(&mut self) -> PResult<Expr> {
        let lo = self.bump().span;
        if self.is_kw("let") {
            return self.parse_if_let(lo);
        }
        let cond = self.cond_expr()?;
        let then = self.parse_block()?;
        let mut hi = then.span;
        let els = if self.eat_kw("else") {
            let e = if self.is_kw("if") {
                self.nested(Self::parse_if)?
            } else {
                let b = self.parse_block()?;
                let span = b.span;
                Expr {
                    kind: ExprKind::Block(b),
                    span,
                }
            };
            hi = e.span;
            Some(Box::new(e))
        } else {
            None
        };
        Ok(Expr {
            kind: ExprKind::If {
                cond: Box::new(cond),
                then,
                els,
            },
            span: lo.to(hi),
        })
    }

    /// Rust habit: `if let P = e { a } else { b }`, read as the `match` it
    /// stands for (`P => a`, the rest `=> b`); the checker reports E0225 with
    /// that `match` as the fix.
    fn parse_if_let(&mut self, lo: Span) -> PResult<Expr> {
        self.bump(); // `let`
        let pat = self.parse_pat()?;
        self.expect_p("=")?;
        let scrut = self.cond_expr()?;
        let then = self.parse_block()?;
        let then_span = then.span;
        let els = if self.eat_kw("else") {
            if self.is_kw("if") {
                self.nested(Self::parse_if)?
            } else {
                let b = self.parse_block()?;
                let span = b.span;
                Expr {
                    kind: ExprKind::Block(b),
                    span,
                }
            }
        } else {
            Expr {
                kind: ExprKind::Unit,
                span: then_span,
            }
        };
        let else_pat = Pat {
            path: vec![Ident {
                name: "_".into(),
                span: els.span,
            }],
            binds: Vec::new(),
            parens: false,
            origin: PatOrigin::IfLetElse,
            span: els.span,
        };
        let span = lo.to(els.span);
        let arms = vec![
            Arm {
                span: pat.span.to(then_span),
                pat,
                body: Expr {
                    kind: ExprKind::Block(then),
                    span: then_span,
                },
            },
            Arm {
                span: els.span,
                pat: else_pat,
                body: els,
            },
        ];
        Ok(Expr {
            kind: ExprKind::Match {
                scrut: Box::new(scrut),
                arms,
            },
            span,
        })
    }

    /// `match e { P => e, .. }` (ADR-0026 §3).
    fn parse_match(&mut self) -> PResult<Expr> {
        let lo = self.bump().span; // `match`
        let scrut = self.cond_expr()?;
        self.expect_p("{")?;
        let mut arms = Vec::new();
        while !self.is_p("}") && !matches!(self.peek(), Tok::Eof) {
            let pat = self.parse_pat()?;
            self.expect_p("=>")?;
            let body = self.parse_expr()?;
            let span = pat.span.to(body.span);
            let block = matches!(body.kind, ExprKind::Block(_));
            arms.push(Arm { pat, body, span });
            if !self.eat_p(",") && !block {
                break;
            }
        }
        let hi = self.expect_p("}")?;
        Ok(Expr {
            kind: ExprKind::Match {
                scrut: Box::new(scrut),
                arms,
            },
            span: lo.to(hi),
        })
    }

    /// A variant pattern: `T::V`, `T::V(a, _)`, `Some(x)`, `None`, or `_`
    /// (a wildcard arm, which the checker rejects with the arms it stands for).
    fn parse_pat(&mut self) -> PResult<Pat> {
        let lo = self.span();
        let literal = matches!(
            self.peek(),
            Tok::Int(_) | Tok::Str(_) | Tok::Byte(_) | Tok::Bytes(_) | Tok::P("-")
        ) || self.is_kw("true")
            || self.is_kw("false");
        if literal || self.is_p("(") || self.is_p("[") || self.is_p("&") {
            return Ok(self.rejected_pat(
                lo,
                "literal, tuple and reference patterns are not allowed",
                "`match` an enum value; compare other values with `if`",
            ));
        }
        let mut path = vec![self.ident("variant")?];
        while self.eat_p("::") {
            path.push(self.ident("variant")?);
        }
        if self.is_p("{") {
            return Ok(self.rejected_pat(
                lo,
                "struct patterns are not allowed",
                "bind the value and read its fields: `p.x`",
            ));
        }
        let mut binds = Vec::new();
        let parens = self.eat_p("(");
        if parens {
            while !self.is_p(")") {
                let nested = !matches!(self.peek(), Tok::Ident(_))
                    || matches!(self.peek_at(1), Tok::P("::") | Tok::P("(") | Tok::P("{"));
                if nested {
                    return Ok(self.rejected_pat(
                        lo,
                        "nested patterns are not allowed",
                        "bind a name and `match` it in the arm body",
                    ));
                }
                let b = self.ident("binding or `_`")?;
                binds.push((b.name != "_").then_some(b));
                if !self.eat_p(",") {
                    break;
                }
            }
            self.expect_p(")")?;
        }
        let mut pat = Pat {
            path,
            binds,
            parens,
            origin: PatOrigin::Written,
            span: lo.to(self.prev_span()),
        };
        // An or-pattern or a guard: the arm keeps its first pattern's
        // bindings, and covers the rest so no second error follows.
        let extra = if self.is_p("|") {
            Some((
                "or-patterns are not allowed",
                "one arm per variant, with the same body",
            ))
        } else if self.is_kw("if") {
            Some((
                "guards are not allowed",
                "test the condition in the arm body: `V(x) => if c { .. } else { .. }`",
            ))
        } else {
            None
        };
        if let Some((msg, fix)) = extra {
            let sp = self.span();
            self.skip_to_arrow();
            self.diags
                .push(Diag::new("E0232", sp.to(self.prev_span()), msg).fix(fix, None));
            pat.origin = PatOrigin::Rejected;
        }
        Ok(pat)
    }

    /// Skips the rest of a pattern, up to the arm's `=>`.
    fn skip_to_arrow(&mut self) {
        while !self.is_p("=>") && !self.is_p("}") && !matches!(self.peek(), Tok::Eof) {
            if self.is_p("(") || self.is_p("[") || self.is_p("{") {
                self.skip_group();
            } else {
                self.bump();
            }
        }
    }

    /// E0232 for a pattern form BMB has no place for; the arm covers the
    /// remaining variants so the `match` reports nothing else.
    fn rejected_pat(&mut self, lo: Span, msg: &str, fix: &str) -> Pat {
        self.skip_to_arrow();
        let span = lo.to(self.prev_span());
        self.diags
            .push(Diag::new("E0232", span, msg).fix(fix, None));
        Pat {
            path: vec![Ident {
                name: "_".into(),
                span,
            }],
            binds: Vec::new(),
            parens: false,
            origin: PatOrigin::Rejected,
            span,
        }
    }

    fn parse_forall(&mut self) -> PResult<Expr> {
        let lo = self.bump().span;
        self.expect_p("|")?;
        let mut binders = Vec::new();
        while !self.is_p("|") {
            let name = self.ident("bound variable")?;
            self.expect_p(":")?;
            let ty = self.parse_type();
            binders.push(Param { name, ty });
            if !self.eat_p(",") {
                break;
            }
        }
        self.expect_p("|")?;
        let body = self.parse_expr()?;
        let span = lo.to(body.span);
        Ok(Expr {
            kind: ExprKind::Forall {
                binders,
                body: Box::new(body),
            },
            span,
        })
    }

    fn parse_path_expr(&mut self) -> PResult<Expr> {
        let lo = self.span();
        // Macro invocation: `name!(..)` / `name![..]` / `name!{..}`.
        if let (Tok::Ident(name), Tok::P("!")) = (self.peek().clone(), self.peek_at(1).clone()) {
            self.bump();
            self.bump();
            self.skip_group();
            // `vec![..]` has no mechanical fix: `[..]` is a fixed array, a
            // `Vec` is built with `push` (ADR-0031 §7, backlog 2G-06).
            let fix = match name.as_str() {
                "println" | "print" | "eprintln" => {
                    "call `print_i64(x)` from an `#[effects(io)]` function"
                }
                "vec" => {
                    "a fixed array is `[a, b]` or `[v; N]`; a `Vec` is `Vec::with_capacity(n)` and `push`"
                }
                "assert" | "assert_eq" | "debug_assert" => {
                    "use `assert(cond);` (a proof obligation, not a runtime check)"
                }
                "panic" | "unreachable" | "todo" | "unimplemented" => {
                    "panics are not allowed; prove the case impossible"
                }
                _ => "write the expansion out explicitly",
            };
            self.diags.push(
                Diag::new(
                    "E0202",
                    lo.to(self.prev_span()),
                    format!("macro `{name}!` is not allowed"),
                )
                .fix(fix, None),
            );
            return Ok(Expr {
                kind: ExprKind::Error,
                span: lo.to(self.prev_span()),
            });
        }
        let mut path = vec![self.ident("name")?];
        while self.is_p("::") {
            self.bump();
            path.push(self.ident("path segment")?);
        }
        if self.is_p("(") {
            let args = self.parse_args()?;
            return Ok(Expr {
                kind: ExprKind::Call { path, args },
                span: lo.to(self.prev_span()),
            });
        }
        // `T { f: e, .. }`: a struct literal starts with `{ name :`, which a
        // block (`while k < n { x = 1; }`) never does. Outside conditions a
        // capitalized `T {` also opens one for the Rust habits `T { x, y }`
        // (field shorthand) and `T { x: 1, ..p }` (update syntax).
        let capital = path
            .last()
            .is_some_and(|i| i.name.starts_with(|c: char| c.is_ascii_uppercase()));
        let habit = !self.no_struct
            && capital
            && (matches!(self.peek_at(1), Tok::P(".."))
                || (matches!(self.peek_at(1), Tok::Ident(_))
                    && matches!(self.peek_at(2), Tok::P(",") | Tok::P("}"))));
        if self.is_p("{")
            && (habit
                || (matches!(self.peek_at(1), Tok::Ident(_))
                    && matches!(self.peek_at(2), Tok::P(":"))))
        {
            self.bump();
            let mut fields = Vec::new();
            let mut rest = None;
            while !self.is_p("}") {
                if self.eat_p("..") {
                    rest = Some(Box::new(self.parse_expr()?));
                    break;
                }
                let fname = self.ident("field name")?;
                let value = if self.eat_p(":") {
                    self.parse_expr()?
                } else {
                    // Rust habit: field shorthand.
                    let r = format!("{0}: {0}", fname.name);
                    self.diags.push(
                        Diag::new(
                            "E0222",
                            fname.span,
                            format!("field shorthand `{}` is not allowed", fname.name),
                        )
                        .fix(format!("write `{r}`"), Some(r)),
                    );
                    Expr {
                        kind: ExprKind::Var(fname.clone()),
                        span: fname.span,
                    }
                };
                fields.push((fname, value));
                if !self.eat_p(",") {
                    break;
                }
            }
            let hi = self.expect_p("}")?;
            return Ok(Expr {
                kind: ExprKind::StructLit { path, fields, rest },
                span: lo.to(hi),
            });
        }
        if path.len() > 1 {
            // A unit variant (`Shape::Dot`); the checker rejects other paths.
            let span = lo.to(self.prev_span());
            return Ok(Expr {
                kind: ExprKind::Path(path),
                span,
            });
        }
        let id = path.pop().unwrap();
        let span = id.span;
        Ok(Expr {
            kind: ExprKind::Var(id),
            span,
        })
    }
}

enum StmtOrTail {
    Stmt(Stmt),
    Tail(Expr),
}

fn bin(op: BinOp, l: Expr, r: Expr) -> Expr {
    let span = l.span.to(r.span);
    Expr {
        kind: ExprKind::Binary(op, Box::new(l), Box::new(r)),
        span,
    }
}

/// `(e)` without its outer parentheses when they enclose the whole text: the
/// argument of a call needs none.
fn unparen(x: &str) -> &str {
    let Some(inner) = x.strip_prefix('(').and_then(|r| r.strip_suffix(')')) else {
        return x;
    };
    let mut depth = 0i32;
    for c in inner.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return x;
        }
    }
    inner
}
