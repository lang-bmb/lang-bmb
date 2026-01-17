//! MIR Optimization Passes
//!
//! This module provides optimization passes that transform MIR programs
//! to improve performance. Optimizations are organized into passes that
//! can be composed and run in sequence.
//!
//! # Optimization Levels
//!
//! - **Debug**: No optimizations (preserves debugging)
//! - **Release**: Standard optimizations (DCE, constant folding, inlining)
//! - **Aggressive**: All optimizations including contract-based
//!
//! # Contract-Based Optimizations (BMB-specific)
//!
//! BMB's contract system enables unique optimizations:
//! - **Bounds Check Elimination**: `pre` conditions prove array bounds
//! - **Null Check Elimination**: `Option<T>` + contracts eliminate null checks
//! - **Purity-Based CSE**: `post` conditions enable aggressive CSE
//! - **Alias Analysis**: Ownership proves non-aliasing for SIMD

use std::collections::{HashMap, HashSet};

use super::{
    CmpOp, Constant, ContractFact, MirBinOp, MirFunction, MirInst, MirProgram, MirUnaryOp,
    Operand, Place, Terminator,
};

/// Optimization pass trait
pub trait OptimizationPass {
    /// Name of the optimization pass
    fn name(&self) -> &'static str;

    /// Run the optimization pass on a function
    /// Returns true if any changes were made
    fn run_on_function(&self, func: &mut MirFunction) -> bool;
}

/// Optimization pipeline
pub struct OptimizationPipeline {
    passes: Vec<Box<dyn OptimizationPass>>,
    max_iterations: usize,
}

impl OptimizationPipeline {
    /// Create a new optimization pipeline
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            max_iterations: 10,
        }
    }

    /// Create pipeline for the given optimization level
    pub fn for_level(level: OptLevel) -> Self {
        let mut pipeline = Self::new();

        match level {
            OptLevel::Debug => {
                // No optimizations in debug mode
            }
            OptLevel::Release => {
                // Standard optimizations
                pipeline.add_pass(Box::new(ConstantFolding));
                pipeline.add_pass(Box::new(DeadCodeElimination));
                pipeline.add_pass(Box::new(SimplifyBranches));
                pipeline.add_pass(Box::new(CopyPropagation));
            }
            OptLevel::Aggressive => {
                // All optimizations
                pipeline.add_pass(Box::new(ConstantFolding));
                pipeline.add_pass(Box::new(DeadCodeElimination));
                pipeline.add_pass(Box::new(SimplifyBranches));
                pipeline.add_pass(Box::new(CopyPropagation));
                pipeline.add_pass(Box::new(CommonSubexpressionElimination));
                pipeline.add_pass(Box::new(ContractBasedOptimization));
                pipeline.add_pass(Box::new(ContractUnreachableElimination));
            }
        }

        pipeline
    }

    /// Add an optimization pass
    pub fn add_pass(&mut self, pass: Box<dyn OptimizationPass>) {
        self.passes.push(pass);
    }

    /// Set maximum iterations for fixed-point optimization
    pub fn set_max_iterations(&mut self, n: usize) {
        self.max_iterations = n;
    }

    /// Run all passes on a program
    pub fn optimize(&self, program: &mut MirProgram) -> OptimizationStats {
        let mut stats = OptimizationStats::new();

        // v0.38.3: Create PureFunctionCSE pass with program-level information
        let pure_cse = PureFunctionCSE::from_program(program);

        // v0.38.4: Create ConstFunctionEval pass with program-level information
        let const_eval = ConstFunctionEval::from_program(program);

        for func in &mut program.functions {
            let func_stats = self.optimize_function_with_program_passes(func, &pure_cse, &const_eval);
            stats.merge(&func_stats);
        }

        stats
    }

    /// Run all passes on a single function until fixed point (with program-level passes)
    fn optimize_function_with_program_passes(
        &self,
        func: &mut MirFunction,
        pure_cse: &PureFunctionCSE,
        const_eval: &ConstFunctionEval,
    ) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        let mut iteration = 0;

        loop {
            let mut changed = false;
            iteration += 1;

            // Run standard passes
            for pass in &self.passes {
                if pass.run_on_function(func) {
                    changed = true;
                    stats.record_pass(pass.name());
                }
            }

            // v0.38.3: Run pure function CSE
            if pure_cse.run_on_function(func) {
                changed = true;
                stats.record_pass(pure_cse.name());
            }

            // v0.38.4: Run const function evaluation
            if const_eval.run_on_function(func) {
                changed = true;
                stats.record_pass(const_eval.name());
            }

            if !changed || iteration >= self.max_iterations {
                break;
            }
        }

        stats.iterations = iteration;
        stats
    }
}

impl Default for OptimizationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization level
#[derive(Debug, Clone, Copy, Default)]
pub enum OptLevel {
    #[default]
    Debug,
    Release,
    Aggressive,
}

/// Statistics from optimization passes
#[derive(Debug, Default)]
pub struct OptimizationStats {
    /// Number of iterations run
    pub iterations: usize,
    /// Pass execution counts
    pub pass_counts: HashMap<String, usize>,
}

impl OptimizationStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_pass(&mut self, name: &str) {
        *self.pass_counts.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn merge(&mut self, other: &OptimizationStats) {
        for (name, count) in &other.pass_counts {
            *self.pass_counts.entry(name.clone()).or_insert(0) += count;
        }
    }
}

// ============================================================================
// Constant Folding Pass
// ============================================================================

/// Constant folding: evaluate constant expressions at compile time
pub struct ConstantFolding;

impl OptimizationPass for ConstantFolding {
    fn name(&self) -> &'static str {
        "constant_folding"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;
        let mut constants: HashMap<String, Constant> = HashMap::new();

