//! Contract verification logic
//!
//! Verifies pre/post conditions for functions.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use std::collections::HashMap;

use crate::ast::{Expr, FnDef, Item, NamedContract, Program, Spanned, Type};
use crate::smt::{
    SmtLibGenerator, SmtTranslator, SmtSolver, SolverResult,
    VerifyResult, Counterexample,
};

/// Contract verifier for BMB programs
pub struct ContractVerifier {
    solver: SmtSolver,
}

impl ContractVerifier {
    /// Create a new contract verifier
    pub fn new() -> Self {
        Self {
            solver: SmtSolver::new(),
        }
    }

    /// Set custom Z3 path
    pub fn with_z3_path(mut self, path: &str) -> Self {
        self.solver = self.solver.with_path(path);
        self
    }

    /// Set timeout in seconds
    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.solver = self.solver.with_timeout(seconds);
        self
    }

    /// Check if the solver is available
    pub fn is_solver_available(&self) -> bool {
        self.solver.is_available()
    }

    /// Verify all functions in a program
    pub fn verify_program(&self, program: &Program) -> VerificationReport {
        let mut report = VerificationReport::new();

        // v0.86: Build function index for contract conflict detection (Phase 83)
        let mut function_index: HashMap<String, &FnDef> = HashMap::new();
        for item in &program.items {
            if let Item::FnDef(func) = item {
                function_index.insert(func.name.node.clone(), func);
            }
        }

        for item in &program.items {
            match item {
                Item::FnDef(func) => {
                    let func_report = self.verify_function_with_index(func, &function_index);
                    report.functions.push(func_report);
                }
                // Struct, Enum, Use, and ExternFn don't need verification
                Item::StructDef(_) | Item::EnumDef(_) | Item::Use(_) | Item::ExternFn(_) => {}
                // v0.20.1: Trait system not yet included in verification
                Item::TraitDef(_) | Item::ImplBlock(_) => {}
                // v0.50.6: Type aliases don't need verification
                Item::TypeAlias(_) => {}
            }
        }

        report
    }

    /// Verify a single function (legacy interface without function index)
    pub fn verify_function(&self, func: &FnDef) -> FunctionReport {
        self.verify_function_with_index(func, &HashMap::new())
    }

    /// Verify a single function's contracts with access to all function definitions
    fn verify_function_with_index(
        &self,
        func: &FnDef,
        function_index: &HashMap<String, &FnDef>,
    ) -> FunctionReport {
        let name = func.name.node.clone();
        let mut report = FunctionReport::new(name.clone());

        // v0.31: Check for @trust attribute - skip verification if present
        if let Some(trust_attr) = func.attributes.iter().find(|a| a.is_trust()) {
            report.pre_result = Some(VerifyResult::Verified);
            report.post_result = Some(VerifyResult::Verified);
            let reason = trust_attr.reason().unwrap_or("no reason provided");
            report.message = Some(format!("Trusted: {}", reason));
            report.trusted = true;
            return report;
        }

        // Check if function has any contracts (pre/post, named contracts, or refinement types)
        let has_return_refinement = matches!(&func.ret_ty.node, Type::Refined { .. });
        let has_contracts = func.pre.is_some()
            || func.post.is_some()
            || !func.contracts.is_empty()
            || has_return_refinement;

        if !has_contracts {
            report.pre_result = Some(VerifyResult::Verified);
            report.post_result = Some(VerifyResult::Verified);
            report.message = Some("No contracts to verify".to_string());
            return report;
        }

        // v0.31: Check for duplicate contracts
        self.detect_duplicate_contracts(func, &mut report);

        // v0.82: Check for trivial contracts (tautologies)
        self.detect_trivial_contracts(func, &mut report);

        // v0.86: Check for unsatisfiable preconditions (dead code)
        self.detect_unsatisfiable_precondition(func, &mut report);

        // v0.86: Check for contract conflicts at call sites (Phase 83)
        self.detect_contract_conflicts(func, function_index, &mut report);

        // Set up translator
        let mut generator = SmtLibGenerator::new();
        let mut translator = SmtTranslator::new();
        translator.setup_function(func, &mut generator);

        // Verify pre-condition if present
        if let Some(pre) = &func.pre {
            report.pre_result = Some(self.verify_pre(&translator, &mut generator.clone(), pre, func));
        } else {
            report.pre_result = Some(VerifyResult::Verified);
        }

        // Verify post-condition if present
        if let Some(post) = &func.post {
            report.post_result = Some(self.verify_post(&translator, &generator, post, func));
        } else {
            report.post_result = Some(VerifyResult::Verified);
        }

        // v0.2: Verify named contracts from where {} blocks
        for contract in &func.contracts {
            let contract_name = contract.name.as_ref().map(|s| s.node.clone());
            let result = self.verify_named_contract(&translator, &generator, contract, func);
            report.contract_results.push((contract_name, result));
        }

        // v0.2: Verify refinement type constraints
        // Parameter refinements are treated as preconditions (already asserted as context)
        // Return type refinements are treated as postconditions
        if let Type::Refined { constraints, .. } = &func.ret_ty.node {
            for constraint in constraints {
                let result = self.verify_return_refinement(&translator, &generator, constraint, func);
                report.refinement_results.push(("return".to_string(), result));
            }
        }

        report
    }

    /// v0.31: Detect duplicate contracts by hashing their expressions
    fn detect_duplicate_contracts(&self, func: &FnDef, report: &mut FunctionReport) {
        use std::collections::HashMap;
        let mut seen_hashes: HashMap<u64, (usize, Option<String>)> = HashMap::new();

        for (idx, contract) in func.contracts.iter().enumerate() {
            let hash = self.hash_expr(&contract.condition.node);
            let contract_name = contract.name.as_ref().map(|s| s.node.clone());

            if let Some((prev_idx, prev_name)) = seen_hashes.get(&hash) {
                let current_desc = contract_name
                    .clone()
                    .unwrap_or_else(|| format!("contract #{}", idx + 1));
                let prev_desc = prev_name
                    .clone()
                    .unwrap_or_else(|| format!("contract #{}", prev_idx + 1));
                report.warnings.push(format!(
                    "Duplicate contract: '{}' has the same condition as '{}'",
                    current_desc, prev_desc
                ));
            } else {
                seen_hashes.insert(hash, (idx, contract_name));
            }
        }
    }

    /// Compute hash of an expression for duplicate detection
    fn hash_expr(&self, expr: &Expr) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Use debug format as a canonical representation
        format!("{:?}", expr).hash(&mut hasher);
        hasher.finish()
    }

    /// v0.82: Detect trivial contracts (tautologies)
    /// A contract is trivial if NOT(contract) is unsatisfiable,
    /// meaning the contract is always true regardless of inputs
    fn detect_trivial_contracts(&self, func: &FnDef, report: &mut FunctionReport) {
        // Set up translator and generator for contract checking
        let mut generator = SmtLibGenerator::new();
        let mut translator = SmtTranslator::new();
        translator.setup_function(func, &mut generator);

        // Check precondition for tautology
        if let Some(pre) = &func.pre
            && self.is_tautology(&translator, &generator, pre)
        {
            report.warnings.push(
                "Trivial contract: precondition is always true (tautology)".to_string()
            );
        }

        // Check postcondition for tautology
        if let Some(post) = &func.post
            && self.is_tautology(&translator, &generator, post)
        {
            report.warnings.push(
                "Trivial contract: postcondition is always true (tautology)".to_string()
            );
        }

        // Check named contracts for tautology
        for contract in &func.contracts {
            if self.is_tautology(&translator, &generator, &contract.condition) {
                let contract_name = contract.name.as_ref()
                    .map(|s| format!("contract '{}'", s.node))
                    .unwrap_or_else(|| "unnamed contract".to_string());
                report.warnings.push(format!(
                    "Trivial contract: {} is always true (tautology)",
                    contract_name
                ));
            }
        }
    }

    /// v0.82: Check if an expression is a tautology (always true)
    /// Returns true if NOT(expr) is unsatisfiable
    fn is_tautology(
        &self,
        translator: &SmtTranslator,
        base_generator: &SmtLibGenerator,
        expr: &Spanned<Expr>,
    ) -> bool {
        let mut generator = base_generator.clone();

        // Translate the expression
        let smt_expr = match translator.translate(expr) {
            Ok(s) => s,
            Err(_) => return false, // Can't check, assume not tautology
        };

        // Assert negation of expression
        generator.assert(&format!("(not {})", smt_expr));

        // Generate SMT script
        let script = generator.generate();

        // If NOT(expr) is UNSAT, expr is always true (tautology)
        match self.solver.solve(&script) {
            Ok(SolverResult::Unsat) => true,  // NOT(expr) unsatisfiable → expr is tautology
            _ => false, // SAT, unknown, or error → not a tautology
        }
    }

    /// v0.86: Detect unsatisfiable preconditions (dead code)
    /// A function with an unsatisfiable precondition can never be called
    fn detect_unsatisfiable_precondition(&self, func: &FnDef, report: &mut FunctionReport) {
        let Some(pre) = &func.pre else { return };

        // Set up translator and generator
        let mut generator = SmtLibGenerator::new();
        let mut translator = SmtTranslator::new();
        translator.setup_function(func, &mut generator);

        // Translate precondition
        let pre_smt = match translator.translate(pre) {
            Ok(s) => s,
            Err(_) => return, // Can't check
        };

        // Assert precondition and check if satisfiable
        generator.assert(&pre_smt);
        let script = generator.generate();

        // If precondition is UNSAT, the function can never be called
        if matches!(self.solver.solve(&script), Ok(SolverResult::Unsat)) {
            report.warnings.push(
                "Dead code: precondition is unsatisfiable; function can never be called".to_string()
            );
        }
    }

    /// v0.86: Detect contract conflicts at call sites (Phase 83)
    /// When calling f(g()), check if g's postcondition conflicts with f's precondition
    fn detect_contract_conflicts(
        &self,
        func: &FnDef,
        function_index: &HashMap<String, &FnDef>,
        report: &mut FunctionReport,
    ) {
        // Find all call expressions in the function body
        self.check_expr_for_conflicts(&func.body.node, function_index, report);
    }

    /// v0.86: Recursively check an expression for contract conflicts
    fn check_expr_for_conflicts(
        &self,
        expr: &Expr,
        function_index: &HashMap<String, &FnDef>,
        report: &mut FunctionReport,
    ) {
        match expr {
            Expr::Call { func: callee_name, args } => {
                // Check each argument for conflicts with callee's precondition
                self.check_call_for_conflicts(callee_name, args, function_index, report);

                // Recursively check arguments
                for arg in args {
                    self.check_expr_for_conflicts(&arg.node, function_index, report);
                }
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.check_expr_for_conflicts(&stmt.node, function_index, report);
                }
            }
            Expr::If { cond, then_branch, else_branch } => {
                self.check_expr_for_conflicts(&cond.node, function_index, report);
                self.check_expr_for_conflicts(&then_branch.node, function_index, report);
                self.check_expr_for_conflicts(&else_branch.node, function_index, report);
            }
            Expr::Let { value, body, .. } => {
                self.check_expr_for_conflicts(&value.node, function_index, report);
                self.check_expr_for_conflicts(&body.node, function_index, report);
            }
            Expr::Binary { left, right, .. } => {
                self.check_expr_for_conflicts(&left.node, function_index, report);
                self.check_expr_for_conflicts(&right.node, function_index, report);
            }
            Expr::Unary { expr: inner, .. } => {
                self.check_expr_for_conflicts(&inner.node, function_index, report);
            }
            Expr::While { cond, body, invariant } => {
                self.check_expr_for_conflicts(&cond.node, function_index, report);
                self.check_expr_for_conflicts(&body.node, function_index, report);
                if let Some(inv) = invariant {
                    self.check_expr_for_conflicts(&inv.node, function_index, report);
                }
            }
            Expr::Loop { body } => {
                self.check_expr_for_conflicts(&body.node, function_index, report);
            }
            Expr::For { iter, body, .. } => {
                self.check_expr_for_conflicts(&iter.node, function_index, report);
                self.check_expr_for_conflicts(&body.node, function_index, report);
            }
            Expr::Match { expr: scrutinee, arms } => {
                self.check_expr_for_conflicts(&scrutinee.node, function_index, report);
                for arm in arms {
                    self.check_expr_for_conflicts(&arm.body.node, function_index, report);
                    if let Some(guard) = &arm.guard {
                        self.check_expr_for_conflicts(&guard.node, function_index, report);
                    }
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                self.check_expr_for_conflicts(&receiver.node, function_index, report);
                for arg in args {
                    self.check_expr_for_conflicts(&arg.node, function_index, report);
                }
            }
            Expr::ArrayLit(elems) | Expr::Tuple(elems) => {
                for elem in elems {
                    self.check_expr_for_conflicts(&elem.node, function_index, report);
                }
            }
            Expr::StructInit { fields, .. } => {
                for (_, value) in fields {
                    self.check_expr_for_conflicts(&value.node, function_index, report);
                }
            }
            Expr::Index { expr: array, index } => {
                self.check_expr_for_conflicts(&array.node, function_index, report);
                self.check_expr_for_conflicts(&index.node, function_index, report);
            }
            // v0.51: Index assignment
            Expr::IndexAssign { array, index, value } => {
                self.check_expr_for_conflicts(&array.node, function_index, report);
                self.check_expr_for_conflicts(&index.node, function_index, report);
                self.check_expr_for_conflicts(&value.node, function_index, report);
            }
            Expr::Closure { body, .. } => {
                self.check_expr_for_conflicts(&body.node, function_index, report);
            }
            Expr::Range { start, end, .. } => {
                self.check_expr_for_conflicts(&start.node, function_index, report);
                self.check_expr_for_conflicts(&end.node, function_index, report);
            }
            Expr::EnumVariant { args, .. } => {
                for arg in args {
                    self.check_expr_for_conflicts(&arg.node, function_index, report);
                }
            }
            Expr::Break { value } | Expr::Return { value } => {
                if let Some(v) = value {
                    self.check_expr_for_conflicts(&v.node, function_index, report);
                }
            }
            Expr::FieldAccess { expr: inner, .. }
            | Expr::TupleField { expr: inner, .. }
            | Expr::Deref(inner)
            | Expr::Ref(inner)
            | Expr::RefMut(inner)
            | Expr::Cast { expr: inner, .. }
            | Expr::StateRef { expr: inner, .. } => {
                self.check_expr_for_conflicts(&inner.node, function_index, report);
            }
            Expr::Assign { value, .. } => {
                self.check_expr_for_conflicts(&value.node, function_index, report);
            }
            // v0.51.23: Field assignment
            Expr::FieldAssign { object, value, .. } => {
                self.check_expr_for_conflicts(&object.node, function_index, report);
                self.check_expr_for_conflicts(&value.node, function_index, report);
            }
            Expr::Forall { body, .. } | Expr::Exists { body, .. } => {
                self.check_expr_for_conflicts(&body.node, function_index, report);
            }
            // Leaf expressions - no recursion needed
            Expr::IntLit(_) | Expr::FloatLit(_) | Expr::BoolLit(_) | Expr::StringLit(_)
            | Expr::CharLit(_) | Expr::Var(_) | Expr::Ret | Expr::Unit | Expr::It
            | Expr::Continue | Expr::Todo { .. } => {}
        }
    }

    /// v0.86: Check a specific call site for contract conflicts
    fn check_call_for_conflicts(
        &self,
        callee_name: &str,
        args: &[Spanned<Expr>],
        function_index: &HashMap<String, &FnDef>,
        report: &mut FunctionReport,
    ) {
        // Get callee function definition
        let Some(callee) = function_index.get(callee_name) else { return };

        // Get callee's precondition
        let Some(callee_pre) = &callee.pre else { return };

        // Check each argument - if it's a call with a postcondition, check for conflict
        for (param_idx, arg) in args.iter().enumerate() {
            if let Expr::Call { func: arg_func_name, .. } = &arg.node {
                // Argument is a function call - get its postcondition
                let Some(arg_func) = function_index.get(arg_func_name) else { continue };
                let Some(arg_post) = &arg_func.post else { continue };

                // Check if arg's postcondition conflicts with callee's precondition
                // We need to substitute the argument function's ret with the callee's param
                if self.check_conflict(callee, callee_pre, arg_func, arg_post, param_idx) {
                    let param_name = callee.params.get(param_idx)
                        .map(|p| p.name.node.as_str())
                        .unwrap_or("arg");
                    report.warnings.push(format!(
                        "Contract conflict: {}() returns value violating {}'s precondition on parameter '{}'",
                        arg_func_name, callee_name, param_name
                    ));
                }
            }
        }
    }

    /// v0.86: Check if argument's postcondition conflicts with callee's precondition
    fn check_conflict(
        &self,
        callee: &FnDef,
        callee_pre: &Spanned<Expr>,
        arg_func: &FnDef,
        arg_post: &Spanned<Expr>,
        param_idx: usize,
    ) -> bool {
        // Set up SMT context
        let mut generator = SmtLibGenerator::new();
        let mut translator = SmtTranslator::new();

        // Declare the callee's parameters
        translator.setup_function(callee, &mut generator);

        // Declare __arg_ret__ for the argument function's return value
        let arg_ret_sort = SmtTranslator::type_to_sort(&arg_func.ret_ty.node);
        generator.declare_var("__arg_ret__", arg_ret_sort);

        // Translate postcondition (need to substitute 'ret' with '__arg_ret__')
        // For simplicity, we'll use __ret__ and map it to the parameter
        let post_smt = match translator.translate(arg_post) {
            Ok(s) => s.replace("__ret__", "__arg_ret__"),
            Err(_) => return false,
        };

        // Assert postcondition
        generator.assert(&post_smt);

        // Map __arg_ret__ to the callee's parameter
        if let Some(param) = callee.params.get(param_idx) {
            generator.assert(&format!("(= {} __arg_ret__)", param.name.node));
        } else {
            return false;
        }

        // Translate and assert precondition
        let pre_smt = match translator.translate(callee_pre) {
            Ok(s) => s,
            Err(_) => return false,
        };
        generator.assert(&pre_smt);

        // Generate SMT script
        let script = generator.generate();

        // If (postcondition AND param=ret AND precondition) is UNSAT, there's a conflict
        matches!(self.solver.solve(&script), Ok(SolverResult::Unsat))
    }

    /// Verify pre-condition: Check that pre is satisfiable
    fn verify_pre(
        &self,
        translator: &SmtTranslator,
        generator: &mut SmtLibGenerator,
        pre: &crate::ast::Spanned<crate::ast::Expr>,
        _func: &FnDef,
    ) -> VerifyResult {
        // Translate pre-condition
        let pre_smt = match translator.translate(pre) {
            Ok(s) => s,
            Err(e) => return VerifyResult::Unknown(format!("translation error: {}", e)),
        };

        // Assert pre-condition
        generator.assert(&pre_smt);

        // Generate SMT script
        let script = generator.generate();

        // Solve
        match self.solver.solve(&script) {
            Ok(SolverResult::Sat(_)) => VerifyResult::Verified, // Pre is satisfiable
            Ok(SolverResult::Unsat) => VerifyResult::Failed(Counterexample {
                assignments: vec![("pre".to_string(), "unsatisfiable".to_string())],
            }),
            Ok(SolverResult::Unknown) | Ok(SolverResult::Timeout) => {
                VerifyResult::Unknown("solver timeout or unknown".to_string())
            }
            Err(e) => VerifyResult::Unknown(format!("solver error: {}", e)),
        }
    }

    /// Verify post-condition: Check that (pre ∧ ret = body) → post
    fn verify_post(
        &self,
        translator: &SmtTranslator,
        base_generator: &SmtLibGenerator,
        post: &crate::ast::Spanned<crate::ast::Expr>,
        func: &FnDef,
    ) -> VerifyResult {
        let mut generator = base_generator.clone();

        // Translate body
        let body_smt = match translator.translate(&func.body) {
            Ok(s) => s,
            Err(e) => return VerifyResult::Unknown(format!("body translation error: {}", e)),
        };

        // Assert: __ret__ = body
        generator.assert(&format!("(= __ret__ {})", body_smt));

        // If there's a pre-condition, assert it
        if let Some(pre) = &func.pre {
            let pre_smt = match translator.translate(pre) {
                Ok(s) => s,
                Err(e) => return VerifyResult::Unknown(format!("pre translation error: {}", e)),
            };
            generator.assert(&pre_smt);
        }

        // Translate post-condition
        let post_smt = match translator.translate(post) {
            Ok(s) => s,
            Err(e) => return VerifyResult::Unknown(format!("post translation error: {}", e)),
        };

        // Assert negation of post-condition (to find counterexample)
        generator.assert(&format!("(not {})", post_smt));

        // Generate SMT script
        let script = generator.generate();

        // Solve
        match self.solver.solve(&script) {
            Ok(SolverResult::Unsat) => VerifyResult::Verified, // No counterexample = verified
            Ok(SolverResult::Sat(model)) => {
                VerifyResult::Failed(Counterexample::from_model(model))
            }
            Ok(SolverResult::Unknown) | Ok(SolverResult::Timeout) => {
                VerifyResult::Unknown("solver timeout or unknown".to_string())
            }
            Err(e) => VerifyResult::Unknown(format!("solver error: {}", e)),
        }
    }

    /// v0.2: Verify named contract from where {} block
    /// Similar to verify_post: Check that (pre ∧ ret = body) → contract_condition
    fn verify_named_contract(
        &self,
        translator: &SmtTranslator,
        base_generator: &SmtLibGenerator,
        contract: &NamedContract,
        func: &FnDef,
    ) -> VerifyResult {
        let mut generator = base_generator.clone();

        // Translate body
        let body_smt = match translator.translate(&func.body) {
            Ok(s) => s,
            Err(e) => return VerifyResult::Unknown(format!("body translation error: {}", e)),
        };

        // Assert: __ret__ = body (or ret_name if specified)
        if let Some(ret_name) = &func.ret_name {
            generator.assert(&format!("(= {} {})", ret_name.node, body_smt));
        } else {
            generator.assert(&format!("(= __ret__ {})", body_smt));
        }

        // If there's a pre-condition, assert it
        if let Some(pre) = &func.pre {
            let pre_smt = match translator.translate(pre) {
                Ok(s) => s,
                Err(e) => return VerifyResult::Unknown(format!("pre translation error: {}", e)),
            };
            generator.assert(&pre_smt);
        }

        // Translate named contract condition
        let contract_smt = match translator.translate(&contract.condition) {
            Ok(s) => s,
            Err(e) => return VerifyResult::Unknown(format!("contract translation error: {}", e)),
        };

        // Assert negation of contract (to find counterexample)
        generator.assert(&format!("(not {})", contract_smt));

        // Generate SMT script
        let script = generator.generate();

        // Solve
        match self.solver.solve(&script) {
            Ok(SolverResult::Unsat) => VerifyResult::Verified, // No counterexample = verified
            Ok(SolverResult::Sat(model)) => {
                VerifyResult::Failed(Counterexample::from_model(model))
            }
            Ok(SolverResult::Unknown) | Ok(SolverResult::Timeout) => {
                VerifyResult::Unknown("solver timeout or unknown".to_string())
            }
            Err(e) => VerifyResult::Unknown(format!("solver error: {}", e)),
        }
    }

    /// v0.2: Verify return type refinement constraint
    /// Check that (pre ∧ ret = body) → refinement_constraint
    fn verify_return_refinement(
        &self,
        translator: &SmtTranslator,
        base_generator: &SmtLibGenerator,
        constraint: &Spanned<Expr>,
        func: &FnDef,
    ) -> VerifyResult {
        let mut generator = base_generator.clone();

        // Get return type sort for __it__ declaration
        let ret_sort = SmtTranslator::type_to_sort(&func.ret_ty.node);

        // Declare __it__ variable for refinement self-reference
        generator.declare_var("__it__", ret_sort);

        // Translate body
        let body_smt = match translator.translate(&func.body) {
            Ok(s) => s,
            Err(e) => return VerifyResult::Unknown(format!("body translation error: {}", e)),
        };

        // Assert: __ret__ = body
        generator.assert(&format!("(= __ret__ {})", body_smt));

        // Assert: __it__ = __ret__ (refinement self-reference equals return value)
        generator.assert("(= __it__ __ret__)");

        // If there's a pre-condition, assert it
        if let Some(pre) = &func.pre {
            let pre_smt = match translator.translate(pre) {
                Ok(s) => s,
                Err(e) => return VerifyResult::Unknown(format!("pre translation error: {}", e)),
            };
            generator.assert(&pre_smt);
        }

        // Translate refinement constraint
        // The 'it' keyword is translated to __it__, which equals __ret__
        let constraint_smt = match translator.translate(constraint) {
            Ok(s) => s,
            Err(e) => return VerifyResult::Unknown(format!("refinement translation error: {}", e)),
        };

        // Assert negation of constraint (to find counterexample)
        generator.assert(&format!("(not {})", constraint_smt));

        // Generate SMT script
        let script = generator.generate();

        // Solve
        match self.solver.solve(&script) {
            Ok(SolverResult::Unsat) => VerifyResult::Verified, // No counterexample = verified
            Ok(SolverResult::Sat(model)) => {
                VerifyResult::Failed(Counterexample::from_model(model))
            }
            Ok(SolverResult::Unknown) | Ok(SolverResult::Timeout) => {
                VerifyResult::Unknown("solver timeout or unknown".to_string())
            }
            Err(e) => VerifyResult::Unknown(format!("solver error: {}", e)),
        }
    }
}

