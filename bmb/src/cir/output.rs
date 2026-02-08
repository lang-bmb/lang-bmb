//! CIR Output Formatting
//!
//! Provides serialization and pretty-printing for CIR structures.

use std::fmt::{self, Write as FmtWrite};
use super::*;

/// Output formatter for CIR
pub struct CirOutput;

impl CirOutput {
    /// Format CIR program as human-readable text
    pub fn format_text(program: &CirProgram) -> String {
        let mut output = String::new();

        writeln!(output, "// CIR Output").unwrap();
        writeln!(output, "// Functions: {}", program.functions.len()).unwrap();
        writeln!(output, "// Extern Functions: {}", program.extern_fns.len()).unwrap();
        writeln!(output, "// Structs: {}", program.structs.len()).unwrap();
        writeln!(output).unwrap();

        // Output struct definitions
        for struct_def in program.structs.values() {
            Self::format_struct(&mut output, struct_def);
            writeln!(output).unwrap();
        }

        // Output extern functions
        for extern_fn in &program.extern_fns {
            Self::format_extern_fn(&mut output, extern_fn);
            writeln!(output).unwrap();
        }

        // Output functions
        for func in &program.functions {
            Self::format_function(&mut output, func);
            writeln!(output).unwrap();
        }

        output
    }

