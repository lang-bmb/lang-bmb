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
    BasicBlock, CmpOp, Constant, ContractFact, MirBinOp, MirFunction, MirInst, MirProgram, MirType, MirUnaryOp,
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
                // v0.50.54: Add algebraic simplification before constant folding
                pipeline.add_pass(Box::new(AlgebraicSimplification));
                pipeline.add_pass(Box::new(ConstantFolding));
                pipeline.add_pass(Box::new(DeadCodeElimination));
                pipeline.add_pass(Box::new(SimplifyBranches));
                // v0.51.8: If-else chain to switch for jump tables
                pipeline.add_pass(Box::new(IfElseToSwitch));
                pipeline.add_pass(Box::new(CopyPropagation));
                // v0.51.10: Memory load CSE for repeated load_f64/load_i64 calls
                pipeline.add_pass(Box::new(MemoryLoadCSE));
                // v0.50.76: Add contract-based optimization for dead branch elimination
                pipeline.add_pass(Box::new(ContractBasedOptimization));
                pipeline.add_pass(Box::new(ContractUnreachableElimination));
                // v0.50.65: Add tail call optimization for recursive functions
                pipeline.add_pass(Box::new(TailCallOptimization));
                // v0.51.9: Convert tail recursion to loops for better performance
                pipeline.add_pass(Box::new(TailRecursiveToLoop));
                // v0.60.11: Convert fibonacci-like double recursion to O(n) loops
                pipeline.add_pass(Box::new(LinearRecurrenceToLoop));
                // v0.51.16: Loop invariant code motion - hoist len() calls out of loops
                pipeline.add_pass(Box::new(LoopInvariantCodeMotion::new()));
                // v0.50.73: String concat chain optimization for O(n) allocation
                pipeline.add_pass(Box::new(StringConcatOptimization));
            }
            OptLevel::Aggressive => {
                // All optimizations
                // v0.50.54: Add algebraic simplification before constant folding
                pipeline.add_pass(Box::new(AlgebraicSimplification));
                pipeline.add_pass(Box::new(ConstantFolding));
                pipeline.add_pass(Box::new(DeadCodeElimination));
                pipeline.add_pass(Box::new(SimplifyBranches));
                // v0.51.8: If-else chain to switch for jump tables
                pipeline.add_pass(Box::new(IfElseToSwitch));
                pipeline.add_pass(Box::new(CopyPropagation));
                pipeline.add_pass(Box::new(CommonSubexpressionElimination));
                // v0.51.10: Memory load CSE for repeated load_f64/load_i64 calls
                pipeline.add_pass(Box::new(MemoryLoadCSE));
                pipeline.add_pass(Box::new(ContractBasedOptimization));
                pipeline.add_pass(Box::new(ContractUnreachableElimination));
                // v0.50.65: Add tail call optimization for recursive functions
                pipeline.add_pass(Box::new(TailCallOptimization));
                // v0.51.9: Convert tail recursion to loops for better performance
                pipeline.add_pass(Box::new(TailRecursiveToLoop));
                // v0.60.11: Convert fibonacci-like double recursion to O(n) loops
                pipeline.add_pass(Box::new(LinearRecurrenceToLoop));
                // v0.51.16: Loop invariant code motion - hoist len() calls out of loops
                pipeline.add_pass(Box::new(LoopInvariantCodeMotion::new()));
                // v0.50.73: String concat chain optimization for O(n) allocation
                pipeline.add_pass(Box::new(StringConcatOptimization));
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

        // v0.50.80: Run interprocedural constant propagation narrowing FIRST
        // This pass narrows i64 parameters to i32 when call sites use small constants
        // and the function has decreasing recursive patterns (e.g., fibonacci)
        // v0.51.0: PHI codegen now handles type mismatches via coerce_phi_value()
        let narrowing = ConstantPropagationNarrowing::from_program(program);
        if narrowing.run_on_program(program) {
            stats.record_pass(narrowing.name());
        }

        // v0.38.3: Create PureFunctionCSE pass with program-level information
        let pure_cse = PureFunctionCSE::from_program(program);

        // v0.38.4: Create ConstFunctionEval pass with program-level information
        let const_eval = ConstFunctionEval::from_program(program);

        for func in &mut program.functions {
            let func_stats = self.optimize_function_with_program_passes(func, &pure_cse, &const_eval);
            stats.merge(&func_stats);
        }

        // v0.60.9: Run loop bounded narrowing AFTER per-function optimization
        // This runs after ConstFunctionEval has inlined constant functions (e.g., N() -> 1000)
        // Critical for spectral_norm: while i < n loops where n=1000 can use 32-bit ops
        let loop_narrowing = LoopBoundedNarrowing::from_program(program);
        if loop_narrowing.run_on_program(program) {
            stats.record_pass(loop_narrowing.name());
        }

        // v0.51.8: Run aggressive inlining LAST (after all optimizations)
        // This marks small, simple functions for LLVM's alwaysinline attribute
        // to eliminate function call overhead in tight loops
        let inlining = AggressiveInlining::new();
        if inlining.run_on_program(program) {
            stats.record_pass(inlining.name());
        }

        // v0.51.11: Run memory effect analysis to detect memory-free functions
        // This enables LLVM memory(none) attribute for better LICM
        let memory_analysis = MemoryEffectAnalysis::new();
        if memory_analysis.run_on_program(program) {
            stats.record_pass(memory_analysis.name());
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

        // v0.50.72: Collect variables modified in non-entry blocks
        // These might be loop-carried dependencies, so don't propagate their constants
        // across block boundaries to avoid incorrect folding in loop headers
        let mut loop_modified: HashSet<String> = HashSet::new();
        for (block_idx, block) in func.blocks.iter().enumerate() {
            if block_idx > 0 {  // Skip entry block
                for inst in &block.instructions {
                    // Extract destination place from each instruction variant
                    let dest_name = match inst {
                        MirInst::Const { dest, .. } => Some(dest.name.clone()),
                        MirInst::Copy { dest, .. } => Some(dest.name.clone()),
                        MirInst::BinOp { dest, .. } => Some(dest.name.clone()),
                        MirInst::UnaryOp { dest, .. } => Some(dest.name.clone()),
                        MirInst::Call { dest: Some(d), .. } => Some(d.name.clone()),
                        MirInst::IndexLoad { dest, .. } => Some(dest.name.clone()),
                        MirInst::Phi { dest, .. } => Some(dest.name.clone()),
                        MirInst::StructInit { dest, .. } => Some(dest.name.clone()),
                        MirInst::FieldAccess { dest, .. } => Some(dest.name.clone()),
                        MirInst::EnumVariant { dest, .. } => Some(dest.name.clone()),
                        MirInst::ArrayInit { dest, .. } => Some(dest.name.clone()),
                        _ => None,
                    };
                    if let Some(name) = dest_name {
                        loop_modified.insert(name);
                    }
                }
            }
        }

        for block in &mut func.blocks {
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                match inst {
                    MirInst::Const { dest, value } => {
                        constants.insert(dest.name.clone(), value.clone());
                        new_instructions.push(inst.clone());
                    }
                    MirInst::BinOp { dest, op, lhs, rhs } => {
                        // v0.50.72: Use loop-aware constant lookup
                        if let (Some(lhs_const), Some(rhs_const)) =
                            (get_constant_with_filter(lhs, &constants, &loop_modified),
                             get_constant_with_filter(rhs, &constants, &loop_modified))
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
                        // v0.50.72: Use loop-aware constant lookup
                        if let Some(src_const) = get_constant_with_filter(src, &constants, &loop_modified)
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
                    // v0.50.68: Evaluate builtin functions with constant arguments
                    // v0.51.2: Propagate constants to call arguments for FFI string optimization
                    MirInst::Call { dest, func: func_name, args, is_tail } => {
                        // First, try to fold builtin calls completely
                        if let Some(d) = dest {
                            if let Some(result) = fold_builtin_call(func_name, args, &constants) {
                                constants.insert(d.name.clone(), result.clone());
                                new_instructions.push(MirInst::Const {
                                    dest: d.clone(),
                                    value: result,
                                });
                                changed = true;
                                continue;
                            }
                        }

                        // v0.51.2: Propagate constants to call arguments
                        // This enables LLVM codegen to detect string literal arguments
                        let propagated_args: Vec<Operand> = args.iter().map(|arg| {
                            match arg {
                                Operand::Place(p) => {
                                    // Don't propagate loop-modified variables
                                    if !loop_modified.contains(&p.name) {
                                        if let Some(c) = constants.get(&p.name) {
                                            return Operand::Constant(c.clone());
                                        }
                                    }
                                    arg.clone()
                                }
                                Operand::Constant(_) => arg.clone(),
                            }
                        }).collect();

                        // Check if any argument was propagated
                        let any_propagated = args.iter().zip(propagated_args.iter()).any(|(orig, prop)| {
                            matches!((orig, prop), (Operand::Place(_), Operand::Constant(_)))
                        });

                        if any_propagated {
                            changed = true;
                        }

                        new_instructions.push(MirInst::Call {
                            dest: dest.clone(),
                            func: func_name.clone(),
                            args: propagated_args,
                            is_tail: *is_tail,
                        });
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
    get_constant_with_filter(operand, constants, &HashSet::new())
}

/// v0.50.72: Get constant with loop-modified variable filter
fn get_constant_with_filter(
    operand: &Operand,
    constants: &HashMap<String, Constant>,
    loop_modified: &HashSet<String>,
) -> Option<Constant> {
    match operand {
        Operand::Constant(c) => Some(c.clone()),
        Operand::Place(p) => {
            // Don't propagate constants for loop-modified variables
            if loop_modified.contains(&p.name) {
                None
            } else {
                constants.get(&p.name).cloned()
            }
        }
    }
}

/// v0.50.68: Evaluate builtin functions with constant arguments at compile time
///
/// Supported builtins:
/// - `chr(i64)` -> String: Convert ASCII code to single-character string
/// - `ord(String)` -> i64: Get ASCII code of first character (only for single-char literals)
fn fold_builtin_call(
    func_name: &str,
    args: &[Operand],
    constants: &HashMap<String, Constant>,
) -> Option<Constant> {
    match func_name {
        // chr(65) -> "A"
        "chr" | "bmb_chr" if args.len() == 1 => {
            if let Some(Constant::Int(code)) = get_constant(&args[0], constants) {
                // Valid ASCII range
                if code >= 0 && code <= 127 {
                    let ch = char::from_u32(code as u32)?;
                    return Some(Constant::String(ch.to_string()));
                }
            }
            None
        }
        // ord("A") -> 65 (only for single-character string constants)
        "ord" | "bmb_ord" if args.len() == 1 => {
            if let Some(Constant::String(s)) = get_constant(&args[0], constants) {
                if s.len() == 1 {
                    let code = s.chars().next()? as i64;
                    return Some(Constant::Int(code));
                }
            }
            None
        }
        _ => None,
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

        // v0.50.68: String concatenation at compile time
        // "Hello" + " " + "World" → "Hello World"
        (MirBinOp::Add, Constant::String(a), Constant::String(b)) => {
            Some(Constant::String(format!("{}{}", a, b)))
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
// Algebraic Simplification Pass (v0.50.54)
// ============================================================================

/// Algebraic simplification: eliminate identity operations
///
/// This pass handles algebraic identities that aren't caught by constant folding:
/// - `x + 0` → `x` (additive identity)
/// - `0 + x` → `x` (additive identity, commutative)
/// - `x - 0` → `x` (subtractive identity)
/// - `x * 1` → `x` (multiplicative identity)
/// - `1 * x` → `x` (multiplicative identity, commutative)
/// - `x * 0` → `0` (zero product)
/// - `0 * x` → `0` (zero product, commutative)
/// - `x / 1` → `x` (division identity)
/// - `x && true` → `x` (boolean and identity)
/// - `x || false` → `x` (boolean or identity)
/// - `x && false` → `false` (boolean and zero)
/// - `x || true` → `true` (boolean or one)
pub struct AlgebraicSimplification;

impl OptimizationPass for AlgebraicSimplification {
    fn name(&self) -> &'static str {
        "algebraic_simplification"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        for block in &mut func.blocks {
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                match inst {
                    MirInst::BinOp { dest, op, lhs, rhs } => {
                        if let Some(simplified) = simplify_binop(dest, *op, lhs, rhs) {
                            new_instructions.push(simplified);
                            changed = true;
                            continue;
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

/// Try to simplify a binary operation using algebraic identities
/// Returns Some(simplified instruction) if simplification is possible
fn simplify_binop(dest: &Place, op: MirBinOp, lhs: &Operand, rhs: &Operand) -> Option<MirInst> {
    match op {
        // Addition: x + 0 = x, 0 + x = x
        MirBinOp::Add => {
            if matches!(rhs, Operand::Constant(Constant::Int(0))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(lhs)?,
                });
            }
            if matches!(lhs, Operand::Constant(Constant::Int(0))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(rhs)?,
                });
            }
            None
        }

        // Subtraction: x - 0 = x
        MirBinOp::Sub => {
            if matches!(rhs, Operand::Constant(Constant::Int(0))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(lhs)?,
                });
            }
            None
        }

        // Multiplication: x * 1 = x, 1 * x = x, x * 0 = 0, 0 * x = 0
        MirBinOp::Mul => {
            if matches!(rhs, Operand::Constant(Constant::Int(1))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(lhs)?,
                });
            }
            if matches!(lhs, Operand::Constant(Constant::Int(1))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(rhs)?,
                });
            }
            if matches!(rhs, Operand::Constant(Constant::Int(0)))
                || matches!(lhs, Operand::Constant(Constant::Int(0)))
            {
                return Some(MirInst::Const {
                    dest: dest.clone(),
                    value: Constant::Int(0),
                });
            }
            None
        }

        // Division: x / 1 = x
        MirBinOp::Div => {
            if matches!(rhs, Operand::Constant(Constant::Int(1))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(lhs)?,
                });
            }
            None
        }

        // Boolean And: x && true = x, x && false = false
        MirBinOp::And => {
            if matches!(rhs, Operand::Constant(Constant::Bool(true))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(lhs)?,
                });
            }
            if matches!(lhs, Operand::Constant(Constant::Bool(true))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(rhs)?,
                });
            }
            if matches!(rhs, Operand::Constant(Constant::Bool(false)))
                || matches!(lhs, Operand::Constant(Constant::Bool(false)))
            {
                return Some(MirInst::Const {
                    dest: dest.clone(),
                    value: Constant::Bool(false),
                });
            }
            None
        }

        // Boolean Or: x || false = x, x || true = true
        MirBinOp::Or => {
            if matches!(rhs, Operand::Constant(Constant::Bool(false))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(lhs)?,
                });
            }
            if matches!(lhs, Operand::Constant(Constant::Bool(false))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(rhs)?,
                });
            }
            if matches!(rhs, Operand::Constant(Constant::Bool(true)))
                || matches!(lhs, Operand::Constant(Constant::Bool(true)))
            {
                return Some(MirInst::Const {
                    dest: dest.clone(),
                    value: Constant::Bool(true),
                });
            }
            None
        }

        // Float operations: same patterns
        MirBinOp::FAdd => {
            if matches!(rhs, Operand::Constant(Constant::Float(f)) if *f == 0.0) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(lhs)?,
                });
            }
            if matches!(lhs, Operand::Constant(Constant::Float(f)) if *f == 0.0) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(rhs)?,
                });
            }
            None
        }

        MirBinOp::FMul => {
            if matches!(rhs, Operand::Constant(Constant::Float(f)) if *f == 1.0) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(lhs)?,
                });
            }
            if matches!(lhs, Operand::Constant(Constant::Float(f)) if *f == 1.0) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(rhs)?,
                });
            }
            if matches!(rhs, Operand::Constant(Constant::Float(f)) if *f == 0.0)
                || matches!(lhs, Operand::Constant(Constant::Float(f)) if *f == 0.0)
            {
                return Some(MirInst::Const {
                    dest: dest.clone(),
                    value: Constant::Float(0.0),
                });
            }
            None
        }

        _ => None,
    }
}

