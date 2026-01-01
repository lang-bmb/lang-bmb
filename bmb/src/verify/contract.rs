//! Contract verification logic
//!
//! Verifies pre/post conditions for functions.

use crate::ast::{FnDef, Item, Program};
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
                // Struct and Enum definitions don't need verification
                Item::StructDef(_) | Item::EnumDef(_) => {}
            }
        }

        report
    }

    /// Verify a single function's contracts
    pub fn verify_function(&self, func: &FnDef) -> FunctionReport {
        let name = func.name.node.clone();
        let mut report = FunctionReport::new(name.clone());

        // Check if function has any contracts
        if func.pre.is_none() && func.post.is_none() {
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
    pub message: Option<String>,
}

impl FunctionReport {
    pub fn new(name: String) -> Self {
        Self {
            name,
            pre_result: None,
            post_result: None,
            message: None,
        }
    }

    /// Check if function is fully verified
    pub fn is_verified(&self) -> bool {
        let pre_ok = matches!(&self.pre_result, Some(VerifyResult::Verified));
        let post_ok = matches!(&self.post_result, Some(VerifyResult::Verified));
        pre_ok && post_ok
    }

    /// Check if function has any failure
    pub fn has_failure(&self) -> bool {
        matches!(&self.pre_result, Some(VerifyResult::Failed(_)))
            || matches!(&self.post_result, Some(VerifyResult::Failed(_)))
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

    #[test]
    fn test_verifier_creation() {
        let _verifier = ContractVerifier::new();
        // Verifier created successfully
    }
}
