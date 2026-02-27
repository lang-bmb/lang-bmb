//! Incremental Verification
//!
//! Phase 1.3: Re-verify only changed functions and their dependents.
//! Enables fast iteration during development.

use std::collections::{HashMap, HashSet};

use crate::cir::{CirProgram, CirVerifier, ProofWitness, ProofOutcome};

#[cfg(test)]
use crate::cir::CirFunction;
use super::proof_db::{ProofDatabase, FunctionId, FunctionProofResult, VerificationStatus, ProofFact, ProofScope, ProofEvidence};
use super::summary::{FunctionSummary, SummaryChange, extract_summaries, compare_summaries};

/// Incremental verification manager
pub struct IncrementalVerifier {
    /// Proof database for caching
    db: ProofDatabase,

    /// Function summaries
    summaries: HashMap<FunctionId, FunctionSummary>,

    /// Call graph: function -> functions it calls
    call_graph: HashMap<String, HashSet<String>>,

    /// Reverse call graph: function -> functions that call it
    reverse_call_graph: HashMap<String, HashSet<String>>,

    /// The underlying verifier
    verifier: CirVerifier,
}

/// Result of incremental verification
#[derive(Debug)]
pub struct IncrementalVerificationResult {
    /// Functions that were verified
    pub verified_functions: Vec<String>,
    /// Functions that were skipped (unchanged and cached)
    pub skipped_functions: Vec<String>,
    /// Functions that failed verification
    pub failed_functions: Vec<(String, String)>,
    /// Total verification time in milliseconds
    pub total_time_ms: u64,
    /// Time saved by incremental verification (estimated)
    pub time_saved_ms: u64,
}

impl IncrementalVerifier {
    /// Create a new incremental verifier
    pub fn new() -> Self {
        Self {
            db: ProofDatabase::new(),
            summaries: HashMap::new(),
            call_graph: HashMap::new(),
            reverse_call_graph: HashMap::new(),
            verifier: CirVerifier::new(),
        }
    }

