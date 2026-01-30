//! CIR to SMT-LIB2 Translator
//!
//! Translates CIR propositions and expressions to SMT-LIB2 format.
//! This is Phase 1.1 of the CIR/PIR roadmap.
//!
//! Unlike the AST-based SMT translator, this works on normalized
//! CIR propositions which makes translation more straightforward.

use std::collections::HashMap;
use std::fmt::Write;

use super::{
    BinOp, CirExpr, CirFunction, CirType, CompareOp, Proposition, UnaryOp,
};

/// SMT-LIB2 sorts (types)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmtSort {
    Int,
    Real,
    Bool,
    /// Bitvector with width
    BitVec(u32),
    /// Array sort: (Array Index Element)
    Array(Box<SmtSort>, Box<SmtSort>),
}

impl SmtSort {
    /// Convert to SMT-LIB2 string
    pub fn to_smt(&self) -> String {
        match self {
            SmtSort::Int => "Int".to_string(),
            SmtSort::Real => "Real".to_string(),
            SmtSort::Bool => "Bool".to_string(),
            SmtSort::BitVec(n) => format!("(_ BitVec {})", n),
            SmtSort::Array(idx, elem) => format!("(Array {} {})", idx.to_smt(), elem.to_smt()),
        }
    }
}

/// SMT-LIB2 generator from CIR
#[derive(Debug, Default)]
pub struct CirSmtGenerator {
    /// Variable declarations
    declarations: Vec<String>,
    /// Assertions
    assertions: Vec<String>,
    /// Function definitions (for pure functions)
    functions: Vec<String>,
    /// Variable types
    var_types: HashMap<String, SmtSort>,
    /// Logic to use
    logic: String,
}