        for block in &mut func.blocks {
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                match inst {
                    MirInst::Const { dest, value } => {
                        constants.insert(dest.name.clone(), value.clone());
                        new_instructions.push(inst.clone());
                    }
                    MirInst::BinOp { dest, op, lhs, rhs } => {
                        if let (Some(lhs_const), Some(rhs_const)) =
                            (get_constant(lhs, &constants), get_constant(rhs, &constants))
                            && let Some(result) = fold_binop(*op, &lhs_const, &rhs_const)
                        {
                            constants.insert(dest.name.clone(), result.clone());
                            new_instructions.push(MirInst::Const {
                                dest: dest.clone(),
                                value: result,
                            });
                            changed = true;
                            continue;
                        }
                        new_instructions.push(inst.clone());
                    }
                    MirInst::UnaryOp { dest, op, src } => {
                        if let Some(src_const) = get_constant(src, &constants)
                            && let Some(result) = fold_unaryop(*op, &src_const)
                        {
                            constants.insert(dest.name.clone(), result.clone());
                            new_instructions.push(MirInst::Const {
                                dest: dest.clone(),
                                value: result,
                            });
                            changed = true;
                            continue;
                        }
                        new_instructions.push(inst.clone());
                    }
                    MirInst::Copy { dest, src } => {
                        if let Some(value) = constants.get(&src.name) {
                            constants.insert(dest.name.clone(), value.clone());
                        }
                        new_instructions.push(inst.clone());
                    }
                    _ => {
                        new_instructions.push(inst.clone());
                    }
                }
            }

            block.instructions = new_instructions;
        }

        changed
    }
}

fn get_constant(operand: &Operand, constants: &HashMap<String, Constant>) -> Option<Constant> {
    match operand {
        Operand::Constant(c) => Some(c.clone()),
        Operand::Place(p) => constants.get(&p.name).cloned(),
    }
}

fn fold_binop(op: MirBinOp, lhs: &Constant, rhs: &Constant) -> Option<Constant> {
    match (op, lhs, rhs) {
        // Integer arithmetic
        (MirBinOp::Add, Constant::Int(a), Constant::Int(b)) => Some(Constant::Int(a + b)),
        (MirBinOp::Sub, Constant::Int(a), Constant::Int(b)) => Some(Constant::Int(a - b)),
        (MirBinOp::Mul, Constant::Int(a), Constant::Int(b)) => Some(Constant::Int(a * b)),
        (MirBinOp::Div, Constant::Int(a), Constant::Int(b)) if *b != 0 => {
            Some(Constant::Int(a / b))
        }
        (MirBinOp::Mod, Constant::Int(a), Constant::Int(b)) if *b != 0 => {
            Some(Constant::Int(a % b))
        }

        // Integer comparison
        (MirBinOp::Eq, Constant::Int(a), Constant::Int(b)) => Some(Constant::Bool(a == b)),
        (MirBinOp::Ne, Constant::Int(a), Constant::Int(b)) => Some(Constant::Bool(a != b)),
        (MirBinOp::Lt, Constant::Int(a), Constant::Int(b)) => Some(Constant::Bool(a < b)),
        (MirBinOp::Le, Constant::Int(a), Constant::Int(b)) => Some(Constant::Bool(a <= b)),
        (MirBinOp::Gt, Constant::Int(a), Constant::Int(b)) => Some(Constant::Bool(a > b)),
        (MirBinOp::Ge, Constant::Int(a), Constant::Int(b)) => Some(Constant::Bool(a >= b)),

        // Boolean operations
        (MirBinOp::And, Constant::Bool(a), Constant::Bool(b)) => Some(Constant::Bool(*a && *b)),
        (MirBinOp::Or, Constant::Bool(a), Constant::Bool(b)) => Some(Constant::Bool(*a || *b)),

        // Float arithmetic
        (MirBinOp::FAdd, Constant::Float(a), Constant::Float(b)) => Some(Constant::Float(a + b)),
        (MirBinOp::FSub, Constant::Float(a), Constant::Float(b)) => Some(Constant::Float(a - b)),
        (MirBinOp::FMul, Constant::Float(a), Constant::Float(b)) => Some(Constant::Float(a * b)),
        (MirBinOp::FDiv, Constant::Float(a), Constant::Float(b)) if *b != 0.0 => {
            Some(Constant::Float(a / b))
        }

        _ => None,
    }
}

fn fold_unaryop(op: MirUnaryOp, src: &Constant) -> Option<Constant> {
    match (op, src) {
        (MirUnaryOp::Neg, Constant::Int(n)) => Some(Constant::Int(-n)),
        (MirUnaryOp::FNeg, Constant::Float(f)) => Some(Constant::Float(-f)),
        (MirUnaryOp::Not, Constant::Bool(b)) => Some(Constant::Bool(!b)),
        _ => None,
    }
}

// ============================================================================
// Dead Code Elimination Pass
// ============================================================================

/// Dead code elimination: remove unused definitions
pub struct DeadCodeElimination;

impl OptimizationPass for DeadCodeElimination {
    fn name(&self) -> &'static str {
        "dead_code_elimination"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Collect all used variables
        let mut used: HashSet<String> = HashSet::new();

        // Mark variables used in terminators
        for block in &func.blocks {
            collect_used_in_terminator(&block.terminator, &mut used);
        }

        // Mark variables used in instructions (backwards)
        for block in &func.blocks {
            for inst in block.instructions.iter().rev() {
                collect_used_in_instruction(inst, &mut used);
            }
        }

        // Remove dead instructions
        for block in &mut func.blocks {
            let original_len = block.instructions.len();
            block.instructions.retain(|inst| {
                if let Some(dest) = get_inst_dest(inst) {
                    // Keep if result is used or has side effects
                    used.contains(&dest.name) || has_side_effects(inst)
                } else {
                    // Keep instructions without destinations (calls, stores)
                    true
                }
            });
            if block.instructions.len() != original_len {
                changed = true;
            }
        }