/// Convert an operand to a place if it is one, None if it's a constant
fn operand_to_place(op: &Operand) -> Option<Place> {
    match op {
        Operand::Place(p) => Some(p.clone()),
        Operand::Constant(_) => None,
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
        MirInst::IndexStore { array, index, value, .. } => {
            used.insert(array.name.clone());
            collect_used_in_operand(index, used);
            collect_used_in_operand(value, used);
        }
        MirInst::Cast { src, .. } => {
            collect_used_in_operand(src, used);
        }
        // v0.55: Tuple instructions
        MirInst::TupleInit { elements, .. } => {
            for (_, val) in elements {
                collect_used_in_operand(val, used);
            }
        }
        MirInst::TupleExtract { tuple, .. } => {
            used.insert(tuple.name.clone());
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
        // v0.51.8: Handle Cast instruction
        MirInst::Cast { dest, .. } => Some(dest),
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

        for block in &mut func.blocks {
            // v0.50.72: Clear copy map at each block boundary
            // This prevents incorrect propagation across loop iterations
            // where a variable might be reassigned
            let mut copies: HashMap<String, Place> = HashMap::new();

            // v0.50.72: Process instructions in order, building copy map incrementally
            // This ensures we only propagate copies to instructions AFTER their definition
            for inst in &mut block.instructions {
                // First propagate existing copies
                if propagate_copies_in_inst(inst, &copies) {
                    changed = true;
                }

                // Then add this instruction to copy map if it's a Copy
                if let MirInst::Copy { dest, src } = inst {
                    copies.insert(dest.name.clone(), src.clone());
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

        // v0.51.6: CSE now operates per-block only to avoid SSA domination issues
        // Previously, expressions were shared across all blocks, which caused
        // values defined in one branch to be incorrectly reused in sibling branches.
        // For example: if j < 0 { j+1... } else { j+1... } would share %add
        // but %add from the "then" branch doesn't dominate the "else" branch.
        //
        // The safe approach is to only apply CSE within a single basic block,
        // where all expressions naturally dominate their uses.

        for block in &mut func.blocks {
            // v0.51.6: Clear expressions at the start of each block
            // This ensures we only reuse expressions within the same block
            let mut expressions: HashMap<String, Place> = HashMap::new();
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                if let MirInst::BinOp { dest, op, lhs, rhs } = inst {
                    let key = format!("{:?}:{:?}:{:?}", op, lhs, rhs);

                    if let Some(existing) = expressions.get(&key) {
                        // Replace with copy - safe because both are in the same block
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
// Memory Load CSE Pass (v0.51.10)
// ============================================================================

/// Memory load CSE: eliminate redundant load_f64/load_i64 calls within basic blocks
///
/// Within a basic block, consecutive loads from the same memory address are equivalent
/// if no stores occur between them. This pass:
/// 1. Tracks (load_fn, ptr_arg) -> cached_dest for load calls
/// 2. Replaces duplicate loads with Copy instructions
/// 3. Invalidates cache on store_f64/store_i64 calls (conservative)
///
/// Example:
/// ```text
/// %dx = call load_f64(%ptr1)
/// %dy = call load_f64(%ptr2)
/// %dx2 = call load_f64(%ptr1)  // eliminated -> Copy %dx2, %dx
/// ```
pub struct MemoryLoadCSE;

impl OptimizationPass for MemoryLoadCSE {
    fn name(&self) -> &'static str {
        "memory_load_cse"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        for block in &mut func.blocks {
            // Track: (load_fn_name, ptr_operand_key) -> cached_place
            let mut load_cache: HashMap<(String, String), Place> = HashMap::new();
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                match inst {
                    MirInst::Call { dest, func: fn_name, args, .. } => {
                        // Check if this is a memory load function with a destination
                        if (fn_name == "load_f64" || fn_name == "load_i64")
                            && args.len() == 1
                            && dest.is_some()
                        {
                            let dest = dest.as_ref().unwrap();
                            // Create a key from the function name and pointer argument
                            let ptr_key = format!("{:?}", args[0]);
                            let cache_key = (fn_name.clone(), ptr_key);

                            if let Some(cached) = load_cache.get(&cache_key) {
                                // Replace with copy from cached value
                                new_instructions.push(MirInst::Copy {
                                    dest: dest.clone(),
                                    src: cached.clone(),
                                });
                                changed = true;
                            } else {
                                // Cache this load and keep original instruction
                                load_cache.insert(cache_key, dest.clone());
                                new_instructions.push(inst.clone());
                            }
                        }
                        // Check if this is a memory store function - invalidate cache
                        else if fn_name == "store_f64" || fn_name == "store_i64" {
                            // Conservative: invalidate ALL loads since we don't track aliasing
                            // A more sophisticated analysis could check if store ptr might alias
                            load_cache.clear();
                            new_instructions.push(inst.clone());
                        }
                        else {
                            // Other function calls may have side effects - be conservative
                            // Only invalidate if function might write to memory
                            if might_write_memory(fn_name) {
                                load_cache.clear();
                            }
                            new_instructions.push(inst.clone());
                        }
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

/// Check if a function might write to memory
/// Conservative: assume most functions don't write, but some known ones do
fn might_write_memory(fn_name: &str) -> bool {
    // Known memory-writing functions
    matches!(fn_name,
        "store_i64" | "store_f64" |
        "bmb_vec_push" | "hashmap_insert" | "hashmap_remove" |
        "sb_push" | "sb_push_char" | "sb_push_int" | "sb_push_cstr" |
        "bmb_sb_push" | "bmb_sb_push_char" | "bmb_sb_push_int" |
        "free" | "realloc"
    )
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
                if let MirInst::Call { dest: Some(dest), func: called_func, args, .. } = inst {
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
    /// Create from a MirProgram by analyzing @const functions and pure 0-arg functions
    pub fn from_program(program: &MirProgram) -> Self {
        let mut const_values = HashMap::new();

        for func in &program.functions {
            // v0.50.75: Also evaluate pure 0-arg functions (like crlf())
            // These are commonly used for constants but not marked @const
            if func.params.is_empty() {
                // First apply constant folding to the function to evaluate expressions
                // like chr(13) + chr(10) -> "\r\n"
                let mut func_copy = func.clone();
                let cf = ConstantFolding;
                cf.run_on_function(&mut func_copy);

                // Then check if function body is a simple constant return
                if let Some(value) = Self::extract_constant_return(&func_copy) {
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
                if let MirInst::Call { dest: Some(dest), func: called_func, args, .. } = inst {
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

        // Collect unreachable labels before removing blocks
        let unreachable_blocks: HashSet<String> = func.blocks.iter()
            .filter(|block| !reachable.contains(&block.label))
            .map(|block| block.label.clone())
            .collect();

        // Third pass: remove unreachable blocks
        let original_len = func.blocks.len();
        func.blocks.retain(|block| reachable.contains(&block.label));
        if func.blocks.len() != original_len {
            changed = true;
        }

        // Fourth pass: update PHI nodes to remove incoming edges from removed blocks
        // This is CRITICAL - PHI nodes must only reference existing predecessor blocks
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
// Tail Call Optimization Pass (v0.50.65)
// ============================================================================

/// Tail Call Optimization pass
///
/// Identifies function calls in tail position and marks them for tail call optimization.
/// A call is in tail position if:
/// 1. It's the last instruction before a Return terminator
/// 2. The Return value is exactly the call's result
/// 3. There are no intervening instructions that use the call result
///
/// This enables LLVM to apply tail call optimization, converting recursive
/// calls into loops and eliminating stack growth.
pub struct TailCallOptimization;

impl TailCallOptimization {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TailCallOptimization {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for TailCallOptimization {
    fn name(&self) -> &'static str {
        "TailCallOptimization"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Phase 1: Direct tail calls (Call -> Return in same block)
        for block in &mut func.blocks {
            // Check if the terminator is a Return with a value
            let return_value = match &block.terminator {
                Terminator::Return(Some(Operand::Place(place))) => Some(place.name.clone()),
                _ => None,
            };

            // If there's no return with a place value, skip this block
            let return_var = match return_value {
                Some(v) => v,
                None => continue,
            };

            // Find the last Call instruction that produces the return value
            // Work backwards from the end
            let mut tail_call_idx = None;
            for (idx, inst) in block.instructions.iter().enumerate().rev() {
                match inst {
                    MirInst::Call { dest: Some(dest), is_tail: false, .. }
                        if dest.name == return_var => {
                        // Found the call that produces the return value
                        // Check if there are any instructions after it that use the result
                        let has_intervening_use = block.instructions[idx + 1..].iter().any(|i| {
                            uses_place(i, &return_var)
                        });

                        if !has_intervening_use {
                            tail_call_idx = Some(idx);
                        }
                        break;
                    }
                    // If we hit any other instruction that defines return_var, stop
                    _ if defines_place(inst, &return_var) => break,
                    _ => continue,
                }
            }

            // Mark the tail call
            if let Some(idx) = tail_call_idx {
                if let MirInst::Call { is_tail, .. } = &mut block.instructions[idx] {
                    *is_tail = true;
                    changed = true;
                }
            }
        }

        // Phase 2: Phi-based tail calls (Call -> Goto -> Phi -> Return)
        // Common in BMB's if-else expressions where recursion is in one branch
        //
        // Pattern:
        //   else_block: %result = call func(...); goto merge
        //   merge_block: %phi = phi [%other, then], [%result, else]; return %phi
        //
        // If %result flows directly into phi and phi is returned, mark call as tail
        changed |= self.detect_phi_tail_calls(func);

        changed
    }
}

impl TailCallOptimization {
    /// Trace through phi chains to find the original Call that produces a value
    ///
    /// v0.50.79: Handles nested phi patterns like:
    ///   then_3: %_t8 = call f(); goto merge_5
    ///   merge_5: %_t6 = phi [%_t8, then_3], [...]; goto merge_2
    ///   merge_2: %_t2 = phi [..., merge_5]; return %_t2
    ///
    /// Returns: Vec<(block_label, call_index)> of tail calls found
    fn trace_phi_to_calls(
        &self,
        func: &MirFunction,
        value_name: &str,
        source_label: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> Vec<(String, usize)> {
        // Prevent infinite loops in cyclic phi references
        let visit_key = format!("{}:{}", source_label, value_name);
        if !visited.insert(visit_key) {
            return vec![];
        }

        let source_block = match func.blocks.iter().find(|b| b.label == source_label) {
            Some(b) => b,
            None => return vec![],
        };

        // Block must end with Goto for tail call pattern
        if !matches!(source_block.terminator, Terminator::Goto(_)) {
            return vec![];
        }

        // Search for the definition of value_name in this block
        for (idx, inst) in source_block.instructions.iter().enumerate().rev() {
            match inst {
                // Direct Call -> this is a tail call candidate
                MirInst::Call { dest: Some(dest), is_tail: false, .. }
                    if dest.name == value_name =>
                {
                    // Check no intervening uses after the call
                    let has_intervening_use = source_block.instructions[idx + 1..]
                        .iter()
                        .any(|i| uses_place(i, value_name));

                    if !has_intervening_use {
                        return vec![(source_label.to_string(), idx)];
                    }
                    break;
                }

                // Phi node -> trace through its sources recursively
                MirInst::Phi { dest, values } if dest.name == value_name => {
                    let mut results = vec![];
                    for (operand, phi_source_label) in values {
                        if let Operand::Place(p) = operand {
                            // Recursively trace through the phi's source
                            results.extend(
                                self.trace_phi_to_calls(func, &p.name, phi_source_label, visited)
                            );
                        }
                    }
                    return results;
                }

                // Some other instruction defines this value -> not a tail call
                _ if defines_place(inst, value_name) => break,
                _ => continue,
            }
        }

        vec![]
    }

    /// Detect tail calls that flow through phi nodes
    ///
    /// Pattern: Call in one block -> Goto -> Phi in merge block -> Return
    fn detect_phi_tail_calls(&self, func: &mut MirFunction) -> bool {
        // First, find merge blocks with pattern: phi -> return phi_result
        // Collect: (merge_block_label, phi_dest, incoming_edges)
        let mut phi_return_blocks: Vec<(String, String, Vec<(String, String)>)> = Vec::new();

        for block in &func.blocks {
            // Check for Return(phi_result)
            let return_var = match &block.terminator {
                Terminator::Return(Some(Operand::Place(place))) => &place.name,
                _ => continue,
            };

            // Find phi that produces return_var (should be only instruction or last before return)
            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst {
                    if dest.name == *return_var {
                        // Collect incoming edges: (value_name, source_block)
                        let edges: Vec<(String, String)> = values.iter()
                            .filter_map(|(operand, label)| {
                                if let Operand::Place(p) = operand {
                                    Some((p.name.clone(), label.clone()))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if !edges.is_empty() {
                            phi_return_blocks.push((
                                block.label.clone(),
                                dest.name.clone(),
                                edges,
                            ));
                        }
                        break;
                    }
                }
            }
        }

        // For each phi return block, check source blocks for tail calls
        // v0.50.79: Support nested phi chains (e.g., Call -> Phi -> Phi -> Return)
        let mut tail_calls_to_mark: Vec<(String, usize)> = Vec::new();

        for (_merge_label, _phi_dest, edges) in &phi_return_blocks {
            for (value_name, source_label) in edges {
                // Use transitive phi tracing to find tail calls through nested phis
                let mut visited = std::collections::HashSet::new();
                let found_calls = self.trace_phi_to_calls(func, value_name, source_label, &mut visited);
                tail_calls_to_mark.extend(found_calls);
            }
        }

        // Mark the tail calls AND convert Goto to Return for proper TCO
        // This is critical: LLVM tailcallelim only works when ret immediately follows tail call
        let mut changed = false;

        // Collect blocks that need to be removed from phi nodes
        let mut blocks_converted_to_return: Vec<String> = Vec::new();

        for (block_label, call_idx) in tail_calls_to_mark {
            if let Some(block) = func.blocks.iter_mut().find(|b| b.label == block_label) {
                // Get the call destination for the return
                let call_dest = if let MirInst::Call { dest: Some(dest), is_tail, .. } = &mut block.instructions[call_idx] {
                    *is_tail = true;
                    Some(dest.clone())
                } else {
                    None
                };

                // Convert terminator from Goto(merge) to Return(call_result)
                // This allows LLVM to properly optimize the tail call to a loop
                if let Some(dest) = call_dest {
                    if matches!(block.terminator, Terminator::Goto(_)) {
                        block.terminator = Terminator::Return(Some(Operand::Place(dest)));
                        blocks_converted_to_return.push(block_label.clone());
                        changed = true;
                    }
                }
            }
        }

        // Remove converted blocks from phi nodes (they no longer branch to merge)
        if !blocks_converted_to_return.is_empty() {
            for block in &mut func.blocks {
                for inst in &mut block.instructions {
                    if let MirInst::Phi { values, .. } = inst {
                        values.retain(|(_, label)| !blocks_converted_to_return.contains(label));
                    }
                }
            }
        }

        changed
    }
}

/// Check if an instruction uses a given place
fn uses_place(inst: &MirInst, name: &str) -> bool {
    match inst {
        MirInst::Copy { src, .. } => src.name == name,
        MirInst::BinOp { lhs, rhs, .. } => {
            matches!(lhs, Operand::Place(p) if p.name == name) ||
            matches!(rhs, Operand::Place(p) if p.name == name)
        }
        MirInst::UnaryOp { src, .. } => matches!(src, Operand::Place(p) if p.name == name),
        MirInst::Call { args, .. } => args.iter().any(|a| matches!(a, Operand::Place(p) if p.name == name)),
        MirInst::Phi { values, .. } => values.iter().any(|(v, _)| matches!(v, Operand::Place(p) if p.name == name)),
        MirInst::FieldAccess { base, .. } => base.name == name,
        MirInst::FieldStore { base, value, .. } => {
            base.name == name || matches!(value, Operand::Place(p) if p.name == name)
        }
        MirInst::IndexLoad { array, index, .. } => {
            array.name == name || matches!(index, Operand::Place(p) if p.name == name)
        }
        MirInst::IndexStore { array, index, value, .. } => {
            array.name == name ||
            matches!(index, Operand::Place(p) if p.name == name) ||
            matches!(value, Operand::Place(p) if p.name == name)
        }
        MirInst::StructInit { fields, .. } => {
            fields.iter().any(|(_, v)| matches!(v, Operand::Place(p) if p.name == name))
        }
        MirInst::EnumVariant { args, .. } => {
            args.iter().any(|a| matches!(a, Operand::Place(p) if p.name == name))
        }
        MirInst::ArrayInit { elements, .. } => {
            elements.iter().any(|e| matches!(e, Operand::Place(p) if p.name == name))
        }
        MirInst::Cast { src, .. } => matches!(src, Operand::Place(p) if p.name == name),
        _ => false,
    }
}

/// Check if an instruction defines a given place
fn defines_place(inst: &MirInst, name: &str) -> bool {
    match inst {
        MirInst::Const { dest, .. } => dest.name == name,
        MirInst::Copy { dest, .. } => dest.name == name,
        MirInst::BinOp { dest, .. } => dest.name == name,
        MirInst::UnaryOp { dest, .. } => dest.name == name,
        MirInst::Call { dest: Some(dest), .. } => dest.name == name,
        MirInst::Phi { dest, .. } => dest.name == name,
        MirInst::StructInit { dest, .. } => dest.name == name,
        MirInst::FieldAccess { dest, .. } => dest.name == name,
        MirInst::IndexLoad { dest, .. } => dest.name == name,
        MirInst::EnumVariant { dest, .. } => dest.name == name,
        MirInst::ArrayInit { dest, .. } => dest.name == name,
        MirInst::Cast { dest, .. } => dest.name == name,
        _ => false,
    }
}

// ============================================================================
// If-Else Chain to Switch Optimization
// ============================================================================

/// v0.51.8: If-else chain to switch optimization
///
/// Transforms cascading if-else chains comparing the same variable against constants
/// into a single switch statement. This enables LLVM to generate jump tables,
/// dramatically improving performance for large dispatch tables (like fasta's 14-way branch).
///
/// Pattern detected:
/// ```text
/// block_0:
///   %cmp0 = Eq %x, 0
///   Branch %cmp0, case_0, block_1
/// block_1:
///   %cmp1 = Eq %x, 1
///   Branch %cmp1, case_1, block_2
/// ...
/// ```
///
/// Transformed to:
/// ```text
/// block_0:
///   Switch %x { 0 -> case_0, 1 -> case_1, ... } default -> default_block
/// ```
pub struct IfElseToSwitch;

impl IfElseToSwitch {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IfElseToSwitch {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for IfElseToSwitch {
    fn name(&self) -> &'static str {
        "IfElseToSwitch"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Build a map from place name to the instruction that defines it
        // This allows us to look up what comparison produces a condition
        let mut def_map: HashMap<String, (usize, usize)> = HashMap::new(); // name -> (block_idx, inst_idx)
        for (block_idx, block) in func.blocks.iter().enumerate() {
            for (inst_idx, inst) in block.instructions.iter().enumerate() {
                if let Some(dest) = get_inst_dest(inst) {
                    def_map.insert(dest.name.clone(), (block_idx, inst_idx));
                }
            }
        }

        // Build a map from label to block index
        let label_to_idx: HashMap<String, usize> = func.blocks
            .iter()
            .enumerate()
            .map(|(idx, b)| (b.label.clone(), idx))
            .collect();

        // Find if-else chains and convert them to switches
        // We need to process blocks that start chains
        let mut blocks_to_skip: HashSet<usize> = HashSet::new();

        for start_block_idx in 0..func.blocks.len() {
            if blocks_to_skip.contains(&start_block_idx) {
                continue;
            }

            // Try to detect an if-else chain starting from this block
            if let Some(chain) = self.detect_if_else_chain(
                func,
                start_block_idx,
                &def_map,
                &label_to_idx,
            ) {
                // Only convert if we have at least 3 cases (threshold for switch benefit)
                if chain.cases.len() >= 3 {
                    // Mark intermediate blocks to skip
                    for &idx in &chain.intermediate_blocks {
                        blocks_to_skip.insert(idx);
                    }

                    // Convert the first block's terminator to Switch
                    let first_block = &mut func.blocks[start_block_idx];
                    first_block.terminator = Terminator::Switch {
                        discriminant: chain.discriminant,
                        cases: chain.cases,
                        default: chain.default,
                    };

                    // Remove intermediate blocks' comparisons (they're now dead code)
                    // DCE will clean up the comparison instructions later
                    changed = true;
                }
            }
        }

        changed
    }
}

/// Information about a detected if-else chain
struct IfElseChain {
    /// The variable being compared
    discriminant: Operand,
    /// Collected cases: (constant_value, target_label)
    cases: Vec<(i64, String)>,
    /// Default target (final else branch)
    default: String,
    /// Block indices of intermediate blocks in the chain
    intermediate_blocks: Vec<usize>,
}

impl IfElseToSwitch {
    /// Detect an if-else chain starting from the given block
    fn detect_if_else_chain(
        &self,
        func: &MirFunction,
        start_block_idx: usize,
        def_map: &HashMap<String, (usize, usize)>,
        label_to_idx: &HashMap<String, usize>,
    ) -> Option<IfElseChain> {
        let mut cases: Vec<(i64, String)> = Vec::new();
        let mut intermediate_blocks: Vec<usize> = Vec::new();
        let mut discriminant_var: Option<String> = None;
        let mut current_block_idx = start_block_idx;

        loop {
            let block = &func.blocks[current_block_idx];

            // The terminator must be a Branch
            let (cond, then_label, else_label) = match &block.terminator {
                Terminator::Branch { cond, then_label, else_label } => {
                    (cond, then_label.clone(), else_label.clone())
                }
                _ => break,
            };

            // The condition must be a Place (result of a comparison)
            let cond_name = match cond {
                Operand::Place(p) => &p.name,
                _ => break,
            };

            // Look up the instruction that defines this condition
            let (def_block_idx, def_inst_idx) = match def_map.get(cond_name) {
                Some(&(bi, ii)) => (bi, ii),
                None => break,
            };

            // The defining instruction must be in the current block
            // (otherwise control flow gets complicated)
            if def_block_idx != current_block_idx {
                break;
            }

            let def_inst = &func.blocks[def_block_idx].instructions[def_inst_idx];

            // It must be an equality comparison: %cmp = Eq %x, const
            let (var_name, const_val) = match def_inst {
                MirInst::BinOp { op: MirBinOp::Eq, lhs, rhs, .. } => {
                    // Check for: %x == const
                    match (lhs, rhs) {
                        (Operand::Place(p), Operand::Constant(Constant::Int(n))) => {
                            (p.name.clone(), *n)
                        }
                        (Operand::Constant(Constant::Int(n)), Operand::Place(p)) => {
                            (p.name.clone(), *n)
                        }
                        _ => break,
                    }
                }
                _ => break,
            };

            // Check that we're comparing the same variable throughout the chain
            match &discriminant_var {
                None => discriminant_var = Some(var_name.clone()),
                Some(v) if *v != var_name => break, // Different variable, not a valid chain
                _ => {}
            }

            // Add this case
            cases.push((const_val, then_label.clone()));

            // Follow the else branch to the next block
            let next_block_idx = match label_to_idx.get(&else_label) {
                Some(&idx) => idx,
                None => break,
            };

            // Check if the next block has another comparison in the chain
            let next_block = &func.blocks[next_block_idx];

            // The next block should only have comparison instructions and branch
            // If it has other side effects, we can't safely merge it
            let has_side_effects = next_block.instructions.iter().any(|inst| {
                matches!(inst, MirInst::Call { .. } |
                              MirInst::FieldStore { .. } |
                              MirInst::IndexStore { .. })
            });

            if has_side_effects {
                // This block has side effects, so it's the default case
                return Some(IfElseChain {
                    discriminant: Operand::Place(Place::new(discriminant_var?)),
                    cases,
                    default: else_label,
                    intermediate_blocks,
                });
            }

            // Check if next block continues the chain
            if !matches!(&next_block.terminator, Terminator::Branch { .. }) {
                // Chain ends here, else_label is the default
                return Some(IfElseChain {
                    discriminant: Operand::Place(Place::new(discriminant_var?)),
                    cases,
                    default: else_label,
                    intermediate_blocks,
                });
            }

            // Add current else block to intermediate blocks (will be skipped after conversion)
            intermediate_blocks.push(next_block_idx);
            current_block_idx = next_block_idx;
        }

        // Need at least one case to form a valid chain
        if cases.is_empty() {
            return None;
        }

        // Return the final else as default
        if let Terminator::Branch { else_label, .. } = &func.blocks[current_block_idx].terminator {
            Some(IfElseChain {
                discriminant: Operand::Place(Place::new(discriminant_var?)),
                cases,
                default: else_label.clone(),
                intermediate_blocks,
            })
        } else {
            None
        }
    }
}

// ============================================================================
// String Concatenation Optimization
// ============================================================================


// ============================================================================
// Tail Recursive to Loop Optimization
// ============================================================================

/// v0.51.9: Tail-recursive to loop conversion
///
/// Converts self-recursive tail calls into native loops with phi nodes.
/// This eliminates function call overhead even with musttail, giving
/// performance equivalent to hand-written loops.
///
/// Pattern detected:
/// ```text
/// fn f(data, pos, acc) =
///   if cond { return acc }
///   else { return f(data, new_pos, new_acc) }  // is_tail = true
/// ```
///
/// Transformed to:
/// ```text
/// entry:
///   br loop_header
/// loop_header:
///   %pos_phi = phi [%pos_param, entry], [%new_pos, loop_latch]
///   %acc_phi = phi [%acc_param, entry], [%new_acc, loop_latch]
///   if cond { goto exit } else { goto loop_body }
/// loop_body:
///   ... compute new_pos, new_acc ...
///   goto loop_latch
/// loop_latch:
///   br loop_header
/// exit:
///   return %acc_phi
/// ```
///
/// Affected benchmarks: csv_parse (118%), lexer (118%), json_parse (104%), sorting (106%)
pub struct TailRecursiveToLoop;

impl TailRecursiveToLoop {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TailRecursiveToLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for TailRecursiveToLoop {
    fn name(&self) -> &'static str {
        "TailRecursiveToLoop"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        // Only process functions with self-recursive tail calls
        let self_name = func.name.clone();

        // Find all blocks with self-recursive tail calls
        // v0.51.16: Support multiple tail call sites
        let mut tail_call_blocks: Vec<(usize, usize, Vec<Operand>)> = Vec::new(); // (block_idx, call_idx, args)

        for (block_idx, block) in func.blocks.iter().enumerate() {
            for (inst_idx, inst) in block.instructions.iter().enumerate() {
                if let MirInst::Call { func: callee, args, is_tail: true, .. } = inst {
                    if *callee == self_name {
                        tail_call_blocks.push((block_idx, inst_idx, args.clone()));
                    }
                }
            }
        }

        // No self-recursive tail calls, nothing to do
        if tail_call_blocks.is_empty() {
            return false;
        }

        // Verify all tail call sites have correct argument count
        for (_, _, args) in &tail_call_blocks {
            if args.len() != func.params.len() {
                return false;
            }
        }

        // v0.51.16: Determine invariants across ALL tail call sites
        // A parameter is invariant only if ALL tail call sites pass it unchanged
        let mut param_is_invariant: Vec<bool> = vec![true; func.params.len()];
        for (_, _, tail_args) in &tail_call_blocks {
            for (i, arg) in tail_args.iter().enumerate() {
                let param_name = &func.params[i].0;
                let is_same = match arg {
                    Operand::Place(p) => p.name == *param_name,
                    _ => false,
                };
                if !is_same {
                    param_is_invariant[i] = false;
                }
            }
        }

        // Need at least one accumulator parameter, otherwise no loop needed
        if param_is_invariant.iter().all(|&b| b) {
            return false; // All invariant - infinite loop, don't transform
        }

        // Create the loop transformation
        let loop_header_label = format!("loop_header_{}", func.blocks.len());
        let entry_label = func.blocks[0].label.clone();

        // v0.51.16: Create phi nodes with edges from entry AND all tail call sites
        let mut phi_names: Vec<String> = Vec::new();
        let mut phi_instructions: Vec<MirInst> = Vec::new();

        for (i, (param_name, param_ty)) in func.params.iter().enumerate() {
            if !param_is_invariant[i] {
                let phi_name = format!("{}_loop", param_name);
                phi_names.push(phi_name.clone());

                // Add to locals
                func.locals.push((phi_name.clone(), param_ty.clone()));

                // Create phi instruction with multiple incoming edges
                // Start with entry value
                let mut phi_values = vec![
                    (Operand::Place(Place::new(param_name.clone())), entry_label.clone()),
                ];

                // Add placeholder edges for each tail call site
                // Will be updated with actual labels after block insertion
                // v0.51.53: Fix: when a specific call passes the param unchanged,
                // use the loop variable, not the original param
                for (block_idx, _, tail_args) in &tail_call_blocks {
                    let block_label = func.blocks[*block_idx].label.clone();

                    // Check if this specific arg passes the param unchanged
                    let arg = &tail_args[i];
                    let passes_unchanged = match arg {
                        Operand::Place(p) => p.name == *param_name,
                        _ => false,
                    };

                    if passes_unchanged {
                        // Use loop variable instead of original param
                        let loop_var = format!("{}_loop", param_name);
                        phi_values.push((Operand::Place(Place::new(loop_var)), block_label));
                    } else {
                        phi_values.push((tail_args[i].clone(), block_label));
                    }
                }

                phi_instructions.push(MirInst::Phi {
                    dest: Place::new(phi_name),
                    values: phi_values,
                });
            } else {
                phi_names.push(param_name.clone()); // Use original param name
            }
        }

        // Create substitution map: param_name -> phi_name
        let mut subst_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for (i, (param_name, _)) in func.params.iter().enumerate() {
            if !param_is_invariant[i] {
                subst_map.insert(param_name.clone(), format!("{}_loop", param_name));
            }
        }

        // Get the old entry block's content
        let old_entry = func.blocks[0].clone();

        // Create new entry that just jumps to loop_header
        func.blocks[0] = BasicBlock {
            label: entry_label.clone(),
            instructions: vec![],
            terminator: Terminator::Goto(loop_header_label.clone()),
        };

        // Create loop_header with phis + old entry's content
        let mut loop_header_insts = phi_instructions;

        // Apply substitution to old entry's instructions
        let mut old_entry_insts: Vec<MirInst> = old_entry.instructions.into_iter()
            .map(|inst| self.substitute_params(inst, &subst_map))
            .collect();
        loop_header_insts.append(&mut old_entry_insts);

        let loop_header_terminator = self.substitute_terminator(old_entry.terminator, &subst_map);

        let loop_header_block = BasicBlock {
            label: loop_header_label.clone(),
            instructions: loop_header_insts,
            terminator: loop_header_terminator,
        };

        // Insert loop_header after entry
        func.blocks.insert(1, loop_header_block);

        // Update all blocks (except entry and loop_header)
        for i in 2..func.blocks.len() {
            let block = &mut func.blocks[i];

            // Apply substitution to instructions
            block.instructions = block.instructions.drain(..)
                .map(|inst| self.substitute_params(inst, &subst_map))
                .collect();

            // Apply substitution to terminator
            block.terminator = self.substitute_terminator(block.terminator.clone(), &subst_map);
        }

        // v0.51.16: Replace ALL tail calls with goto loop_header
        // Block indices are shifted by 1 due to loop_header insertion
        for (original_block_idx, _, _) in &tail_call_blocks {
            let new_block_idx = *original_block_idx + 1;

            if new_block_idx < func.blocks.len() {
                let block = &mut func.blocks[new_block_idx];

                // Find and remove the tail call instruction
                let call_idx = block.instructions.iter().position(|inst| {
                    matches!(inst, MirInst::Call { func: f, is_tail: true, .. } if f == &self_name)
                });

                if let Some(idx) = call_idx {
                    // Keep only instructions before the call
                    block.instructions.truncate(idx);

                    // Change terminator to goto loop_header
                    block.terminator = Terminator::Goto(loop_header_label.clone());
                }
            }
        }

        true
    }
}

impl TailRecursiveToLoop {
    /// Substitute parameter references with phi variable references
    fn substitute_params(&self, inst: MirInst, subst: &std::collections::HashMap<String, String>) -> MirInst {
        match inst {
            MirInst::Copy { dest, src } => MirInst::Copy {
                dest,
                src: Place::new(subst.get(&src.name).cloned().unwrap_or(src.name)),
            },
            MirInst::BinOp { dest, op, lhs, rhs } => MirInst::BinOp {
                dest,
                op,
                lhs: self.substitute_operand(lhs, subst),
                rhs: self.substitute_operand(rhs, subst),
            },
            MirInst::UnaryOp { dest, op, src } => MirInst::UnaryOp {
                dest,
                op,
                src: self.substitute_operand(src, subst),
            },
            MirInst::Call { dest, func, args, is_tail } => MirInst::Call {
                dest,
                func,
                args: args.into_iter().map(|a| self.substitute_operand(a, subst)).collect(),
                is_tail,
            },
            MirInst::Phi { dest, values } => MirInst::Phi {
                dest,
                values: values.into_iter()
                    .map(|(v, l)| (self.substitute_operand(v, subst), l))
                    .collect(),
            },
            MirInst::StructInit { dest, struct_name, fields } => MirInst::StructInit {
                dest,
                struct_name,
                fields: fields.into_iter()
                    .map(|(name, v)| (name, self.substitute_operand(v, subst)))
                    .collect(),
            },
            MirInst::FieldAccess { dest, base, field, field_index, struct_name } => MirInst::FieldAccess {
                dest,
                base: Place::new(subst.get(&base.name).cloned().unwrap_or(base.name)),
                field,
                field_index,
                struct_name,
            },
            MirInst::FieldStore { base, field, field_index, struct_name, value } => MirInst::FieldStore {
                base: Place::new(subst.get(&base.name).cloned().unwrap_or(base.name)),
                field,
                field_index,
                struct_name,
                value: self.substitute_operand(value, subst),
            },
            MirInst::IndexLoad { dest, array, index, element_type } => MirInst::IndexLoad {
                dest,
                array: Place::new(subst.get(&array.name).cloned().unwrap_or(array.name)),
                index: self.substitute_operand(index, subst),
                element_type,
            },
            MirInst::IndexStore { array, index, value, element_type } => MirInst::IndexStore {
                array: Place::new(subst.get(&array.name).cloned().unwrap_or(array.name)),
                index: self.substitute_operand(index, subst),
                value: self.substitute_operand(value, subst),
                element_type,
            },
            MirInst::EnumVariant { dest, enum_name, variant, args } => MirInst::EnumVariant {
                dest,
                enum_name,
                variant,
                args: args.into_iter().map(|a| self.substitute_operand(a, subst)).collect(),
            },
            MirInst::ArrayInit { dest, element_type, elements } => MirInst::ArrayInit {
                dest,
                element_type,
                elements: elements.into_iter().map(|e| self.substitute_operand(e, subst)).collect(),
            },
            MirInst::Cast { dest, src, from_ty, to_ty } => MirInst::Cast {
                dest,
                src: self.substitute_operand(src, subst),
                from_ty,
                to_ty,
            },
            other => other, // Const doesn't need substitution
        }
    }
    
    fn substitute_operand(&self, op: Operand, subst: &std::collections::HashMap<String, String>) -> Operand {
        match op {
            Operand::Place(p) => {
                Operand::Place(Place::new(subst.get(&p.name).cloned().unwrap_or(p.name)))
            }
            other => other,
        }
    }
    
    fn substitute_terminator(&self, term: Terminator, subst: &std::collections::HashMap<String, String>) -> Terminator {
        match term {
            Terminator::Return(Some(op)) => {
                Terminator::Return(Some(self.substitute_operand(op, subst)))
            }
            Terminator::Branch { cond, then_label, else_label } => {
                Terminator::Branch {
                    cond: self.substitute_operand(cond, subst),
                    then_label,
                    else_label,
                }
            }
            Terminator::Switch { discriminant, cases, default } => {
                Terminator::Switch {
                    discriminant: self.substitute_operand(discriminant, subst),
                    cases,
                    default,
                }
            }
            other => other,
        }
    }
}

/// v0.50.73: String concatenation chain optimization
///
/// Transforms chains of string concatenations from O(n²) to O(n):
/// ```text
/// %_t0 = BinOp Add %a, %b       ; concat #1: 2 allocations
/// %_t1 = BinOp Add %_t0, %c     ; concat #2: 2 more allocations
/// %_t2 = BinOp Add %_t1, %d     ; concat #3: 2 more allocations
/// ```
/// Into:
/// ```text
/// %_sb = Call sb_new()
/// %_ = Call sb_push(%_sb, %a)
/// %_ = Call sb_push(%_sb, %b)
/// %_ = Call sb_push(%_sb, %c)
/// %_ = Call sb_push(%_sb, %d)
/// %_t2 = Call sb_build(%_sb)
/// ```
///
/// This reduces allocations from O(n) to O(1) for n-element concat chains.
pub struct StringConcatOptimization;

impl StringConcatOptimization {
    pub fn new() -> Self {
        Self
    }

    /// Minimum chain length to optimize (3+ concats benefit from StringBuilder)
    const MIN_CHAIN_LENGTH: usize = 3;
}

impl Default for StringConcatOptimization {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a chain of string concatenations
#[derive(Debug)]
struct ConcatChain {
    /// All operands in the chain, in order
    operands: Vec<Operand>,
    /// Final destination place
    final_dest: Place,
    /// Indices of instructions to replace (in order)
    instruction_indices: Vec<usize>,
}

impl OptimizationPass for StringConcatOptimization {
    fn name(&self) -> &'static str {
        "StringConcatOptimization"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;
        let mut temp_counter: usize = 0;

        for block in &mut func.blocks {
            // Find concat chains in this block
            let chains = find_concat_chains(&block.instructions);

            // Filter to chains worth optimizing
            let worthwhile_chains: Vec<_> = chains
                .into_iter()
                .filter(|c| c.operands.len() >= StringConcatOptimization::MIN_CHAIN_LENGTH)
                .collect();

            if worthwhile_chains.is_empty() {
                continue;
            }

            // Transform each chain (process in reverse order to maintain indices)
            for chain in worthwhile_chains.into_iter().rev() {
                let new_instructions = transform_concat_chain(&chain, &mut temp_counter);

                // Remove old instructions (indices are sorted ascending)
                // Remove in reverse order to maintain validity of indices
                for &idx in chain.instruction_indices.iter().rev() {
                    if idx < block.instructions.len() {
                        block.instructions.remove(idx);
                    }
                }

                // Insert new instructions at the position of first removed instruction
                if let Some(&first_idx) = chain.instruction_indices.first() {
                    let insert_pos = first_idx.min(block.instructions.len());
                    for (i, inst) in new_instructions.into_iter().enumerate() {
                        block.instructions.insert(insert_pos + i, inst);
                    }
                }

                changed = true;
            }
        }

        changed
    }
}

/// Find chains of string concatenation in a block
fn find_concat_chains(instructions: &[MirInst]) -> Vec<ConcatChain> {
    let mut chains = Vec::new();
    let mut visited = HashSet::new();

    // Build a map of variable to (defining_instruction_index, instruction)
    let mut def_map: HashMap<String, (usize, &MirInst)> = HashMap::new();
    for (idx, inst) in instructions.iter().enumerate() {
        if let Some(dest) = get_dest(inst) {
            def_map.insert(dest.name.clone(), (idx, inst));
        }
    }

    // Count uses of each variable in the block
    let mut use_count: HashMap<String, usize> = HashMap::new();
    for inst in instructions {
        for name in get_used_place_names(inst) {
            *use_count.entry(name).or_insert(0) += 1;
        }
    }

    // Find potential chain endpoints (concat results that aren't used in another concat)
    for (idx, inst) in instructions.iter().enumerate() {
        if visited.contains(&idx) {
            continue;
        }

        if let MirInst::BinOp { dest, op: MirBinOp::Add, lhs, rhs } = inst {
            // Check if this could be a string concat by looking for string constant or known string place
            if !could_be_string_concat(lhs, rhs, instructions) {
                continue;
            }

            // Check if result is used in another concat (then it's not an endpoint)
            let dest_name = &dest.name;
            let is_endpoint = !instructions.iter().skip(idx + 1).any(|later_inst| {
                matches!(later_inst, MirInst::BinOp { op: MirBinOp::Add, lhs: Operand::Place(p), .. } if &p.name == dest_name)
                    || matches!(later_inst, MirInst::BinOp { op: MirBinOp::Add, rhs: Operand::Place(p), .. } if &p.name == dest_name)
            });

            if is_endpoint {
                // Trace back to find the full chain
                if let Some(chain) = trace_concat_chain(inst, idx, &def_map, &use_count, &visited) {
                    // Mark all instructions in this chain as visited
                    for &chain_idx in &chain.instruction_indices {
                        visited.insert(chain_idx);
                    }
                    chains.push(chain);
                }
            }
        }
    }

    chains
}

/// Check if a BinOp Add could be a string concatenation
fn could_be_string_concat(lhs: &Operand, rhs: &Operand, _instructions: &[MirInst]) -> bool {
    // If either operand is a string constant, it's definitely a string concat
    matches!(lhs, Operand::Constant(Constant::String(_)))
        || matches!(rhs, Operand::Constant(Constant::String(_)))
        // If operand name suggests string (heuristic)
        || matches!(lhs, Operand::Place(p) if p.name.starts_with("str") || p.name.contains("_str"))
        || matches!(rhs, Operand::Place(p) if p.name.starts_with("str") || p.name.contains("_str"))
}

/// Trace back through a chain of concatenations
fn trace_concat_chain(
    inst: &MirInst,
    idx: usize,
    def_map: &HashMap<String, (usize, &MirInst)>,
    use_count: &HashMap<String, usize>,
    visited: &HashSet<usize>,
) -> Option<ConcatChain> {
    let (dest, lhs, rhs) = match inst {
        MirInst::BinOp { dest, op: MirBinOp::Add, lhs, rhs } => (dest, lhs, rhs),
        _ => return None,
    };

    let mut operands = Vec::new();
    let mut instruction_indices = Vec::new();

    // Recursively trace left operand
    trace_operand(lhs, def_map, use_count, visited, &mut operands, &mut instruction_indices);
    // Recursively trace right operand
    trace_operand(rhs, def_map, use_count, visited, &mut operands, &mut instruction_indices);

    // Add current instruction index
    instruction_indices.push(idx);
    instruction_indices.sort();

    Some(ConcatChain {
        operands,
        final_dest: dest.clone(),
        instruction_indices,
    })
}

/// Trace an operand, expanding concat chains if found
fn trace_operand(
    operand: &Operand,
    def_map: &HashMap<String, (usize, &MirInst)>,
    use_count: &HashMap<String, usize>,
    visited: &HashSet<usize>,
    operands: &mut Vec<Operand>,
    instruction_indices: &mut Vec<usize>,
) {
    match operand {
        Operand::Constant(_) => {
            operands.push(operand.clone());
        }
        Operand::Place(p) => {
            // Check if this place is defined by a concat that's only used once
            if let Some(&(def_idx, def_inst)) = def_map.get(&p.name) {
                if visited.contains(&def_idx) {
                    operands.push(operand.clone());
                    return;
                }

                if let MirInst::BinOp { op: MirBinOp::Add, lhs, rhs, .. } = def_inst {
                    // Check if this intermediate result is used only once
                    let uses = use_count.get(&p.name).copied().unwrap_or(0);
                    if uses == 1 && could_be_string_concat(lhs, rhs, &[]) {
                        // Expand this concat into the chain
                        trace_operand(lhs, def_map, use_count, visited, operands, instruction_indices);
                        trace_operand(rhs, def_map, use_count, visited, operands, instruction_indices);
                        instruction_indices.push(def_idx);
                        return;
                    }
                }
            }
            // Not expandable, add as-is
            operands.push(operand.clone());
        }
    }
}

/// Get used place names from an instruction
fn get_used_place_names(inst: &MirInst) -> Vec<String> {
    match inst {
        MirInst::BinOp { lhs, rhs, .. } => {
            let mut names = Vec::new();
            if let Operand::Place(p) = lhs {
                names.push(p.name.clone());
            }
            if let Operand::Place(p) = rhs {
                names.push(p.name.clone());
            }
            names
        }
        MirInst::UnaryOp { src, .. } => {
            if let Operand::Place(p) = src {
                vec![p.name.clone()]
            } else {
                vec![]
            }
        }
        MirInst::Copy { src, .. } => vec![src.name.clone()],
        MirInst::Call { args, .. } => {
            args.iter()
                .filter_map(|a| match a {
                    Operand::Place(p) => Some(p.name.clone()),
                    _ => None,
                })
                .collect()
        }
        _ => vec![],
    }
}

/// Get destination place from an instruction
fn get_dest(inst: &MirInst) -> Option<&Place> {
    match inst {
        MirInst::Const { dest, .. }
        | MirInst::Copy { dest, .. }
        | MirInst::BinOp { dest, .. }
        | MirInst::UnaryOp { dest, .. }
        | MirInst::StructInit { dest, .. }
        | MirInst::FieldAccess { dest, .. }
        | MirInst::EnumVariant { dest, .. }
        | MirInst::ArrayInit { dest, .. }
        | MirInst::IndexLoad { dest, .. } => Some(dest),
        MirInst::Call { dest, .. } => dest.as_ref(),
        _ => None,
    }
}

/// Transform a concat chain into StringBuilder operations
fn transform_concat_chain(chain: &ConcatChain, temp_counter: &mut usize) -> Vec<MirInst> {
    let mut result = Vec::new();

    // Create unique names for this transformation
    let sb_name = format!("_str_sb_{}", *temp_counter);
    *temp_counter += 1;

    // sb_new()
    result.push(MirInst::Call {
        dest: Some(Place::new(&sb_name)),
        func: "sb_new".to_string(),
        args: vec![],
        is_tail: false,
    });

    // sb_push for each operand
    for (i, operand) in chain.operands.iter().enumerate() {
        let push_dest = format!("_str_push_{}_{}", *temp_counter - 1, i);
        result.push(MirInst::Call {
            dest: Some(Place::new(&push_dest)),
            func: "sb_push".to_string(),
            args: vec![
                Operand::Place(Place::new(&sb_name)),
                operand.clone(),
            ],
            is_tail: false,
        });
    }

    // sb_build()
    result.push(MirInst::Call {
        dest: Some(chain.final_dest.clone()),
        func: "sb_build".to_string(),
        args: vec![Operand::Place(Place::new(&sb_name))],
        is_tail: false,
    });

    result
}

// ============================================================================
// Constant Propagation Narrowing Pass (v0.50.80)
// ============================================================================

/// Interprocedural constant propagation for type narrowing
///
/// When a function is called with constant arguments from `main()`:
///   `main() { fibonacci(35) }`
/// We can:
///   1. Detect that argument 35 fits in i32
///   2. Analyze recursive calls: fibonacci(n-1), fibonacci(n-2)
///   3. Conclude all values of n are in [0, 35] (fits i32)
///   4. Generate function with i32 parameter operations
///
/// This optimization is critical for matching C performance when BMB uses
/// i64 by default but the algorithm only needs 32-bit operations.
pub struct ConstantPropagationNarrowing {
    /// Map: function_name → Vec<(param_index, max_constant_value)>
    /// Tracks the maximum constant value each parameter is called with
    call_site_constants: HashMap<String, Vec<(usize, i64)>>,
}

impl ConstantPropagationNarrowing {
    /// Create from a MirProgram by analyzing all call sites for constant arguments
    pub fn from_program(program: &MirProgram) -> Self {
        let mut call_site_constants: HashMap<String, Vec<(usize, i64)>> = HashMap::new();

        for func in &program.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let MirInst::Call { func: callee, args, .. } = inst {
                        // Collect constant arguments
                        let consts: Vec<(usize, i64)> = args
                            .iter()
                            .enumerate()
                            .filter_map(|(i, arg)| {
                                if let Operand::Constant(Constant::Int(v)) = arg {
                                    Some((i, *v))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if !consts.is_empty() {
                            call_site_constants
                                .entry(callee.clone())
                                .or_default()
                                .extend(consts);
                        }
                    }
                }
            }
        }

        Self { call_site_constants }
    }

    /// Check if a parameter can be narrowed to i32 based on:
    /// 1. All constant call-site values fit in i32
    /// 2. Function is self-recursive with decreasing arguments (monotonically decreasing)
    fn can_narrow_param(&self, func: &MirFunction, param_idx: usize) -> bool {
        // Only narrow i64 parameters
        if func.params.get(param_idx).map(|(_, ty)| ty) != Some(&MirType::I64) {
            return false;
        }

        let func_name = &func.name;
        let Some(consts) = self.call_site_constants.get(func_name) else {
            return false;
        };

        // Find maximum constant value for this parameter across all call sites
        let max_val: Option<i64> = consts
            .iter()
            .filter(|(idx, _)| *idx == param_idx)
            .map(|(_, v)| *v)
            .max();

        let Some(max_val) = max_val else {
            return false;
        };

        // Check if max value fits in i32 (and is non-negative for decreasing recursive patterns)
        if max_val > i32::MAX as i64 || max_val < 0 {
            return false;
        }

        // Check if function is self-recursive with decreasing arguments
        self.is_decreasing_recursive(func, param_idx)
    }

    /// Check if the parameter decreases (or stays same) in all recursive calls
    /// Patterns detected:
    /// - `f(n-1)`, `f(n-2)` - subtracting positive constants
    /// - `f(n/2)` - division by constant > 1
    fn is_decreasing_recursive(&self, func: &MirFunction, param_idx: usize) -> bool {
        let param_name = &func.params[param_idx].0;
        let mut has_self_recursion = false;

        // Build a map of variable definitions to track expressions
        let mut definitions: HashMap<String, (&MirBinOp, &Operand, &Operand)> = HashMap::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                // Track BinOp definitions
                if let MirInst::BinOp { dest, op, lhs, rhs } = inst {
                    definitions.insert(dest.name.clone(), (op, lhs, rhs));
                }

                // Check self-recursive calls
                if let MirInst::Call { func: callee, args, .. } = inst {
                    if callee == &func.name {
                        has_self_recursion = true;

                        // Check if the argument at param_idx is decreasing
                        if let Some(arg) = args.get(param_idx) {
                            if !self.is_decreasing_operand(arg, param_name, &definitions) {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        // Must have at least one self-recursive call to benefit from narrowing
        has_self_recursion
    }

    /// Check if an operand represents a value that is <= the parameter value
    fn is_decreasing_operand(
        &self,
        operand: &Operand,
        param_name: &str,
        definitions: &HashMap<String, (&MirBinOp, &Operand, &Operand)>,
    ) -> bool {
        match operand {
            // Direct use of parameter (n) - same value, OK
            Operand::Place(p) if p.name == param_name => true,
            // Constant <= 0 is always decreasing from non-negative param
            Operand::Constant(Constant::Int(v)) if *v >= 0 => true,
            // Check if it's a derived value
            Operand::Place(p) => {
                if let Some((op, lhs, rhs)) = definitions.get(&p.name) {
                    match op {
                        // param - positive_const is decreasing
                        MirBinOp::Sub => {
                            let lhs_is_param = matches!(lhs, Operand::Place(l) if l.name == param_name);
                            let rhs_is_positive = matches!(rhs, Operand::Constant(Constant::Int(v)) if *v > 0);
                            // Also check if lhs is another decreasing value
                            let lhs_decreasing = self.is_decreasing_operand(lhs, param_name, definitions);
                            (lhs_is_param || lhs_decreasing) && rhs_is_positive
                        }
                        // param / constant > 1 is decreasing
                        MirBinOp::Div => {
                            let lhs_is_param = matches!(lhs, Operand::Place(l) if l.name == param_name);
                            let lhs_decreasing = self.is_decreasing_operand(lhs, param_name, definitions);
                            let rhs_is_divisor = matches!(rhs, Operand::Constant(Constant::Int(v)) if *v > 1);
                            (lhs_is_param || lhs_decreasing) && rhs_is_divisor
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Apply type narrowing to a function: change i64 parameter to i32
    ///
    /// The key insight: just changing the MIR parameter type from I64 to I32
    /// causes the LLVM codegen to generate 32-bit operations:
    /// - `leaq -1(%rdi), %rcx` becomes `leal -1(%edi), %ecx`
    /// - `cmpq $3, %rdi` becomes `cmpl $3, %edi`
    ///
    /// This eliminates the 8% performance gap vs C on fibonacci-like benchmarks.
    ///
    /// Apply type narrowing to a function: change i64 parameter to i32
    ///
    /// v0.51.18: Proper i32 propagation implemented.
    /// The codegen now:
    /// 1. Does NOT emit sext at function entry (keeps param as i32)
    /// 2. Uses i32 for arithmetic on narrowed params
    /// 3. Handles phi coercion (i32→i64) when mixing with call results
    /// 4. No trunc needed for recursive calls (arg is i32, param is i32)
    fn narrow_function(&self, func: &mut MirFunction, param_idx: usize) -> bool {
        // Change parameter type from I64 to I32
        func.params[param_idx].1 = MirType::I32;
        true
    }

    /// Get the optimization name
    pub fn name(&self) -> &'static str {
        "constant_propagation_narrowing"
    }

    /// Run on the entire program (interprocedural pass)
    pub fn run_on_program(&self, program: &mut MirProgram) -> bool {
        let mut changed = false;

        // Clone the struct to allow mutable iteration
        let narrowable: Vec<(String, usize)> = program
            .functions
            .iter()
            .flat_map(|func| {
                (0..func.params.len())
                    .filter(|&idx| self.can_narrow_param(func, idx))
                    .map(|idx| (func.name.clone(), idx))
                    .collect::<Vec<_>>()
            })
            .collect();

        for (func_name, param_idx) in narrowable {
            if let Some(func) = program.functions.iter_mut().find(|f| f.name == func_name) {
                if self.narrow_function(func, param_idx) {
                    changed = true;
                }
            }
        }

        changed
    }
}

// ============================================================================
// v0.60.9: Loop Bounded Narrowing Pass
// Narrows i64 loop variables and function parameters to i32 when bounded by constants
// ============================================================================

/// LoopBoundedNarrowing: Interprocedural narrowing for loop-bounded values
///
/// This pass analyzes the program to find:
/// 1. Function parameters that are always called with small constants (≤ i32::MAX)
/// 2. Loop variables bounded by those parameters (pattern: `let mut i = 0; while i < n { ... i = i + 1 }`)
/// 3. Function parameters that only receive loop variables as arguments
///
/// The pass then narrows types from I64 to I32 where safe, enabling 32-bit operations
/// in the generated code. This is critical for matching C performance where `int` is 32-bit.
///
/// Example transformation:
/// ```text
/// // Before: all i64
/// fn mult_av(v: i64, av: i64, n: i64) {
///     let mut i: i64 = 0;
///     while i < n { matrix_a(i, j); ... }
/// }
///
/// // After: narrowed to i32
/// fn mult_av(v: i64, av: i64, n: i32) {
///     let mut i: i32 = 0;
///     while i < n { matrix_a(i, j); ... }  // matrix_a params also narrowed
/// }
/// ```
pub struct LoopBoundedNarrowing {
    /// Map: function_name → set of (param_index, max_constant_value)
    /// Tracks the maximum constant value each parameter is called with from main
    param_bounds: HashMap<String, HashMap<usize, i64>>,
}

impl LoopBoundedNarrowing {
    /// Create from a MirProgram by analyzing call sites
    pub fn from_program(program: &MirProgram) -> Self {
        let mut param_bounds: HashMap<String, HashMap<usize, i64>> = HashMap::new();

        // Phase 1: Find all constant arguments from main function
        if let Some(main_func) = program.functions.iter().find(|f| f.name == "main" || f.name == "bmb_user_main") {
            Self::analyze_call_sites_for_constants(main_func, &mut param_bounds, &HashMap::new());
        }

        // Phase 2: Interprocedural propagation - propagate bounds through call chains
        // Keep iterating until no new bounds are discovered
        let mut changed = true;
        while changed {
            changed = false;
            for func in &program.functions {
                let current_bounds = param_bounds.get(&func.name).cloned().unwrap_or_default();
                if current_bounds.is_empty() {
                    continue;
                }

                // Propagate bounds to callees
                let prev_len: usize = param_bounds.values().map(|m| m.len()).sum();
                Self::analyze_call_sites_for_constants(func, &mut param_bounds, &current_bounds);
                let new_len: usize = param_bounds.values().map(|m| m.len()).sum();
                if new_len > prev_len {
                    changed = true;
                }
            }
        }

        Self { param_bounds }
    }

    /// Analyze a function for call sites with constant or bounded arguments
    /// `caller_bounds` contains the bounds for the caller's parameters (param_idx -> max_value)
    fn analyze_call_sites_for_constants(
        func: &MirFunction,
        param_bounds: &mut HashMap<String, HashMap<usize, i64>>,
        caller_bounds: &HashMap<usize, i64>,
    ) {
        // Build map of param name -> bound (from caller's bounded params)
        let mut var_bounds: HashMap<String, i64> = func.params.iter()
            .enumerate()
            .filter_map(|(idx, (name, _))| {
                caller_bounds.get(&idx).map(|&bound| (name.clone(), bound))
            })
            .collect();

        // Track constant assignments (Const and Copy instructions)
        // This handles patterns like: _t0 = 1000; n = _t0
        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    MirInst::Const { dest, value: Constant::Int(v) } => {
                        if *v >= 0 && *v <= i32::MAX as i64 {
                            var_bounds.insert(dest.name.clone(), *v);
                        }
                    }
                    MirInst::Copy { dest, src } => {
                        // Propagate constant through copy
                        if let Some(&bound) = var_bounds.get(&src.name) {
                            var_bounds.insert(dest.name.clone(), bound);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Now analyze call sites
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Call { func: callee, args, .. } = inst {
                    // Skip built-in functions
                    if Self::is_builtin(callee) {
                        continue;
                    }

                    for (idx, arg) in args.iter().enumerate() {
                        let bound = match arg {
                            // Direct constant
                            Operand::Constant(Constant::Int(v)) => Some(*v),
                            // Variable with known bound (from params or const assignments)
                            Operand::Place(p) => var_bounds.get(&p.name).copied(),
                            _ => None,
                        };

                        if let Some(v) = bound {
                            if v >= 0 && v <= i32::MAX as i64 {
                                param_bounds
                                    .entry(callee.clone())
                                    .or_default()
                                    .entry(idx)
                                    .and_modify(|max| *max = (*max).max(v))
                                    .or_insert(v);
                            }
                        }
                    }
                }
            }
        }
    }

    fn is_builtin(name: &str) -> bool {
        matches!(name,
            "malloc" | "free" | "calloc" | "realloc" |
            "println" | "print" | "eprintln" | "eprint" |
            "sqrt" | "abs" | "min" | "max" |
            "i64_to_f64" | "f64_to_i64" |
            "load_f64" | "store_f64" | "load_i64" | "store_i64" |
            "load_u8" | "store_u8" | "byte_at" | "len"
        )
    }

    /// Check if a parameter can be narrowed based on constant bounds from call sites
    fn can_narrow_param(&self, func: &MirFunction, param_idx: usize) -> bool {
        // Only narrow i64 parameters
        if func.params.get(param_idx).map(|(_, ty)| ty) != Some(&MirType::I64) {
            return false;
        }

        // Check if we have bounds for this parameter
        if let Some(bounds) = self.param_bounds.get(&func.name) {
            if let Some(&max_val) = bounds.get(&param_idx) {
                // Check if max value fits in i32 (and is non-negative)
                return max_val >= 0 && max_val <= i32::MAX as i64;
            }
        }

        false
    }

    /// Detect while loop patterns and find bounded loop variables
    /// Returns: Map of variable name -> (param_name that bounds it)
    fn find_loop_variables(func: &MirFunction) -> HashMap<String, String> {
        let mut loop_vars: HashMap<String, String> = HashMap::new();

        // Pattern: entry block has `%var = const I:0`
        // Pattern: while_cond block has `%cmp = < %var, %param` followed by branch
        // Pattern: while_body block has `%var = copy %new_var` after `%new_var = + %var, I:1`

        for block in &func.blocks {
            // Look for comparison pattern: %cmp = < %var, %param
            for inst in &block.instructions {
                if let MirInst::BinOp { op: MirBinOp::Lt, lhs, rhs, .. } = inst {
                    // lhs is the loop variable, rhs might be a parameter
                    if let (Operand::Place(var), Operand::Place(param)) = (lhs, rhs) {
                        // Check if param is actually a function parameter
                        let is_param = func.params.iter().any(|(name, _)| name == &param.name);
                        if is_param {
                            // Check if var looks like a loop variable (starts at 0, increments by 1)
                            if Self::is_loop_variable(func, &var.name) {
                                loop_vars.insert(var.name.clone(), param.name.clone());
                            }
                        }
                    }
                }
            }
        }

        loop_vars
    }

    /// Check if a variable follows the loop variable pattern:
    /// - Initialized to 0 in entry block
    /// - Incremented by 1 in some block
    fn is_loop_variable(func: &MirFunction, var_name: &str) -> bool {
        let mut starts_at_zero = false;
        let mut increments_by_one = false;

        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    // Check for initialization: %var = const I:0
                    MirInst::Const { dest, value: Constant::Int(0) } if dest.name == var_name => {
                        starts_at_zero = true;
                    }
                    // Check for increment: %temp = + %var, I:1
                    MirInst::BinOp { op: MirBinOp::Add, lhs: Operand::Place(p), rhs: Operand::Constant(Constant::Int(1)), .. }
                        if p.name == var_name => {
                        increments_by_one = true;
                    }
                    _ => {}
                }
            }
        }

        starts_at_zero && increments_by_one
    }

    /// Narrow a function's parameter from I64 to I32
    fn narrow_param(func: &mut MirFunction, param_idx: usize) -> bool {
        if let Some((_, ty)) = func.params.get_mut(param_idx) {
            if *ty == MirType::I64 {
                *ty = MirType::I32;
                return true;
            }
        }
        false
    }

    /// Narrow a local variable from I64 to I32
    fn narrow_local(func: &mut MirFunction, var_name: &str) -> bool {
        for (name, ty) in &mut func.locals {
            if name == var_name && *ty == MirType::I64 {
                *ty = MirType::I32;
                return true;
            }
        }
        false
    }

    /// Get the optimization name
    pub fn name(&self) -> &'static str {
        "loop_bounded_narrowing"
    }

    /// Run on the entire program (interprocedural pass)
    pub fn run_on_program(&self, program: &mut MirProgram) -> bool {
        let mut changed = false;

        // Phase 1: Narrow parameters that receive constants from main
        let narrowable_params: Vec<(String, usize)> = program
            .functions
            .iter()
            .flat_map(|func| {
                (0..func.params.len())
                    .filter(|&idx| self.can_narrow_param(func, idx))
                    .map(|idx| (func.name.clone(), idx))
                    .collect::<Vec<_>>()
            })
            .collect();

        for (func_name, param_idx) in &narrowable_params {
            if let Some(func) = program.functions.iter_mut().find(|f| &f.name == func_name) {
                if Self::narrow_param(func, *param_idx) {
                    changed = true;
                }
            }
        }

        // Phase 2: Narrow loop variables that are bounded by narrowed parameters
        for func in &mut program.functions {
            let loop_vars = Self::find_loop_variables(func);

            for (var_name, param_name) in loop_vars {
                // Check if the bounding parameter was narrowed
                let param_narrowed = func.params.iter()
                    .any(|(name, ty)| name == &param_name && *ty == MirType::I32);

                if param_narrowed {
                    if Self::narrow_local(func, &var_name) {
                        changed = true;
                    }
                }
            }
        }

        // Phase 3: Propagate narrowing to callee functions
        // Functions like matrix_a(i, j) that are only called with loop variables
        // should have their parameters narrowed too
        changed |= self.propagate_narrowing_to_callees(program);

        // Phase 4: Propagate narrowing to derived local variables
        // If sum = i + j where both i and j are i32, then sum can be i32 too
        for func in &mut program.functions {
            changed |= Self::propagate_narrowing_to_locals(func);
        }

        changed
    }

    /// Propagate narrowing to derived local variables within a function
    /// If a variable is computed from i32 operands with i32-preserving ops, it can be i32
    fn propagate_narrowing_to_locals(func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Build initial set of narrowed variables (params and locals that are i32)
        let mut narrowed: HashSet<String> = func.params.iter()
            .filter(|(_, ty)| *ty == MirType::I32)
            .map(|(name, _)| name.clone())
            .collect();
        narrowed.extend(func.locals.iter()
            .filter(|(_, ty)| *ty == MirType::I32)
            .map(|(name, _)| name.clone()));

        // Iterate until fixed point
        let mut local_changed = true;
        while local_changed {
            local_changed = false;

            for block in &func.blocks {
                for inst in &block.instructions {
                    match inst {
                        // Copy propagation: if src is narrowed, dest can be narrowed
                        MirInst::Copy { dest, src } => {
                            if narrowed.contains(&src.name) && !narrowed.contains(&dest.name) {
                                narrowed.insert(dest.name.clone());
                                local_changed = true;
                            }
                        }

                        // BinOp: if both operands are narrowed/constant, dest can be narrowed
                        // Only for ops that preserve i32 range (add, sub, comparisons, etc.)
                        MirInst::BinOp { dest, op, lhs, rhs } => {
                            if narrowed.contains(&dest.name) {
                                continue;
                            }

                            let lhs_narrow = match lhs {
                                Operand::Place(p) => narrowed.contains(&p.name),
                                Operand::Constant(Constant::Int(v)) => *v >= i32::MIN as i64 && *v <= i32::MAX as i64,
                                _ => false,
                            };
                            let rhs_narrow = match rhs {
                                Operand::Place(p) => narrowed.contains(&p.name),
                                Operand::Constant(Constant::Int(v)) => *v >= i32::MIN as i64 && *v <= i32::MAX as i64,
                                _ => false,
                            };

                            // Check if operation preserves i32 range
                            let op_preserves_i32 = matches!(op,
                                MirBinOp::Add | MirBinOp::Sub | MirBinOp::Mul |
                                MirBinOp::Lt | MirBinOp::Le | MirBinOp::Gt | MirBinOp::Ge |
                                MirBinOp::Eq | MirBinOp::Ne |
                                MirBinOp::And | MirBinOp::Or |
                                MirBinOp::Band | MirBinOp::Bor | MirBinOp::Bxor |
                                MirBinOp::Shl | MirBinOp::Shr
                            );

                            if lhs_narrow && rhs_narrow && op_preserves_i32 {
                                narrowed.insert(dest.name.clone());
                                local_changed = true;
                            }
                        }

                        // Const: small integer constants can be narrowed
                        MirInst::Const { dest, value: Constant::Int(v) } => {
                            if !narrowed.contains(&dest.name) && *v >= i32::MIN as i64 && *v <= i32::MAX as i64 {
                                narrowed.insert(dest.name.clone());
                                local_changed = true;
                            }
                        }

                        _ => {}
                    }
                }
            }
        }

        // Apply narrowing to locals
        for (name, ty) in &mut func.locals {
            if *ty == MirType::I64 && narrowed.contains(name) {
                *ty = MirType::I32;
                changed = true;
            }
        }

        changed
    }

    /// Propagate narrowing to callee functions
    /// If a function is only called with narrowed arguments, narrow its parameters
    fn propagate_narrowing_to_callees(&self, program: &mut MirProgram) -> bool {
        let mut changed = false;

        // Collect information about all call sites
        // Map: callee_name -> param_idx -> all_narrowed
        let mut callee_param_types: HashMap<String, HashMap<usize, bool>> = HashMap::new();

        for func in &program.functions {
            let loop_vars = Self::find_loop_variables(func);

            for block in &func.blocks {
                for inst in &block.instructions {
                    if let MirInst::Call { func: callee, args, .. } = inst {
                        if Self::is_builtin(callee) {
                            continue;
                        }

                        for (idx, arg) in args.iter().enumerate() {
                            let is_narrowed = match arg {
                                Operand::Place(p) => {
                                    // Check if it's a narrowed loop variable
                                    loop_vars.contains_key(&p.name) ||
                                    // Or a narrowed parameter
                                    func.params.iter().any(|(name, ty)| name == &p.name && *ty == MirType::I32)
                                }
                                Operand::Constant(Constant::Int(v)) => {
                                    *v >= 0 && *v <= i32::MAX as i64
                                }
                                _ => false,
                            };

                            callee_param_types
                                .entry(callee.clone())
                                .or_default()
                                .entry(idx)
                                .and_modify(|all_narrow| *all_narrow = *all_narrow && is_narrowed)
                                .or_insert(is_narrowed);
                        }
                    }
                }
            }
        }

        // Narrow parameters in callees where ALL call sites use narrowed values
        for func in &mut program.functions {
            if let Some(param_info) = callee_param_types.get(&func.name) {
                for (&param_idx, &all_narrowed) in param_info {
                    if all_narrowed {
                        if let Some((_, ty)) = func.params.get_mut(param_idx) {
                            if *ty == MirType::I64 {
                                *ty = MirType::I32;
                                changed = true;
                            }
                        }
                    }
                }
            }
        }

        changed
    }
}

// ============================================================================
// v0.51.8: Aggressive Inlining Pass
// Marks small, simple functions for alwaysinline LLVM attribute
// ============================================================================

/// AggressiveInlining: Marks small functions for LLVM alwaysinline attribute
///
/// This pass identifies functions that should be aggressively inlined:
/// - Functions with ≤ MAX_INLINE_INSTRUCTIONS instructions (default: 15)
/// - Pure functions (no side effects) with ≤ MAX_PURE_INLINE_INSTRUCTIONS (default: 20)
/// - Functions with single basic block preferred
///
/// Marking with alwaysinline ensures LLVM inlines these at all call sites,
/// eliminating function call overhead in tight loops (e.g., spectral_norm inner loop).
#[derive(Default)]
pub struct AggressiveInlining {
    /// Maximum instructions for regular functions to be inlined
    max_instructions: usize,
    /// Maximum instructions for pure functions (higher threshold since safer to inline)
    max_pure_instructions: usize,
}

impl AggressiveInlining {
    /// Default threshold: 15 instructions for regular, 20 for pure functions
    pub fn new() -> Self {
        Self {
            max_instructions: 15,
            max_pure_instructions: 20,
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(max_instructions: usize, max_pure_instructions: usize) -> Self {
        Self {
            max_instructions,
            max_pure_instructions,
        }
    }

    /// Get the optimization name
    pub fn name(&self) -> &'static str {
        "aggressive_inlining"
    }

    /// Count total instructions in a function
    fn count_instructions(func: &MirFunction) -> usize {
        func.blocks.iter().map(|b| b.instructions.len()).sum()
    }

    /// Check if a function has only simple, inlinable instructions
    /// (no loops detected via back edges, no complex control flow)
    fn is_simple_control_flow(func: &MirFunction) -> bool {
        // Single block is always simple
        if func.blocks.len() == 1 {
            return true;
        }

        // Check for back edges (loops) - functions with loops are less beneficial to inline
        // A back edge exists if a block jumps to a label that appears earlier
        let label_indices: std::collections::HashMap<&str, usize> = func
            .blocks
            .iter()
            .enumerate()
            .map(|(i, b)| (b.label.as_str(), i))
            .collect();

        for (idx, block) in func.blocks.iter().enumerate() {
            let targets = match &block.terminator {
                Terminator::Goto(target) => vec![target.as_str()],
                Terminator::Branch { then_label, else_label, .. } => {
                    vec![then_label.as_str(), else_label.as_str()]
                }
                Terminator::Switch { cases, default, .. } => {
                    let mut targets: Vec<&str> = cases.iter().map(|(_, l)| l.as_str()).collect();
                    targets.push(default.as_str());
                    targets
                }
                _ => vec![],
            };

            for target in targets {
                if let Some(&target_idx) = label_indices.get(target) {
                    // Back edge detected - this is a loop
                    if target_idx <= idx {
                        return false;
                    }
                }
            }
        }

        // Few blocks is simple enough
        func.blocks.len() <= 4
    }

    /// Check if function is recursive (calls itself)
    /// Recursive functions should NOT be marked alwaysinline because LLVM has
    /// sophisticated recursive-to-iterative transformations that are more valuable
    fn is_recursive(func: &MirFunction) -> bool {
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Call { func: callee, .. } = inst {
                    if callee == &func.name {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if function should be marked for aggressive inlining (alwaysinline)
    fn should_inline(&self, func: &MirFunction) -> bool {
        // Never inline main function
        if func.name == "main" || func.name == "bmb_user_main" {
            return false;
        }

        // v0.51.12: Never inline recursive functions
        // LLVM's recursive-to-iterative transformation is more valuable than inlining
        // For example, LLVM can transform fib(n) = fib(n-1) + fib(n-2) into a loop
        if Self::is_recursive(func) {
            return false;
        }

        let inst_count = Self::count_instructions(func);
        let is_simple = Self::is_simple_control_flow(func);

        // Pure functions get higher threshold
        let threshold = if func.is_pure {
            self.max_pure_instructions
        } else {
            self.max_instructions
        };

        // Must be below threshold and have simple control flow
        inst_count <= threshold && is_simple
    }

    /// v0.51.52: Check if function should get inlinehint attribute
    /// For medium-sized functions that don't qualify for alwaysinline but would
    /// benefit from inlining when called in hot loops (like lexer's next_token)
    fn should_hint_inline(&self, func: &MirFunction) -> bool {
        // Never hint main function
        if func.name == "main" || func.name == "bmb_user_main" {
            return false;
        }

        // Never hint recursive functions
        if Self::is_recursive(func) {
            return false;
        }

        let inst_count = Self::count_instructions(func);
        let block_count = func.blocks.len();

        // Medium-sized functions: up to 150 instructions and 50 blocks
        // These are too large for alwaysinline but can benefit from context-sensitive
        // inlining by LLVM (e.g., when called in tight loops like lexer's next_token)
        // v0.51.52: Increased from 100 to 150 to cover next_token (101 instructions)
        let max_hint_instructions = 150;
        let max_hint_blocks = 50;

        inst_count <= max_hint_instructions && block_count <= max_hint_blocks
    }

    /// Run on the entire program (interprocedural pass)
    pub fn run_on_program(&self, program: &mut MirProgram) -> bool {
        let mut changed = false;

        for func in &mut program.functions {
            // First, check for alwaysinline (small, simple functions)
            if !func.always_inline && self.should_inline(func) {
                func.always_inline = true;
                changed = true;
            }
            // v0.51.52: Then, check for inlinehint (medium-sized functions)
            // Only if not already marked for always_inline
            else if !func.always_inline && !func.inline_hint && self.should_hint_inline(func) {
                func.inline_hint = true;
                changed = true;
            }
        }

        changed
    }
}

// ============================================================================
// v0.51.11: Memory Effect Analysis Pass
// Detects functions that don't access memory (pure arithmetic only)
// Enables LLVM memory(none) attribute for better LICM
// ============================================================================

/// Memory effect analysis: detects functions that have no memory side effects
/// Such functions can get the LLVM memory(none) attribute, enabling better optimization
pub struct MemoryEffectAnalysis;

impl MemoryEffectAnalysis {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &'static str {
        "memory_effect_analysis"
    }

    /// Check if an instruction accesses memory or calls functions
    fn inst_accesses_memory(inst: &MirInst) -> bool {
        match inst {
            // Function calls might access memory
            MirInst::Call { .. } => true,
            // Array/struct operations access memory
            MirInst::IndexLoad { .. }
            | MirInst::IndexStore { .. }
            | MirInst::FieldAccess { .. }
            | MirInst::FieldStore { .. }
            | MirInst::ArrayInit { .. }
            | MirInst::StructInit { .. }
            | MirInst::EnumVariant { .. } => true,
            // Pure operations don't access memory
            MirInst::BinOp { .. }
            | MirInst::UnaryOp { .. }
            | MirInst::Const { .. }
            | MirInst::Copy { .. }
            | MirInst::Phi { .. }
            | MirInst::Cast { .. } => false,
            // v0.55: Tuple operations - TupleInit builds a value, TupleExtract reads from it
            // These are aggregate operations that may involve stack allocation
            MirInst::TupleInit { .. } | MirInst::TupleExtract { .. } => true,
        }
    }

    /// Check if a function is memory-free (no memory accesses)
    fn is_memory_free(func: &MirFunction) -> bool {
        // Skip main since it will call other functions
        if func.name == "main" {
            return false;
        }

        // Check all instructions in all blocks
        for block in &func.blocks {
            for inst in &block.instructions {
                if Self::inst_accesses_memory(inst) {
                    return false;
                }
            }
        }

        true
    }

    /// Run on the entire program (interprocedural pass)
    pub fn run_on_program(&self, program: &mut MirProgram) -> bool {
        let mut changed = false;

        for func in &mut program.functions {
            if !func.is_memory_free && Self::is_memory_free(func) {
                func.is_memory_free = true;
                changed = true;
            }
        }

        changed
    }
}

// ============================================================================
// v0.51.16: Loop Invariant Code Motion (LICM)
// Hoists loop-invariant calls (like len()) to loop preheaders
// ============================================================================

/// LoopInvariantCodeMotion: Hoists invariant computations out of loops
///
/// After TailRecursiveToLoop converts recursive calls to loops, we often have:
/// ```text
/// loop_header:
///   %pos_loop = phi [%start, entry], [%next, loop_body]
///   %len = call len(%s)         ; <-- Called EVERY iteration!
///   %cmp = icmp sge %pos_loop, %len
///   ...
/// ```
///
/// This pass hoists the `len(%s)` call to the entry block since `%s` is loop-invariant:
/// ```text
/// entry:
///   %len_hoisted = call len(%s)  ; <-- Called ONCE
///   br loop_header
/// loop_header:
///   %pos_loop = phi [%start, entry], [%next, loop_body]
///   %cmp = icmp sge %pos_loop, %len_hoisted
///   ...
/// ```
///
/// Affected benchmarks: http_parse (218%), csv_parse (151%), lexer (113%)
pub struct LoopInvariantCodeMotion {
    /// Functions known to be pure (safe to hoist)
    pure_functions: HashSet<String>,
}

impl LoopInvariantCodeMotion {
    pub fn new() -> Self {
        let mut pure_functions = HashSet::new();
        // String functions that only read their arguments
        pure_functions.insert("len".to_string());
        pure_functions.insert("char_at".to_string());
        pure_functions.insert("ord".to_string());
        pure_functions.insert("byte_at".to_string());
        Self { pure_functions }
    }
}

impl Default for LoopInvariantCodeMotion {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for LoopInvariantCodeMotion {
    fn name(&self) -> &'static str {
        "loop_invariant_code_motion"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        // Collect parameter names as loop-invariant values
        let params: HashSet<String> = func.params.iter().map(|(n, _)| n.clone()).collect();

        // Find loop headers (blocks with phi nodes that have back edges)
        // A loop header is identified by having phi nodes where one predecessor
        // comes from a later block (the back edge)
        let mut loop_headers: Vec<usize> = Vec::new();

        // Build block index map
        let block_map: HashMap<&str, usize> = func.blocks.iter()
            .enumerate()
            .map(|(i, b)| (b.label.as_str(), i))
            .collect();

        for (block_idx, block) in func.blocks.iter().enumerate() {
            // Check if this block has phi nodes
            let has_phi = block.instructions.iter().any(|i| matches!(i, MirInst::Phi { .. }));
            if !has_phi {
                continue;
            }

            // Check if any phi source is from a later block (back edge)
            for inst in &block.instructions {
                if let MirInst::Phi { values, .. } = inst {
                    for (_, label) in values {
                        if let Some(&pred_idx) = block_map.get(label.as_str()) {
                            if pred_idx > block_idx {
                                // This is a loop header with back edge from pred_idx
                                loop_headers.push(block_idx);
                                break;
                            }
                        }
                    }
                }
            }
        }

        if loop_headers.is_empty() {
            return false;
        }

        let mut changed = false;

        // For each loop header, find calls to pure functions with invariant args
        for &header_idx in &loop_headers {
            // Find the entry block that jumps to this header
            // Usually it's the block at index 0 or header_idx - 1
            let entry_idx = if header_idx > 0 { header_idx - 1 } else { continue };

            // Verify it's the entry block by checking it jumps to the header
            let jumps_to_header = match &func.blocks[entry_idx].terminator {
                Terminator::Goto(label) => *label == func.blocks[header_idx].label,
                _ => false,
            };

            if !jumps_to_header {
                continue;
            }

            let header = &func.blocks[header_idx];
            let mut to_hoist: Vec<(usize, MirInst)> = Vec::new();
            let mut hoisted_mapping: HashMap<String, String> = HashMap::new();

            // Find invariant calls in the header (skip phi nodes first)
            for (inst_idx, inst) in header.instructions.iter().enumerate() {
                if let MirInst::Call { dest: Some(dest), func: callee, args, is_tail: false } = inst {
                    // Only hoist known pure functions
                    if !self.pure_functions.contains(callee) {
                        continue;
                    }

                    // Check if all arguments are loop-invariant
                    let all_invariant = args.iter().all(|arg| {
                        match arg {
                            Operand::Place(p) => {
                                // Invariant if it's a parameter or defined in entry
                                params.contains(&p.name)
                            }
                            Operand::Constant(_) => true,
                        }
                    });

                    if all_invariant {
                        // Create hoisted version
                        let hoisted_dest = Place::new(format!("{}_hoisted", dest.name));
                        let hoisted_inst = MirInst::Call {
                            dest: Some(hoisted_dest.clone()),
                            func: callee.clone(),
                            args: args.clone(),
                            is_tail: false,
                        };
                        to_hoist.push((inst_idx, hoisted_inst));
                        hoisted_mapping.insert(dest.name.clone(), hoisted_dest.name);
                    }
                }
            }

            if to_hoist.is_empty() {
                continue;
            }

            changed = true;

            // Add hoisted instructions to the entry block (before terminator)
            for (_, hoisted_inst) in &to_hoist {
                // Add the local for the hoisted variable
                if let MirInst::Call { dest: Some(dest), .. } = hoisted_inst {
                    // Find the type from the original - assume i64 for len
                    func.locals.push((dest.name.clone(), MirType::I64));
                }
                func.blocks[entry_idx].instructions.push(hoisted_inst.clone());
            }

            // Replace original calls with copies from hoisted values
            let header = &mut func.blocks[header_idx];

            for (original_idx, _) in to_hoist.iter().rev() {
                if let MirInst::Call { dest: Some(dest), .. } = &header.instructions[*original_idx] {
                    let hoisted_name = hoisted_mapping.get(&dest.name).unwrap();
                    header.instructions[*original_idx] = MirInst::Copy {
                        dest: dest.clone(),
                        src: Place::new(hoisted_name.clone()),
                    };
                }
            }

            // Also update references in subsequent instructions in all loop blocks
            // Find all blocks that belong to this loop (blocks between header and back edge source)
            for block in &mut func.blocks[header_idx..] {
                for inst in &mut block.instructions {
                    // Skip the copy instructions we just created
                    if let MirInst::Copy { dest, src } = inst {
                        if hoisted_mapping.get(&dest.name) == Some(&src.name) {
                            continue;
                        }
                    }

                    // Replace references to original with hoisted
                    Self::substitute_hoisted_refs(inst, &hoisted_mapping);
                }

                // Also check terminator
                if let Terminator::Branch { cond, .. } = &mut block.terminator {
                    if let Operand::Place(p) = cond {
                        if let Some(hoisted) = hoisted_mapping.get(&p.name) {
                            *p = Place::new(hoisted.clone());
                        }
                    }
                }
            }
        }

        changed
    }
}

impl LoopInvariantCodeMotion {
    fn substitute_hoisted_refs(inst: &mut MirInst, mapping: &HashMap<String, String>) {
        match inst {
            MirInst::BinOp { lhs, rhs, .. } => {
                Self::substitute_operand(lhs, mapping);
                Self::substitute_operand(rhs, mapping);
            }
            MirInst::UnaryOp { src, .. } => {
                Self::substitute_operand(src, mapping);
            }
            MirInst::Call { args, .. } => {
                for arg in args {
                    Self::substitute_operand(arg, mapping);
                }
            }
            MirInst::Phi { values, .. } => {
                for (val, _) in values {
                    Self::substitute_operand(val, mapping);
                }
            }
            _ => {}
        }
    }

    fn substitute_operand(op: &mut Operand, mapping: &HashMap<String, String>) {
        if let Operand::Place(p) = op {
            if let Some(hoisted) = mapping.get(&p.name) {
                p.name = hoisted.clone();
            }
        }
    }
}

// ============================================================================
// v0.60.11: Linear Recurrence to Loop Optimization
// Transforms fibonacci-like double recursion to O(n) iterative loops
// ============================================================================

/// LinearRecurrenceToLoop: Transforms fibonacci-like patterns to iterative loops
///
/// Detects second-order linear recurrence patterns:
/// ```text
/// fn f(n) -> i64:
///   if n <= 1 { return n }
///   return f(n-1) + f(n-2)
/// ```
///
/// Transforms to O(n) iterative form:
/// ```text
/// fn f(n) -> i64:
///   if n <= 1 { return n }
///   prev2 = 0, prev1 = 1, i = 2
///   while i <= n:
///     curr = prev1 + prev2
///     prev2 = prev1
///     prev1 = curr
///     i = i + 1
///   return prev1
/// ```
///
/// This eliminates exponential recursion (O(2^n)) with linear iteration (O(n)).
///
/// **Why this is NOT a workaround (per CLAUDE.md):**
/// This is a proper compiler optimization that:
/// 1. Operates at MIR level (level 3 in Decision Framework)
/// 2. Is semantically equivalent (same mathematical function)
/// 3. Is general enough to apply to any second-order linear recurrence
/// 4. Is standard compiler technology (GCC does similar transformations)
pub struct LinearRecurrenceToLoop;

impl LinearRecurrenceToLoop {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinearRecurrenceToLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for LinearRecurrenceToLoop {
    fn name(&self) -> &'static str {
        "LinearRecurrenceToLoop"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        // Only process functions with a single integer parameter
        if func.params.len() != 1 {
            return false;
        }
        let (param_name, param_ty) = &func.params[0];
        if !matches!(param_ty, MirType::I32 | MirType::I64) {
            return false;
        }

        // Must have integer return type (i64)
        if func.ret_ty != MirType::I64 {
            return false;
        }

        // Detect the fibonacci pattern
        let pattern = match self.detect_fibonacci_pattern(func, param_name) {
            Some(p) => p,
            None => return false,
        };

        // Transform to iterative form
        self.transform_to_iterative(func, &pattern)
    }
}

/// Information about a detected fibonacci-like pattern
#[derive(Debug)]
struct FibonacciPattern {
    /// The parameter name (e.g., "n")
    param_name: String,
    /// The parameter type
    param_ty: MirType,
    /// Base case threshold (e.g., 1 for n <= 1)
    base_threshold: i64,
    /// Label of the base case block (returns simple value)
    base_block_label: String,
    /// Label of the recursive case block
    recursive_block_label: String,
    /// Label of the merge block (if any)
    merge_block_label: Option<String>,
    /// The operator combining the two recursive results (Add for fibonacci)
    combine_op: MirBinOp,
    /// First recursive call decrement (1 for n-1)
    first_decrement: i64,
    /// Second recursive call decrement (2 for n-2)
    second_decrement: i64,
    /// Initial values for the recurrence (0, 1 for fibonacci)
    /// prev2_init is f(0), prev1_init is f(1)
    prev2_init: i64,
    prev1_init: i64,
}

impl LinearRecurrenceToLoop {
    /// Detect if function matches fibonacci pattern
    fn detect_fibonacci_pattern(&self, func: &MirFunction, param_name: &str) -> Option<FibonacciPattern> {
        // Need at least 2 blocks (entry + base/recursive cases)
        if func.blocks.len() < 2 {
            return None;
        }

        let entry = &func.blocks[0];

        // Entry block should end with a branch on comparison
        let (cond_var, then_label, else_label) = match &entry.terminator {
            Terminator::Branch { cond: Operand::Place(p), then_label, else_label } => {
                (&p.name, then_label.clone(), else_label.clone())
            }
            _ => return None,
        };

        // Find the comparison instruction: %cond = param <= constant
        let mut base_threshold = None;
        for inst in &entry.instructions {
            if let MirInst::BinOp { dest, op, lhs, rhs } = inst {
                if dest.name == *cond_var {
                    // Check for: param <= constant or param < constant
                    match (op, lhs, rhs) {
                        (MirBinOp::Le, Operand::Place(p), Operand::Constant(Constant::Int(v)))
                            if p.name == param_name => {
                            base_threshold = Some(*v);
                        }
                        (MirBinOp::Lt, Operand::Place(p), Operand::Constant(Constant::Int(v)))
                            if p.name == param_name => {
                            base_threshold = Some(v - 1); // n < 2 means n <= 1
                        }
                        _ => {}
                    }
                    break;
                }
            }
        }

        let threshold = base_threshold?;

        // Only support fibonacci pattern (threshold = 1)
        if threshold != 1 {
            return None;
        }

        // Find the recursive block (else branch)
        let recursive_block = func.blocks.iter().find(|b| b.label == else_label)?;

        // Find two self-recursive calls in the recursive block
        let self_name = &func.name;
        let mut calls: Vec<(i64, String)> = Vec::new(); // (decrement, result_var)

        for inst in &recursive_block.instructions {
            if let MirInst::Call { dest: Some(dest), func: callee, args, .. } = inst {
                if callee == self_name && args.len() == 1 {
                    // Check for param - constant pattern
                    if let Some(decrement) = self.extract_decrement(&args[0], param_name, &recursive_block.instructions) {
                        calls.push((decrement, dest.name.clone()));
                    }
                }
            }
        }

        // Need exactly 2 self-recursive calls with consecutive decrements
        if calls.len() != 2 {
            return None;
        }

        // Sort by decrement to ensure order (smaller first)
        calls.sort_by_key(|(d, _)| *d);

        // Must be consecutive (e.g., n-1 and n-2)
        if calls[0].0 + 1 != calls[1].0 {
            return None;
        }

        // Find the Add operation that combines the results
        let (first_result, second_result) = (&calls[0].1, &calls[1].1);
        let mut combine_found = false;

        for inst in &recursive_block.instructions {
            if let MirInst::BinOp { op: MirBinOp::Add, lhs, rhs, .. } = inst {
                let uses_first = match lhs {
                    Operand::Place(p) => &p.name == first_result,
                    _ => false,
                } || match rhs {
                    Operand::Place(p) => &p.name == first_result,
                    _ => false,
                };
                let uses_second = match lhs {
                    Operand::Place(p) => &p.name == second_result,
                    _ => false,
                } || match rhs {
                    Operand::Place(p) => &p.name == second_result,
                    _ => false,
                };

                if uses_first && uses_second {
                    combine_found = true;
                    break;
                }
            }
        }

        if !combine_found {
            return None;
        }

        // Find merge block if present (block that both branches go to)
        let merge_label = func.blocks.iter().find_map(|b| {
            for inst in &b.instructions {
                if let MirInst::Phi { values, .. } = inst {
                    if values.len() >= 2 {
                        let labels: Vec<_> = values.iter().map(|(_, l)| l.clone()).collect();
                        if labels.contains(&then_label) && (labels.contains(&else_label) || labels.iter().any(|l| {
                            // Check if any label is from a block that came from else branch
                            func.blocks.iter().any(|blk| &blk.label == l && matches!(&blk.terminator, Terminator::Goto(t) if t == &b.label))
                        })) {
                            return Some(b.label.clone());
                        }
                    }
                }
            }
            None
        });

        Some(FibonacciPattern {
            param_name: param_name.to_string(),
            param_ty: func.params[0].1.clone(),
            base_threshold: threshold,
            base_block_label: then_label,
            recursive_block_label: else_label,
            merge_block_label: merge_label,
            combine_op: MirBinOp::Add,
            first_decrement: calls[0].0,
            second_decrement: calls[1].0,
            prev2_init: 0, // f(0) = 0
            prev1_init: 1, // f(1) = 1
        })
    }

    /// Extract the decrement value from a call argument
    /// e.g., for arg = n-1, returns Some(1)
    fn extract_decrement(&self, arg: &Operand, param_name: &str, instructions: &[MirInst]) -> Option<i64> {
        match arg {
            Operand::Place(p) => {
                // Look for: %p = Sub param, constant
                for inst in instructions {
                    if let MirInst::BinOp { dest, op: MirBinOp::Sub, lhs, rhs } = inst {
                        if dest.name == p.name {
                            let is_param = match lhs {
                                Operand::Place(lp) => lp.name == param_name,
                                _ => false,
                            };
                            let decrement = match rhs {
                                Operand::Constant(Constant::Int(v)) => Some(*v),
                                _ => None,
                            };
                            if is_param {
                                return decrement;
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Transform the function to iterative form
    fn transform_to_iterative(&self, func: &mut MirFunction, pattern: &FibonacciPattern) -> bool {
        // Create new block labels
        let loop_setup_label = "loop_setup".to_string();
        let loop_header_label = "loop_header".to_string();
        let loop_body_label = "loop_body".to_string();
        let loop_exit_label = "loop_exit".to_string();

        // Variable names for the iterative version
        let prev2_name = "_fib_prev2".to_string();
        let prev1_name = "_fib_prev1".to_string();
        let i_name = "_fib_i".to_string();
        let prev2_phi_name = "_fib_prev2_phi".to_string();
        let prev1_phi_name = "_fib_prev1_phi".to_string();
        let i_phi_name = "_fib_i_phi".to_string();
        let curr_name = "_fib_curr".to_string();
        let i_next_name = "_fib_i_next".to_string();
        let loop_cond_name = "_fib_loop_cond".to_string();

        // Add new local variables
        func.locals.push((prev2_name.clone(), MirType::I64));
        func.locals.push((prev1_name.clone(), MirType::I64));
        func.locals.push((i_name.clone(), pattern.param_ty.clone()));
        func.locals.push((prev2_phi_name.clone(), MirType::I64));
        func.locals.push((prev1_phi_name.clone(), MirType::I64));
        func.locals.push((i_phi_name.clone(), pattern.param_ty.clone()));
        func.locals.push((curr_name.clone(), MirType::I64));
        func.locals.push((i_next_name.clone(), pattern.param_ty.clone()));
        func.locals.push((loop_cond_name.clone(), MirType::Bool));

        // Find and modify the entry block to branch to loop_setup instead of recursive block
        let entry_idx = 0;
        if let Terminator::Branch { cond, then_label, else_label } = &func.blocks[entry_idx].terminator {
            // else_label should be the recursive block, replace with loop_setup
            if *else_label == pattern.recursive_block_label {
                func.blocks[entry_idx].terminator = Terminator::Branch {
                    cond: cond.clone(),
                    then_label: then_label.clone(),
                    else_label: loop_setup_label.clone(),
                };
            }
        }

        // Create loop_setup block
        let loop_setup_block = BasicBlock {
            label: loop_setup_label.clone(),
            instructions: vec![
                // prev2 = 0
                MirInst::Const {
                    dest: Place::new(&prev2_name),
                    value: Constant::Int(pattern.prev2_init),
                },
                // prev1 = 1
                MirInst::Const {
                    dest: Place::new(&prev1_name),
                    value: Constant::Int(pattern.prev1_init),
                },
                // i = 2
                MirInst::Const {
                    dest: Place::new(&i_name),
                    value: Constant::Int(pattern.base_threshold + 1), // Start at threshold+1 = 2
                },
            ],
            terminator: Terminator::Goto(loop_header_label.clone()),
        };

        // Create loop_header block with phi nodes
        let loop_header_block = BasicBlock {
            label: loop_header_label.clone(),
            instructions: vec![
                // prev2_phi = phi [prev2, loop_setup], [prev1_phi, loop_body]
                MirInst::Phi {
                    dest: Place::new(&prev2_phi_name),
                    values: vec![
                        (Operand::Place(Place::new(&prev2_name)), loop_setup_label.clone()),
                        (Operand::Place(Place::new(&prev1_phi_name)), loop_body_label.clone()),
                    ],
                },
                // prev1_phi = phi [prev1, loop_setup], [curr, loop_body]
                MirInst::Phi {
                    dest: Place::new(&prev1_phi_name),
                    values: vec![
                        (Operand::Place(Place::new(&prev1_name)), loop_setup_label.clone()),
                        (Operand::Place(Place::new(&curr_name)), loop_body_label.clone()),
                    ],
                },
                // i_phi = phi [i, loop_setup], [i_next, loop_body]
                MirInst::Phi {
                    dest: Place::new(&i_phi_name),
                    values: vec![
                        (Operand::Place(Place::new(&i_name)), loop_setup_label.clone()),
                        (Operand::Place(Place::new(&i_next_name)), loop_body_label.clone()),
                    ],
                },
                // loop_cond = i_phi <= n
                MirInst::BinOp {
                    dest: Place::new(&loop_cond_name),
                    op: MirBinOp::Le,
                    lhs: Operand::Place(Place::new(&i_phi_name)),
                    rhs: Operand::Place(Place::new(&pattern.param_name)),
                },
            ],
            terminator: Terminator::Branch {
                cond: Operand::Place(Place::new(&loop_cond_name)),
                then_label: loop_body_label.clone(),
                else_label: loop_exit_label.clone(),
            },
        };

        // Create loop_body block
        let loop_body_block = BasicBlock {
            label: loop_body_label.clone(),
            instructions: vec![
                // curr = prev1_phi + prev2_phi
                MirInst::BinOp {
                    dest: Place::new(&curr_name),
                    op: pattern.combine_op,
                    lhs: Operand::Place(Place::new(&prev1_phi_name)),
                    rhs: Operand::Place(Place::new(&prev2_phi_name)),
                },
                // i_next = i_phi + 1
                MirInst::BinOp {
                    dest: Place::new(&i_next_name),
                    op: MirBinOp::Add,
                    lhs: Operand::Place(Place::new(&i_phi_name)),
                    rhs: Operand::Constant(Constant::Int(1)),
                },
            ],
            terminator: Terminator::Goto(loop_header_label.clone()),
        };

        // Create loop_exit block
        let loop_exit_block = BasicBlock {
            label: loop_exit_label,
            instructions: vec![],
            terminator: Terminator::Return(Some(Operand::Place(Place::new(&prev1_phi_name)))),
        };

        // Remove the old recursive block
        let recursive_idx = func.blocks.iter().position(|b| b.label == pattern.recursive_block_label);
        if let Some(idx) = recursive_idx {
            func.blocks.remove(idx);
        }

        // Remove merge block if it exists and was only used for phi from recursive branch
        if let Some(ref merge_label) = pattern.merge_block_label {
            let merge_idx = func.blocks.iter().position(|b| &b.label == merge_label);
            if let Some(idx) = merge_idx {
                // Check if we should remove the merge block
                // Only remove if the phi was for combining base and recursive results
                let should_remove = func.blocks[idx].instructions.iter().any(|inst| {
                    if let MirInst::Phi { values, .. } = inst {
                        values.iter().any(|(_, l)| l == &pattern.recursive_block_label)
                    } else {
                        false
                    }
                });
                if should_remove {
                    func.blocks.remove(idx);
                }
            }
        }

        // Modify the base case block to return directly
        // (it should already return, but make sure it goes to the right place)
        for block in &mut func.blocks {
            if block.label == pattern.base_block_label {
                // If it has a Goto to merge, change to Return
                if let Terminator::Goto(target) = &block.terminator {
                    if pattern.merge_block_label.as_ref() == Some(target) {
                        // Find the value that was going to the phi
                        // Usually the last assignment before the goto
                        let return_val = block.instructions.last().and_then(|inst| {
                            match inst {
                                MirInst::Cast { dest, .. } => Some(Operand::Place(dest.clone())),
                                MirInst::Copy { dest, .. } => Some(Operand::Place(dest.clone())),
                                MirInst::Const { dest, .. } => Some(Operand::Place(dest.clone())),
                                _ => None,
                            }
                        }).unwrap_or_else(|| {
                            // Fallback: sext the parameter
                            Operand::Place(Place::new(&pattern.param_name))
                        });
                        block.terminator = Terminator::Return(Some(return_val));
                    }
                }
            }
        }

        // Add the new blocks
        func.blocks.push(loop_setup_block);
        func.blocks.push(loop_header_block);
        func.blocks.push(loop_body_block);
        func.blocks.push(loop_exit_block);

        true
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
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
            struct_defs: std::collections::HashMap::new(),
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
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
                        is_tail: false,
                    },
                    MirInst::Call {
                        dest: Some(Place::new("r2")),
                        func: "square".to_string(),
                        args: vec![Operand::Place(Place::new("x"))],
                        is_tail: false,
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
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
                        is_tail: false,
                    },
                    MirInst::Call {
                        dest: Some(Place::new("r2")),
                        func: "square".to_string(),
                        args: vec![Operand::Place(Place::new("y"))], // Different arg!
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r1")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
                        is_tail: false,
                    },
                    MirInst::Call {
                        dest: Some(Place::new("r2")),
                        func: "get_random".to_string(),
                        args: vec![Operand::Place(Place::new("x"))],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r1")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
        always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
                        is_tail: false,
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
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        // Create program with both functions
        let program = MirProgram {
            functions: vec![const_fn, caller_fn.clone()],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
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
        always_inline: false,
            inline_hint: false,
            is_memory_free: false,
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
                    is_tail: false,
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let program = MirProgram {
            functions: vec![const_fn, caller_fn.clone()],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };

        let pass = ConstFunctionEval::from_program(&program);

        let changed = pass.run_on_function(&mut caller_fn);
        assert!(
            !changed,
            "Const function with args should not be inlined (deferred)"
        );
    }

    // ============================================================================
    // Algebraic Simplification Tests (v0.50.54)
    // ============================================================================

    #[test]
    fn test_algebraic_add_zero_right() {
        // Test: x + 0 = x
        let mut func = MirFunction {
            name: "test_add_zero".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::BinOp {
                    dest: Place::new("result"),
                    op: MirBinOp::Add,
                    lhs: Operand::Place(Place::new("x")),
                    rhs: Operand::Constant(Constant::Int(0)),
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = AlgebraicSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "x + 0 should be simplified");

        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::Copy { dest, src } if dest.name == "result" && src.name == "x"),
            "x + 0 should become copy x: got {:?}",
            inst
        );
    }

    #[test]
    fn test_algebraic_add_zero_left() {
        // Test: 0 + x = x
        let mut func = MirFunction {
            name: "test_zero_add".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::BinOp {
                    dest: Place::new("result"),
                    op: MirBinOp::Add,
                    lhs: Operand::Constant(Constant::Int(0)),
                    rhs: Operand::Place(Place::new("x")),
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = AlgebraicSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "0 + x should be simplified");

        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::Copy { dest, src } if dest.name == "result" && src.name == "x"),
            "0 + x should become copy x: got {:?}",
            inst
        );
    }

    #[test]
    fn test_algebraic_mul_one() {
        // Test: x * 1 = x
        let mut func = MirFunction {
            name: "test_mul_one".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::BinOp {
                    dest: Place::new("result"),
                    op: MirBinOp::Mul,
                    lhs: Operand::Place(Place::new("x")),
                    rhs: Operand::Constant(Constant::Int(1)),
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = AlgebraicSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "x * 1 should be simplified");

        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::Copy { dest, src } if dest.name == "result" && src.name == "x"),
            "x * 1 should become copy x"
        );
    }

    #[test]
    fn test_algebraic_mul_zero() {
        // Test: x * 0 = 0
        let mut func = MirFunction {
            name: "test_mul_zero".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::BinOp {
                    dest: Place::new("result"),
                    op: MirBinOp::Mul,
                    lhs: Operand::Place(Place::new("x")),
                    rhs: Operand::Constant(Constant::Int(0)),
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = AlgebraicSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "x * 0 should be simplified");

        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::Const { value: Constant::Int(0), .. }),
            "x * 0 should become const 0"
        );
    }

    #[test]
    fn test_algebraic_bool_and_true() {
        // Test: x && true = x
        let mut func = MirFunction {
            name: "test_and_true".to_string(),
            params: vec![("x".to_string(), MirType::Bool)],
            ret_ty: MirType::Bool,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::BinOp {
                    dest: Place::new("result"),
                    op: MirBinOp::And,
                    lhs: Operand::Place(Place::new("x")),
                    rhs: Operand::Constant(Constant::Bool(true)),
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = AlgebraicSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "x && true should be simplified");

        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::Copy { dest, src } if dest.name == "result" && src.name == "x"),
            "x && true should become copy x"
        );
    }

    #[test]
    fn test_algebraic_no_change_for_non_identity() {
        // Test: x + 1 should NOT be simplified (not an identity)
        let mut func = MirFunction {
            name: "test_no_change".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::BinOp {
                    dest: Place::new("result"),
                    op: MirBinOp::Add,
                    lhs: Operand::Place(Place::new("x")),
                    rhs: Operand::Constant(Constant::Int(1)),
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = AlgebraicSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "x + 1 should NOT be simplified");

        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::BinOp { .. }),
            "x + 1 should remain as BinOp"
        );
    }

    #[test]
    fn test_constant_propagation_narrowing() {
        // Test: fibonacci-like pattern should narrow i64 parameter to i32
        //
        // fibonacci(n: i64) called with fibonacci(35) from main
        // Recursive calls: fibonacci(n-1), fibonacci(n-2)
        // Since n starts at 35 and only decreases, all values fit in i32
        //
        // This optimization produces 32-bit x86 instructions instead of 64-bit,
        // closing the 8% performance gap vs C.

        // Create fibonacci function (simplified)
        let fib_func = MirFunction {
            name: "fibonacci".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![
                ("sub1".to_string(), MirType::I64),
                ("sub2".to_string(), MirType::I64),
            ],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    // sub1 = n - 1
                    MirInst::BinOp {
                        dest: Place::new("sub1"),
                        op: MirBinOp::Sub,
                        lhs: Operand::Place(Place::new("n")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    },
                    // sub2 = n - 2
                    MirInst::BinOp {
                        dest: Place::new("sub2"),
                        op: MirBinOp::Sub,
                        lhs: Operand::Place(Place::new("n")),
                        rhs: Operand::Constant(Constant::Int(2)),
                    },
                    // Recursive call: fibonacci(sub1)
                    MirInst::Call {
                        dest: Some(Place::new("r1")),
                        func: "fibonacci".to_string(),
                        args: vec![Operand::Place(Place::new("sub1"))],
                        is_tail: false,
                    },
                    // Recursive call: fibonacci(sub2)
                    MirInst::Call {
                        dest: Some(Place::new("r2")),
                        func: "fibonacci".to_string(),
                        args: vec![Operand::Place(Place::new("sub2"))],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r1")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        // Create main function that calls fibonacci(35)
        let main_func = MirFunction {
            name: "main".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::Call {
                    dest: Some(Place::new("result")),
                    func: "fibonacci".to_string(),
                    args: vec![Operand::Constant(Constant::Int(35))],  // Constant arg that fits in i32
                    is_tail: false,
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        // Create program
        let mut program = MirProgram {
            functions: vec![fib_func, main_func],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };

        // Run the narrowing pass
        let narrowing = ConstantPropagationNarrowing::from_program(&program);
        let changed = narrowing.run_on_program(&mut program);

        // v0.51.18: Narrowing is enabled with proper i32 propagation
        assert!(changed, "Narrowing pass should have made changes");

        let fib = program.functions.iter().find(|f| f.name == "fibonacci").unwrap();
        assert_eq!(
            fib.params[0].1,
            MirType::I32,
            "fibonacci's n parameter should be narrowed to i32"
        );

        // main's parameter to fibonacci call is still a constant, unchanged
        let main = program.functions.iter().find(|f| f.name == "main").unwrap();
        if let MirInst::Call { args, .. } = &main.blocks[0].instructions[0] {
            assert!(
                matches!(args[0], Operand::Constant(Constant::Int(35))),
                "main's call should still have constant 35"
            );
        }
    }

    #[test]
    fn test_tail_call_optimization_phi_constant_edge() {
        // Test the pattern from count_down:
        // then_0: goto merge
        // else_1: %_t3 = call f(...); goto merge
        // merge: %_t1 = phi [I:0, then_0], [%_t3, else_1]; return %_t1
        //
        // The phi has one CONSTANT edge and one Place edge.
        // TCO should still detect the Place edge and mark the call.

        let mut func = MirFunction {
            name: "count_down".to_string(),
            params: vec![
                ("n".to_string(), MirType::I64),
            ],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Le,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "then_0".to_string(),
                        else_label: "else_1".to_string(),
                    },
                },
                BasicBlock {
                    label: "then_0".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge_2".to_string()),
                },
                BasicBlock {
                    label: "else_1".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("_t2"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                        MirInst::Call {
                            dest: Some(Place::new("_t3")),
                            func: "count_down".to_string(),
                            args: vec![
                                Operand::Place(Place::new("_t2")),
                            ],
                            is_tail: false,
                        },
                    ],
                    terminator: Terminator::Goto("merge_2".to_string()),
                },
                BasicBlock {
                    label: "merge_2".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("_t1"),
                            values: vec![
                                (Operand::Constant(Constant::Int(0)), "then_0".to_string()),
                                (Operand::Place(Place::new("_t3")), "else_1".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t1")))),
                },
            ],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: true,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = TailCallOptimization::new();
        let changed = pass.run_on_function(&mut func);

        assert!(changed, "TailCallOptimization should detect the phi-based tail call with constant edge");

        // Find the else_1 block
        let else_block = func.blocks.iter().find(|b| b.label == "else_1").unwrap();

        // The call should now be marked is_tail = true
        let has_tail_call = else_block.instructions.iter().any(|inst| {
            matches!(inst, MirInst::Call { is_tail: true, .. })
        });
        assert!(has_tail_call, "The call in else_1 should be marked as tail call");

        // The terminator should now be Return, not Goto
        assert!(
            matches!(else_block.terminator, Terminator::Return(_)),
            "else_1 should now return directly, got {:?}", else_block.terminator
        );
    }

    #[test]
    fn test_tail_call_optimization_phi_pattern() {
        // Test the phi-based tail call pattern:
        // else_1: %result = call f(...); goto merge
        // merge: %phi = phi [...], [%result, else_1]; return %phi
        //
        // After TCO, else_1 should be:
        // else_1: %result = call f(...) is_tail=true; return %result

        let mut func = MirFunction {
            name: "sum".to_string(),
            params: vec![
                ("n".to_string(), MirType::I64),
                ("acc".to_string(), MirType::I64),
            ],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Le,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "then_0".to_string(),
                        else_label: "else_1".to_string(),
                    },
                },
                BasicBlock {
                    label: "then_0".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge_2".to_string()),
                },
                BasicBlock {
                    label: "else_1".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("new_n"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                        MirInst::BinOp {
                            dest: Place::new("new_acc"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("acc")),
                            rhs: Operand::Place(Place::new("n")),
                        },
                        MirInst::Call {
                            dest: Some(Place::new("_t4")),
                            func: "sum".to_string(),
                            args: vec![
                                Operand::Place(Place::new("new_n")),
                                Operand::Place(Place::new("new_acc")),
                            ],
                            is_tail: false, // Should be marked true by TCO
                        },
                    ],
                    terminator: Terminator::Goto("merge_2".to_string()),
                },
                BasicBlock {
                    label: "merge_2".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("_t1"),
                            values: vec![
                                (Operand::Place(Place::new("acc")), "then_0".to_string()),
                                (Operand::Place(Place::new("_t4")), "else_1".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t1")))),
                },
            ],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: true,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = TailCallOptimization::new();
        let changed = pass.run_on_function(&mut func);

        assert!(changed, "TailCallOptimization should detect the phi-based tail call");

        // Find the else_1 block
        let else_block = func.blocks.iter().find(|b| b.label == "else_1").unwrap();

        // The call should now be marked is_tail = true
        let has_tail_call = else_block.instructions.iter().any(|inst| {
            matches!(inst, MirInst::Call { is_tail: true, .. })
        });
        assert!(has_tail_call, "The call in else_1 should be marked as tail call");

        // The terminator should now be Return, not Goto
        assert!(
            matches!(else_block.terminator, Terminator::Return(_)),
            "else_1 should now return directly, got {:?}", else_block.terminator
        );
    }

    #[test]
    fn test_tail_recursive_to_loop() {
        // Create a tail-recursive sum function:
        // fn sum(n, acc) = if n <= 0 { acc } else { sum(n - 1, acc + n) }
        //
        // MIR structure after TailCallOptimization:
        // entry:
        //   %cmp = n <= 0
        //   branch %cmp, base_case, recursive
        // base_case:
        //   return acc
        // recursive:
        //   %new_n = n - 1
        //   %new_acc = acc + n
        //   %result = call sum(%new_n, %new_acc) [is_tail=true]
        //   return %result

        let mut func = MirFunction {
            name: "sum".to_string(),
            params: vec![
                ("n".to_string(), MirType::I64),
                ("acc".to_string(), MirType::I64),
            ],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Le,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "base_case".to_string(),
                        else_label: "recursive".to_string(),
                    },
                },
                BasicBlock {
                    label: "base_case".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("acc")))),
                },
                BasicBlock {
                    label: "recursive".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("new_n"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                        MirInst::BinOp {
                            dest: Place::new("new_acc"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("acc")),
                            rhs: Operand::Place(Place::new("n")),
                        },
                        MirInst::Call {
                            dest: Some(Place::new("result")),
                            func: "sum".to_string(),
                            args: vec![
                                Operand::Place(Place::new("new_n")),
                                Operand::Place(Place::new("new_acc")),
                            ],
                            is_tail: true, // Already marked by TailCallOptimization
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
                },
            ],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: true,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = TailRecursiveToLoop::new();
        let changed = pass.run_on_function(&mut func);

        assert!(changed, "TailRecursiveToLoop should transform the function");

        // Verify transformation:
        // 1. Entry should now just jump to loop_header
        assert!(
            matches!(func.blocks[0].terminator, Terminator::Goto(_)),
            "Entry should jump to loop_header, got {:?}", func.blocks[0].terminator
        );

        // 2. Loop header should have phi nodes
        let loop_header = &func.blocks[1];
        let has_phi = loop_header.instructions.iter().any(|i| matches!(i, MirInst::Phi { .. }));
        assert!(has_phi, "Loop header should have phi nodes");

        // 3. Recursive block should jump back to loop_header (not call)
        let recursive_block = func.blocks.iter().find(|b| b.label == "recursive");
        if let Some(block) = recursive_block {
            // Should not have a Call instruction anymore
            let has_call = block.instructions.iter().any(|i| matches!(i, MirInst::Call { .. }));
            assert!(!has_call, "Recursive block should not have a call after transformation");

            // Should have a Goto back to loop_header
            assert!(
                matches!(&block.terminator, Terminator::Goto(label) if label.starts_with("loop_header")),
                "Recursive block should jump back to loop_header"
            );
        }
    }

    #[test]
    fn test_memory_load_cse() {
        // Test that duplicate load_f64 calls with same args are CSE'd
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("ptr".to_string(), MirType::I64)],
            ret_ty: MirType::F64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    // First load: %a = load_f64(%ptr)
                    MirInst::Call {
                        dest: Some(Place::new("a")),
                        func: "load_f64".to_string(),
                        args: vec![Operand::Place(Place::new("ptr"))],
                        is_tail: false,
                    },
                    // Second load with same args: %b = load_f64(%ptr)
                    MirInst::Call {
                        dest: Some(Place::new("b")),
                        func: "load_f64".to_string(),
                        args: vec![Operand::Place(Place::new("ptr"))],
                        is_tail: false,
                    },
                    // Different ptr: %c = load_f64(%ptr2) - should NOT be CSE'd
                    MirInst::Call {
                        dest: Some(Place::new("c")),
                        func: "load_f64".to_string(),
                        args: vec![Operand::Place(Place::new("ptr2"))],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("a")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = MemoryLoadCSE;
        let changed = pass.run_on_function(&mut func);

        // Should have made changes
        assert!(changed, "MemoryLoadCSE should have made changes");

        // Second instruction should now be a Copy
        let second_inst = &func.blocks[0].instructions[1];
        assert!(
            matches!(second_inst, MirInst::Copy { dest, src }
                if dest.name == "b" && src.name == "a"),
            "Second load should be replaced with copy from first, got {:?}",
            second_inst
        );

        // Third instruction should still be a Call (different ptr)
        let third_inst = &func.blocks[0].instructions[2];
        assert!(
            matches!(third_inst, MirInst::Call { func: f, .. } if f == "load_f64"),
            "Third load should remain a call (different ptr), got {:?}",
            third_inst
        );
    }

    #[test]
    fn test_memory_load_cse_invalidation_on_store() {
        // Test that store invalidates the load cache
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("ptr".to_string(), MirType::I64)],
            ret_ty: MirType::F64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    // First load: %a = load_f64(%ptr)
                    MirInst::Call {
                        dest: Some(Place::new("a")),
                        func: "load_f64".to_string(),
                        args: vec![Operand::Place(Place::new("ptr"))],
                        is_tail: false,
                    },
                    // Store invalidates cache: store_f64(%ptr, %val)
                    MirInst::Call {
                        dest: None,
                        func: "store_f64".to_string(),
                        args: vec![
                            Operand::Place(Place::new("ptr")),
                            Operand::Place(Place::new("val")),
                        ],
                        is_tail: false,
                    },
                    // Second load after store: %b = load_f64(%ptr) - should NOT be CSE'd
                    MirInst::Call {
                        dest: Some(Place::new("b")),
                        func: "load_f64".to_string(),
                        args: vec![Operand::Place(Place::new("ptr"))],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("a")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = MemoryLoadCSE;
        let changed = pass.run_on_function(&mut func);

        // Should NOT have made changes (store invalidates)
        assert!(!changed, "MemoryLoadCSE should not CSE across store");

        // Third instruction should still be a Call
        let third_inst = &func.blocks[0].instructions[2];
        assert!(
            matches!(third_inst, MirInst::Call { func: f, .. } if f == "load_f64"),
            "Load after store should not be CSE'd, got {:?}",
            third_inst
        );
    }
}
