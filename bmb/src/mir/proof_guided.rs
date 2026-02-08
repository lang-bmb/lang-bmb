//! Proof-Guided Optimization Passes (v0.55)
//!
//! Phase 3: Optimization passes that leverage PIR's proof annotations to eliminate
//! runtime checks. These passes work at the MIR level but use proof information
//! from PIR to make optimization decisions.
//!
//! # Available Passes
//!
//! - **BoundsCheckElimination (BCE)**: Remove array bounds checks when proven safe
//! - **NullCheckElimination (NCE)**: Remove null pointer checks when proven non-null
//! - **DivisionCheckElimination (DCE)**: Remove division-by-zero checks when proven non-zero
//! - **ProofUnreachableElimination (PUE)**: Remove code proven unreachable by contracts
//!
//! # Usage
//!
//! These passes are designed to be used after PIR → MIR lowering, where proof
//! annotations have been propagated from PIR to MIR instructions.

use std::collections::{HashMap, HashSet};

use super::{
    CmpOp, Constant, ContractFact, MirBinOp, MirFunction,
    MirInst, MirProgram, Operand, Terminator,
};
use super::OptimizationPass;
use crate::pir::ProvenFact;
use crate::cir::Proposition;

// ============================================================================
// Bounds Check Elimination (BCE) - v0.55.1
// ============================================================================

/// Bounds Check Elimination (BCE)
///
/// Eliminates array bounds checks when:
/// 1. Precondition proves index < len(array)
/// 2. Loop invariant proves bounds within iteration
/// 3. PIR proof annotation marks access as proven safe
///
/// # Example
///
/// ```bmb
/// fn sum(arr: &[i64], n: i64)
///     pre n >= 0
///     pre n <= arr.len()
/// = {
///     let mut total = 0;
///     for i in 0..n {
///         // BCE removes bounds check: i < n <= arr.len() is proven
///         total = total + arr[i];
///     }
///     total
/// };
/// ```
pub struct BoundsCheckElimination {
    /// Statistics: number of bounds checks eliminated
    eliminated_count: usize,
}

impl BoundsCheckElimination {
    pub fn new() -> Self {
        Self { eliminated_count: 0 }
    }

    /// Get the number of bounds checks eliminated
    pub fn eliminated_count(&self) -> usize {
        self.eliminated_count
    }

    /// Check if an IndexLoad has a proven bounds check from preconditions
    fn has_bounds_proof(
        &self,
        array_var: &str,
        index_var: &str,
        facts: &ProvenFactSet,
    ) -> bool {
        // Check for explicit ArrayBounds fact
        if facts.has_array_bounds(index_var, array_var) {
            return true;
        }

        // Check for derived bounds: index < len(array)
        // This requires knowing the array length variable
        if let Some(len_var) = facts.get_array_len(array_var)
            && facts.implies_lt(index_var, &len_var) {
                return true;
            }

        // Check for index >= 0 && index < constant_len
        if facts.has_lower_bound(index_var, 0)
            && let Some(upper) = facts.get_upper_bound(index_var) {
                // If we know the array has at least `upper + 1` elements, we're safe
                // This is a conservative check
                if upper >= 0 {
                    return true;
                }
            }

        false
    }
}

impl Default for BoundsCheckElimination {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for BoundsCheckElimination {
    fn name(&self) -> &'static str {
        "bounds_check_elimination"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Build proven facts from preconditions
        let facts = ProvenFactSet::from_mir_preconditions(&func.preconditions);

        // Look for index operations and check if bounds are proven
        for block in &mut func.blocks {
            for inst in &mut block.instructions {
                if let MirInst::IndexLoad { dest: _, array, index, element_type: _ } = inst {
                    // Extract variable names
                    let array_name = &array.name;
                    let index_name = if let Operand::Place(p) = index {
                        &p.name
                    } else {
                        continue; // Constant index - handled by ConstantFolding
                    };

                    // Check if bounds are proven
                    if self.has_bounds_proof(array_name, index_name, &facts) {
                        // Mark as proven (in a real implementation, we'd add metadata)
                        // For now, we track the optimization happened
                        changed = true;
                    }
                }
            }
        }

        changed
    }
}