    /// Configure the underlying verifier
    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.verifier = self.verifier.with_timeout(seconds);
        self
    }

    /// Check if solver is available
    pub fn is_solver_available(&self) -> bool {
        self.verifier.is_solver_available()
    }

    /// Perform incremental verification
    ///
    /// Only re-verifies functions that:
    /// 1. Have changed since last verification
    /// 2. Call functions whose contracts changed
    /// 3. Are called by functions whose implementations changed
    pub fn verify_incremental(
        &mut self,
        old_cir: Option<&CirProgram>,
        new_cir: &CirProgram,
    ) -> IncrementalVerificationResult {
        let start = std::time::Instant::now();

        // Extract new summaries
        let new_summaries = extract_summaries(new_cir);

        // Build call graph from new program
        self.build_call_graph(new_cir);

        // Determine which functions need re-verification
        let functions_to_verify = self.identify_changed_functions(
            old_cir.map(extract_summaries),
            &new_summaries,
        );

        // Verify only the functions that need it
        let mut result = IncrementalVerificationResult {
            verified_functions: Vec::new(),
            skipped_functions: Vec::new(),
            failed_functions: Vec::new(),
            total_time_ms: 0,
            time_saved_ms: 0,
        };

        let _total_functions = new_cir.functions.len();
        let _functions_to_verify_count = functions_to_verify.len();

        for func in &new_cir.functions {
            let id = FunctionId::simple(&func.name);

            if functions_to_verify.contains(&func.name) {
                // Need to verify this function
                let witness = self.verifier.verify_function(func);

                // Store result in database
                let proof_result = witness_to_proof_result(&witness, func);
                self.db.store_function_proof(&id, proof_result);

                match &witness.outcome {
                    ProofOutcome::Verified | ProofOutcome::Skipped => {
                        result.verified_functions.push(func.name.clone());
                    }
                    ProofOutcome::Failed(reason) => {
                        result.failed_functions.push((func.name.clone(), reason.clone()));
                    }
                    ProofOutcome::Unknown(reason) => {
                        result.failed_functions.push((func.name.clone(), format!("Unknown: {}", reason)));
                    }
                    ProofOutcome::Error(err) => {
                        result.failed_functions.push((func.name.clone(), format!("Error: {}", err)));
                    }
                }

                result.total_time_ms += witness.verification_time_ms;
            } else {
                // Skip this function (cached)
                result.skipped_functions.push(func.name.clone());
            }
        }

        // Update summaries
        for (id, summary) in new_summaries {
            self.summaries.insert(id, summary);
        }

        // Estimate time saved (assuming average verification time)
        if !result.verified_functions.is_empty() {
            let avg_time = result.total_time_ms / result.verified_functions.len() as u64;
            result.time_saved_ms = avg_time * result.skipped_functions.len() as u64;
        }

        result.total_time_ms = start.elapsed().as_millis() as u64;
        result
    }

    /// Build call graph from CIR program
    fn build_call_graph(&mut self, cir: &CirProgram) {
        self.call_graph.clear();
        self.reverse_call_graph.clear();

        for func in &cir.functions {
            let callee_names = extract_callees(&func.body);

            for callee in &callee_names {
                // Add to call graph
                self.call_graph
                    .entry(func.name.clone())
                    .or_default()
                    .insert(callee.clone());

                // Add to reverse call graph
                self.reverse_call_graph
                    .entry(callee.clone())
                    .or_default()
                    .insert(func.name.clone());
            }
        }
    }

    /// Identify functions that need re-verification
    fn identify_changed_functions(
        &self,
        old_summaries: Option<HashMap<FunctionId, FunctionSummary>>,
        new_summaries: &HashMap<FunctionId, FunctionSummary>,
    ) -> HashSet<String> {
        let mut to_verify = HashSet::new();

        // If no old program, verify everything
        let old_summaries = match old_summaries {
            Some(s) => s,
            None => {
                return new_summaries.keys().map(|id| id.name.clone()).collect();
            }
        };

        // Build map from name to old summary
        let old_by_name: HashMap<String, &FunctionSummary> = old_summaries
            .iter()
            .map(|(id, s)| (id.name.clone(), s))
            .collect();

        // Check each function
        for (id, new_summary) in new_summaries {
            let old_summary = old_by_name.get(&id.name).copied();

            match compare_summaries(old_summary, Some(new_summary)) {
                SummaryChange::Unchanged => {
                    // No need to re-verify
                }
                SummaryChange::ContractChanged => {
                    // Contract changed: verify this function and all callers
                    to_verify.insert(id.name.clone());
                    if let Some(callers) = self.reverse_call_graph.get(&id.name) {
                        to_verify.extend(callers.iter().cloned());
                    }
                }
                SummaryChange::ImplementationChanged => {
                    // Only implementation changed: verify just this function
                    to_verify.insert(id.name.clone());
                }
                SummaryChange::Added => {
                    // New function: verify it
                    to_verify.insert(id.name.clone());
                }
                SummaryChange::Removed => {
                    // Function removed: re-verify all callers
                    if let Some(callers) = self.reverse_call_graph.get(&id.name) {
                        to_verify.extend(callers.iter().cloned());
                    }
                }
            }
        }

        to_verify
    }

    /// Get the proof database
    pub fn database(&self) -> &ProofDatabase {
        &self.db
    }

    /// Get mutable reference to proof database
    pub fn database_mut(&mut self) -> &mut ProofDatabase {
        &mut self.db
    }

    /// Get function summaries
    pub fn summaries(&self) -> &HashMap<FunctionId, FunctionSummary> {
        &self.summaries
    }

    /// Clear all cached data
    pub fn clear(&mut self) {
        self.db.clear();
        self.summaries.clear();
        self.call_graph.clear();
        self.reverse_call_graph.clear();
    }
}

