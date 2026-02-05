//! CIR-based Contract Verification
//!
//! Phase 1.2: Verification driver that uses CIR for contract verification.
//! This provides cleaner SMT generation compared to direct AST translation.

use crate::smt::{SmtSolver, SolverResult, Counterexample};
use super::{
    CirProgram, CirFunction, Proposition,
    smt::{CirSmtGenerator, SmtError},
};

#[cfg(test)]
use super::{CirParam, CirType, CirExpr, NamedProposition, EffectSet, CompareOp};

/// Proof witness representing verification outcome
#[derive(Debug, Clone)]
pub struct ProofWitness {
    /// Function name being verified
    pub function: String,
    /// Verification outcome
    pub outcome: ProofOutcome,
    /// SMT script used (for debugging)
    pub smt_script: Option<String>,
    /// Counterexample if verification failed
    pub counterexample: Option<Counterexample>,
    /// Verification time in milliseconds
    pub verification_time_ms: u64,
}

/// Proof verification outcome
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofOutcome {
    /// Contract verified successfully
    Verified,
    /// Contract verification failed with counterexample
    Failed(String),
    /// Solver returned unknown (timeout or complexity limit)
    Unknown(String),
    /// Verification was skipped (no contracts)
    Skipped,
    /// Verification error occurred
    Error(String),
}

impl ProofWitness {
    pub fn verified(function: String, smt_script: Option<String>, time_ms: u64) -> Self {
        Self {
            function,
            outcome: ProofOutcome::Verified,
            smt_script,
            counterexample: None,
            verification_time_ms: time_ms,
        }
    }

    pub fn failed(function: String, reason: String, smt_script: Option<String>, time_ms: u64) -> Self {
        Self {
            function,
            outcome: ProofOutcome::Failed(reason),
            smt_script,
            counterexample: None,
            verification_time_ms: time_ms,
        }
    }

    pub fn skipped(function: String) -> Self {
        Self {
            function,
            outcome: ProofOutcome::Skipped,
            smt_script: None,
            counterexample: None,
            verification_time_ms: 0,
        }
    }

    pub fn error(function: String, error: String) -> Self {
        Self {
            function,
            outcome: ProofOutcome::Error(error),
            smt_script: None,
            counterexample: None,
            verification_time_ms: 0,
        }
    }

    pub fn is_verified(&self) -> bool {
        matches!(self.outcome, ProofOutcome::Verified)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.outcome, ProofOutcome::Failed(_))
    }
}

/// CIR-based contract verifier
pub struct CirVerifier {
    solver: SmtSolver,
    /// Keep SMT scripts for debugging
    keep_smt_scripts: bool,
    /// Verbose output
    verbose: bool,
}

