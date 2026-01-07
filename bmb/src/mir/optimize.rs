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
    Constant, MirBinOp, MirFunction, MirInst, MirProgram, MirUnaryOp,
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

        for func in &mut program.functions {
            let func_stats = self.optimize_function(func);
            stats.merge(&func_stats);
        }

        stats
    }

    /// Run all passes on a single function until fixed point
    fn optimize_function(&self, func: &mut MirFunction) -> OptimizationStats {
        let mut stats = OptimizationStats::new();
        let mut iteration = 0;

        loop {
            let mut changed = false;
            iteration += 1;

            for pass in &self.passes {
                if pass.run_on_function(func) {
                    changed = true;
                    stats.record_pass(pass.name());
                }
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
            {
                if let Operand::Constant(Constant::Bool(b)) = cond {
                    let target = if *b {
                        then_label.clone()
                    } else {
                        else_label.clone()
                    };
                    block.terminator = Terminator::Goto(target);
                    changed = true;
                }
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
    if let Operand::Place(p) = op {
        if let Some(src) = copies.get(&p.name) {
            *p = src.clone();
            return true;
        }
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
// Contract-Based Optimization Pass (BMB-specific)
// ============================================================================

/// Contract-based optimizations unique to BMB
///
/// These optimizations leverage BMB's contract system:
/// - Bounds check elimination based on `pre` conditions
/// - Null check elimination with Option<T> contracts
/// - Purity-based CSE using `post` conditions
pub struct ContractBasedOptimization;

impl OptimizationPass for ContractBasedOptimization {
    fn name(&self) -> &'static str {
        "contract_based_optimization"
    }

    fn run_on_function(&self, _func: &mut MirFunction) -> bool {
        // TODO: Implement contract-based optimizations
        //
        // Future implementation will:
        // 1. Parse preconditions from function metadata
        // 2. Track proven facts through the function
        // 3. Eliminate redundant checks based on proven facts
        //
        // Example: fn sum_range(arr: &[i32], start: usize, end: usize)
        //   pre start <= end
        //   pre end <= len(arr)
        //
        // With these preconditions, we can prove:
        // - All array accesses in start..end are in bounds
        // - Loop invariants about start <= i < end
        //
        // This enables eliminating bounds checks in the loop body.

        false // Placeholder - no changes yet
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
}