        changed
    }
}

fn collect_used_in_terminator(term: &Terminator, used: &mut HashSet<String>) {
    match term {
        Terminator::Return(Some(op)) => collect_used_in_operand(op, used),
        Terminator::Branch { cond, .. } => collect_used_in_operand(cond, used),
        Terminator::Switch { discriminant, .. } => collect_used_in_operand(discriminant, used),
        _ => {}
    }
}

fn collect_used_in_instruction(inst: &MirInst, used: &mut HashSet<String>) {
    match inst {
        MirInst::Const { .. } => {}
        MirInst::Copy { src, .. } => {
            used.insert(src.name.clone());
        }
        MirInst::BinOp { lhs, rhs, .. } => {
            collect_used_in_operand(lhs, used);
            collect_used_in_operand(rhs, used);
        }
        MirInst::UnaryOp { src, .. } => {
            collect_used_in_operand(src, used);
        }
        MirInst::Call { args, .. } => {
            for arg in args {
                collect_used_in_operand(arg, used);
            }
        }
        MirInst::Phi { values, .. } => {
            for (op, _) in values {
                collect_used_in_operand(op, used);
            }
        }
        MirInst::StructInit { fields, .. } => {
            for (_, val) in fields {
                collect_used_in_operand(val, used);
            }
        }
        MirInst::FieldAccess { base, .. } => {
            used.insert(base.name.clone());
        }
        MirInst::FieldStore { base, value, .. } => {
            used.insert(base.name.clone());
            collect_used_in_operand(value, used);
        }
        MirInst::EnumVariant { args, .. } => {
            for arg in args {
                collect_used_in_operand(arg, used);
            }
        }
        MirInst::ArrayInit { elements, .. } => {
            for elem in elements {
                collect_used_in_operand(elem, used);
            }
        }
        MirInst::IndexLoad { array, index, .. } => {
            used.insert(array.name.clone());
            collect_used_in_operand(index, used);
        }
        MirInst::IndexStore { array, index, value } => {
            used.insert(array.name.clone());
            collect_used_in_operand(index, used);
            collect_used_in_operand(value, used);
        }
    }
}

fn collect_used_in_operand(op: &Operand, used: &mut HashSet<String>) {
    if let Operand::Place(p) = op {
        used.insert(p.name.clone());
    }
}

fn get_inst_dest(inst: &MirInst) -> Option<&Place> {
    match inst {
        MirInst::Const { dest, .. } => Some(dest),
        MirInst::Copy { dest, .. } => Some(dest),
        MirInst::BinOp { dest, .. } => Some(dest),
        MirInst::UnaryOp { dest, .. } => Some(dest),
        MirInst::Call { dest, .. } => dest.as_ref(),
        MirInst::Phi { dest, .. } => Some(dest),
        MirInst::StructInit { dest, .. } => Some(dest),
        MirInst::FieldAccess { dest, .. } => Some(dest),
        MirInst::EnumVariant { dest, .. } => Some(dest),
        MirInst::ArrayInit { dest, .. } => Some(dest),
        MirInst::IndexLoad { dest, .. } => Some(dest),
        _ => None,
    }
}

fn has_side_effects(inst: &MirInst) -> bool {
    matches!(
        inst,
        MirInst::Call { .. } | MirInst::FieldStore { .. } | MirInst::IndexStore { .. }
    )
}

// ============================================================================
// Simplify Branches Pass
// ============================================================================

/// Simplify branches: eliminate branches with constant conditions
pub struct SimplifyBranches;

impl OptimizationPass for SimplifyBranches {
    fn name(&self) -> &'static str {
        "simplify_branches"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        for block in &mut func.blocks {
            if let Terminator::Branch {
                cond,
                then_label,
                else_label,
            } = &block.terminator
                && let Operand::Constant(Constant::Bool(b)) = cond
            {
                let target = if *b {
                    then_label.clone()
                } else {
                    else_label.clone()
                };
                block.terminator = Terminator::Goto(target);
                changed = true;
            }
        }

        changed
    }
}

// ============================================================================
// Copy Propagation Pass
// ============================================================================

/// Copy propagation: replace copies with original values
pub struct CopyPropagation;

impl OptimizationPass for CopyPropagation {
    fn name(&self) -> &'static str {
        "copy_propagation"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;
        let mut copies: HashMap<String, Place> = HashMap::new();

        for block in &mut func.blocks {
            // Build copy map
            for inst in &block.instructions {
                if let MirInst::Copy { dest, src } = inst {
                    copies.insert(dest.name.clone(), src.clone());
                }
            }

            // Propagate copies
            for inst in &mut block.instructions {
                if propagate_copies_in_inst(inst, &copies) {
                    changed = true;
                }
            }

            if propagate_copies_in_term(&mut block.terminator, &copies) {
                changed = true;
            }
        }

        changed
    }
}

fn propagate_copies_in_inst(inst: &mut MirInst, copies: &HashMap<String, Place>) -> bool {
    let mut changed = false;

    match inst {
        MirInst::BinOp { lhs, rhs, .. } => {
            if propagate_operand(lhs, copies) {
                changed = true;
            }
            if propagate_operand(rhs, copies) {
                changed = true;
            }
        }
        MirInst::UnaryOp { src, .. } => {
            if propagate_operand(src, copies) {
                changed = true;
            }
        }
        MirInst::Call { args, .. } => {
            for arg in args {
                if propagate_operand(arg, copies) {
                    changed = true;
                }
            }
        }
        _ => {}
    }

    changed
}

fn propagate_copies_in_term(term: &mut Terminator, copies: &HashMap<String, Place>) -> bool {
    match term {
        Terminator::Return(Some(op)) => propagate_operand(op, copies),
        Terminator::Branch { cond, .. } => propagate_operand(cond, copies),
        Terminator::Switch { discriminant, .. } => propagate_operand(discriminant, copies),
        _ => false,
    }
}