impl CirSmtGenerator {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
            assertions: Vec::new(),
            functions: Vec::new(),
            var_types: HashMap::new(),
            logic: "QF_LIA".to_string(), // Default: Quantifier-Free Linear Integer Arithmetic
        }
    }

    /// Set the logic to use
    pub fn set_logic(&mut self, logic: &str) {
        self.logic = logic.to_string();
    }

    /// Use quantified logic with arrays
    pub fn use_array_logic(&mut self) {
        self.logic = "AUFLIA".to_string(); // Arrays + Uninterpreted Functions + Linear Integer Arithmetic
    }

    /// Declare a variable
    pub fn declare_var(&mut self, name: &str, sort: SmtSort) {
        self.declarations.push(format!(
            "(declare-const {} {})",
            sanitize_name(name),
            sort.to_smt()
        ));
        self.var_types.insert(name.to_string(), sort);
    }

    /// Declare a function
    pub fn declare_fun(&mut self, name: &str, params: &[SmtSort], ret: SmtSort) {
        let params_str: Vec<String> = params.iter().map(|s| s.to_smt()).collect();
        self.functions.push(format!(
            "(declare-fun {} ({}) {})",
            sanitize_name(name),
            params_str.join(" "),
            ret.to_smt()
        ));
    }

    /// Add an assertion
    pub fn assert(&mut self, expr: &str) {
        self.assertions.push(format!("(assert {})", expr));
    }

    /// Add a proposition as assertion
    pub fn assert_proposition(&mut self, prop: &Proposition) -> Result<(), SmtError> {
        let smt = self.translate_proposition(prop)?;
        self.assert(&smt);
        Ok(())
    }

    /// Generate verification query for a function
    /// Checks: preconditions => body_post => postconditions
    pub fn generate_verification_query(&mut self, func: &CirFunction) -> Result<String, SmtError> {
        // Declare parameters
        for param in &func.params {
            let sort = self.cir_type_to_sort(&param.ty);
            self.declare_var(&param.name, sort);
        }

        // Declare return value
        let ret_sort = self.cir_type_to_sort(&func.ret_ty);
        self.declare_var(&func.ret_name, ret_sort);

        // Build precondition conjunction
        let pre_props: Vec<String> = func.preconditions
            .iter()
            .filter_map(|np| self.translate_proposition(&np.proposition).ok())
            .collect();

        let precond = if pre_props.is_empty() {
            "true".to_string()
        } else if pre_props.len() == 1 {
            pre_props.into_iter().next().unwrap()
        } else {
            format!("(and {})", pre_props.join(" "))
        };

        // Build postcondition conjunction
        let post_props: Vec<String> = func.postconditions
            .iter()
            .filter_map(|np| self.translate_proposition(&np.proposition).ok())
            .collect();

        let postcond = if post_props.is_empty() {
            "true".to_string()
        } else if post_props.len() == 1 {
            post_props.into_iter().next().unwrap()
        } else {
            format!("(and {})", post_props.join(" "))
        };

        // Assert: precondition AND NOT postcondition (to find counterexample)
        // If unsat, the postcondition is valid given precondition
        self.assert(&format!("(and {} (not {}))", precond, postcond));

        Ok(self.generate())
    }

    /// Generate SMT-LIB2 script
    pub fn generate(&self) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "; Generated by BMB CIR SMT Generator").unwrap();
        writeln!(output, "(set-logic {})", self.logic).unwrap();
        writeln!(output).unwrap();

        // Function declarations
        for func in &self.functions {
            writeln!(output, "{}", func).unwrap();
        }
        if !self.functions.is_empty() {
            writeln!(output).unwrap();
        }

        // Variable declarations
        for decl in &self.declarations {
            writeln!(output, "{}", decl).unwrap();
        }
        if !self.declarations.is_empty() {
            writeln!(output).unwrap();
        }

        // Assertions
        for assertion in &self.assertions {
            writeln!(output, "{}", assertion).unwrap();
        }
        writeln!(output).unwrap();

        // Check satisfiability
        writeln!(output, "(check-sat)").unwrap();
        writeln!(output, "(get-model)").unwrap();

        output
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.declarations.clear();
        self.assertions.clear();
        self.functions.clear();
        self.var_types.clear();
    }

    /// Translate a CIR proposition to SMT-LIB2 string
    pub fn translate_proposition(&self, prop: &Proposition) -> Result<String, SmtError> {
        match prop {
            Proposition::True => Ok("true".to_string()),
            Proposition::False => Ok("false".to_string()),

            Proposition::Compare { lhs, op, rhs } => {
                let l = self.translate_expr(lhs)?;
                let r = self.translate_expr(rhs)?;
                let op_str = match op {
                    CompareOp::Lt => "<",
                    CompareOp::Le => "<=",
                    CompareOp::Gt => ">",
                    CompareOp::Ge => ">=",
                    CompareOp::Eq => "=",
                    CompareOp::Ne => return Ok(format!("(not (= {} {}))", l, r)),
                };
                Ok(format!("({} {} {})", op_str, l, r))
            }

            Proposition::Not(inner) => {
                let inner_smt = self.translate_proposition(inner)?;
                Ok(format!("(not {})", inner_smt))
            }

            Proposition::And(props) => {
                if props.is_empty() {
                    return Ok("true".to_string());
                }
                let parts: Result<Vec<_>, _> = props.iter()
                    .map(|p| self.translate_proposition(p))
                    .collect();
                let parts = parts?;
                if parts.len() == 1 {
                    Ok(parts.into_iter().next().unwrap())
                } else {
                    Ok(format!("(and {})", parts.join(" ")))
                }
            }

            Proposition::Or(props) => {
                if props.is_empty() {
                    return Ok("false".to_string());
                }
                let parts: Result<Vec<_>, _> = props.iter()
                    .map(|p| self.translate_proposition(p))
                    .collect();
                let parts = parts?;
                if parts.len() == 1 {
                    Ok(parts.into_iter().next().unwrap())
                } else {
                    Ok(format!("(or {})", parts.join(" ")))
                }
            }

            Proposition::Implies(lhs, rhs) => {
                let l = self.translate_proposition(lhs)?;
                let r = self.translate_proposition(rhs)?;
                Ok(format!("(=> {} {})", l, r))
            }

            Proposition::Forall { var, ty, body } => {
                let sort = self.cir_type_to_sort(ty);
                let body_smt = self.translate_proposition(body)?;
                Ok(format!("(forall (({} {})) {})", sanitize_name(var), sort.to_smt(), body_smt))
            }

            Proposition::Exists { var, ty, body } => {
                let sort = self.cir_type_to_sort(ty);
                let body_smt = self.translate_proposition(body)?;
                Ok(format!("(exists (({} {})) {})", sanitize_name(var), sort.to_smt(), body_smt))
            }

            Proposition::Predicate { name, args } => {
                let args_smt: Result<Vec<_>, _> = args.iter()
                    .map(|a| self.translate_expr(a))
                    .collect();
                let args_smt = args_smt?;
                if args_smt.is_empty() {
                    Ok(sanitize_name(name))
                } else {
                    Ok(format!("({} {})", sanitize_name(name), args_smt.join(" ")))
                }
            }

            Proposition::InBounds { index, array } => {
                let idx = self.translate_expr(index)?;
                let arr = self.translate_expr(array)?;
                // in_bounds(i, arr) = 0 <= i && i < len(arr)
                Ok(format!("(and (>= {} 0) (< {} (len {})))", idx, idx, arr))
            }

            Proposition::NonNull(expr) => {
                let e = self.translate_expr(expr)?;
                // non_null(ptr) = ptr != 0
                Ok(format!("(not (= {} 0))", e))
            }

            Proposition::Old(expr, prop) => {
                // Old value reference - translate as expr_old
                let e = self.translate_expr(expr)?;
                let p = self.translate_proposition(prop)?;
                // This is simplified - proper handling would use pre-state variables
                Ok(format!("(let (({}_old {})) {})", e, e, p))
            }
        }
    }

    /// Translate a CIR expression to SMT-LIB2 string
    pub fn translate_expr(&self, expr: &CirExpr) -> Result<String, SmtError> {
        match expr {
            CirExpr::IntLit(n) => {
                if *n >= 0 {
                    Ok(n.to_string())
                } else {
                    Ok(format!("(- {})", -n))
                }
            }

            CirExpr::FloatLit(bits) => {
                // Approximate as integer for SMT
                let f = f64::from_bits(*bits);
                let n = f as i64;
                if n >= 0 {
                    Ok(n.to_string())
                } else {
                    Ok(format!("(- {})", -n))
                }
            }

            CirExpr::BoolLit(b) => Ok(b.to_string()),

            CirExpr::StringLit(_) => {
                // Strings not fully supported - use 0
                Ok("0".to_string())
            }

            CirExpr::Var(name) => Ok(sanitize_name(name)),

            CirExpr::BinOp { op, lhs, rhs } => {
                let l = self.translate_expr(lhs)?;
                let r = self.translate_expr(rhs)?;
                self.translate_binop(*op, &l, &r)
            }

            CirExpr::UnaryOp { op, operand } => {
                let e = self.translate_expr(operand)?;
                self.translate_unaryop(*op, &e)
            }

            CirExpr::Call { func, args } => {
                let args_smt: Result<Vec<_>, _> = args.iter()
                    .map(|a| self.translate_expr(a))
                    .collect();
                let args_smt = args_smt?;
                if args_smt.is_empty() {
                    Ok(sanitize_name(func))
                } else {
                    Ok(format!("({} {})", sanitize_name(func), args_smt.join(" ")))
                }
            }

            CirExpr::Index { base, index } => {
                let b = self.translate_expr(base)?;
                let i = self.translate_expr(index)?;
                // Array access: (select arr idx)
                Ok(format!("(select {} {})", b, i))
            }

            CirExpr::Field { base, field } => {
                let b = self.translate_expr(base)?;
                // Field access as function application
                Ok(format!("({}_get_{} {})", b, sanitize_name(field), b))
            }

            CirExpr::If { cond, then_branch, else_branch } => {
                let c = self.translate_expr(cond)?;
                let t = self.translate_expr(then_branch)?;
                let e = self.translate_expr(else_branch)?;
                Ok(format!("(ite {} {} {})", c, t, e))
            }

            CirExpr::Let { name, value, body, .. } => {
                let v = self.translate_expr(value)?;
                let b = self.translate_expr(body)?;
                Ok(format!("(let (({} {})) {})", sanitize_name(name), v, b))
            }

            CirExpr::LetMut { name, value, body, .. } => {
                // Same as Let for SMT purposes
                let v = self.translate_expr(value)?;
                let b = self.translate_expr(body)?;
                Ok(format!("(let (({} {})) {})", sanitize_name(name), v, b))
            }

            CirExpr::Len(arr) => {
                let a = self.translate_expr(arr)?;
                Ok(format!("(len {})", a))
            }

            CirExpr::Old(expr) => {
                let e = self.translate_expr(expr)?;
                Ok(format!("{}_old", e))
            }

            CirExpr::Unit => Ok("true".to_string()),

            // Unsupported in SMT
            CirExpr::Assign { .. } |
            CirExpr::While { .. } |
            CirExpr::Loop { .. } |
            CirExpr::For { .. } |
            CirExpr::Break(_) |
            CirExpr::Continue |
            CirExpr::Block(_) |
            CirExpr::Struct { .. } |
            CirExpr::Array(_) |
            CirExpr::Tuple(_) |
            CirExpr::Ref(_) |
            CirExpr::RefMut(_) |
            CirExpr::Deref(_) |
            CirExpr::Range { .. } |
            CirExpr::EnumVariant { .. } |
            CirExpr::StateRef { .. } |
            CirExpr::Closure { .. } |
            CirExpr::Cast { .. } |
            CirExpr::Sizeof(_) |
            CirExpr::Forall { .. } |
            CirExpr::Exists { .. } |
            CirExpr::Todo(_) |
            CirExpr::IndexAssign { .. } |
            CirExpr::FieldAssign { .. } |
            CirExpr::DerefStore { .. } => {
                Err(SmtError::UnsupportedExpression(format!("{:?}", expr)))
            }
        }
    }

    fn translate_binop(&self, op: BinOp, left: &str, right: &str) -> Result<String, SmtError> {
        let op_str = match op {
            BinOp::Add | BinOp::AddWrap | BinOp::AddChecked | BinOp::AddSat => "+",
            BinOp::Sub | BinOp::SubWrap | BinOp::SubChecked | BinOp::SubSat => "-",
            BinOp::Mul | BinOp::MulWrap | BinOp::MulChecked | BinOp::MulSat => "*",
            BinOp::Div => "div",
            BinOp::Mod => "mod",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Eq => "=",
            BinOp::Ne => return Ok(format!("(not (= {} {}))", left, right)),
            BinOp::And => "and",
            BinOp::Or => "or",
            BinOp::Implies => "=>",
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                return Err(SmtError::UnsupportedOperator(format!("{:?}", op)));
            }
        };
        Ok(format!("({} {} {})", op_str, left, right))
    }

    fn translate_unaryop(&self, op: UnaryOp, expr: &str) -> Result<String, SmtError> {
        match op {
            UnaryOp::Neg => Ok(format!("(- {})", expr)),
            UnaryOp::Not => Ok(format!("(not {})", expr)),
            UnaryOp::BitNot => Err(SmtError::UnsupportedOperator("bitwise not".to_string())),
        }
    }

    /// Convert CIR type to SMT sort
    pub fn cir_type_to_sort(&self, ty: &CirType) -> SmtSort {
        match ty {
            CirType::Bool => SmtSort::Bool,
            CirType::I8 | CirType::I16 | CirType::I32 | CirType::I64 | CirType::I128 |
            CirType::U8 | CirType::U16 | CirType::U32 | CirType::U64 | CirType::U128 |
            CirType::Char => SmtSort::Int,
            CirType::F32 | CirType::F64 => SmtSort::Real,
            CirType::String => SmtSort::Int,
            CirType::Unit => SmtSort::Bool,
            CirType::Array(elem, _) => {
                let elem_sort = self.cir_type_to_sort(elem);
                SmtSort::Array(Box::new(SmtSort::Int), Box::new(elem_sort))
            }
            CirType::Slice(elem) => {
                let elem_sort = self.cir_type_to_sort(elem);
                SmtSort::Array(Box::new(SmtSort::Int), Box::new(elem_sort))
            }
            CirType::Ref(inner) | CirType::RefMut(inner) | CirType::Ptr(inner) => {
                self.cir_type_to_sort(inner)
            }
            CirType::Option(inner) => self.cir_type_to_sort(inner),
            CirType::Range(_) => SmtSort::Int,
            CirType::Struct(_) | CirType::Enum(_) => SmtSort::Int,
            CirType::TypeParam(_) => SmtSort::Int,
            CirType::Generic(_, _) => SmtSort::Int,
            CirType::Tuple(_) => SmtSort::Int,
            CirType::Fn { .. } => SmtSort::Int,
            CirType::Infer => SmtSort::Int,
            CirType::Never => SmtSort::Bool,
        }
    }
}