impl Default for ContractVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Report for an entire program's verification
#[derive(Debug)]
pub struct VerificationReport {
    pub functions: Vec<FunctionReport>,
}

impl VerificationReport {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    /// Check if all verifications passed
    pub fn all_verified(&self) -> bool {
        self.functions.iter().all(|f| f.is_verified())
    }

    /// Get number of verified functions
    pub fn verified_count(&self) -> usize {
        self.functions.iter().filter(|f| f.is_verified()).count()
    }

    /// Get number of failed functions
    pub fn failed_count(&self) -> usize {
        self.functions.iter().filter(|f| f.has_failure()).count()
    }
}

impl Default for VerificationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VerificationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for func in &self.functions {
            writeln!(f, "{}", func)?;
        }

        writeln!(f)?;
        if self.all_verified() {
            writeln!(f, "All {} function(s) verified successfully.", self.functions.len())?;
        } else {
            writeln!(
                f,
                "Verified: {}/{}, Failed: {}",
                self.verified_count(),
                self.functions.len(),
                self.failed_count()
            )?;
        }

        Ok(())
    }
}

/// Report for a single function's verification
#[derive(Debug)]
pub struct FunctionReport {
    pub name: String,
    pub pre_result: Option<VerifyResult>,
    pub post_result: Option<VerifyResult>,
    /// v0.2: Named contract results from where {} blocks
    pub contract_results: Vec<(Option<String>, VerifyResult)>,
    /// v0.2: Refinement type constraint results (param_name or "return", constraint description)
    pub refinement_results: Vec<(String, VerifyResult)>,
    pub message: Option<String>,
    /// v0.31: Whether this function was trusted via @trust attribute
    pub trusted: bool,
    /// v0.31: Warnings (e.g., duplicate contracts)
    pub warnings: Vec<String>,
}