fn propagate_operand(op: &mut Operand, copies: &HashMap<String, Place>) -> bool {
    if let Operand::Place(p) = op
        && let Some(src) = copies.get(&p.name)
    {
        *p = src.clone();
        return true;
    }
    false
}

// ============================================================================
// Common Subexpression Elimination Pass
// ============================================================================

/// Common subexpression elimination: reuse computed values
pub struct CommonSubexpressionElimination;

impl OptimizationPass for CommonSubexpressionElimination {
    fn name(&self) -> &'static str {
        "common_subexpression_elimination"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;
        let mut expressions: HashMap<String, Place> = HashMap::new();

        for block in &mut func.blocks {
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                if let MirInst::BinOp { dest, op, lhs, rhs } = inst {
                    let key = format!("{:?}:{:?}:{:?}", op, lhs, rhs);

                    if let Some(existing) = expressions.get(&key) {
                        // Replace with copy
                        new_instructions.push(MirInst::Copy {
                            dest: dest.clone(),
                            src: existing.clone(),
                        });
                        changed = true;
                    } else {
                        expressions.insert(key, dest.clone());
                        new_instructions.push(inst.clone());
                    }
                } else {
                    new_instructions.push(inst.clone());
                }
            }

            block.instructions = new_instructions;
        }

        changed
    }
}

// ============================================================================
// Pure Function CSE Pass (v0.38.3)
// ============================================================================

/// Common subexpression elimination for @pure function calls
///
/// Pure functions have no side effects and always return the same result
/// for the same inputs. This allows us to eliminate duplicate calls.
///
/// Example:
/// ```text
/// @pure fn square(x: i64) -> i64 = x * x;
///
/// fn example(n: i64) -> i64 = square(n) + square(n); // second call eliminated
/// ```
pub struct PureFunctionCSE {
    /// Set of function names marked @pure
    pure_functions: HashSet<String>,
}

impl PureFunctionCSE {
    /// Create a new PureFunctionCSE pass with the given pure function set
    pub fn new(pure_functions: HashSet<String>) -> Self {
        Self { pure_functions }
    }

    /// Create from a MirProgram by collecting all @pure functions
    pub fn from_program(program: &MirProgram) -> Self {
        let pure_functions: HashSet<String> = program
            .functions
            .iter()
            .filter(|f| f.is_pure || f.is_const) // @const implies @pure
            .map(|f| f.name.clone())
            .collect();
        Self { pure_functions }
    }
}

impl OptimizationPass for PureFunctionCSE {
    fn name(&self) -> &'static str {
        "pure_function_cse"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;
        // Map from (func_name, args...) -> result place
        let mut call_results: HashMap<String, Place> = HashMap::new();

        for block in &mut func.blocks {
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                if let MirInst::Call { dest: Some(dest), func: called_func, args } = inst {
                    // Only optimize if the called function is pure
                    if self.pure_functions.contains(called_func) {
                        // Create a key from function name and arguments
                        let key = format!("call:{}:{:?}", called_func, args);

                        if let Some(existing) = call_results.get(&key) {
                            // Replace with copy from previous result
                            new_instructions.push(MirInst::Copy {
                                dest: dest.clone(),
                                src: existing.clone(),
                            });
                            changed = true;
                            continue;
                        } else {
                            // First call - record the result
                            call_results.insert(key, dest.clone());
                        }
                    }
                }
                new_instructions.push(inst.clone());
            }

            block.instructions = new_instructions;
        }

        changed
    }
}

// ============================================================================
// Const Function Evaluation Pass (v0.38.4)
// ============================================================================

/// Compile-time evaluation of @const function calls
///
/// @const functions are a superset of @pure functions. When a @const function
/// returns a constant value and is called with constant arguments, the call
/// can be replaced with the constant result at compile time.
///
/// This is a simplified implementation that handles:
/// 1. @const functions that just return a constant
/// 2. @const functions with no parameters
///
/// Full compile-time evaluation (interpreting function bodies) is deferred
/// to future enhancements.
pub struct ConstFunctionEval {
    /// Map of const function name -> constant return value (if simple)
    const_values: HashMap<String, Constant>,
}

impl ConstFunctionEval {
    /// Create from a MirProgram by analyzing @const functions
    pub fn from_program(program: &MirProgram) -> Self {
        let mut const_values = HashMap::new();

        for func in &program.functions {
            if func.is_const && func.params.is_empty() {
                // Check if function body is a simple constant return
                if let Some(value) = Self::extract_constant_return(func) {
                    const_values.insert(func.name.clone(), value);
                }
            }
        }

        Self { const_values }
    }

    /// Try to extract a constant return value from a simple @const function
    fn extract_constant_return(func: &MirFunction) -> Option<Constant> {
        // Must have exactly one block
        if func.blocks.len() != 1 {
            return None;
        }

        let block = &func.blocks[0];

        // Check if it's a direct return of a constant
        if let Terminator::Return(Some(Operand::Constant(c))) = &block.terminator {
            return Some(c.clone());
        }

        // Check if it's a return of a variable that was set to a constant
        if let Terminator::Return(Some(Operand::Place(place))) = &block.terminator {
            // Look for const assignment to this place
            for inst in &block.instructions {
                if let MirInst::Const { dest, value } = inst
                    && dest.name == place.name
                {
                    return Some(value.clone());
                }
            }
        }

        None
    }
}

