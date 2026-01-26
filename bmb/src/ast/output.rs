//! AST Output Formatters
//!
//! Phase 14: S-expression output format for debugging and tooling

use super::expr::{BinOp, Expr, LiteralPattern, Pattern, RangeKind, StateKind, UnOp};
use super::types::Type;
use super::{
    EnumDef, ExternFn, FnDef, ImplBlock, Item, Program, StructDef, TraitDef, TypeAliasDef, UseStmt, Visibility,
};

/// Format AST as S-expression (Lisp-like notation)
pub fn to_sexpr(program: &Program) -> String {
    let mut output = String::new();
    output.push_str("(program\n");
    for item in &program.items {
        output.push_str(&format_item(item, 1));
    }
    output.push(')');
    output
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn format_item(item: &Item, level: usize) -> String {
    match item {
        Item::FnDef(f) => format_fn_def(f, level),
        Item::StructDef(s) => format_struct_def(s, level),
        Item::EnumDef(e) => format_enum_def(e, level),
        Item::ExternFn(e) => format_extern_fn(e, level),
        Item::Use(u) => format_use_stmt(u, level),
        Item::TraitDef(t) => format_trait_def(t, level),
        Item::ImplBlock(i) => format_impl_block(i, level),
        // v0.50.6: Type alias
        Item::TypeAlias(t) => format_type_alias(t, level),
    }
}

fn format_visibility(vis: &Visibility) -> &'static str {
    match vis {
        Visibility::Public => "pub",
        Visibility::Private => "priv",
    }
}

fn format_fn_def(f: &FnDef, level: usize) -> String {
    let ind = indent(level);
    let mut s = format!("{}(fn {} ", ind, f.name.node);

    // Visibility
    s.push_str(&format!(":{} ", format_visibility(&f.visibility)));

    // Type params
    if !f.type_params.is_empty() {
        s.push('<');
        s.push_str(
            &f.type_params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(" "),
        );
        s.push_str("> ");
    }

    // Params
    s.push('(');
    s.push_str(
        &f.params
            .iter()
            .map(|p| format!("({} {})", p.name.node, format_type(&p.ty.node)))
            .collect::<Vec<_>>()
            .join(" "),
    );
    s.push_str(") -> ");

    // Return type
    s.push_str(&format_type(&f.ret_ty.node));

    // Pre/Post conditions
    if let Some(pre) = &f.pre {
        s.push_str(&format!("\n{}  :pre {}", ind, format_expr(&pre.node)));
    }
    if let Some(post) = &f.post {
        s.push_str(&format!("\n{}  :post {}", ind, format_expr(&post.node)));
    }

    // Body
    s.push_str(&format!("\n{}  ", ind));
    s.push_str(&format_expr(&f.body.node));
    s.push_str(")\n");
    s
}

fn format_struct_def(s: &StructDef, level: usize) -> String {
    let ind = indent(level);
    let mut out = format!("{}(struct {} ", ind, s.name.node);

    // Type params
    if !s.type_params.is_empty() {
        out.push('<');
        out.push_str(
            &s.type_params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(" "),
        );
        out.push_str("> ");
    }

    out.push('(');
    out.push_str(
        &s.fields
            .iter()
            .map(|f| format!("({} {})", f.name.node, format_type(&f.ty.node)))
            .collect::<Vec<_>>()
            .join(" "),
    );
    out.push_str("))\n");
    out
}