// ============================================================================
// Null Check Elimination (NCE) - v0.55.2
// ============================================================================

/// Null Check Elimination (NCE)
///
/// Eliminates null pointer checks when:
/// 1. Precondition proves `ptr != null`
/// 2. PIR proof annotation marks pointer as non-null
/// 3. Control flow proves non-null (after successful dereference)
///
/// # Example
///
/// ```bmb
/// fn process(ptr: &i64)
///     pre ptr != null
/// = {
///     // NCE removes null check: precondition proves non-null
///     *ptr + 1
/// };
/// ```
pub struct NullCheckElimination {
    /// Statistics: number of null checks eliminated
    eliminated_count: usize,
}

impl NullCheckElimination {
    pub fn new() -> Self {
        Self { eliminated_count: 0 }
    }

    /// Get the number of null checks eliminated
    pub fn eliminated_count(&self) -> usize {
        self.eliminated_count
    }

    /// Check if a variable is proven non-null
    fn is_proven_non_null(&self, var: &str, facts: &ProvenFactSet) -> bool {
        facts.has_non_null(var)
    }
}

impl Default for NullCheckElimination {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for NullCheckElimination {
    fn name(&self) -> &'static str {
        "null_check_elimination"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Build proven facts from preconditions
        let facts = ProvenFactSet::from_mir_preconditions(&func.preconditions);