impl CirVerifier {
    pub fn new() -> Self {
        Self {
            solver: SmtSolver::new(),
            keep_smt_scripts: false,
            verbose: false,
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

    /// Keep SMT scripts in proof witnesses
    pub fn with_smt_scripts(mut self, keep: bool) -> Self {
        self.keep_smt_scripts = keep;
        self
    }

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Check if solver is available
    pub fn is_solver_available(&self) -> bool {
        self.solver.is_available()
    }

    /// Verify all functions in a CIR program
    pub fn verify_program(&self, program: &CirProgram) -> CirVerificationReport {
        let mut report = CirVerificationReport::new();

        for func in &program.functions {
            let witness = self.verify_function(func);
            report.witnesses.push(witness);
        }

        report.compute_summary();
        report
    }

    /// Verify a single CIR function
    pub fn verify_function(&self, func: &CirFunction) -> ProofWitness {
        let func_name = func.name.clone();

        // Check if function has contracts
        if func.preconditions.is_empty() && func.postconditions.is_empty() {
            return ProofWitness::skipped(func_name);
        }

        // Generate SMT script
        let mut generator = CirSmtGenerator::new();

        // Use appropriate logic
        if self.needs_quantifiers(func) {
            generator.use_array_logic();
        }

        let smt_script = match generator.generate_verification_query(func) {
            Ok(script) => script,
            Err(e) => {
                return ProofWitness::error(func_name, format!("SMT generation error: {}", e));
            }
        };

        if self.verbose {
            eprintln!("=== SMT Script for {} ===", func_name);
            eprintln!("{}", smt_script);
            eprintln!("=== End SMT Script ===\n");
        }

        // Run solver
        let start = std::time::Instant::now();
        let result = self.solver.solve(&smt_script);
        let elapsed = start.elapsed().as_millis() as u64;

        let smt_for_witness = if self.keep_smt_scripts {
            Some(smt_script)
        } else {
            None
        };

        match result {
            Ok(SolverResult::Unsat) => {
                // Unsat means no counterexample exists, so contract is valid
                ProofWitness::verified(func_name, smt_for_witness, elapsed)
            }
            Ok(SolverResult::Sat(model)) => {
                // Sat means counterexample found, contract is violated
                let mut witness = ProofWitness::failed(
                    func_name,
                    "Counterexample found".to_string(),
                    smt_for_witness,
                    elapsed,
                );
                witness.counterexample = Some(Counterexample::from_model(model));
                witness
            }
            Ok(SolverResult::Unknown) => {
                ProofWitness {
                    function: func_name,
                    outcome: ProofOutcome::Unknown("Solver returned unknown".to_string()),
                    smt_script: smt_for_witness,
                    counterexample: None,
                    verification_time_ms: elapsed,
                }
            }
            Ok(SolverResult::Timeout) => {
                ProofWitness {
                    function: func_name,
                    outcome: ProofOutcome::Unknown("Solver timeout".to_string()),
                    smt_script: smt_for_witness,
                    counterexample: None,
                    verification_time_ms: elapsed,
                }
            }
            Err(e) => {
                ProofWitness::error(func_name, format!("Solver error: {}", e))
            }
        }
    }

    /// Verify a specific proposition holds given preconditions
    pub fn verify_proposition(
        &self,
        func: &CirFunction,
        prop: &Proposition,
    ) -> Result<ProofOutcome, SmtError> {
        let mut generator = CirSmtGenerator::new();

        // Declare parameters
        for param in &func.params {
            let sort = generator.cir_type_to_sort(&param.ty);
            generator.declare_var(&param.name, sort);
        }

        // Assert preconditions as assumptions
        for pre in &func.preconditions {
            generator.assert_proposition(&pre.proposition)?;
        }

        // Assert negation of target proposition (to find counterexample)
        let prop_smt = generator.translate_proposition(prop)?;
        generator.assert(&format!("(not {})", prop_smt));

        let smt_script = generator.generate();

        match self.solver.solve(&smt_script) {
            Ok(SolverResult::Unsat) => Ok(ProofOutcome::Verified),
            Ok(SolverResult::Sat(_)) => Ok(ProofOutcome::Failed("Counterexample exists".to_string())),
            Ok(SolverResult::Unknown) => Ok(ProofOutcome::Unknown("Unknown".to_string())),
            Ok(SolverResult::Timeout) => Ok(ProofOutcome::Unknown("Timeout".to_string())),
            Err(e) => Ok(ProofOutcome::Error(format!("{}", e))),
        }
    }

    /// Check if function needs quantified logic
    fn needs_quantifiers(&self, func: &CirFunction) -> bool {
        for pre in &func.preconditions {
            if self.proposition_has_quantifier(&pre.proposition) {
                return true;
            }
        }
        for post in &func.postconditions {
            if self.proposition_has_quantifier(&post.proposition) {
                return true;
            }
        }
        for inv in &func.loop_invariants {
            if self.proposition_has_quantifier(&inv.invariant) {
                return true;
            }
        }
        false
    }

    fn proposition_has_quantifier(&self, prop: &Proposition) -> bool {
        match prop {
            Proposition::Forall { .. } | Proposition::Exists { .. } => true,
            Proposition::Not(inner) => self.proposition_has_quantifier(inner),
            Proposition::And(props) | Proposition::Or(props) => {
                props.iter().any(|p| self.proposition_has_quantifier(p))
            }
            Proposition::Implies(l, r) => {
                self.proposition_has_quantifier(l) || self.proposition_has_quantifier(r)
            }
            Proposition::Old(_, inner) => self.proposition_has_quantifier(inner),
            _ => false,
        }
    }
}

impl Default for CirVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Verification report for a CIR program
#[derive(Debug)]
pub struct CirVerificationReport {
    pub witnesses: Vec<ProofWitness>,
    pub total_functions: usize,
    pub verified_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub error_count: usize,
    pub unknown_count: usize,
    pub total_time_ms: u64,
}

impl CirVerificationReport {
    pub fn new() -> Self {
        Self {
            witnesses: Vec::new(),
            total_functions: 0,
            verified_count: 0,
            failed_count: 0,
            skipped_count: 0,
            error_count: 0,
            unknown_count: 0,
            total_time_ms: 0,
        }
    }

    fn compute_summary(&mut self) {
        self.total_functions = self.witnesses.len();
        self.verified_count = 0;
        self.failed_count = 0;
        self.skipped_count = 0;
        self.error_count = 0;
        self.unknown_count = 0;
        self.total_time_ms = 0;

        for w in &self.witnesses {
            self.total_time_ms += w.verification_time_ms;
            match &w.outcome {
                ProofOutcome::Verified => self.verified_count += 1,
                ProofOutcome::Failed(_) => self.failed_count += 1,
                ProofOutcome::Skipped => self.skipped_count += 1,
                ProofOutcome::Error(_) => self.error_count += 1,
                ProofOutcome::Unknown(_) => self.unknown_count += 1,
            }
        }
    }

    pub fn all_verified(&self) -> bool {
        self.failed_count == 0 && self.error_count == 0
    }

    pub fn has_failures(&self) -> bool {
        self.failed_count > 0
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Verified: {}, Failed: {}, Skipped: {}, Unknown: {}, Errors: {} ({}ms)",
            self.verified_count,
            self.failed_count,
            self.skipped_count,
            self.unknown_count,
            self.error_count,
            self.total_time_ms,
        )
    }
}

impl Default for CirVerificationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_function() -> CirFunction {
        CirFunction {
            name: "test_fn".to_string(),
            type_params: vec![],
            params: vec![
                CirParam {
                    name: "x".to_string(),
                    ty: CirType::I64,
                    constraints: vec![],
                },
            ],
            ret_name: "result".to_string(),
            ret_ty: CirType::I64,
            preconditions: vec![
                NamedProposition {
                    name: None,
                    proposition: Proposition::Compare {
                        lhs: Box::new(CirExpr::Var("x".to_string())),
                        op: CompareOp::Gt,
                        rhs: Box::new(CirExpr::IntLit(0)),
                    },
                },
            ],
            postconditions: vec![
                NamedProposition {
                    name: None,
                    proposition: Proposition::Compare {
                        lhs: Box::new(CirExpr::Var("result".to_string())),
                        op: CompareOp::Gt,
                        rhs: Box::new(CirExpr::IntLit(0)),
                    },
                },
            ],
            loop_invariants: vec![],
            effects: EffectSet::pure(),
            body: CirExpr::Var("x".to_string()),
        }
    }