fn format_enum_def(e: &EnumDef, level: usize) -> String {
    let ind = indent(level);
    let mut out = format!("{}(enum {} ", ind, e.name.node);

    // Type params
    if !e.type_params.is_empty() {
        out.push('<');
        out.push_str(
            &e.type_params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(" "),
        );
        out.push_str("> ");
    }

    out.push('(');
    out.push_str(
        &e.variants
            .iter()
            .map(|v| {
                if v.fields.is_empty() {
                    v.name.node.clone()
                } else {
                    format!(
                        "({} {})",
                        v.name.node,
                        v.fields
                            .iter()
                            .map(|t| format_type(&t.node))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    );
    out.push_str("))\n");
    out
}

fn format_extern_fn(e: &ExternFn, level: usize) -> String {
    let ind = indent(level);
    let params = e
        .params
        .iter()
        .map(|p| format!("({} {})", p.name.node, format_type(&p.ty.node)))
        .collect::<Vec<_>>()
        .join(" ");
    // v0.20.2: Include ABI in output
    let abi_str = format!(" :abi \"{}\"", e.abi);
    format!(
        "{}(extern-fn {} ({}) -> {}{})\n",
        ind,
        e.name.node,
        params,
        format_type(&e.ret_ty.node),
        abi_str
    )
}

fn format_use_stmt(u: &UseStmt, level: usize) -> String {
    let ind = indent(level);
    let path_str = u
        .path
        .iter()
        .map(|s| s.node.as_str())
        .collect::<Vec<_>>()
        .join("::");
    format!("{}(use {})\n", ind, path_str)
}

// v0.20.1: Trait definition formatting
fn format_trait_def(t: &TraitDef, level: usize) -> String {
    let ind = indent(level);
    let methods = t
        .methods
        .iter()
        .map(|m| {
            let params = m
                .params
                .iter()
                .map(|p| format!("({} {})", p.name.node, format_type(&p.ty.node)))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{}  (fn {} ({}) -> {})", ind, m.name.node, params, format_type(&m.ret_ty.node))
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{}(trait {}\n{})\n",
        ind, t.name.node, methods
    )
}

// v0.20.1: Impl block formatting
fn format_impl_block(i: &ImplBlock, level: usize) -> String {
    let ind = indent(level);
    let methods = i
        .methods
        .iter()
        .map(|m| format_fn_def(m, level + 1))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "{}(impl {} for {}\n{}{})\n",
        ind,
        i.trait_name.node,
        format_type(&i.target_type.node),
        methods,
        ind
    )
}

// v0.50.6: Type alias formatting
fn format_type_alias(t: &TypeAliasDef, level: usize) -> String {
    let ind = indent(level);
    let mut out = format!("{}(type {} ", ind, t.name.node);

    // Type params
    if !t.type_params.is_empty() {
        out.push('<');
        out.push_str(
            &t.type_params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(" "),
        );
        out.push_str("> ");
    }

    // Target type
    out.push_str(&format_type(&t.target.node));

    // Refinement condition
    if let Some(ref refinement) = t.refinement {
        out.push_str(&format!(" :where {}", format_expr(&refinement.node)));
    }

    out.push_str(")\n");
    out
}

/// v0.84: Format type as string (span-agnostic)
/// Used for semantic duplication detection
pub fn format_type(ty: &Type) -> String {
    match ty {
        Type::I64 => "i64".to_string(),
        Type::I32 => "i32".to_string(),
        // v0.38: Unsigned types
        Type::U32 => "u32".to_string(),
        Type::U64 => "u64".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Unit => "()".to_string(),
        Type::String => "String".to_string(),
        // v0.64: Char type
        Type::Char => "char".to_string(),
        Type::Range(inner) => format!("(Range {})", format_type(inner)),
        Type::Named(name) => name.clone(),
        Type::TypeVar(name) => format!("'{}", name),
        Type::Generic { name, type_args } => {
            let args = type_args
                .iter()
                .map(|a| format_type(a))
                .collect::<Vec<_>>()
                .join(" ");
            format!("({} {})", name, args)
        }
        Type::Struct { name, .. } => name.clone(),
        Type::Enum { name, .. } => name.clone(),
        Type::Array(inner, size) => format!("[{} {}]", format_type(inner), size),
        Type::Ref(inner) => format!("(&{})", format_type(inner)),
        Type::RefMut(inner) => format!("(&mut {})", format_type(inner)),
        Type::Refined { base, constraints } => {
            let constrs = constraints
                .iter()
                .map(|c| format_expr(&c.node))
                .collect::<Vec<_>>()
                .join(" ");
            format!("(refined {} {})", format_type(base), constrs)
        }
        // v0.20.0: Fn type
        Type::Fn { params, ret } => {
            let params_str = params
                .iter()
                .map(|p| format_type(p))
                .collect::<Vec<_>>()
                .join(" ");
            format!("(fn ({}) {})", params_str, format_type(ret))
        }
        // v0.31: Never type
        Type::Never => "!".to_string(),
        // v0.37: Nullable type
        Type::Nullable(inner) => format!("{}?", format_type(inner)),
        // v0.42: Tuple type
        Type::Tuple(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|t| format_type(t)).collect();
            format!("({})", elems_str.join(", "))
        }
        // v0.51.37: Pointer type
        Type::Ptr(inner) => format!("*{}", format_type(inner)),
    }
}

/// v0.84: Format expression as S-expression (span-agnostic)
/// Used for semantic duplication detection
pub fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::IntLit(n) => n.to_string(),
        Expr::FloatLit(f) => f.to_string(),
        Expr::BoolLit(b) => b.to_string(),
        Expr::StringLit(s) => format!("\"{}\"", s.escape_default()),
        // v0.64: Character literal
        Expr::CharLit(c) => format!("'{}'", c.escape_default()),
        // v0.51.40: Null pointer literal
        Expr::Null => "null".to_string(),
        // v0.51.41: Sizeof expression
        Expr::Sizeof { ty } => format!("(sizeof {})", format_type(&ty.node)),
        Expr::Unit => "()".to_string(),
        Expr::Var(name) => name.clone(),
        Expr::Ret => "ret".to_string(),
        Expr::It => "it".to_string(),

        Expr::Binary { left, op, right } => {
            format!(
                "({} {} {})",
                format_binop(op),
                format_expr(&left.node),
                format_expr(&right.node)
            )
        }

        Expr::Unary { op, expr } => {
            format!("({} {})", format_unop(op), format_expr(&expr.node))
        }

        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            format!(
                "(if {} {} {})",
                format_expr(&cond.node),
                format_expr(&then_branch.node),
                format_expr(&else_branch.node)
            )
        }

        Expr::Let {
            name,
            mutable,
            ty,
            value,
            body,
        } => {
            let mut_str = if *mutable { "mut " } else { "" };
            let ty_str = ty
                .as_ref()
                .map(|t| format!(" : {}", format_type(&t.node)))
                .unwrap_or_default();
            format!(
                "(let {}{}{} {} {})",
                mut_str,
                name,
                ty_str,
                format_expr(&value.node),
                format_expr(&body.node)
            )
        }

        Expr::Assign { name, value } => {
            format!("(set! {} {})", name, format_expr(&value.node))
        }

        // v0.37: Include invariant if present
        Expr::While { cond, invariant, body } => {
            match invariant {
                Some(inv) => format!(
                    "(while {} :invariant {} {})",
                    format_expr(&cond.node),
                    format_expr(&inv.node),
                    format_expr(&body.node)
                ),
                None => format!(
                    "(while {} {})",
                    format_expr(&cond.node),
                    format_expr(&body.node)
                ),
            }
        }

        Expr::For { var, iter, body } => {
            format!(
                "(for {} {} {})",
                var,
                format_expr(&iter.node),
                format_expr(&body.node)
            )
        }

        Expr::Range { start, end, kind } => {
            let op = match kind {
                RangeKind::Exclusive => "..",
                RangeKind::Inclusive => "..=",
            };
            format!(
                "(range{} {} {})",
                op,
                format_expr(&start.node),
                format_expr(&end.node)
            )
        }

        Expr::Call { func, args } => {
            if args.is_empty() {
                format!("({})", func)
            } else {
                let args_str = args
                    .iter()
                    .map(|a| format_expr(&a.node))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("({} {})", func, args_str)
            }
        }

        Expr::Block(exprs) => {
            if exprs.is_empty() {
                "(block)".to_string()
            } else {
                let body = exprs
                    .iter()
                    .map(|e| format_expr(&e.node))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("(block {})", body)
            }
        }

        Expr::StructInit { name, fields } => {
            let fs = fields
                .iter()
                .map(|(n, v)| format!("({} {})", n.node, format_expr(&v.node)))
                .collect::<Vec<_>>()
                .join(" ");
            format!("(new {} {})", name, fs)
        }

        Expr::FieldAccess { expr, field } => {
            format!("(. {} {})", format_expr(&expr.node), field.node)
        }

        // v0.51.23: Field assignment
        Expr::FieldAssign { object, field, value } => {
            format!("(set-field {} {} {})", format_expr(&object.node), field.node, format_expr(&value.node))
        }

        // v0.43: Tuple field access
        Expr::TupleField { expr, index } => {
            format!("(tuple-field {} {})", format_expr(&expr.node), index)
        }

        Expr::EnumVariant {
            enum_name,
            variant,
            args,
        } => {
            if args.is_empty() {
                format!("{}::{}", enum_name, variant)
            } else {
                let args_str = args
                    .iter()
                    .map(|a| format_expr(&a.node))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("({}::{} {})", enum_name, variant, args_str)
            }
        }

        Expr::Match { expr, arms } => {
            let arms_str = arms
                .iter()
                .map(|arm| {
                    format!(
                        "({} {})",
                        format_pattern(&arm.pattern.node),
                        format_expr(&arm.body.node)
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!("(match {} {})", format_expr(&expr.node), arms_str)
        }

        Expr::Ref(inner) => format!("(& {})", format_expr(&inner.node)),
        Expr::RefMut(inner) => format!("(&mut {})", format_expr(&inner.node)),
        Expr::Deref(inner) => format!("(* {})", format_expr(&inner.node)),

        Expr::ArrayLit(elems) => {
            let es = elems
                .iter()
                .map(|e| format_expr(&e.node))
                .collect::<Vec<_>>()
                .join(" ");
            format!("[{}]", es)
        }

        // v0.42: Tuple expression
        Expr::Tuple(elems) => {
            let es = elems
                .iter()
                .map(|e| format_expr(&e.node))
                .collect::<Vec<_>>()
                .join(" ");
            format!("(tuple {})", es)
        }

        Expr::Index { expr, index } => {
            format!(
                "(index {} {})",
                format_expr(&expr.node),
                format_expr(&index.node)
            )
        }

        Expr::IndexAssign { array, index, value } => {
            format!(
                "(index-assign {} {} {})",
                format_expr(&array.node),
                format_expr(&index.node),
                format_expr(&value.node)
            )
        }

        Expr::MethodCall {
            receiver,
            method,
            args,
        } => {
            if args.is_empty() {
                format!("(.{} {})", method, format_expr(&receiver.node))
            } else {
                let args_str = args
                    .iter()
                    .map(|a| format_expr(&a.node))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("(.{} {} {})", method, format_expr(&receiver.node), args_str)
            }
        }

        Expr::StateRef { expr, state } => {
            let state_str = match state {
                StateKind::Pre => "pre",
                StateKind::Post => "post",
            };
            format!("({} {})", state_str, format_expr(&expr.node))
        }

        // v0.20.0: Closure expressions
        Expr::Closure { params, ret_ty, body } => {
            let params_str = params
                .iter()
                .map(|p| {
                    if let Some(ty) = &p.ty {
                        format!("({}: {})", p.name.node, format_type(&ty.node))
                    } else {
                        p.name.node.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            let ret_str = ret_ty
                .as_ref()
                .map(|t| format!(" -> {}", format_type(&t.node)))
                .unwrap_or_default();
            format!("(fn |{}|{} {})", params_str, ret_str, format_expr(&body.node))
        }

        // v0.31: Todo expression
        Expr::Todo { message } => {
            match message {
                Some(msg) => format!("(todo \"{}\")", msg.escape_default()),
                None => "(todo)".to_string(),
            }
        }

        // v0.36: Additional control flow
        Expr::Loop { body } => format!("(loop {})", format_expr(&body.node)),
        Expr::Break { value } => match value {
            Some(v) => format!("(break {})", format_expr(&v.node)),
            None => "(break)".to_string(),
        },
        Expr::Continue => "(continue)".to_string(),
        Expr::Return { value } => match value {
            Some(v) => format!("(return {})", format_expr(&v.node)),
            None => "(return)".to_string(),
        },

        // v0.37: Quantifiers
        Expr::Forall { var, ty, body } => {
            format!(
                "(forall {} : {} {})",
                var.node,
                format_type(&ty.node),
                format_expr(&body.node)
            )
        }
        Expr::Exists { var, ty, body } => {
            format!(
                "(exists {} : {} {})",
                var.node,
                format_type(&ty.node),
                format_expr(&body.node)
            )
        }
        // v0.39: Type cast
        Expr::Cast { expr, ty } => {
            format!("({} as {})", format_expr(&expr.node), format_type(&ty.node))
        }
    }
}

fn format_literal_pattern(lit: &LiteralPattern) -> String {
    match lit {
        LiteralPattern::Int(n) => n.to_string(),
        LiteralPattern::Float(f) => f.to_string(),
        LiteralPattern::Bool(b) => b.to_string(),
        LiteralPattern::String(s) => format!("\"{}\"", s),
    }
}

fn format_pattern(pat: &Pattern) -> String {
    match pat {
        Pattern::Wildcard => "_".to_string(),
        Pattern::Var(name) => name.clone(),
        Pattern::Literal(lit) => format_literal_pattern(lit),
        // v0.41: Nested patterns in enum bindings
        Pattern::EnumVariant {
            enum_name,
            variant,
            bindings,
        } => {
            if bindings.is_empty() {
                format!("{}::{}", enum_name, variant)
            } else {
                let bs = bindings
                    .iter()
                    .map(|b| format_pattern(&b.node))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("({}::{} {})", enum_name, variant, bs)
            }
        }
        Pattern::Struct { name, fields } => {
            let fs = fields
                .iter()
                .map(|(n, p)| format!("({} {})", n.node, format_pattern(&p.node)))
                .collect::<Vec<_>>()
                .join(" ");
            format!("({} {})", name, fs)
        }
        // v0.39: Range pattern
        Pattern::Range { start, end, inclusive } => {
            let op = if *inclusive { "..=" } else { ".." };
            format!("(range {} {} {})", format_literal_pattern(start), op, format_literal_pattern(end))
        }
        // v0.40: Or-pattern
        Pattern::Or(alts) => {
            let alts_str: Vec<_> = alts.iter().map(|p| format_pattern(&p.node)).collect();
            format!("(or {})", alts_str.join(" "))
        }
        // v0.41: Binding pattern
        Pattern::Binding { name, pattern } => {
            format!("(@ {} {})", name, format_pattern(&pattern.node))
        }
        // v0.42: Tuple pattern
        Pattern::Tuple(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|p| format_pattern(&p.node)).collect();
            format!("(tuple {})", elems_str.join(" "))
        }
        // v0.44: Array pattern
        Pattern::Array(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|p| format_pattern(&p.node)).collect();
            format!("[{}]", elems_str.join(" "))
        }
        // v0.45: Array rest pattern
        Pattern::ArrayRest { prefix, suffix } => {
            let prefix_str: Vec<_> = prefix.iter().map(|p| format_pattern(&p.node)).collect();
            let suffix_str: Vec<_> = suffix.iter().map(|p| format_pattern(&p.node)).collect();
            match (prefix.is_empty(), suffix.is_empty()) {
                (true, true) => "[..]".to_string(),
                (false, true) => format!("[{} ..]", prefix_str.join(" ")),
                (true, false) => format!("[.. {}]", suffix_str.join(" ")),
                (false, false) => format!("[{} .. {}]", prefix_str.join(" "), suffix_str.join(" ")),
            }
        }
    }
}

fn format_binop(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        // v0.37: Wrapping arithmetic
        BinOp::AddWrap => "+%",
        BinOp::SubWrap => "-%",
        BinOp::MulWrap => "*%",
        // v0.38: Checked arithmetic
        BinOp::AddChecked => "+?",
        BinOp::SubChecked => "-?",
        BinOp::MulChecked => "*?",
        // v0.38: Saturating arithmetic
        BinOp::AddSat => "+|",
        BinOp::SubSat => "-|",
        BinOp::MulSat => "*|",
        BinOp::And => "and",
        BinOp::Or => "or",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        // v0.32: Shift operators
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        // v0.36: Bitwise operators
        BinOp::Band => "band",
        BinOp::Bor => "bor",
        BinOp::Bxor => "bxor",
        // v0.36: Logical implication
        BinOp::Implies => "implies",
    }
}

fn format_unop(op: &UnOp) -> &'static str {
    match op {
        UnOp::Neg => "-",
        UnOp::Not => "not",
        // v0.36: Bitwise not
        UnOp::Bnot => "bnot",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn parse_and_format(source: &str) -> String {
        let tokens = tokenize(source).expect("tokenize");
        let ast = parse("test.bmb", source, tokens).expect("parse");
        to_sexpr(&ast)
    }

    #[test]
    fn test_sexpr_simple_fn() {
        let source = "fn add(a: i64, b: i64) -> i64 = a + b;";
        let sexpr = parse_and_format(source);
        assert!(sexpr.contains("(fn add"));
        assert!(sexpr.contains("(+ a b)"));
    }

    #[test]
    fn test_sexpr_generic_enum() {
        let source = "enum Option<T> { Some(T), None }";
        let sexpr = parse_and_format(source);
        // Parser outputs T as Named type (not TypeVar) at parse time
        // Type variables are resolved during type checking
        assert!(sexpr.contains("(enum Option <T>"));
        assert!(sexpr.contains("(Some T)"));
        assert!(sexpr.contains("None"));
    }

    #[test]
    fn test_sexpr_struct() {
        let source = "struct Point { x: i64, y: i64 }";
        let sexpr = parse_and_format(source);
        assert!(sexpr.contains("(struct Point"));
        assert!(sexpr.contains("(x i64)"));
        assert!(sexpr.contains("(y i64)"));
    }

    #[test]
    fn test_sexpr_match() {
        let source = r#"
            enum Color { Red, Blue }
            fn test(c: Color) -> i64 = match c {
                Color::Red => 1,
                Color::Blue => 2,
            };
        "#;
        let sexpr = parse_and_format(source);
        assert!(sexpr.contains("(match c"));
        assert!(sexpr.contains("Color::Red"));
    }

    #[test]
    fn test_sexpr_method_call() {
        let source = "fn test(x: i64) -> bool = x.is_some();";
        let sexpr = parse_and_format(source);
        assert!(sexpr.contains("(.is_some x)"));
    }
}