/// Sanitize a name for SMT-LIB2
fn sanitize_name(name: &str) -> String {
    // SMT-LIB2 allows alphanumeric and underscore, but must start with letter or _
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_alphanumeric() || c == '_' {
            result.push(c);
        } else if c == ':' {
            result.push('_');
        } else if i == 0 && c.is_numeric() {
            result.push('_');
            result.push(c);
        } else {
            result.push('_');
        }
    }
    if result.is_empty() || result.chars().next().unwrap().is_numeric() {
        result = format!("_{}", result);
    }
    result
}

/// SMT translation errors
#[derive(Debug, Clone)]
pub enum SmtError {
    UnsupportedExpression(String),
    UnsupportedOperator(String),
    UnsupportedType(String),
    VerificationFailed(String),
}

impl std::fmt::Display for SmtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmtError::UnsupportedExpression(e) => write!(f, "unsupported expression: {}", e),
            SmtError::UnsupportedOperator(op) => write!(f, "unsupported operator: {}", op),
            SmtError::UnsupportedType(ty) => write!(f, "unsupported type: {}", ty),
            SmtError::VerificationFailed(msg) => write!(f, "verification failed: {}", msg),
        }
    }
}

impl std::error::Error for SmtError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_comparison() {
        let generator = CirSmtGenerator::new();
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Gt,
            rhs: Box::new(CirExpr::IntLit(0)),
        };
        let smt = generator.translate_proposition(&prop).unwrap();
        assert_eq!(smt, "(> x 0)");
    }

    #[test]
    fn test_conjunction() {
        let generator = CirSmtGenerator::new();
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
        let smt = generator.translate_proposition(&prop).unwrap();
        assert_eq!(smt, "(and (>= x 0) (< x len))");
    }

    #[test]
    fn test_implication() {
        let generator = CirSmtGenerator::new();
        let prop = Proposition::Implies(
            Box::new(Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            }),
            Box::new(Proposition::Compare {
                lhs: Box::new(CirExpr::Var("y".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            }),
        );
        let smt = generator.translate_proposition(&prop).unwrap();
        assert_eq!(smt, "(=> (> x 0) (> y 0))");
    }

    #[test]
    fn test_forall() {
        let generator = CirSmtGenerator::new();
        let prop = Proposition::Forall {
            var: "i".to_string(),
            ty: CirType::I64,
            body: Box::new(Proposition::Compare {
                lhs: Box::new(CirExpr::Var("i".to_string())),
                op: CompareOp::Ge,
                rhs: Box::new(CirExpr::IntLit(0)),
            }),
        };
        let smt = generator.translate_proposition(&prop).unwrap();
        assert_eq!(smt, "(forall ((i Int)) (>= i 0))");
    }

    #[test]
    fn test_not_equal() {
        let generator = CirSmtGenerator::new();
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Ne,
            rhs: Box::new(CirExpr::IntLit(0)),
        };
        let smt = generator.translate_proposition(&prop).unwrap();
        assert_eq!(smt, "(not (= x 0))");
    }

    #[test]
    fn test_binary_expression() {
        let generator = CirSmtGenerator::new();
        let expr = CirExpr::BinOp {
            op: BinOp::Add,
            lhs: Box::new(CirExpr::Var("x".to_string())),
            rhs: Box::new(CirExpr::IntLit(1)),
        };
        let smt = generator.translate_expr(&expr).unwrap();
        assert_eq!(smt, "(+ x 1)");
    }

    #[test]
    fn test_generate_script() {
        let mut generator = CirSmtGenerator::new();
        generator.declare_var("x", SmtSort::Int);
        generator.assert("(> x 0)");
        let script = generator.generate();
        assert!(script.contains("(declare-const x Int)"));
        assert!(script.contains("(assert (> x 0))"));
        assert!(script.contains("(check-sat)"));
    }
}