impl OptimizationPass for ConstFunctionEval {
    fn name(&self) -> &'static str {
        "const_function_eval"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        for block in &mut func.blocks {
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                if let MirInst::Call { dest: Some(dest), func: called_func, args } = inst {
                    // Only evaluate if function is known const and has no args
                    if args.is_empty()
                        && let Some(value) = self.const_values.get(called_func)
                    {
                        // Replace call with constant
                        new_instructions.push(MirInst::Const {
                            dest: dest.clone(),
                            value: value.clone(),
                        });
                        changed = true;
                        continue;
                    }
                }
                new_instructions.push(inst.clone());
            }

            block.instructions = new_instructions;
        }

        changed
    }
}

// ============================================================================
// Contract-Based Optimization Pass (BMB-specific)
// ============================================================================

/// Contract-based optimizations unique to BMB (v0.38)
///
/// These optimizations leverage BMB's contract system:
/// - Bounds check elimination based on `pre` conditions
/// - Null check elimination with `Option<T>` contracts
/// - Purity-based CSE using `post` conditions
/// - Unreachable branch elimination using `post` conditions
pub struct ContractBasedOptimization;

impl OptimizationPass for ContractBasedOptimization {
    fn name(&self) -> &'static str {
        "contract_based_optimization"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Build set of proven facts from preconditions
        let proven_facts = ProvenFacts::from_preconditions(&func.preconditions);

        // Phase 1: Eliminate redundant comparisons based on proven facts
        for block in &mut func.blocks {
            for inst in &mut block.instructions {
                if self.try_eliminate_redundant_check(inst, &proven_facts) {
                    changed = true;
                }
            }

            // Phase 2: Simplify branches based on proven facts
            if self.try_simplify_branch(&mut block.terminator, &proven_facts) {
                changed = true;
            }
        }

        changed
    }
}

impl ContractBasedOptimization {
    /// Try to eliminate redundant checks based on proven facts
    /// Returns true if the instruction was modified
    fn try_eliminate_redundant_check(&self, inst: &mut MirInst, facts: &ProvenFacts) -> bool {
        // First, extract info without borrowing inst mutably
        let replacement = match inst {
            MirInst::BinOp { dest, op, lhs, rhs } => {
                let cmp_op = match op {
                    MirBinOp::Lt => CmpOp::Lt,
                    MirBinOp::Le => CmpOp::Le,
                    MirBinOp::Gt => CmpOp::Gt,
                    MirBinOp::Ge => CmpOp::Ge,
                    _ => return false,
                };

                // Check if this comparison is implied by preconditions
                facts.evaluate_comparison(lhs, cmp_op, rhs).map(|result| MirInst::Const {
                        dest: dest.clone(),
                        value: Constant::Bool(result),
                    })
            }
            _ => None,
        };

        // Apply replacement if found
        if let Some(new_inst) = replacement {
            *inst = new_inst;
            true
        } else {
            false
        }
    }

    /// Try to simplify branches based on proven facts
    fn try_simplify_branch(&self, term: &mut Terminator, facts: &ProvenFacts) -> bool {
        if let Terminator::Branch { cond, then_label, else_label } = term {
            // If condition is a known-true/false variable, simplify to unconditional
            if let Operand::Place(place) = cond
                && let Some(value) = facts.get_bool_value(&place.name) {
                    let target = if value {
                        then_label.clone()
                    } else {
                        else_label.clone()
                    };
                    *term = Terminator::Goto(target);
                    return true;
                }
        }
        false
    }
}

/// Proven facts from preconditions, used for optimization
struct ProvenFacts {
    /// Variable bounds: var -> (lower_bound, upper_bound) where bounds are Option<i64>
    var_bounds: HashMap<String, (Option<i64>, Option<i64>)>,
    /// Variable-variable relationships
    var_relations: Vec<ContractFact>,
    /// Known boolean values
    bool_values: HashMap<String, bool>,
}

impl ProvenFacts {
    /// Build proven facts from a list of preconditions
    fn from_preconditions(preconditions: &[ContractFact]) -> Self {
        let mut facts = ProvenFacts {
            var_bounds: HashMap::new(),
            var_relations: Vec::new(),
            bool_values: HashMap::new(),
        };

        for fact in preconditions {
            match fact {
                ContractFact::VarCmp { var, op, value } => {
                    let entry = facts.var_bounds.entry(var.clone()).or_insert((None, None));
                    match op {
                        CmpOp::Ge => {
                            // x >= value means lower bound is value
                            entry.0 = Some(entry.0.map_or(*value, |v| v.max(*value)));
                        }
                        CmpOp::Gt => {
                            // x > value means lower bound is value + 1
                            entry.0 = Some(entry.0.map_or(value + 1, |v| v.max(value + 1)));
                        }
                        CmpOp::Le => {
                            // x <= value means upper bound is value
                            entry.1 = Some(entry.1.map_or(*value, |v| v.min(*value)));
                        }
                        CmpOp::Lt => {
                            // x < value means upper bound is value - 1
                            entry.1 = Some(entry.1.map_or(value - 1, |v| v.min(value - 1)));
                        }
                        CmpOp::Eq => {
                            // x == value means both bounds are value
                            entry.0 = Some(*value);
                            entry.1 = Some(*value);
                        }
                        _ => {}
                    }
                }
                ContractFact::VarVarCmp { .. } | ContractFact::ArrayBounds { .. } => {
                    facts.var_relations.push(fact.clone());
                }
                ContractFact::NonNull { var } => {
                    // NonNull doesn't directly affect numeric bounds
                    // but could be used for null check elimination
                    facts.bool_values.insert(format!("{}_is_null", var), false);
                }
            }
        }

        facts
    }