impl FunctionReport {
    pub fn new(name: String) -> Self {
        Self {
            name,
            pre_result: None,
            post_result: None,
            contract_results: Vec::new(),
            refinement_results: Vec::new(),
            message: None,
            trusted: false,
            warnings: Vec::new(),
        }
    }

    /// Check if function is fully verified
    pub fn is_verified(&self) -> bool {
        let pre_ok = matches!(&self.pre_result, Some(VerifyResult::Verified));
        let post_ok = matches!(&self.post_result, Some(VerifyResult::Verified));
        // v0.2: Check named contracts from where {} blocks
        let contracts_ok = self.contract_results.iter()
            .all(|(_, result)| matches!(result, VerifyResult::Verified));
        // v0.2: Check refinement type constraints
        let refinements_ok = self.refinement_results.iter()
            .all(|(_, result)| matches!(result, VerifyResult::Verified));
        pre_ok && post_ok && contracts_ok && refinements_ok
    }

    /// Check if function has any failure
    pub fn has_failure(&self) -> bool {
        matches!(&self.pre_result, Some(VerifyResult::Failed(_)))
            || matches!(&self.post_result, Some(VerifyResult::Failed(_)))
            // v0.2: Check named contracts from where {} blocks
            || self.contract_results.iter()
                .any(|(_, result)| matches!(result, VerifyResult::Failed(_)))
            // v0.2: Check refinement type constraints
            || self.refinement_results.iter()
                .any(|(_, result)| matches!(result, VerifyResult::Failed(_)))
    }
}

