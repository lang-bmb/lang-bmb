//! Contract verification logic
//!
//! Verifies pre/post conditions for functions.

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

        for item in &program.items {
            match item {
                Item::FnDef(func) => {
                    let func_report = self.verify_function(func);
                    report.functions.push(func_report);
                }
                // Struct, Enum, Use, and ExternFn don't need verification
                Item::StructDef(_) | Item::EnumDef(_) | Item::Use(_) | Item::ExternFn(_) => {}
                // v0.20.1: Trait system not yet included in verification
                Item::TraitDef(_) | Item::ImplBlock(_) => {}
            }
        }

        report
    }

    /// Verify a single function's contracts
    pub fn verify_function(&self, func: &FnDef) -> FunctionReport {
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
}