    /// Evaluate a comparison given proven facts
    /// Returns Some(true/false) if the result is known, None otherwise
    fn evaluate_comparison(&self, lhs: &Operand, op: CmpOp, rhs: &Operand) -> Option<bool> {
        // Pattern: var op constant
        if let (Operand::Place(lhs_place), Operand::Constant(Constant::Int(rhs_val))) = (lhs, rhs)
            && let Some((lower, upper)) = self.var_bounds.get(&lhs_place.name) {
                return self.check_bounds(*lower, *upper, op, *rhs_val);
            }

        // Pattern: constant op var
        if let (Operand::Constant(Constant::Int(lhs_val)), Operand::Place(rhs_place)) = (lhs, rhs)
            && let Some((lower, upper)) = self.var_bounds.get(&rhs_place.name) {
                // Flip the comparison: c op x becomes x flipped_op c
                let flipped_op = match op {
                    CmpOp::Lt => CmpOp::Gt,
                    CmpOp::Le => CmpOp::Ge,
                    CmpOp::Gt => CmpOp::Lt,
                    CmpOp::Ge => CmpOp::Le,
                    other => other,
                };
                return self.check_bounds(*lower, *upper, flipped_op, *lhs_val);
            }

        None
    }

    /// Check if a comparison is always true/false given bounds
    fn check_bounds(&self, lower: Option<i64>, upper: Option<i64>, op: CmpOp, value: i64) -> Option<bool> {
        match op {
            CmpOp::Ge => {
                // x >= value: true if lower >= value
                if let Some(l) = lower
                    && l >= value {
                        return Some(true);
                    }
                // false if upper < value
                if let Some(u) = upper
                    && u < value {
                        return Some(false);
                    }
            }
            CmpOp::Gt => {
                // x > value: true if lower > value
                if let Some(l) = lower
                    && l > value {
                        return Some(true);
                    }
                // false if upper <= value
                if let Some(u) = upper
                    && u <= value {
                        return Some(false);
                    }
            }
            CmpOp::Le => {
                // x <= value: true if upper <= value
                if let Some(u) = upper
                    && u <= value {
                        return Some(true);
                    }
                // false if lower > value
                if let Some(l) = lower
                    && l > value {
                        return Some(false);
                    }
            }
            CmpOp::Lt => {
                // x < value: true if upper < value
                if let Some(u) = upper
                    && u < value {
                        return Some(true);
                    }
                // false if lower >= value
                if let Some(l) = lower
                    && l >= value {
                        return Some(false);
                    }
            }
            _ => {}
        }
        None
    }

    /// Get a known boolean value for a variable
    fn get_bool_value(&self, var: &str) -> Option<bool> {
        self.bool_values.get(var).copied()
    }
}

// ============================================================================
// Contract-Driven Unreachable Code Elimination (v0.38.0.2)
// ============================================================================

/// Contract-driven unreachable code elimination
///
/// This optimization removes blocks that are provably unreachable based on
/// contract facts (preconditions and postconditions). It works by:
///
/// 1. Building proven facts from preconditions
/// 2. Propagating facts through the CFG
/// 3. Identifying branches where one arm is provably never taken
/// 4. Removing unreachable blocks
pub struct ContractUnreachableElimination;

impl OptimizationPass for ContractUnreachableElimination {
    fn name(&self) -> &'static str {
        "contract_unreachable_elimination"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Build proven facts from preconditions
        let proven_facts = ProvenFacts::from_preconditions(&func.preconditions);

        // First pass: mark unreachable branches as unconditional jumps
        let mut unreachable_labels: HashSet<String> = HashSet::new();

        for block in &mut func.blocks {
            if let Terminator::Branch { cond, then_label, else_label } = &block.terminator {
                // Try to evaluate the branch condition based on proven facts
                if let Some(always_true) = self.evaluate_branch_condition(cond, &proven_facts, &block.instructions) {
                    let (target, dead) = if always_true {
                        (then_label.clone(), else_label.clone())
                    } else {
                        (else_label.clone(), then_label.clone())
                    };
                    block.terminator = Terminator::Goto(target);
                    unreachable_labels.insert(dead);
                    changed = true;
                }
            }
        }

        // Second pass: find all reachable blocks (starting from entry)
        let reachable = self.find_reachable_blocks(func);

        // Third pass: remove unreachable blocks
        let original_len = func.blocks.len();
        func.blocks.retain(|block| reachable.contains(&block.label));
        if func.blocks.len() != original_len {
            changed = true;
        }

        changed
    }
}

impl ContractUnreachableElimination {
    /// Evaluate a branch condition based on proven facts and local definitions
    fn evaluate_branch_condition(
        &self,
        cond: &Operand,
        facts: &ProvenFacts,
        instructions: &[MirInst],
    ) -> Option<bool> {
        // Case 1: condition is a constant
        if let Operand::Constant(Constant::Bool(b)) = cond {
            return Some(*b);
        }

        // Case 2: condition is a variable with a known value
        if let Operand::Place(place) = cond {
            // Check if we have a known bool value
            if let Some(value) = facts.get_bool_value(&place.name) {
                return Some(value);
            }

            // Check if the variable was defined as a constant in this block
            for inst in instructions.iter().rev() {
                match inst {
                    MirInst::Const { dest, value: Constant::Bool(b) }
                        if dest.name == place.name =>
                    {
                        return Some(*b);
                    }
                    // Check for comparison result that we can evaluate
                    MirInst::BinOp { dest, op, lhs, rhs }
                        if dest.name == place.name =>
                    {
                        let cmp_op = match op {
                            MirBinOp::Lt => CmpOp::Lt,
                            MirBinOp::Le => CmpOp::Le,
                            MirBinOp::Gt => CmpOp::Gt,
                            MirBinOp::Ge => CmpOp::Ge,
                            MirBinOp::Eq => CmpOp::Eq,
                            MirBinOp::Ne => CmpOp::Ne,
                            _ => return None,
                        };
                        return facts.evaluate_comparison(lhs, cmp_op, rhs);
                    }
                    _ => {}
                }
            }
        }

        None
    }