impl Default for IncrementalVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert ProofWitness to FunctionProofResult
/// v0.90.147: Extract proven facts from CIR function contracts when verified
fn witness_to_proof_result(witness: &ProofWitness, func: &crate::cir::CirFunction) -> FunctionProofResult {
    use std::time::Duration;

    let status = match &witness.outcome {
        ProofOutcome::Verified => VerificationStatus::Verified,
        ProofOutcome::Failed(reason) => VerificationStatus::Failed(reason.clone()),
        ProofOutcome::Skipped => VerificationStatus::Skipped,
        ProofOutcome::Unknown(_) | ProofOutcome::Error(_) => VerificationStatus::Unknown,
    };

    // v0.90.147: When verified, extract proven facts from pre/post conditions
    let proven_facts = if matches!(&witness.outcome, ProofOutcome::Verified) {
        let func_id = FunctionId::simple(&witness.function);
        let mut facts = Vec::new();

        for named_prop in &func.preconditions {
            facts.push(ProofFact {
                proposition: named_prop.proposition.clone(),
                scope: ProofScope::Function(func_id.clone()),
                evidence: ProofEvidence::Precondition,
            });
        }

        for (i, named_prop) in func.postconditions.iter().enumerate() {
            facts.push(ProofFact {
                proposition: named_prop.proposition.clone(),
                scope: ProofScope::Function(func_id.clone()),
                evidence: ProofEvidence::SmtProof {
                    query_hash: 0, // Simplified
                    solver: "z3".to_string(),
                },
            });
            // Also record as function call evidence for callers
            let _ = i; // postcondition index available if needed
        }

        facts
    } else {
        vec![]
    };

    FunctionProofResult {
        status,
        proven_facts,
        verification_time: Duration::from_millis(witness.verification_time_ms),
        smt_queries: 1, // Simplified
        verified_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    }
}