    #[test]
    fn test_proof_witness_creation() {
        let w = ProofWitness::verified("foo".to_string(), None, 100);
        assert!(w.is_verified());
        assert!(!w.is_failed());

        let w2 = ProofWitness::failed("bar".to_string(), "reason".to_string(), None, 50);
        assert!(w2.is_failed());
        assert!(!w2.is_verified());

        let w3 = ProofWitness::skipped("baz".to_string());
        assert!(!w3.is_verified());
        assert!(!w3.is_failed());
    }

    #[test]
    fn test_verifier_creation() {
        let verifier = CirVerifier::new()
            .with_timeout(30)
            .with_verbose(true)
            .with_smt_scripts(true);

        // Just test that it builds
        assert!(verifier.verbose);
        assert!(verifier.keep_smt_scripts);
    }

    #[test]
    fn test_verification_report() {
        let mut report = CirVerificationReport::new();
        report.witnesses.push(ProofWitness::verified("f1".to_string(), None, 10));
        report.witnesses.push(ProofWitness::failed("f2".to_string(), "err".to_string(), None, 20));
        report.witnesses.push(ProofWitness::skipped("f3".to_string()));
        report.compute_summary();

        assert_eq!(report.total_functions, 3);
        assert_eq!(report.verified_count, 1);
        assert_eq!(report.failed_count, 1);
        assert_eq!(report.skipped_count, 1);
        assert_eq!(report.total_time_ms, 30);
        assert!(report.has_failures());
        assert!(!report.all_verified());
    }

    #[test]
    fn test_needs_quantifiers() {
        let verifier = CirVerifier::new();
        let mut func = make_test_function();

        // No quantifiers
        assert!(!verifier.needs_quantifiers(&func));

        // Add forall in precondition
        func.preconditions.push(NamedProposition {
            name: None,
            proposition: Proposition::Forall {
                var: "i".to_string(),
                ty: CirType::I64,
                body: Box::new(Proposition::True),
            },
        });
        assert!(verifier.needs_quantifiers(&func));
    }
}