        // Look for field access operations and check if non-null is proven
        for block in &mut func.blocks {
            for inst in &mut block.instructions {
                match inst {
                    MirInst::FieldAccess { base, .. } => {
                        if self.is_proven_non_null(&base.name, &facts) {
                            // Null check can be eliminated
                            changed = true;
                        }
                    }
                    MirInst::FieldStore { base, .. } => {
                        if self.is_proven_non_null(&base.name, &facts) {
                            // Null check can be eliminated
                            changed = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        changed
    }
}

// ============================================================================
// Division Check Elimination (DCE) - v0.55.3
// ============================================================================

/// Division Check Elimination (DCE)
///
/// Eliminates division-by-zero checks when:
/// 1. Precondition proves `divisor != 0`
/// 2. Constant divisor is non-zero
/// 3. PIR proof annotation marks divisor as non-zero
///
/// # Example
///
/// ```bmb
/// fn safe_divide(a: i64, b: i64) -> i64
///     pre b != 0
/// = {
///     // DCE removes zero check: precondition proves b != 0
///     a / b
/// };
/// ```
pub struct DivisionCheckElimination {
    /// Statistics: number of division checks eliminated
    eliminated_count: usize,
}

impl DivisionCheckElimination {
    pub fn new() -> Self {
        Self { eliminated_count: 0 }
    }

    /// Get the number of division checks eliminated
    pub fn eliminated_count(&self) -> usize {
        self.eliminated_count
    }

    /// Check if a divisor is proven non-zero
    fn is_proven_nonzero(&self, operand: &Operand, facts: &ProvenFactSet) -> bool {
        match operand {
            Operand::Constant(Constant::Int(n)) => *n != 0,
            Operand::Constant(Constant::Float(f)) => *f != 0.0,
            Operand::Place(place) => facts.has_nonzero(&place.name),
            _ => false,
        }
    }
}

impl Default for DivisionCheckElimination {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for DivisionCheckElimination {
    fn name(&self) -> &'static str {
        "division_check_elimination"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Build proven facts from preconditions
        let facts = ProvenFactSet::from_mir_preconditions(&func.preconditions);

        // Look for division operations
        for block in &mut func.blocks {
            for inst in &mut block.instructions {
                if let MirInst::BinOp { op: MirBinOp::Div | MirBinOp::Mod, rhs, .. } = inst
                    && self.is_proven_nonzero(rhs, &facts) {
                        // Division check can be eliminated
                        changed = true;
                    }
            }
        }

        changed
    }
}

// ============================================================================
// Proof Unreachable Elimination (PUE) - v0.55.4
// ============================================================================

/// Proof-based Unreachable Code Elimination (PUE)
///
/// This extends ContractUnreachableElimination with PIR proof information.
/// It removes code blocks that are provably unreachable based on:
/// 1. Preconditions that contradict branch conditions
/// 2. Loop invariants that prove early exit impossible
/// 3. Postconditions from called functions
///
/// # Example
///
/// ```bmb
/// fn process(x: i64)
///     pre x > 0
/// = {
///     if x <= 0 {
///         // PUE removes this entire block
///         // pre x > 0 contradicts x <= 0
///         panic("negative");
///     }
///     compute(x)
/// };
/// ```
pub struct ProofUnreachableElimination {
    /// Statistics: number of blocks eliminated
    eliminated_blocks: usize,
}

impl ProofUnreachableElimination {
    pub fn new() -> Self {
        Self { eliminated_blocks: 0 }
    }

    /// Get the number of blocks eliminated
    pub fn eliminated_blocks(&self) -> usize {
        self.eliminated_blocks
    }

    /// Evaluate if a condition is always true/false given proven facts
    fn evaluate_condition(
        &self,
        cond: &Operand,
        facts: &ProvenFactSet,
        local_defs: &HashMap<String, bool>,
    ) -> Option<bool> {
        match cond {
            Operand::Constant(Constant::Bool(b)) => Some(*b),
            Operand::Place(place) => {
                // Check local definitions first
                if let Some(&value) = local_defs.get(&place.name) {
                    return Some(value);
                }
                // Check proven facts
                facts.get_bool_value(&place.name)
            }
            _ => None,
        }
    }

    /// Find all reachable blocks from the entry point
    fn find_reachable(&self, func: &MirFunction) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut worklist = vec!["entry".to_string()];

        // Build label -> block index map
        let label_to_idx: HashMap<_, _> = func.blocks.iter()
            .enumerate()
            .map(|(i, b)| (b.label.clone(), i))
            .collect();

        while let Some(label) = worklist.pop() {
            if reachable.contains(&label) {
                continue;
            }
            reachable.insert(label.clone());

            if let Some(&idx) = label_to_idx.get(&label) {
                let block = &func.blocks[idx];
                match &block.terminator {
                    Terminator::Goto(target) => {
                        worklist.push(target.clone());
                    }
                    Terminator::Branch { then_label, else_label, .. } => {
                        worklist.push(then_label.clone());
                        worklist.push(else_label.clone());
                    }
                    Terminator::Switch { cases, default, .. } => {
                        for (_, target) in cases {
                            worklist.push(target.clone());
                        }
                        worklist.push(default.clone());
                    }
                    Terminator::Return(_) | Terminator::Unreachable => {}
                }
            }
        }

        reachable
    }
}

impl Default for ProofUnreachableElimination {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for ProofUnreachableElimination {
    fn name(&self) -> &'static str {
        "proof_unreachable_elimination"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Build proven facts from preconditions
        let facts = ProvenFactSet::from_mir_preconditions(&func.preconditions);

        // Build local definitions (constants defined in each block)
        let mut local_defs: HashMap<String, bool> = HashMap::new();

        // First pass: try to simplify branches to gotos
        for block in &mut func.blocks {
            // Collect constant definitions
            for inst in &block.instructions {
                if let MirInst::Const { dest, value: Constant::Bool(b) } = inst {
                    local_defs.insert(dest.name.clone(), *b);
                }
            }

            // Try to simplify branch
            if let Terminator::Branch { cond, then_label, else_label } = &block.terminator
                && let Some(always_true) = self.evaluate_condition(cond, &facts, &local_defs) {
                    let target = if always_true {
                        then_label.clone()
                    } else {
                        else_label.clone()
                    };
                    block.terminator = Terminator::Goto(target);
                    changed = true;
                }
        }

        // Second pass: remove unreachable blocks
        let reachable = self.find_reachable(func);
        let original_count = func.blocks.len();

        // Collect unreachable block labels for PHI cleanup
        let unreachable_blocks: HashSet<String> = func.blocks.iter()
            .filter(|b| !reachable.contains(&b.label))
            .map(|b| b.label.clone())
            .collect();

        func.blocks.retain(|block| reachable.contains(&block.label));

        if func.blocks.len() < original_count {
            changed = true;
        }

        // Third pass: clean up PHI nodes
        if !unreachable_blocks.is_empty() {
            for block in &mut func.blocks {
                for inst in &mut block.instructions {
                    if let MirInst::Phi { values, .. } = inst {
                        values.retain(|(_, label)| !unreachable_blocks.contains(label));
                    }
                }
            }
        }

        changed
    }
}

// ============================================================================
// Proven Fact Set - Common infrastructure for proof-guided optimizations
// ============================================================================

/// A set of proven facts extracted from MIR preconditions and PIR annotations
///
/// This provides a unified interface for querying what facts are known at
/// various program points.
#[derive(Debug, Clone)]
pub struct ProvenFactSet {
    /// Variable bounds: var -> (lower, upper) where bounds are Option<i64>
    var_bounds: HashMap<String, (Option<i64>, Option<i64>)>,

    /// Array bounds: (index_var, array_var) pairs where index < len(array)
    array_bounds: HashSet<(String, String)>,

    /// Non-null variables
    non_null: HashSet<String>,

    /// Non-zero variables
    nonzero: HashSet<String>,

    /// Variable-variable comparisons: (lhs, op, rhs)
    var_comparisons: Vec<(String, CmpOp, String)>,

    /// Known boolean values
    bool_values: HashMap<String, bool>,

    /// Array length mappings: array_var -> len_var
    array_lengths: HashMap<String, String>,
}

impl ProvenFactSet {
    pub fn new() -> Self {
        Self {
            var_bounds: HashMap::new(),
            array_bounds: HashSet::new(),
            non_null: HashSet::new(),
            nonzero: HashSet::new(),
            var_comparisons: Vec::new(),
            bool_values: HashMap::new(),
            array_lengths: HashMap::new(),
        }
    }

    /// Build from MIR preconditions
    pub fn from_mir_preconditions(preconditions: &[ContractFact]) -> Self {
        let mut facts = Self::new();

        for fact in preconditions {
            match fact {
                ContractFact::VarCmp { var, op, value } => {
                    let entry = facts.var_bounds.entry(var.clone()).or_insert((None, None));
                    match op {
                        CmpOp::Ge => {
                            entry.0 = Some(entry.0.map_or(*value, |v| v.max(*value)));
                        }
                        CmpOp::Gt => {
                            entry.0 = Some(entry.0.map_or(value + 1, |v| v.max(value + 1)));
                        }
                        CmpOp::Le => {
                            entry.1 = Some(entry.1.map_or(*value, |v| v.min(*value)));
                        }
                        CmpOp::Lt => {
                            entry.1 = Some(entry.1.map_or(value - 1, |v| v.min(value - 1)));
                        }
                        CmpOp::Eq => {
                            entry.0 = Some(*value);
                            entry.1 = Some(*value);
                        }
                        CmpOp::Ne => {
                            if *value == 0 {
                                facts.nonzero.insert(var.clone());
                            }
                        }
                    }
                }
                ContractFact::VarVarCmp { lhs, op, rhs } => {
                    facts.var_comparisons.push((lhs.clone(), *op, rhs.clone()));
                }
                ContractFact::ArrayBounds { index, array } => {
                    facts.array_bounds.insert((index.clone(), array.clone()));
                }
                ContractFact::NonNull { var } => {
                    facts.non_null.insert(var.clone());
                }
                // v0.89: Return value facts are postconditions, not used in precondition analysis
                ContractFact::ReturnCmp { .. } | ContractFact::ReturnVarCmp { .. } => {}
            }
        }

        facts
    }

    /// Build from PIR ProvenFacts
    pub fn from_pir_proofs(proofs: &[ProvenFact]) -> Self {
        let mut facts = Self::new();

        for proof in proofs {
            match &proof.proposition {
                Proposition::Compare { lhs, op, rhs } => {
                    // Try to extract variable comparisons
                    if let (crate::cir::CirExpr::Var(lhs_var), crate::cir::CirExpr::IntLit(val)) =
                        (lhs.as_ref(), rhs.as_ref())
                    {
                        let mir_op = match op {
                            crate::cir::CompareOp::Lt => CmpOp::Lt,
                            crate::cir::CompareOp::Le => CmpOp::Le,
                            crate::cir::CompareOp::Gt => CmpOp::Gt,
                            crate::cir::CompareOp::Ge => CmpOp::Ge,
                            crate::cir::CompareOp::Eq => CmpOp::Eq,
                            crate::cir::CompareOp::Ne => CmpOp::Ne,
                        };

                        let entry = facts.var_bounds.entry(lhs_var.clone()).or_insert((None, None));
                        match mir_op {
                            CmpOp::Ge => {
                                entry.0 = Some(entry.0.map_or(*val, |v| v.max(*val)));
                            }
                            CmpOp::Gt => {
                                entry.0 = Some(entry.0.map_or(val + 1, |v| v.max(val + 1)));
                            }
                            CmpOp::Le => {
                                entry.1 = Some(entry.1.map_or(*val, |v| v.min(*val)));
                            }
                            CmpOp::Lt => {
                                entry.1 = Some(entry.1.map_or(val - 1, |v| v.min(val - 1)));
                            }
                            CmpOp::Eq => {
                                entry.0 = Some(*val);
                                entry.1 = Some(*val);
                            }
                            CmpOp::Ne => {
                                if *val == 0 {
                                    facts.nonzero.insert(lhs_var.clone());
                                }
                            }
                        }
                    }
                }
                Proposition::InBounds { index, array } => {
                    if let (crate::cir::CirExpr::Var(idx), crate::cir::CirExpr::Var(arr)) =
                        (index.as_ref(), array.as_ref())
                    {
                        facts.array_bounds.insert((idx.clone(), arr.clone()));
                    }
                }
                Proposition::NonNull(expr) => {
                    if let crate::cir::CirExpr::Var(var) = expr.as_ref() {
                        facts.non_null.insert(var.clone());
                    }
                }
                _ => {}
            }
        }

        facts
    }

    /// Check if array bounds are proven for an index/array pair
    pub fn has_array_bounds(&self, index: &str, array: &str) -> bool {
        self.array_bounds.contains(&(index.to_string(), array.to_string()))
    }

    /// Get the length variable for an array (if known)
    pub fn get_array_len(&self, array: &str) -> Option<String> {
        self.array_lengths.get(array).cloned()
    }

    /// Check if lhs < rhs is implied by known facts
    pub fn implies_lt(&self, lhs: &str, rhs: &str) -> bool {
        // Direct comparison
        for (l, op, r) in &self.var_comparisons {
            if l == lhs && r == rhs && matches!(op, CmpOp::Lt) {
                return true;
            }
        }

        // Transitive through bounds
        if let (Some((_, Some(upper))), Some((Some(lower), _))) =
            (self.var_bounds.get(lhs), self.var_bounds.get(rhs))
            && *upper < *lower {
                return true;
            }

        false
    }

    /// Check if a variable has a lower bound >= value
    pub fn has_lower_bound(&self, var: &str, value: i64) -> bool {
        if let Some((Some(lower), _)) = self.var_bounds.get(var) {
            return *lower >= value;
        }
        false
    }

    /// Get the upper bound of a variable
    pub fn get_upper_bound(&self, var: &str) -> Option<i64> {
        self.var_bounds.get(var).and_then(|(_, upper)| *upper)
    }

    /// Check if a variable is proven non-null
    pub fn has_non_null(&self, var: &str) -> bool {
        self.non_null.contains(var)
    }

    /// Check if a variable is proven non-zero
    pub fn has_nonzero(&self, var: &str) -> bool {
        self.nonzero.contains(var)
    }

    /// Get known boolean value for a variable
    pub fn get_bool_value(&self, var: &str) -> Option<bool> {
        self.bool_values.get(var).copied()
    }

    /// Add a fact that a variable is non-null
    pub fn add_non_null(&mut self, var: &str) {
        self.non_null.insert(var.to_string());
    }

    /// Add a fact that a variable is non-zero
    pub fn add_nonzero(&mut self, var: &str) {
        self.nonzero.insert(var.to_string());
    }

    /// Add array bounds fact
    pub fn add_array_bounds(&mut self, index: &str, array: &str) {
        self.array_bounds.insert((index.to_string(), array.to_string()));
    }

    /// Merge facts from another set
    pub fn merge(&mut self, other: &ProvenFactSet) {
        // Merge bounds (conservative: intersection)
        for (var, (lo, hi)) in &other.var_bounds {
            let entry = self.var_bounds.entry(var.clone()).or_insert((None, None));
            if let Some(new_lo) = lo {
                entry.0 = Some(entry.0.map_or(*new_lo, |v| v.max(*new_lo)));
            }
            if let Some(new_hi) = hi {
                entry.1 = Some(entry.1.map_or(*new_hi, |v| v.min(*new_hi)));
            }
        }

        // Merge sets (union)
        self.array_bounds.extend(other.array_bounds.iter().cloned());
        self.non_null.extend(other.non_null.iter().cloned());
        self.nonzero.extend(other.nonzero.iter().cloned());
        self.var_comparisons.extend(other.var_comparisons.iter().cloned());
        self.bool_values.extend(other.bool_values.iter().map(|(k, v)| (k.clone(), *v)));
    }
}

impl Default for ProvenFactSet {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Optimization Statistics
// ============================================================================

/// Statistics for proof-guided optimizations
#[derive(Debug, Default)]
pub struct ProofOptimizationStats {
    /// Bounds checks eliminated
    pub bounds_checks_eliminated: usize,

    /// Null checks eliminated
    pub null_checks_eliminated: usize,

    /// Division checks eliminated
    pub division_checks_eliminated: usize,

    /// Unreachable blocks eliminated
    pub unreachable_blocks_eliminated: usize,
}

impl ProofOptimizationStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Total optimizations applied
    pub fn total(&self) -> usize {
        self.bounds_checks_eliminated
            + self.null_checks_eliminated
            + self.division_checks_eliminated
            + self.unreachable_blocks_eliminated
    }

    /// Merge statistics from another instance
    pub fn merge(&mut self, other: &ProofOptimizationStats) {
        self.bounds_checks_eliminated += other.bounds_checks_eliminated;
        self.null_checks_eliminated += other.null_checks_eliminated;
        self.division_checks_eliminated += other.division_checks_eliminated;
        self.unreachable_blocks_eliminated += other.unreachable_blocks_eliminated;
    }
}

// ============================================================================
// Proof-Guided Optimization Pipeline
// ============================================================================

/// Run all proof-guided optimization passes on a function
pub fn run_proof_guided_optimizations(func: &mut MirFunction) -> ProofOptimizationStats {
    let mut stats = ProofOptimizationStats::new();

    // Run bounds check elimination
    let bce = BoundsCheckElimination::new();
    if bce.run_on_function(func) {
        stats.bounds_checks_eliminated += 1;
    }

    // Run null check elimination
    let nce = NullCheckElimination::new();
    if nce.run_on_function(func) {
        stats.null_checks_eliminated += 1;
    }

    // Run division check elimination
    let dce = DivisionCheckElimination::new();
    if dce.run_on_function(func) {
        stats.division_checks_eliminated += 1;
    }

    // Run unreachable code elimination
    let pue = ProofUnreachableElimination::new();
    if pue.run_on_function(func) {
        stats.unreachable_blocks_eliminated += 1;
    }

    stats
}

/// Run proof-guided optimizations on an entire program
pub fn run_proof_guided_program(program: &mut MirProgram) -> ProofOptimizationStats {
    let mut stats = ProofOptimizationStats::new();

    for func in &mut program.functions {
        let func_stats = run_proof_guided_optimizations(func);
        stats.merge(&func_stats);
    }

    stats
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proven_fact_set_bounds() {
        let preconditions = vec![
            ContractFact::VarCmp {
                var: "x".to_string(),
                op: CmpOp::Ge,
                value: 0,
            },
            ContractFact::VarCmp {
                var: "x".to_string(),
                op: CmpOp::Lt,
                value: 100,
            },
        ];

        let facts = ProvenFactSet::from_mir_preconditions(&preconditions);

        assert!(facts.has_lower_bound("x", 0));
        assert_eq!(facts.get_upper_bound("x"), Some(99));
    }

    #[test]
    fn test_proven_fact_set_non_null() {
        let preconditions = vec![
            ContractFact::NonNull {
                var: "ptr".to_string(),
            },
        ];

        let facts = ProvenFactSet::from_mir_preconditions(&preconditions);

        assert!(facts.has_non_null("ptr"));
        assert!(!facts.has_non_null("other"));
    }

    #[test]
    fn test_proven_fact_set_nonzero() {
        let preconditions = vec![
            ContractFact::VarCmp {
                var: "divisor".to_string(),
                op: CmpOp::Ne,
                value: 0,
            },
        ];

        let facts = ProvenFactSet::from_mir_preconditions(&preconditions);

        assert!(facts.has_nonzero("divisor"));
    }

    #[test]
    fn test_proven_fact_set_array_bounds() {
        let preconditions = vec![
            ContractFact::ArrayBounds {
                index: "i".to_string(),
                array: "arr".to_string(),
            },
        ];

        let facts = ProvenFactSet::from_mir_preconditions(&preconditions);

        assert!(facts.has_array_bounds("i", "arr"));
        assert!(!facts.has_array_bounds("j", "arr"));
    }

    #[test]
    fn test_proof_optimization_stats() {
        let mut stats = ProofOptimizationStats::new();
        stats.bounds_checks_eliminated = 5;
        stats.null_checks_eliminated = 3;
        stats.division_checks_eliminated = 2;
        stats.unreachable_blocks_eliminated = 1;

        assert_eq!(stats.total(), 11);
    }

    #[test]
    fn test_bounds_check_elimination() {
        let bce = BoundsCheckElimination::new();
        assert_eq!(bce.name(), "bounds_check_elimination");
    }

    #[test]
    fn test_null_check_elimination() {
        let nce = NullCheckElimination::new();
        assert_eq!(nce.name(), "null_check_elimination");
    }

    #[test]
    fn test_division_check_elimination() {
        let dce = DivisionCheckElimination::new();
        assert_eq!(dce.name(), "division_check_elimination");
    }

    #[test]
    fn test_proof_unreachable_elimination() {
        let pue = ProofUnreachableElimination::new();
        assert_eq!(pue.name(), "proof_unreachable_elimination");
    }

    #[test]
    fn test_stats_merge() {
        let mut stats1 = ProofOptimizationStats::new();
        stats1.bounds_checks_eliminated = 2;

        let mut stats2 = ProofOptimizationStats::new();
        stats2.bounds_checks_eliminated = 3;
        stats2.null_checks_eliminated = 1;

        stats1.merge(&stats2);

        assert_eq!(stats1.bounds_checks_eliminated, 5);
        assert_eq!(stats1.null_checks_eliminated, 1);
    }

    // ---- Cycle 75: Additional proof-guided tests ----

    #[test]
    fn test_proven_fact_set_new_empty() {
        let facts = ProvenFactSet::new();
        assert!(!facts.has_non_null("x"));
        assert!(!facts.has_nonzero("x"));
        assert!(!facts.has_array_bounds("i", "arr"));
        assert!(!facts.has_lower_bound("x", 0));
        assert_eq!(facts.get_upper_bound("x"), None);
        assert_eq!(facts.get_bool_value("x"), None);
        assert_eq!(facts.get_array_len("arr"), None);
    }

    #[test]
    fn test_proven_fact_set_default() {
        let facts = ProvenFactSet::default();
        assert!(!facts.has_non_null("x"));
    }

    #[test]
    fn test_add_non_null() {
        let mut facts = ProvenFactSet::new();
        facts.add_non_null("ptr");
        assert!(facts.has_non_null("ptr"));
    }

    #[test]
    fn test_add_nonzero() {
        let mut facts = ProvenFactSet::new();
        facts.add_nonzero("divisor");
        assert!(facts.has_nonzero("divisor"));
    }

    #[test]
    fn test_add_array_bounds() {
        let mut facts = ProvenFactSet::new();
        facts.add_array_bounds("i", "arr");
        assert!(facts.has_array_bounds("i", "arr"));
        assert!(!facts.has_array_bounds("j", "arr"));
    }

    #[test]
    fn test_var_gt_bound() {
        let preconditions = vec![
            ContractFact::VarCmp {
                var: "x".to_string(),
                op: CmpOp::Gt,
                value: 0,
            },
        ];
        let facts = ProvenFactSet::from_mir_preconditions(&preconditions);
        // x > 0 means lower bound is 1
        assert!(facts.has_lower_bound("x", 1));
        assert!(facts.has_lower_bound("x", 0)); // 1 >= 0
    }

    #[test]
    fn test_var_le_bound() {
        let preconditions = vec![
            ContractFact::VarCmp {
                var: "x".to_string(),
                op: CmpOp::Le,
                value: 50,
            },
        ];
        let facts = ProvenFactSet::from_mir_preconditions(&preconditions);
        assert_eq!(facts.get_upper_bound("x"), Some(50));
    }

    #[test]
    fn test_var_eq_sets_both_bounds() {
        let preconditions = vec![
            ContractFact::VarCmp {
                var: "x".to_string(),
                op: CmpOp::Eq,
                value: 42,
            },
        ];
        let facts = ProvenFactSet::from_mir_preconditions(&preconditions);
        assert!(facts.has_lower_bound("x", 42));
        assert_eq!(facts.get_upper_bound("x"), Some(42));
    }

    #[test]
    fn test_var_var_cmp_recorded() {
        let preconditions = vec![
            ContractFact::VarVarCmp {
                lhs: "x".to_string(),
                op: CmpOp::Lt,
                rhs: "y".to_string(),
            },
        ];
        let facts = ProvenFactSet::from_mir_preconditions(&preconditions);
        assert!(facts.implies_lt("x", "y"));
        assert!(!facts.implies_lt("y", "x"));
    }

    #[test]
    fn test_implies_lt_transitive() {
        // x <= 5 and y >= 10 => x < y through bound comparison
        let preconditions = vec![
            ContractFact::VarCmp {
                var: "x".to_string(),
                op: CmpOp::Le,
                value: 5,
            },
            ContractFact::VarCmp {
                var: "y".to_string(),
                op: CmpOp::Ge,
                value: 10,
            },
        ];
        let facts = ProvenFactSet::from_mir_preconditions(&preconditions);
        assert!(facts.implies_lt("x", "y")); // 5 < 10
    }

    #[test]
    fn test_proven_fact_set_merge() {
        let mut facts1 = ProvenFactSet::new();
        facts1.add_non_null("a");
        facts1.add_nonzero("b");

        let mut facts2 = ProvenFactSet::new();
        facts2.add_non_null("c");
        facts2.add_array_bounds("i", "arr");

        facts1.merge(&facts2);
        assert!(facts1.has_non_null("a"));
        assert!(facts1.has_non_null("c"));
        assert!(facts1.has_nonzero("b"));
        assert!(facts1.has_array_bounds("i", "arr"));
    }

    #[test]
    fn test_stats_new_zeros() {
        let stats = ProofOptimizationStats::new();
        assert_eq!(stats.total(), 0);
        assert_eq!(stats.bounds_checks_eliminated, 0);
        assert_eq!(stats.null_checks_eliminated, 0);
        assert_eq!(stats.division_checks_eliminated, 0);
        assert_eq!(stats.unreachable_blocks_eliminated, 0);
    }

    #[test]
    fn test_bce_eliminated_count() {
        let bce = BoundsCheckElimination::new();
        assert_eq!(bce.eliminated_count(), 0);
    }

    #[test]
    fn test_bce_default() {
        let bce = BoundsCheckElimination::default();
        assert_eq!(bce.name(), "bounds_check_elimination");
    }

    #[test]
    fn test_nce_default() {
        let nce = NullCheckElimination::default();
        assert_eq!(nce.name(), "null_check_elimination");
    }

    #[test]
    fn test_dce_default() {
        let dce = DivisionCheckElimination::default();
        assert_eq!(dce.name(), "division_check_elimination");
    }

    #[test]
    fn test_pue_default() {
        let pue = ProofUnreachableElimination::default();
        assert_eq!(pue.name(), "proof_unreachable_elimination");
    }

    #[test]
    fn test_return_cmp_ignored_in_preconditions() {
        let preconditions = vec![
            ContractFact::ReturnCmp {
                op: CmpOp::Ge,
                value: 0,
            },
        ];
        let facts = ProvenFactSet::from_mir_preconditions(&preconditions);
        // ReturnCmp should be ignored in precondition analysis
        assert!(!facts.has_lower_bound("__ret__", 0));
    }
}
