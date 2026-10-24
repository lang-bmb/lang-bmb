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
}

/// Rust constructs recognised only to be rejected with a precise message.
const UNSUPPORTED_ITEMS: &[&str] = &[
    "struct",
    "enum",
    "impl",
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

impl<'a> Parser<'a> {
    pub fn new(toks: Vec<Token>, diags: &'a mut Diags) -> Self {
        Parser {
            toks,
            pos: 0,
            diags,
        }
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
            Tok::P(p) => format!("`{p}`"),
            Tok::Eof => "end of file".into(),
        }
    }
    fn err_expected(&mut self, what: &str) {
        let found = Self::describe(self.peek());
        self.diags.push(Diag::new(
            "E0103",
            self.span(),
            format!("expected {what}, found {found}"),
        ));
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
                matches!(s.as_str(), "fn" | "spec" | "mod" | "pub")
                    || UNSUPPORTED_ITEMS.contains(&s.as_str())
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
                            let sp = self.bump().span;
                            args.push(AttrArg::Ident(Ident { name: s, span: sp }));
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
        let lo = self.bump().span; // `mod`
        let name = self.ident("module name")?;
        self.expect_p("{")?;
        let mut fns = Vec::new();
        let mut mods = Vec::new();
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
            } else if let Tok::Ident(s) = self.peek().clone()
                && UNSUPPORTED_ITEMS.contains(&s.as_str())
            {
                let sp = self.span();
                let fix = match s.as_str() {
                    "use" => "refer to other units by full path, e.g. `unit::f(..)`",
                    "const" | "static" => "use a `spec fn` returning the value, or a literal",
                    _ => "milestone 1 supports `fn` and `spec fn` over integers, bool and arrays",
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
        if self.is_p("<") {
            self.diags.push(Diag::new(
                "E0207",
                self.span(),
                "generic parameters are not supported in milestone 1",
            ));
            self.skip_angle();
        }
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
            let pname = self.ident("parameter name")?;
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
                    .fix("add `-> ()`", Some(" -> ()".into())),
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
                            self.bump();
                            v as u64
                        }
                        _ => {
                            self.err_expected("array length");
                            0
                        }
                    };
                    let hi = self.span();
                    let _ = self.expect_p("]");
                    return Type {
                        kind: TypeKind::Array(Box::new(elem), n),
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
            Tok::Ident(s) => {
                self.bump();
                let kind = match s.as_str() {
                    "i64" => TypeKind::I64,
                    "u64" => TypeKind::U64,
                    "usize" => TypeKind::Usize,
                    "bool" => TypeKind::Bool,
                    _ => {
                        let fix = match s.as_str() {
                            "i8" | "i16" | "i32" | "i128" | "isize" => Some("i64"),
                            "u8" | "u16" | "u32" | "u128" => Some("u64"),
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
                "match" | "break" | "continue" => {
                    self.bump();
                    let fix = if s == "match" {
                        "use `if` / `else`"
                    } else {
                        "move the exit condition into the `while` condition"
                    };
                    self.diags.push(
                        Diag::new(
                            "E0207",
                            lo,
                            format!("`{s}` is not supported in milestone 1"),
                        )
                        .fix(fix, None),
                    );
                    if s == "match" {
                        self.recover_loop_body();
                    }
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
        if self.is_p("}") || matches!(e.kind, ExprKind::If { .. } | ExprKind::Block(_)) {
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
        let name = self.ident("variable name")?;
        let ty = if self.eat_p(":") {
            self.parse_type()
        } else {
            self.diags.push(
                Diag::new(
                    "E0201",
                    name.span,
                    format!("`let {}` requires an explicit type", name.name),
                )
                .fix("add `: T` after the name", None),
            );
            Type {
                kind: TypeKind::Error,
                span: name.span,
            }
        };
        self.expect_p("=")?;
        let init = self.parse_expr()?;
        let hi = self.expect_p(";")?;
        Ok(Stmt::Let {
            mutable,
            name,
            ty,
            init,
            span: lo.to(hi),
        })
    }

    fn parse_while(&mut self) -> PResult<Stmt> {
        let lo = self.bump().span;
        let cond = self.parse_expr()?;
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
            let rhs = self.parse_expr()?;
            let span = lhs.span.to(rhs.span);
            return Ok(Expr {
                kind: ExprKind::Binary(BinOp::Implies, Box::new(lhs), Box::new(rhs)),
                span,
            });
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_and()?;
        while self.eat_p("||") {
            let rhs = self.parse_and()?;
            lhs = bin(BinOp::Or, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_cmp()?;
        while self.eat_p("&&") {
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
            let rhs = self.parse_mul()?;
            lhs = bin(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> PResult<Expr> {
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
            let rhs = self.parse_unary()?;
            lhs = bin(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
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
        let mut e = self.parse_primary()?;
        loop {
            if self.is_p(".") {
                self.bump();
                let name = self.ident("method name")?;
                if !self.is_p("(") {
                    self.diags.push(Diag::new(
                        "E0207",
                        name.span,
                        "field access is not supported in milestone 1",
                    ));
                    return Err(());
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
                let sp = self.bump().span;
                let ty = self.parse_type();
                self.diags.push(
                    Diag::new("E0205", sp.to(ty.span), "`as` casts are not allowed")
                        .fix("keep values in one integer type; checked conversions come after milestone 1", None),
                );
                e = Expr {
                    kind: ExprKind::Error,
                    span: e.span.to(ty.span),
                };
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
                    self.diags.push(Diag::new(
                        "E0207",
                        self.span(),
                        "tuples are not supported in milestone 1",
                    ));
                    return Err(());
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
                _ => self.parse_path_expr(),
            },
            _ => {
                self.err_expected("expression");
                Err(())
            }
        }
    }

    fn parse_if(&mut self) -> PResult<Expr> {
        let lo = self.bump().span;
        let cond = self.parse_expr()?;
        let then = self.parse_block()?;
        let mut hi = then.span;
        let els = if self.eat_kw("else") {
            let e = if self.is_kw("if") {
                self.parse_if()?
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
            let fix = match name.as_str() {
                "println" | "print" | "eprintln" => {
                    "call `print_i64(x)` from an `#[effects(io)]` function"
                }
                "vec" => "use a fixed array: `[a, b, c]` or `[v; N]`",
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
        if path.len() > 1 {
            let span = lo.to(self.prev_span());
            self.diags.push(Diag::new(
                "E0301",
                span,
                "a path must be followed by call arguments",
            ));
            return Err(());
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