    /// Format CIR program as JSON
    pub fn format_json(program: &CirProgram) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(program)
    }

    fn format_struct(output: &mut String, struct_def: &CirStruct) {
        writeln!(output, "struct {} {{", struct_def.name).unwrap();
        for (field_name, field_ty) in &struct_def.fields {
            writeln!(output, "    {}: {},", field_name, field_ty).unwrap();
        }
        if !struct_def.invariants.is_empty() {
            writeln!(output, "    // Invariants:").unwrap();
            for inv in &struct_def.invariants {
                writeln!(output, "    //   {}", Self::format_proposition(inv)).unwrap();
            }
        }
        writeln!(output, "}}").unwrap();
    }

    fn format_extern_fn(output: &mut String, extern_fn: &CirExternFn) {
        write!(output, "extern \"{}\" fn {}(", extern_fn.module, extern_fn.name).unwrap();
        for (i, param) in extern_fn.params.iter().enumerate() {
            if i > 0 {
                write!(output, ", ").unwrap();
            }
            write!(output, "{}", param).unwrap();
        }
        writeln!(output, ") -> {} [{}]", extern_fn.ret_ty, Self::format_effects(&extern_fn.effects)).unwrap();
    }

    fn format_function(output: &mut String, func: &CirFunction) {
        // Function signature
        write!(output, "fn {}(", func.name).unwrap();
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                write!(output, ", ").unwrap();
            }
            write!(output, "{}: {}", param.name, param.ty).unwrap();
        }
        writeln!(output, ") -> {}: {} {{", func.ret_name, func.ret_ty).unwrap();

        // Preconditions
        if !func.preconditions.is_empty() {
            writeln!(output, "    // Preconditions:").unwrap();
            for pre in &func.preconditions {
                if let Some(name) = &pre.name {
                    writeln!(output, "    //   @pre({}) {}", name, Self::format_proposition(&pre.proposition)).unwrap();
                } else {
                    writeln!(output, "    //   @pre {}", Self::format_proposition(&pre.proposition)).unwrap();
                }
            }
        }

        // Postconditions
        if !func.postconditions.is_empty() {
            writeln!(output, "    // Postconditions:").unwrap();
            for post in &func.postconditions {
                if let Some(name) = &post.name {
                    writeln!(output, "    //   @post({}) {}", name, Self::format_proposition(&post.proposition)).unwrap();
                } else {
                    writeln!(output, "    //   @post {}", Self::format_proposition(&post.proposition)).unwrap();
                }
            }
        }

        // Loop invariants
        if !func.loop_invariants.is_empty() {
            writeln!(output, "    // Loop Invariants:").unwrap();
            for inv in &func.loop_invariants {
                writeln!(output, "    //   loop[{}]: {}", inv.loop_id, Self::format_proposition(&inv.invariant)).unwrap();
            }
        }

        // Effects
        writeln!(output, "    // Effects: {}", Self::format_effects(&func.effects)).unwrap();

        // Body (simplified)
        writeln!(output, "    // Body: <...>").unwrap();

        writeln!(output, "}}").unwrap();
    }

    fn format_proposition(prop: &Proposition) -> String {
        match prop {
            Proposition::True => "true".to_string(),
            Proposition::False => "false".to_string(),

            Proposition::Compare { lhs, op, rhs } => {
                format!("{} {} {}", Self::format_expr(lhs), op, Self::format_expr(rhs))
            }

            Proposition::Not(inner) => {
                format!("!{}", Self::format_proposition(inner))
            }

            Proposition::And(props) => {
                let parts: Vec<_> = props.iter().map(Self::format_proposition).collect();
                format!("({})", parts.join(" && "))
            }

            Proposition::Or(props) => {
                let parts: Vec<_> = props.iter().map(Self::format_proposition).collect();
                format!("({})", parts.join(" || "))
            }

            Proposition::Implies(lhs, rhs) => {
                format!("({} => {})", Self::format_proposition(lhs), Self::format_proposition(rhs))
            }

            Proposition::Forall { var, ty, body } => {
                format!("forall {}: {}. {}", var, ty, Self::format_proposition(body))
            }

            Proposition::Exists { var, ty, body } => {
                format!("exists {}: {}. {}", var, ty, Self::format_proposition(body))
            }

            Proposition::Predicate { name, args } => {
                let args_str: Vec<_> = args.iter().map(Self::format_expr).collect();
                format!("{}({})", name, args_str.join(", "))
            }

            Proposition::InBounds { index, array } => {
                format!("in_bounds({}, {})", Self::format_expr(index), Self::format_expr(array))
            }

            Proposition::NonNull(expr) => {
                format!("non_null({})", Self::format_expr(expr))
            }

            Proposition::Old(expr, prop) => {
                format!("old({}) : {}", Self::format_expr(expr), Self::format_proposition(prop))
            }
        }
    }

    fn format_expr(expr: &CirExpr) -> String {
        match expr {
            CirExpr::IntLit(n) => n.to_string(),
            CirExpr::FloatLit(bits) => format!("{:.6}", f64::from_bits(*bits)),
            CirExpr::BoolLit(b) => b.to_string(),
            CirExpr::StringLit(s) => format!("\"{}\"", s),
            CirExpr::Var(name) => name.clone(),
            CirExpr::BinOp { op, lhs, rhs } => {
                format!("({} {:?} {})", Self::format_expr(lhs), op, Self::format_expr(rhs))
            }
            CirExpr::UnaryOp { op, operand } => {
                format!("{:?}({})", op, Self::format_expr(operand))
            }
            CirExpr::Call { func, args } => {
                let args_str: Vec<_> = args.iter().map(Self::format_expr).collect();
                format!("{}({})", func, args_str.join(", "))
            }
            CirExpr::Index { base, index } => {
                format!("{}[{}]", Self::format_expr(base), Self::format_expr(index))
            }
            CirExpr::Field { base, field } => {
                format!("{}.{}", Self::format_expr(base), field)
            }
            CirExpr::Len(expr) => {
                format!("len({})", Self::format_expr(expr))
            }
            CirExpr::Unit => "()".to_string(),
            _ => "<expr>".to_string(),
        }
    }

    fn format_effects(effects: &EffectSet) -> String {
        let mut parts = Vec::new();

        if effects.is_const {
            parts.push("const");
        } else if effects.is_pure {
            parts.push("pure");
        }

        if effects.reads {
            parts.push("reads");
        }
        if effects.writes {
            parts.push("writes");
        }
        if effects.io {
            parts.push("io");
        }
        if effects.allocates {
            parts.push("alloc");
        }
        if effects.diverges {
            parts.push("diverge");
        }

        if parts.is_empty() {
            "impure".to_string()
        } else {
            parts.join(", ")
        }
    }
}