    /// Find all reachable blocks starting from entry
    fn find_reachable_blocks(&self, func: &MirFunction) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut worklist = Vec::new();

        // Start from entry block (first block)
        if let Some(entry) = func.blocks.first() {
            worklist.push(entry.label.clone());
        }

        while let Some(label) = worklist.pop() {
            if reachable.contains(&label) {
                continue;
            }
            reachable.insert(label.clone());

            // Find the block and get its successors
            if let Some(block) = func.blocks.iter().find(|b| b.label == label) {
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
                    Terminator::Return(_) | Terminator::Unreachable => {
                        // No successors
                    }
                }
            }
        }

        reachable
    }
}

// ============================================================================
// Module Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{BasicBlock, MirType};

    fn make_test_function() -> MirFunction {
        MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const {
                        dest: Place::new("a"),
                        value: Constant::Int(5),
                    },
                    MirInst::Const {
                        dest: Place::new("b"),
                        value: Constant::Int(3),
                    },
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("a")),
                        rhs: Operand::Place(Place::new("b")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("c")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        }
    }

    #[test]
    fn test_constant_folding() {
        let mut func = make_test_function();
        let pass = ConstantFolding;

        let changed = pass.run_on_function(&mut func);
        assert!(changed);

        // The add should be folded to a constant
        let last_inst = &func.blocks[0].instructions[2];
        assert!(matches!(last_inst, MirInst::Const { value: Constant::Int(8), .. }));
    }

    #[test]
    fn test_dead_code_elimination() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const {
                        dest: Place::new("unused"),
                        value: Constant::Int(42),
                    },
                    MirInst::Const {
                        dest: Place::new("result"),
                        value: Constant::Int(1),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        let pass = DeadCodeElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);

        // The unused constant should be removed
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn test_optimization_pipeline() {
        let mut program = MirProgram {
            functions: vec![make_test_function()],
            extern_fns: vec![],
        };

        let pipeline = OptimizationPipeline::for_level(OptLevel::Release);
        let stats = pipeline.optimize(&mut program);

        assert!(stats.pass_counts.contains_key("constant_folding"));
    }

    #[test]
    fn test_contract_based_optimization() {
        // Test: precondition "x >= 0" should eliminate "x >= 0" check
        let mut func = MirFunction {
            name: "test_bounds".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::Bool,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    // %cmp = x >= 0  (should be eliminated to true)
                    MirInst::BinOp {
                        dest: Place::new("cmp"),
                        op: MirBinOp::Ge,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(0)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("cmp")))),
            }],
            preconditions: vec![
                ContractFact::VarCmp {
                    var: "x".to_string(),
                    op: CmpOp::Ge,
                    value: 0,
                },
            ],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        let pass = ContractBasedOptimization;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Contract-based optimization should have made changes");

        // The comparison should be replaced with constant true
        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::Const { value: Constant::Bool(true), .. }),
            "x >= 0 should be optimized to true when precondition is x >= 0"
        );
    }

    #[test]
    fn test_contract_bounds_elimination() {
        // Test: precondition "x >= 5" should prove "x >= 3" is always true
        let mut func = MirFunction {
            name: "test_bounds2".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::Bool,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("cmp"),
                        op: MirBinOp::Ge,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(3)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("cmp")))),
            }],
            preconditions: vec![
                ContractFact::VarCmp {
                    var: "x".to_string(),
                    op: CmpOp::Ge,
                    value: 5,  // x >= 5 implies x >= 3
                },
            ],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        let pass = ContractBasedOptimization;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);

        let inst = &func.blocks[0].instructions[0];
        assert!(matches!(inst, MirInst::Const { value: Constant::Bool(true), .. }));
    }

    #[test]
    fn test_contract_unreachable_elimination() {
        // Test: precondition "x >= 0" should eliminate branch to negative case
        // if x >= 0 then goto positive else goto negative
        // The negative block should be removed since x >= 0 is always true
        let mut func = MirFunction {
            name: "test_unreachable".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Ge,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "positive".to_string(),
                        else_label: "negative".to_string(),
                    },
                },
                BasicBlock {
                    label: "positive".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
                },
                BasicBlock {
                    label: "negative".to_string(),
                    instructions: vec![
                        MirInst::UnaryOp {
                            dest: Place::new("neg_x"),
                            op: MirUnaryOp::Neg,
                            src: Operand::Place(Place::new("x")),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("neg_x")))),
                },
            ],
            preconditions: vec![
                ContractFact::VarCmp {
                    var: "x".to_string(),
                    op: CmpOp::Ge,
                    value: 0,
                },
            ],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        let pass = ContractUnreachableElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Unreachable elimination should have made changes");

        // The negative block should be removed
        assert_eq!(func.blocks.len(), 2, "Should have 2 blocks (entry + positive)");
        assert!(
            !func.blocks.iter().any(|b| b.label == "negative"),
            "Negative block should be removed"
        );

        // The entry block should now have Goto instead of Branch
        assert!(
            matches!(func.blocks[0].terminator, Terminator::Goto(_)),
            "Entry terminator should be Goto"
        );
    }

    #[test]
    fn test_contract_unreachable_keeps_both_branches() {
        // Test: when no precondition, both branches should be kept
        let mut func = MirFunction {
            name: "test_both_reachable".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Ge,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "positive".to_string(),
                        else_label: "negative".to_string(),
                    },
                },
                BasicBlock {
                    label: "positive".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
                },
                BasicBlock {
                    label: "negative".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
                },
            ],
            preconditions: vec![], // No preconditions
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        let pass = ContractUnreachableElimination;
        let changed = pass.run_on_function(&mut func);

        // No changes should be made - both branches are reachable
        assert!(!changed, "Should not make changes without preconditions");
        assert_eq!(func.blocks.len(), 3, "All blocks should be kept");
    }

    #[test]
    fn test_contract_unreachable_constant_condition() {
        // Test: constant true condition should eliminate else branch
        let mut func = MirFunction {
            name: "test_const_cond".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const {
                            dest: Place::new("always_true"),
                            value: Constant::Bool(true),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("always_true")),
                        then_label: "taken".to_string(),
                        else_label: "dead".to_string(),
                    },
                },
                BasicBlock {
                    label: "taken".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                },
                BasicBlock {
                    label: "dead".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
                },
            ],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        let pass = ContractUnreachableElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should eliminate dead branch");

        assert_eq!(func.blocks.len(), 2, "Dead block should be removed");
        assert!(
            !func.blocks.iter().any(|b| b.label == "dead"),
            "Dead block should not exist"
        );
    }

    #[test]
    fn test_pure_function_cse() {
        // Test: duplicate calls to @pure function should be eliminated
        // %r1 = call square(x)
        // %r2 = call square(x)  <- should become %r2 = copy %r1
        let mut func = MirFunction {
            name: "test_pure_cse".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("r1")),
                        func: "square".to_string(),
                        args: vec![Operand::Place(Place::new("x"))],
                    },
                    MirInst::Call {
                        dest: Some(Place::new("r2")),
                        func: "square".to_string(),
                        args: vec![Operand::Place(Place::new("x"))],
                    },
                    MirInst::BinOp {
                        dest: Place::new("result"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("r1")),
                        rhs: Operand::Place(Place::new("r2")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        // Create a pure function set containing "square"
        let mut pure_functions = HashSet::new();
        pure_functions.insert("square".to_string());
        let pass = PureFunctionCSE::new(pure_functions);

        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Pure function CSE should have made changes");

        // Second call should be replaced with Copy
        let second_inst = &func.blocks[0].instructions[1];
        assert!(
            matches!(second_inst, MirInst::Copy { dest, src }
                if dest.name == "r2" && src.name == "r1"),
            "Second call should be replaced with copy from first result"
        );
    }

    #[test]
    fn test_pure_function_cse_different_args() {
        // Test: calls with different args should NOT be eliminated
        let mut func = MirFunction {
            name: "test_pure_cse_diff".to_string(),
            params: vec![
                ("x".to_string(), MirType::I64),
                ("y".to_string(), MirType::I64),
            ],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("r1")),
                        func: "square".to_string(),
                        args: vec![Operand::Place(Place::new("x"))],
                    },
                    MirInst::Call {
                        dest: Some(Place::new("r2")),
                        func: "square".to_string(),
                        args: vec![Operand::Place(Place::new("y"))], // Different arg!
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r1")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        let mut pure_functions = HashSet::new();
        pure_functions.insert("square".to_string());
        let pass = PureFunctionCSE::new(pure_functions);

        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Different args should not be eliminated");
    }

    #[test]
    fn test_non_pure_function_not_eliminated() {
        // Test: calls to non-pure functions should NOT be eliminated
        let mut func = MirFunction {
            name: "test_non_pure".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("r1")),
                        func: "get_random".to_string(), // Not pure
                        args: vec![Operand::Place(Place::new("x"))],
                    },
                    MirInst::Call {
                        dest: Some(Place::new("r2")),
                        func: "get_random".to_string(),
                        args: vec![Operand::Place(Place::new("x"))],
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r1")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        // Empty pure function set - no functions are pure
        let pure_functions = HashSet::new();
        let pass = PureFunctionCSE::new(pure_functions);

        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Non-pure functions should not be eliminated");
    }

    #[test]
    fn test_const_function_eval() {
        // Test: calls to @const functions with constant return values should be inlined
        // @const fn get_magic() -> i64 = 42;
        // fn test() -> i64 = get_magic() + 1;  // should become 42 + 1
        let const_fn = MirFunction {
            name: "get_magic".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(42)))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: true,
            is_const: true,
        };

        let mut caller_fn = MirFunction {
            name: "test_caller".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("magic")),
                        func: "get_magic".to_string(),
                        args: vec![],
                    },
                    MirInst::BinOp {
                        dest: Place::new("result"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("magic")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        // Create program with both functions
        let program = MirProgram {
            functions: vec![const_fn, caller_fn.clone()],
            extern_fns: vec![],
        };

        // Create pass from program
        let pass = ConstFunctionEval::from_program(&program);

        let changed = pass.run_on_function(&mut caller_fn);
        assert!(changed, "Const function eval should have made changes");

        // First instruction should now be Const, not Call
        let first_inst = &caller_fn.blocks[0].instructions[0];
        assert!(
            matches!(first_inst, MirInst::Const { dest, value: Constant::Int(42) }
                if dest.name == "magic"),
            "Call to const function should be replaced with constant: got {:?}",
            first_inst
        );
    }

    #[test]
    fn test_const_function_with_args_not_inlined() {
        // Test: @const functions with arguments should NOT be inlined
        // @const fn square(x: i64) -> i64 = x * x;
        // These require compile-time evaluation which is deferred
        let const_fn = MirFunction {
            name: "square".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::BinOp {
                    dest: Place::new("result"),
                    op: MirBinOp::Mul,
                    lhs: Operand::Place(Place::new("x")),
                    rhs: Operand::Place(Place::new("x")),
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: true,
            is_const: true,
        };

        let mut caller_fn = MirFunction {
            name: "test_caller".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::Call {
                    dest: Some(Place::new("result")),
                    func: "square".to_string(),
                    args: vec![Operand::Constant(Constant::Int(5))],
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
        };

        let program = MirProgram {
            functions: vec![const_fn, caller_fn.clone()],
            extern_fns: vec![],
        };

        let pass = ConstFunctionEval::from_program(&program);

        let changed = pass.run_on_function(&mut caller_fn);
        assert!(
            !changed,
            "Const function with args should not be inlined (deferred)"
        );
    }
}