impl std::fmt::Display for FunctionReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Pre-condition result
        if let Some(ref result) = self.pre_result {
            match result {
                VerifyResult::Verified => writeln!(f, "✓ {}: pre verified", self.name)?,
                VerifyResult::Failed(ce) => {
                    writeln!(f, "✗ {}: pre verification failed", self.name)?;
                    write!(f, "  {}", ce)?;
                }
                VerifyResult::Unknown(msg) => {
                    writeln!(f, "? {}: pre unknown ({})", self.name, msg)?
                }
                VerifyResult::SolverNotAvailable => {
                    writeln!(f, "! {}: solver not available", self.name)?
                }
            }
        }

        // Post-condition result
        if let Some(ref result) = self.post_result {
            match result {
                VerifyResult::Verified => writeln!(f, "✓ {}: post verified", self.name)?,
                VerifyResult::Failed(ce) => {
                    writeln!(f, "✗ {}: post verification failed", self.name)?;
                    write!(f, "  {}", ce)?;
                }
                VerifyResult::Unknown(msg) => {
                    writeln!(f, "? {}: post unknown ({})", self.name, msg)?
                }
                VerifyResult::SolverNotAvailable => {
                    writeln!(f, "! {}: solver not available", self.name)?
                }
            }
        }

        // v0.2: Named contract results from where {} blocks
        for (name, result) in &self.contract_results {
            let contract_name = name.as_deref().unwrap_or("unnamed");
            match result {
                VerifyResult::Verified => {
                    writeln!(f, "✓ {}: contract '{}' verified", self.name, contract_name)?
                }
                VerifyResult::Failed(ce) => {
                    writeln!(f, "✗ {}: contract '{}' violated", self.name, contract_name)?;
                    write!(f, "  {}", ce)?;
                }
                VerifyResult::Unknown(msg) => {
                    writeln!(f, "? {}: contract '{}' unknown ({})", self.name, contract_name, msg)?
                }
                VerifyResult::SolverNotAvailable => {
                    writeln!(f, "! {}: solver not available for contract '{}'", self.name, contract_name)?
                }
            }
        }

        // v0.2: Refinement type constraint results
        for (location, result) in &self.refinement_results {
            match result {
                VerifyResult::Verified => {
                    writeln!(f, "✓ {}: refinement '{}' verified", self.name, location)?
                }
                VerifyResult::Failed(ce) => {
                    writeln!(f, "✗ {}: refinement '{}' violated", self.name, location)?;
                    write!(f, "  {}", ce)?;
                }
                VerifyResult::Unknown(msg) => {
                    writeln!(f, "? {}: refinement '{}' unknown ({})", self.name, location, msg)?
                }
                VerifyResult::SolverNotAvailable => {
                    writeln!(f, "! {}: solver not available for refinement '{}'", self.name, location)?
                }
            }
        }

        // Optional message
        if let Some(ref msg) = self.message {
            writeln!(f, "  Note: {}", msg)?;
        }

        // v0.31: Warnings (e.g., duplicate contracts)
        for warning in &self.warnings {
            writeln!(f, "⚠ {}: {}", self.name, warning)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Span, Spanned, Visibility};

    fn dummy_span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned { node, span: dummy_span() }
    }

    #[test]
    fn test_verifier_creation() {
        let _verifier = ContractVerifier::new();
        // Verifier created successfully
    }

    #[test]
    fn test_function_report_no_contracts() {
        let report = FunctionReport::new("test".to_string());
        // Empty report is not verified (no results yet)
        assert!(!report.is_verified());
        assert!(!report.has_failure());
    }

    #[test]
    fn test_function_report_all_verified() {
        let mut report = FunctionReport::new("test".to_string());
        report.pre_result = Some(VerifyResult::Verified);
        report.post_result = Some(VerifyResult::Verified);
        report.contract_results.push((Some("c1".to_string()), VerifyResult::Verified));
        report.refinement_results.push(("return".to_string(), VerifyResult::Verified));

        assert!(report.is_verified());
        assert!(!report.has_failure());
    }

    #[test]
    fn test_function_report_contract_failure() {
        let mut report = FunctionReport::new("test".to_string());
        report.pre_result = Some(VerifyResult::Verified);
        report.post_result = Some(VerifyResult::Verified);
        report.contract_results.push((
            Some("c1".to_string()),
            VerifyResult::Failed(Counterexample { assignments: vec![] }),
        ));

        assert!(!report.is_verified());
        assert!(report.has_failure());
    }

    #[test]
    fn test_function_report_refinement_failure() {
        let mut report = FunctionReport::new("test".to_string());
        report.pre_result = Some(VerifyResult::Verified);
        report.post_result = Some(VerifyResult::Verified);
        report.refinement_results.push((
            "return".to_string(),
            VerifyResult::Failed(Counterexample { assignments: vec![] }),
        ));

        assert!(!report.is_verified());
        assert!(report.has_failure());
    }

    #[test]
    fn test_verification_report_counts() {
        let mut report = VerificationReport::new();

        let mut f1 = FunctionReport::new("f1".to_string());
        f1.pre_result = Some(VerifyResult::Verified);
        f1.post_result = Some(VerifyResult::Verified);

        let mut f2 = FunctionReport::new("f2".to_string());
        f2.pre_result = Some(VerifyResult::Verified);
        f2.post_result = Some(VerifyResult::Failed(Counterexample { assignments: vec![] }));

        report.functions.push(f1);
        report.functions.push(f2);

        assert_eq!(report.verified_count(), 1);
        assert_eq!(report.failed_count(), 1);
        assert!(!report.all_verified());
    }

    #[test]
    fn test_verify_function_no_contracts() {
        let verifier = ContractVerifier::new();
        let func = FnDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: spanned("test".to_string()),
            type_params: vec![],
            params: vec![],
            ret_name: None,
            ret_ty: spanned(Type::I64),
            pre: None,
            post: None,
            contracts: vec![],
            body: spanned(Expr::IntLit(42)),
            span: dummy_span(),
        };

        let report = verifier.verify_function(&func);
        assert!(report.is_verified());
        assert!(report.message.is_some());
        assert!(report.message.unwrap().contains("No contracts"));
    }

    #[test]
    fn test_duplicate_contract_detection() {
        use crate::ast::NamedContract;

        let verifier = ContractVerifier::new();

        // Create a function with duplicate contracts
        let same_condition = spanned(Expr::Binary {
            left: Box::new(spanned(Expr::Var("x".to_string()))),
            op: crate::ast::BinOp::Ge,
            right: Box::new(spanned(Expr::IntLit(0))),
        });

        let func = FnDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: spanned("test_func".to_string()),
            type_params: vec![],
            params: vec![crate::ast::Param {
                name: spanned("x".to_string()),
                ty: spanned(Type::I64),
            }],
            ret_name: Some(spanned("r".to_string())),
            ret_ty: spanned(Type::I64),
            pre: None,
            post: None,
            contracts: vec![
                NamedContract {
                    name: Some(spanned("positive".to_string())),
                    condition: same_condition.clone(),
                    span: dummy_span(),
                },
                NamedContract {
                    name: Some(spanned("also_positive".to_string())),
                    condition: same_condition.clone(),
                    span: dummy_span(),
                },
            ],
            body: spanned(Expr::Var("x".to_string())),
            span: dummy_span(),
        };

        let mut report = FunctionReport::new("test_func".to_string());
        verifier.detect_duplicate_contracts(&func, &mut report);

        // Should have detected the duplicate
        assert_eq!(report.warnings.len(), 1);
        assert!(report.warnings[0].contains("Duplicate contract"));
        assert!(report.warnings[0].contains("also_positive"));
        assert!(report.warnings[0].contains("positive"));
    }

    #[test]
    fn test_trivial_contract_detection() {
        // v0.82: Test trivial contract detection
        // This test only runs when Z3 is available
        let verifier = ContractVerifier::new();
        if !verifier.is_solver_available() {
            // Skip test if Z3 is not available
            return;
        }

        // Create a function with trivial postcondition: ret == ret
        let func = FnDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: spanned("trivial_fn".to_string()),
            type_params: vec![],
            params: vec![crate::ast::Param {
                name: spanned("x".to_string()),
                ty: spanned(Type::I64),
            }],
            ret_name: None,
            ret_ty: spanned(Type::I64),
            pre: None,
            post: Some(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::Var("ret".to_string()))),
                op: crate::ast::BinOp::Eq,
                right: Box::new(spanned(Expr::Var("ret".to_string()))),
            })),
            contracts: vec![],
            body: spanned(Expr::Var("x".to_string())),
            span: dummy_span(),
        };

        let mut report = FunctionReport::new("trivial_fn".to_string());
        verifier.detect_trivial_contracts(&func, &mut report);

        // Should have detected the trivial postcondition
        assert!(report.warnings.iter().any(|w| w.contains("Trivial contract")));
        assert!(report.warnings.iter().any(|w| w.contains("postcondition")));
    }

    #[test]
    fn test_trivial_true_literal() {
        // v0.82: Test trivial contract with literal true
        let verifier = ContractVerifier::new();
        if !verifier.is_solver_available() {
            return;
        }

        // Create a function with trivial precondition: true
        let func = FnDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: spanned("trivial_pre".to_string()),
            type_params: vec![],
            params: vec![crate::ast::Param {
                name: spanned("x".to_string()),
                ty: spanned(Type::I64),
            }],
            ret_name: None,
            ret_ty: spanned(Type::I64),
            pre: Some(spanned(Expr::BoolLit(true))),
            post: None,
            contracts: vec![],
            body: spanned(Expr::Var("x".to_string())),
            span: dummy_span(),
        };

        let mut report = FunctionReport::new("trivial_pre".to_string());
        verifier.detect_trivial_contracts(&func, &mut report);

        // Should have detected the trivial precondition
        assert!(report.warnings.iter().any(|w| w.contains("Trivial contract")));
        assert!(report.warnings.iter().any(|w| w.contains("precondition")));
    }

    #[test]
    fn test_non_trivial_contract() {
        // v0.82: Test that meaningful contracts are NOT flagged as trivial
        let verifier = ContractVerifier::new();
        if !verifier.is_solver_available() {
            return;
        }

        // Create a function with non-trivial postcondition: ret > 0
        let func = FnDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: spanned("non_trivial_fn".to_string()),
            type_params: vec![],
            params: vec![crate::ast::Param {
                name: spanned("x".to_string()),
                ty: spanned(Type::I64),
            }],
            ret_name: None,
            ret_ty: spanned(Type::I64),
            pre: Some(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::Var("x".to_string()))),
                op: crate::ast::BinOp::Gt,
                right: Box::new(spanned(Expr::IntLit(0))),
            })),
            post: Some(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::Var("ret".to_string()))),
                op: crate::ast::BinOp::Gt,
                right: Box::new(spanned(Expr::IntLit(0))),
            })),
            contracts: vec![],
            body: spanned(Expr::Binary {
                left: Box::new(spanned(Expr::Var("x".to_string()))),
                op: crate::ast::BinOp::Add,
                right: Box::new(spanned(Expr::IntLit(1))),
            }),
            span: dummy_span(),
        };

        let mut report = FunctionReport::new("non_trivial_fn".to_string());
        verifier.detect_trivial_contracts(&func, &mut report);

        // Should NOT have detected any trivial contracts
        assert!(
            !report.warnings.iter().any(|w| w.contains("Trivial contract")),
            "Non-trivial contracts should not be flagged"
        );
    }

    #[test]
    fn test_unsatisfiable_precondition() {
        // v0.86: Test unsatisfiable precondition detection (dead code)
        let verifier = ContractVerifier::new();
        if !verifier.is_solver_available() {
            return;
        }

        // Create a function with unsatisfiable precondition: x > 0 AND x < 0
        let func = FnDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: spanned("impossible".to_string()),
            type_params: vec![],
            params: vec![crate::ast::Param {
                name: spanned("x".to_string()),
                ty: spanned(Type::I64),
            }],
            ret_name: None,
            ret_ty: spanned(Type::I64),
            pre: Some(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::Binary {
                    left: Box::new(spanned(Expr::Var("x".to_string()))),
                    op: crate::ast::BinOp::Gt,
                    right: Box::new(spanned(Expr::IntLit(0))),
                })),
                op: crate::ast::BinOp::And,
                right: Box::new(spanned(Expr::Binary {
                    left: Box::new(spanned(Expr::Var("x".to_string()))),
                    op: crate::ast::BinOp::Lt,
                    right: Box::new(spanned(Expr::IntLit(0))),
                })),
            })),
            post: None,
            contracts: vec![],
            body: spanned(Expr::Var("x".to_string())),
            span: dummy_span(),
        };

        let mut report = FunctionReport::new("impossible".to_string());
        verifier.detect_unsatisfiable_precondition(&func, &mut report);

        // Should have detected dead code
        assert!(
            report.warnings.iter().any(|w| w.contains("Dead code")),
            "Unsatisfiable precondition should be flagged as dead code"
        );
    }

    #[test]
    fn test_satisfiable_precondition() {
        // v0.86: Test that satisfiable precondition is NOT flagged
        let verifier = ContractVerifier::new();
        if !verifier.is_solver_available() {
            return;
        }

        // Create a function with satisfiable precondition: x > 0
        let func = FnDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: spanned("possible".to_string()),
            type_params: vec![],
            params: vec![crate::ast::Param {
                name: spanned("x".to_string()),
                ty: spanned(Type::I64),
            }],
            ret_name: None,
            ret_ty: spanned(Type::I64),
            pre: Some(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::Var("x".to_string()))),
                op: crate::ast::BinOp::Gt,
                right: Box::new(spanned(Expr::IntLit(0))),
            })),
            post: None,
            contracts: vec![],
            body: spanned(Expr::Var("x".to_string())),
            span: dummy_span(),
        };

        let mut report = FunctionReport::new("possible".to_string());
        verifier.detect_unsatisfiable_precondition(&func, &mut report);

        // Should NOT have detected dead code
        assert!(
            !report.warnings.iter().any(|w| w.contains("Dead code")),
            "Satisfiable precondition should not be flagged as dead code"
        );
    }
}