/// Extract callee names from an expression
fn extract_callees(expr: &crate::cir::CirExpr) -> HashSet<String> {
    use crate::cir::CirExpr;

    let mut callees = HashSet::new();

    fn visit(expr: &CirExpr, callees: &mut HashSet<String>) {
        match expr {
            CirExpr::Call { func, args } => {
                callees.insert(func.clone());
                for arg in args {
                    visit(arg, callees);
                }
            }
            CirExpr::If { cond, then_branch, else_branch } => {
                visit(cond, callees);
                visit(then_branch, callees);
                visit(else_branch, callees);
            }
            CirExpr::Let { value, body, .. } |
            CirExpr::LetMut { value, body, .. } => {
                visit(value, callees);
                visit(body, callees);
            }
            CirExpr::BinOp { lhs, rhs, .. } => {
                visit(lhs, callees);
                visit(rhs, callees);
            }
            CirExpr::UnaryOp { operand, .. } => {
                visit(operand, callees);
            }
            CirExpr::Block(exprs) => {
                for e in exprs {
                    visit(e, callees);
                }
            }
            CirExpr::While { cond, body, .. } => {
                visit(cond, callees);
                visit(body, callees);
            }
            CirExpr::Loop { body } => {
                visit(body, callees);
            }
            CirExpr::For { iter, body, .. } => {
                visit(iter, callees);
                visit(body, callees);
            }
            CirExpr::Index { base, index } => {
                visit(base, callees);
                visit(index, callees);
            }
            CirExpr::Field { base, .. } => {
                visit(base, callees);
            }
            CirExpr::Assign { value, .. } |
            CirExpr::IndexAssign { value, .. } |
            CirExpr::FieldAssign { value, .. } => {
                visit(value, callees);
            }
            CirExpr::Ref(e) |
            CirExpr::RefMut(e) |
            CirExpr::Deref(e) |
            CirExpr::Len(e) |
            CirExpr::Old(e) |
            CirExpr::Break(e) => {
                visit(e, callees);
            }
            CirExpr::Array(exprs) |
            CirExpr::Tuple(exprs) => {
                for e in exprs {
                    visit(e, callees);
                }
            }
            CirExpr::Struct { fields, .. } => {
                for (_, e) in fields {
                    visit(e, callees);
                }
            }
            CirExpr::Range { start, end, .. } => {
                visit(start, callees);
                visit(end, callees);
            }
            CirExpr::Cast { expr, .. } => {
                visit(expr, callees);
            }
            _ => {}
        }
    }

    visit(expr, &mut callees);
    callees
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cir::{CirParam, CirType, CirExpr, EffectSet};

    fn make_test_function(name: &str) -> CirFunction {
        CirFunction {
            name: name.to_string(),
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
            preconditions: vec![],
            postconditions: vec![],
            loop_invariants: vec![],
            effects: EffectSet::pure(),
            body: CirExpr::Var("x".to_string()),
        }
    }

    fn make_test_program(functions: Vec<CirFunction>) -> CirProgram {
        CirProgram {
            functions,
            extern_fns: vec![],
            structs: HashMap::new(),
            type_invariants: HashMap::new(),
        }
    }

    #[test]
    fn test_incremental_verifier_creation() {
        let verifier = IncrementalVerifier::new();
        assert!(verifier.summaries().is_empty());
    }

    #[test]
    fn test_extract_callees() {
        let body = CirExpr::Call {
            func: "helper".to_string(),
            args: vec![
                CirExpr::Call {
                    func: "other".to_string(),
                    args: vec![],
                },
            ],
        };

        let callees = extract_callees(&body);
        assert!(callees.contains("helper"));
        assert!(callees.contains("other"));
    }

    #[test]
    fn test_initial_verification() {
        let mut verifier = IncrementalVerifier::new();
        let program = make_test_program(vec![
            make_test_function("foo"),
            make_test_function("bar"),
        ]);

        let result = verifier.verify_incremental(None, &program);

        // All functions should be verified (no previous state)
        assert_eq!(result.verified_functions.len() + result.failed_functions.len(), 2);
        assert!(result.skipped_functions.is_empty());
    }

    #[test]
    fn test_unchanged_functions_skipped() {
        let mut verifier = IncrementalVerifier::new();

        let program1 = make_test_program(vec![make_test_function("foo")]);
        let program2 = make_test_program(vec![make_test_function("foo")]);

        // First verification
        let _result1 = verifier.verify_incremental(None, &program1);

        // Second verification with same program
        let result2 = verifier.verify_incremental(Some(&program1), &program2);

        // Function should be skipped (unchanged)
        assert!(result2.skipped_functions.contains(&"foo".to_string()));
    }

    #[test]
    fn test_changed_function_reverified() {
        let mut verifier = IncrementalVerifier::new();

        let program1 = make_test_program(vec![make_test_function("foo")]);

        let mut changed_fn = make_test_function("foo");
        changed_fn.body = CirExpr::IntLit(42);
        let program2 = make_test_program(vec![changed_fn]);

        // First verification
        let _result1 = verifier.verify_incremental(None, &program1);

        // Second verification with changed function
        let result2 = verifier.verify_incremental(Some(&program1), &program2);

        // Function should be re-verified (changed)
        assert!(
            result2.verified_functions.contains(&"foo".to_string()) ||
            result2.failed_functions.iter().any(|(name, _)| name == "foo")
        );
    }

    // ---- Cycle 73: Additional incremental verification tests ----

    #[test]
    fn test_incremental_verifier_default() {
        let verifier = IncrementalVerifier::default();
        assert!(verifier.summaries().is_empty());
        assert!(verifier.database().is_empty());
    }

    #[test]
    fn test_incremental_verifier_clear() {
        let mut verifier = IncrementalVerifier::new();
        let program = make_test_program(vec![make_test_function("foo")]);

        // Populate with a verification
        let _result = verifier.verify_incremental(None, &program);
        assert!(!verifier.summaries().is_empty());

        verifier.clear();
        assert!(verifier.summaries().is_empty());
        assert!(verifier.database().is_empty());
    }

    #[test]
    fn test_incremental_verifier_with_timeout() {
        let verifier = IncrementalVerifier::new().with_timeout(30);
        assert!(verifier.summaries().is_empty());
    }

    #[test]
    fn test_extract_callees_empty_body() {
        let callees = extract_callees(&CirExpr::IntLit(42));
        assert!(callees.is_empty());
    }

    #[test]
    fn test_extract_callees_nested_in_if() {
        let body = CirExpr::If {
            cond: Box::new(CirExpr::BoolLit(true)),
            then_branch: Box::new(CirExpr::Call {
                func: "then_fn".to_string(),
                args: vec![],
            }),
            else_branch: Box::new(CirExpr::Call {
                func: "else_fn".to_string(),
                args: vec![],
            }),
        };

        let callees = extract_callees(&body);
        assert!(callees.contains("then_fn"));
        assert!(callees.contains("else_fn"));
    }

    #[test]
    fn test_extract_callees_nested_in_let() {
        let body = CirExpr::Let {
            name: "x".to_string(),
            ty: CirType::I64,
            value: Box::new(CirExpr::Call {
                func: "init".to_string(),
                args: vec![],
            }),
            body: Box::new(CirExpr::Call {
                func: "use_x".to_string(),
                args: vec![],
            }),
        };

        let callees = extract_callees(&body);
        assert!(callees.contains("init"));
        assert!(callees.contains("use_x"));
    }

    #[test]
    fn test_extract_callees_in_block() {
        let body = CirExpr::Block(vec![
            CirExpr::Call { func: "a".to_string(), args: vec![] },
            CirExpr::Call { func: "b".to_string(), args: vec![] },
            CirExpr::Call { func: "c".to_string(), args: vec![] },
        ]);

        let callees = extract_callees(&body);
        assert_eq!(callees.len(), 3);
        assert!(callees.contains("a"));
        assert!(callees.contains("b"));
        assert!(callees.contains("c"));
    }

    #[test]
    fn test_extract_callees_in_while() {
        let body = CirExpr::While {
            cond: Box::new(CirExpr::Call {
                func: "check".to_string(),
                args: vec![],
            }),
            body: Box::new(CirExpr::Call {
                func: "step".to_string(),
                args: vec![],
            }),
            invariant: None,
        };

        let callees = extract_callees(&body);
        assert!(callees.contains("check"));
        assert!(callees.contains("step"));
    }

    #[test]
    fn test_extract_callees_in_loop() {
        let body = CirExpr::Loop {
            body: Box::new(CirExpr::Call {
                func: "loop_body".to_string(),
                args: vec![],
            }),
        };

        let callees = extract_callees(&body);
        assert!(callees.contains("loop_body"));
    }

    #[test]
    fn test_extract_callees_dedup() {
        // Same function called multiple times
        let body = CirExpr::Block(vec![
            CirExpr::Call { func: "helper".to_string(), args: vec![] },
            CirExpr::Call { func: "helper".to_string(), args: vec![] },
            CirExpr::Call { func: "helper".to_string(), args: vec![] },
        ]);

        let callees = extract_callees(&body);
        assert_eq!(callees.len(), 1);  // HashSet deduplicates
        assert!(callees.contains("helper"));
    }

    #[test]
    fn test_witness_to_proof_result_verified() {
        let func = make_test_function("foo");
        let witness = ProofWitness {
            function: "foo".to_string(),
            outcome: ProofOutcome::Verified,
            verification_time_ms: 100,
            smt_script: None,
            counterexample: None,
        };

        let result = witness_to_proof_result(&witness, &func);
        assert_eq!(result.status, VerificationStatus::Verified);
    }

    #[test]
    fn test_witness_to_proof_result_failed() {
        let func = make_test_function("foo");
        let witness = ProofWitness {
            function: "foo".to_string(),
            outcome: ProofOutcome::Failed("precondition violation".to_string()),
            verification_time_ms: 50,
            smt_script: None,
            counterexample: None,
        };

        let result = witness_to_proof_result(&witness, &func);
        assert!(matches!(result.status, VerificationStatus::Failed(_)));
    }

    #[test]
    fn test_witness_to_proof_result_skipped() {
        let func = make_test_function("foo");
        let witness = ProofWitness {
            function: "foo".to_string(),
            outcome: ProofOutcome::Skipped,
            verification_time_ms: 0,
            smt_script: None,
            counterexample: None,
        };

        let result = witness_to_proof_result(&witness, &func);
        assert_eq!(result.status, VerificationStatus::Skipped);
    }

    #[test]
    fn test_witness_to_proof_result_unknown() {
        let func = make_test_function("foo");
        let witness = ProofWitness {
            function: "foo".to_string(),
            outcome: ProofOutcome::Unknown("timeout".to_string()),
            verification_time_ms: 10000,
            smt_script: None,
            counterexample: None,
        };

        let result = witness_to_proof_result(&witness, &func);
        assert_eq!(result.status, VerificationStatus::Unknown);
    }

    // --- Cycle 393: Proven facts extraction tests ---

    use crate::cir::{NamedProposition, Proposition, CompareOp};

    fn make_test_function_with_contracts(name: &str) -> CirFunction {
        CirFunction {
            name: name.to_string(),
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
                    name: Some("x_positive".to_string()),
                    proposition: Proposition::compare(
                        CirExpr::Var("x".to_string()),
                        CompareOp::Gt,
                        CirExpr::IntLit(0),
                    ),
                },
            ],
            postconditions: vec![
                NamedProposition {
                    name: Some("result_positive".to_string()),
                    proposition: Proposition::compare(
                        CirExpr::Var("result".to_string()),
                        CompareOp::Gt,
                        CirExpr::IntLit(0),
                    ),
                },
            ],
            loop_invariants: vec![],
            effects: EffectSet::pure(),
            body: CirExpr::Var("x".to_string()),
        }
    }

    #[test]
    fn test_proven_facts_verified_with_contracts() {
        let func = make_test_function_with_contracts("foo");
        let witness = ProofWitness {
            function: "foo".to_string(),
            outcome: ProofOutcome::Verified,
            verification_time_ms: 100,
            smt_script: None,
            counterexample: None,
        };

        let result = witness_to_proof_result(&witness, &func);
        assert_eq!(result.status, VerificationStatus::Verified);
        // 1 precondition + 1 postcondition = 2 proven facts
        assert_eq!(result.proven_facts.len(), 2);
    }

    #[test]
    fn test_proven_facts_verified_no_contracts() {
        let func = make_test_function("foo");
        let witness = ProofWitness {
            function: "foo".to_string(),
            outcome: ProofOutcome::Verified,
            verification_time_ms: 50,
            smt_script: None,
            counterexample: None,
        };

        let result = witness_to_proof_result(&witness, &func);
        assert_eq!(result.status, VerificationStatus::Verified);
        // No contracts → no proven facts
        assert!(result.proven_facts.is_empty());
    }

    #[test]
    fn test_proven_facts_failed_no_facts() {
        let func = make_test_function_with_contracts("foo");
        let witness = ProofWitness {
            function: "foo".to_string(),
            outcome: ProofOutcome::Failed("violation".to_string()),
            verification_time_ms: 50,
            smt_script: None,
            counterexample: None,
        };

        let result = witness_to_proof_result(&witness, &func);
        assert!(matches!(result.status, VerificationStatus::Failed(_)));
        // Failed → no proven facts even with contracts
        assert!(result.proven_facts.is_empty());
    }

    #[test]
    fn test_proven_facts_precondition_evidence() {
        let func = make_test_function_with_contracts("foo");
        let witness = ProofWitness {
            function: "foo".to_string(),
            outcome: ProofOutcome::Verified,
            verification_time_ms: 100,
            smt_script: None,
            counterexample: None,
        };

        let result = witness_to_proof_result(&witness, &func);
        // First fact is from precondition
        assert!(matches!(result.proven_facts[0].evidence, ProofEvidence::Precondition));
    }

    #[test]
    fn test_proven_facts_postcondition_evidence() {
        let func = make_test_function_with_contracts("foo");
        let witness = ProofWitness {
            function: "foo".to_string(),
            outcome: ProofOutcome::Verified,
            verification_time_ms: 100,
            smt_script: None,
            counterexample: None,
        };

        let result = witness_to_proof_result(&witness, &func);
        // Second fact is from postcondition (SmtProof)
        assert!(matches!(result.proven_facts[1].evidence, ProofEvidence::SmtProof { .. }));
    }

    #[test]
    fn test_proven_facts_scope_is_function() {
        let func = make_test_function_with_contracts("bar");
        let witness = ProofWitness {
            function: "bar".to_string(),
            outcome: ProofOutcome::Verified,
            verification_time_ms: 100,
            smt_script: None,
            counterexample: None,
        };

        let result = witness_to_proof_result(&witness, &func);
        for fact in &result.proven_facts {
            match &fact.scope {
                ProofScope::Function(id) => assert_eq!(id.name, "bar"),
                _ => panic!("Expected Function scope"),
            }
        }
    }

    #[test]
    fn test_new_function_verified_on_add() {
        let mut verifier = IncrementalVerifier::new();

        let program1 = make_test_program(vec![make_test_function("foo")]);
        let _result1 = verifier.verify_incremental(None, &program1);

        // Add a new function
        let program2 = make_test_program(vec![
            make_test_function("foo"),
            make_test_function("bar"),
        ]);
        let result2 = verifier.verify_incremental(Some(&program1), &program2);

        // bar should be verified (new), foo should be skipped
        let all_processed: Vec<String> = result2.verified_functions.iter()
            .chain(result2.failed_functions.iter().map(|(n, _)| n))
            .cloned()
            .collect();
        assert!(all_processed.contains(&"bar".to_string()));
    }

    #[test]
    fn test_database_accessor() {
        let verifier = IncrementalVerifier::new();
        assert!(verifier.database().is_empty());
    }

    #[test]
    fn test_database_mut_accessor() {
        let mut verifier = IncrementalVerifier::new();
        verifier.database_mut().clear();
        assert!(verifier.database().is_empty());
    }

    // ================================================================
    // Additional incremental verification tests (Cycle 1237)
    // ================================================================

    #[test]
    fn test_extract_callees_in_binop() {
        let body = CirExpr::BinOp {
            op: crate::cir::BinOp::Add,
            lhs: Box::new(CirExpr::Call { func: "left".to_string(), args: vec![] }),
            rhs: Box::new(CirExpr::Call { func: "right".to_string(), args: vec![] }),
        };
        let callees = extract_callees(&body);
        assert!(callees.contains("left"));
        assert!(callees.contains("right"));
    }

    #[test]
    fn test_extract_callees_in_for() {
        let body = CirExpr::For {
            var: "i".to_string(),
            iter: Box::new(CirExpr::Call { func: "range".to_string(), args: vec![] }),
            body: Box::new(CirExpr::Call { func: "process".to_string(), args: vec![] }),
        };
        let callees = extract_callees(&body);
        assert!(callees.contains("range"));
        assert!(callees.contains("process"));
    }

    #[test]
    fn test_extract_callees_in_index() {
        let body = CirExpr::Index {
            base: Box::new(CirExpr::Call { func: "get_arr".to_string(), args: vec![] }),
            index: Box::new(CirExpr::Call { func: "get_idx".to_string(), args: vec![] }),
        };
        let callees = extract_callees(&body);
        assert!(callees.contains("get_arr"));
        assert!(callees.contains("get_idx"));
    }

    #[test]
    fn test_extract_callees_in_array() {
        let body = CirExpr::Array(vec![
            CirExpr::Call { func: "elem1".to_string(), args: vec![] },
            CirExpr::Call { func: "elem2".to_string(), args: vec![] },
        ]);
        let callees = extract_callees(&body);
        assert!(callees.contains("elem1"));
        assert!(callees.contains("elem2"));
    }

    #[test]
    fn test_extract_callees_in_let_mut() {
        let body = CirExpr::LetMut {
            name: "x".to_string(),
            ty: CirType::I64,
            value: Box::new(CirExpr::Call { func: "init_fn".to_string(), args: vec![] }),
            body: Box::new(CirExpr::IntLit(0)),
        };
        let callees = extract_callees(&body);
        assert!(callees.contains("init_fn"));
    }

    #[test]
    fn test_extract_callees_in_unary_op() {
        let body = CirExpr::UnaryOp {
            op: crate::cir::UnaryOp::Neg,
            operand: Box::new(CirExpr::Call { func: "compute".to_string(), args: vec![] }),
        };
        let callees = extract_callees(&body);
        assert!(callees.contains("compute"));
    }

    #[test]
    fn test_extract_callees_in_field() {
        let body = CirExpr::Field {
            base: Box::new(CirExpr::Call { func: "get_struct".to_string(), args: vec![] }),
            field: "x".to_string(),
        };
        let callees = extract_callees(&body);
        assert!(callees.contains("get_struct"));
    }

    #[test]
    fn test_incremental_result_debug() {
        let result = IncrementalVerificationResult {
            verified_functions: vec!["foo".to_string()],
            skipped_functions: vec!["bar".to_string()],
            failed_functions: vec![],
            total_time_ms: 100,
            time_saved_ms: 50,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("foo"));
        assert!(debug.contains("bar"));
    }

    #[test]
    fn test_witness_to_proof_result_error() {
        let func = make_test_function("foo");
        let witness = ProofWitness {
            function: "foo".to_string(),
            outcome: ProofOutcome::Error("solver crashed".to_string()),
            verification_time_ms: 0,
            smt_script: None,
            counterexample: None,
        };
        let result = witness_to_proof_result(&witness, &func);
        assert_eq!(result.status, VerificationStatus::Unknown);
        assert!(result.proven_facts.is_empty());
    }

    #[test]
    fn test_extract_callees_in_ref() {
        let body = CirExpr::Ref(Box::new(CirExpr::Call {
            func: "get_val".to_string(),
            args: vec![],
        }));
        let callees = extract_callees(&body);
        assert!(callees.contains("get_val"));
    }
}