impl fmt::Display for Proposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", CirOutput::format_proposition(self))
    }
}

impl fmt::Display for CirExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", CirOutput::format_expr(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_proposition_compare() {
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Gt,
            rhs: Box::new(CirExpr::IntLit(0)),
        };
        assert_eq!(CirOutput::format_proposition(&prop), "x > 0");
    }

    #[test]
    fn test_format_proposition_and() {
        let prop = Proposition::And(vec![
            Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Ge,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
            Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Lt,
                rhs: Box::new(CirExpr::Var("len".to_string())),
            },
        ]);
        assert_eq!(CirOutput::format_proposition(&prop), "(x >= 0 && x < len)");
    }

    #[test]
    fn test_format_effects_pure() {
        let effects = EffectSet::pure();
        assert_eq!(CirOutput::format_effects(&effects), "pure");
    }

    #[test]
    fn test_format_effects_const() {
        let effects = EffectSet::const_();
        assert_eq!(CirOutput::format_effects(&effects), "const");
    }

    // ---- Cycle 67: Additional output tests ----

    #[test]
    fn test_format_proposition_true_false() {
        assert_eq!(CirOutput::format_proposition(&Proposition::True), "true");
        assert_eq!(CirOutput::format_proposition(&Proposition::False), "false");
    }

    #[test]
    fn test_format_proposition_not() {
        let prop = Proposition::Not(Box::new(Proposition::True));
        assert_eq!(CirOutput::format_proposition(&prop), "!true");
    }

    #[test]
    fn test_format_proposition_or() {
        let prop = Proposition::Or(vec![
            Proposition::True,
            Proposition::False,
        ]);
        assert_eq!(CirOutput::format_proposition(&prop), "(true || false)");
    }

    #[test]
    fn test_format_proposition_implies() {
        let prop = Proposition::Implies(
            Box::new(Proposition::True),
            Box::new(Proposition::False),
        );
        assert_eq!(CirOutput::format_proposition(&prop), "(true => false)");
    }

    #[test]
    fn test_format_proposition_forall() {
        let prop = Proposition::Forall {
            var: "i".to_string(),
            ty: CirType::I64,
            body: Box::new(Proposition::True),
        };
        assert_eq!(CirOutput::format_proposition(&prop), "forall i: i64. true");
    }

    #[test]
    fn test_format_proposition_exists() {
        let prop = Proposition::Exists {
            var: "x".to_string(),
            ty: CirType::Bool,
            body: Box::new(Proposition::False),
        };
        assert_eq!(CirOutput::format_proposition(&prop), "exists x: bool. false");
    }

    #[test]
    fn test_format_proposition_predicate() {
        let prop = Proposition::Predicate {
            name: "is_sorted".to_string(),
            args: vec![CirExpr::Var("arr".to_string())],
        };
        assert_eq!(CirOutput::format_proposition(&prop), "is_sorted(arr)");
    }

    #[test]
    fn test_format_proposition_in_bounds() {
        let prop = Proposition::InBounds {
            index: Box::new(CirExpr::Var("i".to_string())),
            array: Box::new(CirExpr::Var("arr".to_string())),
        };
        assert_eq!(CirOutput::format_proposition(&prop), "in_bounds(i, arr)");
    }

    #[test]
    fn test_format_proposition_non_null() {
        let prop = Proposition::NonNull(Box::new(CirExpr::Var("ptr".to_string())));
        assert_eq!(CirOutput::format_proposition(&prop), "non_null(ptr)");
    }

    #[test]
    fn test_format_proposition_old() {
        let prop = Proposition::Old(
            Box::new(CirExpr::Var("x".to_string())),
            Box::new(Proposition::True),
        );
        assert_eq!(CirOutput::format_proposition(&prop), "old(x) : true");
    }

    #[test]
    fn test_format_expr_literals() {
        assert_eq!(CirOutput::format_expr(&CirExpr::IntLit(42)), "42");
        assert_eq!(CirOutput::format_expr(&CirExpr::BoolLit(true)), "true");
        assert_eq!(CirOutput::format_expr(&CirExpr::StringLit("hello".to_string())), "\"hello\"");
        assert_eq!(CirOutput::format_expr(&CirExpr::Unit), "()");
    }

    #[test]
    fn test_format_expr_float() {
        let bits = 1.5_f64.to_bits();
        let result = CirOutput::format_expr(&CirExpr::FloatLit(bits));
        assert!(result.starts_with("1.5"), "Expected 1.5..., got {}", result);
    }

    #[test]
    fn test_format_expr_call() {
        let expr = CirExpr::Call {
            func: "add".to_string(),
            args: vec![CirExpr::IntLit(1), CirExpr::IntLit(2)],
        };
        assert_eq!(CirOutput::format_expr(&expr), "add(1, 2)");
    }

    #[test]
    fn test_format_expr_index() {
        let expr = CirExpr::Index {
            base: Box::new(CirExpr::Var("arr".to_string())),
            index: Box::new(CirExpr::IntLit(0)),
        };
        assert_eq!(CirOutput::format_expr(&expr), "arr[0]");
    }

    #[test]
    fn test_format_expr_field() {
        let expr = CirExpr::Field {
            base: Box::new(CirExpr::Var("point".to_string())),
            field: "x".to_string(),
        };
        assert_eq!(CirOutput::format_expr(&expr), "point.x");
    }

    #[test]
    fn test_format_expr_len() {
        let expr = CirExpr::Len(Box::new(CirExpr::Var("arr".to_string())));
        assert_eq!(CirOutput::format_expr(&expr), "len(arr)");
    }

    #[test]
    fn test_format_effects_impure() {
        let effects = EffectSet::impure();
        assert_eq!(CirOutput::format_effects(&effects), "impure");
    }

    #[test]
    fn test_format_effects_with_io() {
        let mut effects = EffectSet::pure();
        effects.io = true;
        assert_eq!(CirOutput::format_effects(&effects), "pure, io");
    }

    #[test]
    fn test_format_effects_reads_writes() {
        let mut effects = EffectSet::impure();
        effects.reads = true;
        effects.writes = true;
        assert_eq!(CirOutput::format_effects(&effects), "reads, writes");
    }

    #[test]
    fn test_format_text_empty_program() {
        let program = CirProgram {
            functions: vec![],
            extern_fns: vec![],
            structs: std::collections::HashMap::new(),
            type_invariants: std::collections::HashMap::new(),
        };
        let text = CirOutput::format_text(&program);
        assert!(text.contains("// CIR Output"));
        assert!(text.contains("Functions: 0"));
        assert!(text.contains("Extern Functions: 0"));
        assert!(text.contains("Structs: 0"));
    }

    #[test]
    fn test_format_json_empty_program() {
        let program = CirProgram {
            functions: vec![],
            extern_fns: vec![],
            structs: std::collections::HashMap::new(),
            type_invariants: std::collections::HashMap::new(),
        };
        let json = CirOutput::format_json(&program).unwrap();
        assert!(json.contains("functions"));
        assert!(json.contains("extern_fns"));
    }

    #[test]
    fn test_proposition_display_trait() {
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Eq,
            rhs: Box::new(CirExpr::IntLit(0)),
        };
        assert_eq!(format!("{}", prop), "x == 0");
    }

    #[test]
    fn test_cir_expr_display_trait() {
        let expr = CirExpr::Var("hello".to_string());
        assert_eq!(format!("{}", expr), "hello");
    }
}
