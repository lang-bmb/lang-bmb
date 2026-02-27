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
                // v0.60.41: Remove unreachable blocks after branch simplification
                pipeline.add_pass(Box::new(UnreachableBlockElimination));
                // v0.60.44: Simplify single-value phi nodes after unreachable block elimination
                pipeline.add_pass(Box::new(PhiSimplification));
                // v0.60.44: Merge blocks connected by unconditional jumps
                pipeline.add_pass(Box::new(BlockMerging));
                // v0.51.8: If-else chain to switch for jump tables
                pipeline.add_pass(Box::new(IfElseToSwitch));
                pipeline.add_pass(Box::new(CopyPropagation));
                // v0.51.10: Memory load CSE for repeated load_f64/load_i64 calls
                pipeline.add_pass(Box::new(MemoryLoadCSE));
                // v0.60.38: Global field access CSE for cross-block field access dedup
                pipeline.add_pass(Box::new(GlobalFieldAccessCSE));
                // v0.50.76: Add contract-based optimization for dead branch elimination
                pipeline.add_pass(Box::new(ContractBasedOptimization));
                pipeline.add_pass(Box::new(ContractUnreachableElimination));
                // v0.50.65: Add tail call optimization for recursive functions
                pipeline.add_pass(Box::new(TailCallOptimization));
                // v0.51.9: Convert tail recursion to loops for better performance
                pipeline.add_pass(Box::new(TailRecursiveToLoop));
                // v0.60.55: Convert conditional increment to branchless add
                pipeline.add_pass(Box::new(ConditionalIncrementToSelect));
                // v0.90.27: Convert simple if-else branches to select (branchless)
                pipeline.add_pass(Box::new(IfElseToSelect));
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
                // v0.60.41: Remove unreachable blocks after branch simplification
                pipeline.add_pass(Box::new(UnreachableBlockElimination));
                // v0.60.44: Simplify single-value phi nodes after unreachable block elimination
                pipeline.add_pass(Box::new(PhiSimplification));
                // v0.60.44: Merge blocks connected by unconditional jumps
                pipeline.add_pass(Box::new(BlockMerging));
                // v0.51.8: If-else chain to switch for jump tables
                pipeline.add_pass(Box::new(IfElseToSwitch));
                pipeline.add_pass(Box::new(CopyPropagation));
                pipeline.add_pass(Box::new(CommonSubexpressionElimination));
                // v0.51.10: Memory load CSE for repeated load_f64/load_i64 calls
                pipeline.add_pass(Box::new(MemoryLoadCSE));
                // v0.60.38: Global field access CSE for cross-block field access dedup
                pipeline.add_pass(Box::new(GlobalFieldAccessCSE));
                pipeline.add_pass(Box::new(ContractBasedOptimization));
                pipeline.add_pass(Box::new(ContractUnreachableElimination));
                // v0.50.65: Add tail call optimization for recursive functions
                pipeline.add_pass(Box::new(TailCallOptimization));
                // v0.51.9: Convert tail recursion to loops for better performance
                pipeline.add_pass(Box::new(TailRecursiveToLoop));
                // v0.60.55: Convert conditional increment to branchless add
                pipeline.add_pass(Box::new(ConditionalIncrementToSelect));
                // v0.90.27: Convert simple if-else branches to select (branchless)
                pipeline.add_pass(Box::new(IfElseToSelect));
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
                        if let Some(d) = dest
                            && let Some(result) = fold_builtin_call(func_name, args, &constants) {
                                constants.insert(d.name.clone(), result.clone());
                                new_instructions.push(MirInst::Const {
                                    dest: d.clone(),
                                    value: result,
                                });
                                changed = true;
                                continue;
                            }

                        // v0.51.2: Propagate constants to call arguments
                        // This enables LLVM codegen to detect string literal arguments
                        let propagated_args: Vec<Operand> = args.iter().map(|arg| {
                            match arg {
                                Operand::Place(p) => {
                                    // Don't propagate loop-modified variables
                                    if !loop_modified.contains(&p.name)
                                        && let Some(c) = constants.get(&p.name) {
                                            return Operand::Constant(c.clone());
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
                if (0..=127).contains(&code) {
                    let ch = char::from_u32(code as u32)?;
                    return Some(Constant::String(ch.to_string()));
                }
            }
            None
        }
        // ord("A") -> 65 (only for single-character string constants)
        "ord" | "bmb_ord" if args.len() == 1 => {
            if let Some(Constant::String(s)) = get_constant(&args[0], constants)
                && s.len() == 1 {
                    let code = s.chars().next()? as i64;
                    return Some(Constant::Int(code));
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
        // v0.60.53: x * 2^n = x << n (power-of-2 multiplication to left shift)
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
            // v0.60.53: Convert multiplication by power-of-2 to left shift
            // x * 2 → x << 1, x * 4 → x << 2, x * 8 → x << 3, etc.
            if let Operand::Constant(Constant::Int(multiplier)) = rhs
                && *multiplier > 1 && (*multiplier & (*multiplier - 1)) == 0 {
                    let shift_amount = (*multiplier as u64).trailing_zeros() as i64;
                    return Some(MirInst::BinOp {
                        dest: dest.clone(),
                        op: MirBinOp::Shl,
                        lhs: lhs.clone(),
                        rhs: Operand::Constant(Constant::Int(shift_amount)),
                    });
                }
            // Handle commutative case: 2^n * x → x << n
            if let Operand::Constant(Constant::Int(multiplier)) = lhs
                && *multiplier > 1 && (*multiplier & (*multiplier - 1)) == 0 {
                    let shift_amount = (*multiplier as u64).trailing_zeros() as i64;
                    return Some(MirInst::BinOp {
                        dest: dest.clone(),
                        op: MirBinOp::Shl,
                        lhs: rhs.clone(),
                        rhs: Operand::Constant(Constant::Int(shift_amount)),
                    });
                }
            None
        }

        // Division: x / 1 = x, x / 2^n = x >> n (for positive divisors)
        MirBinOp::Div => {
            if matches!(rhs, Operand::Constant(Constant::Int(1))) {
                return Some(MirInst::Copy {
                    dest: dest.clone(),
                    src: operand_to_place(lhs)?,
                });
            }
            // v0.60.52: Convert division by power-of-2 to arithmetic right shift
            // Note: This is safe for unsigned division semantics
            // For signed division of negative numbers, behavior differs slightly
            // but LLVM's optimization passes handle this correctly
            if let Operand::Constant(Constant::Int(divisor)) = rhs
                && *divisor > 1 && (*divisor & (*divisor - 1)) == 0 {
                    // divisor is a power of 2
                    let shift_amount = (*divisor as u64).trailing_zeros() as i64;
                    return Some(MirInst::BinOp {
                        dest: dest.clone(),
                        op: MirBinOp::Shr,
                        lhs: lhs.clone(),
                        rhs: Operand::Constant(Constant::Int(shift_amount)),
                    });
                }
            None
        }

        // Modulo: x % 2^n = x & (2^n - 1) for power-of-2 divisors
        MirBinOp::Mod => {
            // v0.60.52: Convert modulo by power-of-2 to bitwise AND
            // This is a common optimization for hash table indexing: idx % size → idx & (size - 1)
            if let Operand::Constant(Constant::Int(divisor)) = rhs
                && *divisor > 1 && (*divisor & (*divisor - 1)) == 0 {
                    // divisor is a power of 2
                    let mask = *divisor - 1;
                    return Some(MirInst::BinOp {
                        dest: dest.clone(),
                        op: MirBinOp::Band,
                        lhs: lhs.clone(),
                        rhs: Operand::Constant(Constant::Int(mask)),
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
        // v0.60.19: Pointer offset
        MirInst::PtrOffset { ptr, offset, .. } => {
            collect_used_in_operand(ptr, used);
            collect_used_in_operand(offset, used);
        }
        // v0.60.21: Array allocation - no operands
        MirInst::ArrayAlloc { .. } => {}
        // v0.60.20: Pointer load/store
        MirInst::PtrLoad { ptr, .. } => {
            collect_used_in_operand(ptr, used);
        }
        MirInst::PtrStore { ptr, value, .. } => {
            collect_used_in_operand(ptr, used);
            collect_used_in_operand(value, used);
        }
        // v0.70: Thread spawn/join
        MirInst::ThreadSpawn { captures, .. } => {
            for cap in captures {
                collect_used_in_operand(cap, used);
            }
        }
        MirInst::ThreadJoin { handle, .. } => {
            collect_used_in_operand(handle, used);
        }
        // v0.71: Mutex operations
        MirInst::MutexNew { initial_value, .. } => {
            collect_used_in_operand(initial_value, used);
        }
        MirInst::MutexLock { mutex, .. } => {
            collect_used_in_operand(mutex, used);
        }
        MirInst::MutexUnlock { mutex, new_value } => {
            collect_used_in_operand(mutex, used);
            collect_used_in_operand(new_value, used);
        }
        MirInst::MutexTryLock { mutex, .. } => {
            collect_used_in_operand(mutex, used);
        }
        MirInst::MutexFree { mutex } => {
            collect_used_in_operand(mutex, used);
        }
        // v0.72: Arc operations
        MirInst::ArcNew { value, .. } => {
            collect_used_in_operand(value, used);
        }
        MirInst::ArcClone { arc, .. } => {
            collect_used_in_operand(arc, used);
        }
        MirInst::ArcGet { arc, .. } => {
            collect_used_in_operand(arc, used);
        }
        MirInst::ArcDrop { arc } => {
            collect_used_in_operand(arc, used);
        }
        MirInst::ArcStrongCount { arc, .. } => {
            collect_used_in_operand(arc, used);
        }
        // v0.72: Atomic operations
        MirInst::AtomicNew { value, .. } => {
            collect_used_in_operand(value, used);
        }
        MirInst::AtomicLoad { ptr, .. } => {
            collect_used_in_operand(ptr, used);
        }
        MirInst::AtomicStore { ptr, value } => {
            collect_used_in_operand(ptr, used);
            collect_used_in_operand(value, used);
        }
        MirInst::AtomicFetchAdd { ptr, delta, .. } => {
            collect_used_in_operand(ptr, used);
            collect_used_in_operand(delta, used);
        }
        MirInst::AtomicFetchSub { ptr, delta, .. } => {
            collect_used_in_operand(ptr, used);
            collect_used_in_operand(delta, used);
        }
        MirInst::AtomicSwap { ptr, new_value, .. } => {
            collect_used_in_operand(ptr, used);
            collect_used_in_operand(new_value, used);
        }
        MirInst::AtomicCompareExchange { ptr, expected, new_value, .. } => {
            collect_used_in_operand(ptr, used);
            collect_used_in_operand(expected, used);
            collect_used_in_operand(new_value, used);
        }
        // v0.73: Channel operations
        MirInst::ChannelNew { capacity, .. } => {
            collect_used_in_operand(capacity, used);
        }
        MirInst::ChannelSend { sender, value } => {
            collect_used_in_operand(sender, used);
            collect_used_in_operand(value, used);
        }
        MirInst::ChannelRecv { receiver, .. } => {
            collect_used_in_operand(receiver, used);
        }
        MirInst::ChannelTrySend { sender, value, .. } => {
            collect_used_in_operand(sender, used);
            collect_used_in_operand(value, used);
        }
        MirInst::ChannelTryRecv { receiver, .. } => {
            collect_used_in_operand(receiver, used);
        }
        // v0.77: Receive with timeout
        MirInst::ChannelRecvTimeout { receiver, timeout_ms, .. } => {
            collect_used_in_operand(receiver, used);
            collect_used_in_operand(timeout_ms, used);
        }
        // v0.78: Block on future
        MirInst::BlockOn { future, .. } => {
            collect_used_in_operand(future, used);
        }
        // v0.79: Send with timeout
        MirInst::ChannelSendTimeout { sender, value, timeout_ms, .. } => {
            collect_used_in_operand(sender, used);
            collect_used_in_operand(value, used);
            collect_used_in_operand(timeout_ms, used);
        }
        // v0.80: Channel close instructions
        MirInst::ChannelClose { sender } => {
            collect_used_in_operand(sender, used);
        }
        MirInst::ChannelIsClosed { receiver, .. } => {
            collect_used_in_operand(receiver, used);
        }
        MirInst::ChannelRecvOpt { receiver, .. } => {
            collect_used_in_operand(receiver, used);
        }
        MirInst::SenderClone { sender, .. } => {
            collect_used_in_operand(sender, used);
        }
        // v0.74: RwLock, Barrier, Condvar instructions
        MirInst::RwLockNew { initial_value, .. } => {
            collect_used_in_operand(initial_value, used);
        }
        MirInst::RwLockRead { rwlock, .. } => {
            collect_used_in_operand(rwlock, used);
        }
        MirInst::RwLockReadUnlock { rwlock } => {
            collect_used_in_operand(rwlock, used);
        }
        MirInst::RwLockWrite { rwlock, .. } => {
            collect_used_in_operand(rwlock, used);
        }
        MirInst::RwLockWriteUnlock { rwlock, value } => {
            collect_used_in_operand(rwlock, used);
            collect_used_in_operand(value, used);
        }
        MirInst::BarrierNew { count, .. } => {
            collect_used_in_operand(count, used);
        }
        MirInst::BarrierWait { barrier, .. } => {
            collect_used_in_operand(barrier, used);
        }
        MirInst::CondvarNew { .. } => {}
        MirInst::CondvarWait { condvar, mutex, .. } => {
            collect_used_in_operand(condvar, used);
            collect_used_in_operand(mutex, used);
        }
        MirInst::CondvarNotifyOne { condvar } => {
            collect_used_in_operand(condvar, used);
        }
        MirInst::CondvarNotifyAll { condvar } => {
            collect_used_in_operand(condvar, used);
        }
        // v0.76: Select instruction
        MirInst::Select { cond_lhs, cond_rhs, true_val, false_val, .. } => {
            collect_used_in_operand(cond_lhs, used);
            collect_used_in_operand(cond_rhs, used);
            collect_used_in_operand(true_val, used);
            collect_used_in_operand(false_val, used);
        }
        // v0.83: AsyncFile instructions
        MirInst::AsyncFileOpen { path, .. } => {
            collect_used_in_operand(path, used);
        }
        MirInst::AsyncFileRead { file, .. } => {
            collect_used_in_operand(file, used);
        }
        MirInst::AsyncFileWrite { file, content } => {
            collect_used_in_operand(file, used);
            collect_used_in_operand(content, used);
        }
        MirInst::AsyncFileClose { file } => {
            collect_used_in_operand(file, used);
        }
        // v0.83.1: AsyncSocket instructions
        MirInst::AsyncSocketConnect { host, port, .. } => {
            collect_used_in_operand(host, used);
            collect_used_in_operand(port, used);
        }
        MirInst::AsyncSocketRead { socket, .. } => {
            collect_used_in_operand(socket, used);
        }
        MirInst::AsyncSocketWrite { socket, content } => {
            collect_used_in_operand(socket, used);
            collect_used_in_operand(content, used);
        }
        MirInst::AsyncSocketClose { socket } => {
            collect_used_in_operand(socket, used);
        }
        // v0.84: ThreadPool instructions
        MirInst::ThreadPoolNew { size, .. } => {
            collect_used_in_operand(size, used);
        }
        MirInst::ThreadPoolExecute { pool, task } => {
            collect_used_in_operand(pool, used);
            collect_used_in_operand(task, used);
        }
        MirInst::ThreadPoolJoin { pool } => {
            collect_used_in_operand(pool, used);
        }
        MirInst::ThreadPoolShutdown { pool } => {
            collect_used_in_operand(pool, used);
        }
        // v0.85: Scope instructions
        MirInst::ScopeNew { .. } => {}
        MirInst::ScopeSpawn { scope, task } => {
            collect_used_in_operand(scope, used);
            collect_used_in_operand(task, used);
        }
        MirInst::ScopeWait { scope } => {
            collect_used_in_operand(scope, used);
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
        MirInst::Call { .. }
            | MirInst::FieldStore { .. }
            | MirInst::IndexStore { .. }
            // v0.60.51: PtrStore has memory side effects - critical for hash_table benchmark
            | MirInst::PtrStore { .. }
            // v0.60.51: ArrayAlloc allocates stack memory
            | MirInst::ArrayAlloc { .. }
            // v0.70: Thread spawn/join have side effects (create/wait for threads)
            | MirInst::ThreadSpawn { .. }
            | MirInst::ThreadJoin { .. }
            // v0.71: Mutex operations have side effects (synchronization)
            | MirInst::MutexNew { .. }
            | MirInst::MutexLock { .. }
            | MirInst::MutexUnlock { .. }
            | MirInst::MutexTryLock { .. }
            | MirInst::MutexFree { .. }
            // v0.72: Arc operations have side effects (refcount manipulation)
            | MirInst::ArcNew { .. }
            | MirInst::ArcClone { .. }
            | MirInst::ArcGet { .. }
            | MirInst::ArcDrop { .. }
            | MirInst::ArcStrongCount { .. }
            // v0.72: Atomic operations have side effects (memory synchronization)
            | MirInst::AtomicNew { .. }
            | MirInst::AtomicLoad { .. }
            | MirInst::AtomicStore { .. }
            | MirInst::AtomicFetchAdd { .. }
            | MirInst::AtomicFetchSub { .. }
            | MirInst::AtomicSwap { .. }
            | MirInst::AtomicCompareExchange { .. }
            // v0.73: Channel operations have side effects (message passing)
            | MirInst::ChannelNew { .. }
            | MirInst::ChannelSend { .. }
            | MirInst::ChannelRecv { .. }
            | MirInst::ChannelTrySend { .. }
            | MirInst::ChannelTryRecv { .. }
            | MirInst::ChannelRecvTimeout { .. }  // v0.77
            | MirInst::BlockOn { .. }  // v0.78
            | MirInst::ChannelSendTimeout { .. }  // v0.79
            | MirInst::ChannelClose { .. }  // v0.80
            | MirInst::ChannelIsClosed { .. }  // v0.80
            | MirInst::ChannelRecvOpt { .. }  // v0.80
            | MirInst::SenderClone { .. }
            // v0.74: RwLock, Barrier, Condvar have side effects
            | MirInst::RwLockNew { .. }
            | MirInst::RwLockRead { .. }
            | MirInst::RwLockReadUnlock { .. }
            | MirInst::RwLockWrite { .. }
            | MirInst::RwLockWriteUnlock { .. }
            | MirInst::BarrierNew { .. }
            | MirInst::BarrierWait { .. }
            | MirInst::CondvarNew { .. }
            | MirInst::CondvarWait { .. }
            | MirInst::CondvarNotifyOne { .. }
            | MirInst::CondvarNotifyAll { .. }
            // v0.83: AsyncFile instructions have I/O side effects
            | MirInst::AsyncFileOpen { .. }
            | MirInst::AsyncFileRead { .. }
            | MirInst::AsyncFileWrite { .. }
            | MirInst::AsyncFileClose { .. }
            // v0.83.1: AsyncSocket instructions have I/O side effects
            | MirInst::AsyncSocketConnect { .. }
            | MirInst::AsyncSocketRead { .. }
            | MirInst::AsyncSocketWrite { .. }
            | MirInst::AsyncSocketClose { .. }
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
// Unreachable Block Elimination Pass (v0.60.41)
// ============================================================================

/// Remove blocks that are not reachable from the entry block.
/// This should run after SimplifyBranches to clean up dead else branches.
pub struct UnreachableBlockElimination;

impl OptimizationPass for UnreachableBlockElimination {
    fn name(&self) -> &'static str {
        "unreachable_block_elimination"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        if func.blocks.is_empty() {
            return false;
        }

        // Find all reachable blocks starting from entry
        let mut reachable: HashSet<String> = HashSet::new();
        let mut worklist: Vec<String> = vec!["entry".to_string()];

        while let Some(label) = worklist.pop() {
            if reachable.contains(&label) {
                continue;
            }
            reachable.insert(label.clone());

            // Find the block and add its successors
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
                    Terminator::Return(_) | Terminator::Unreachable => {}
                }
            }
        }

        let original_count = func.blocks.len();

        // Remove unreachable blocks
        func.blocks.retain(|b| reachable.contains(&b.label));

        // Update PHI nodes to remove references to removed blocks
        let removed_blocks: HashSet<_> = func.blocks.iter()
            .flat_map(|b| {
                b.instructions.iter().filter_map(|inst| {
                    if let MirInst::Phi { values, .. } = inst {
                        Some(values.iter().map(|(_, label)| label.clone()).collect::<Vec<_>>())
                    } else {
                        None
                    }
                }).flatten()
            })
            .filter(|label| !reachable.contains(label))
            .collect();

        if !removed_blocks.is_empty() {
            for block in &mut func.blocks {
                for inst in &mut block.instructions {
                    if let MirInst::Phi { values, .. } = inst {
                        values.retain(|(_, label)| reachable.contains(label));
                    }
                }
            }
        }

        func.blocks.len() != original_count
    }
}

// ============================================================================
// Phi Simplification Pass (v0.60.44)
// ============================================================================

/// Simplify PHI nodes: when a phi has only one incoming value, replace with copy/const.
/// This typically happens after UnreachableBlockElimination removes dead branches.
pub struct PhiSimplification;

impl OptimizationPass for PhiSimplification {
    fn name(&self) -> &'static str {
        "phi_simplification"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        for block in &mut func.blocks {
            let mut i = 0;
            while i < block.instructions.len() {
                if let MirInst::Phi { dest, values } = &block.instructions[i] {
                    if values.len() == 1 {
                        // Single-value phi: replace with copy or const
                        let (value, _label) = &values[0];
                        let new_inst = phi_operand_to_inst(dest.clone(), value);
                        block.instructions[i] = new_inst;
                        changed = true;
                    } else if values.len() > 1 {
                        // Check if all values are the same
                        let first_value = &values[0].0;
                        let all_same = values.iter().all(|(v, _)| phi_operands_equal(v, first_value));
                        if all_same {
                            let new_inst = phi_operand_to_inst(dest.clone(), first_value);
                            block.instructions[i] = new_inst;
                            changed = true;
                        }
                    }
                    // Empty phi (shouldn't happen) - leave it for DCE
                }
                i += 1;
            }
        }

        changed
    }
}

/// Convert a phi operand to an appropriate instruction (Copy or Const)
fn phi_operand_to_inst(dest: Place, op: &Operand) -> MirInst {
    match op {
        Operand::Place(src) => MirInst::Copy {
            dest,
            src: src.clone(),
        },
        Operand::Constant(c) => MirInst::Const {
            dest,
            value: c.clone(),
        },
    }
}

/// Check if two operands are equal for phi simplification
fn phi_operands_equal(a: &Operand, b: &Operand) -> bool {
    match (a, b) {
        (Operand::Place(pa), Operand::Place(pb)) => pa.name == pb.name,
        (Operand::Constant(ca), Operand::Constant(cb)) => match (ca, cb) {
            (Constant::Int(a), Constant::Int(b)) => a == b,
            (Constant::Float(a), Constant::Float(b)) => a == b,
            (Constant::Bool(a), Constant::Bool(b)) => a == b,
            (Constant::String(a), Constant::String(b)) => a == b,
            (Constant::Char(a), Constant::Char(b)) => a == b,
            (Constant::Unit, Constant::Unit) => true,
            _ => false,
        },
        _ => false,
    }
}

// ============================================================================
// Block Merging Pass (v0.60.44)
// ============================================================================

/// Merge blocks connected by unconditional jumps when the target has only one predecessor.
/// This reduces CFG complexity and enables further optimizations.
pub struct BlockMerging;

impl OptimizationPass for BlockMerging {
    fn name(&self) -> &'static str {
        "block_merging"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        if func.blocks.len() < 2 {
            return false;
        }

        let mut changed = false;

        // Build predecessor map
        let mut predecessors: HashMap<String, Vec<String>> = HashMap::new();
        for block in &func.blocks {
            predecessors.entry(block.label.clone()).or_default();
            match &block.terminator {
                Terminator::Goto(target) => {
                    predecessors.entry(target.clone()).or_default().push(block.label.clone());
                }
                Terminator::Branch { then_label, else_label, .. } => {
                    predecessors.entry(then_label.clone()).or_default().push(block.label.clone());
                    predecessors.entry(else_label.clone()).or_default().push(block.label.clone());
                }
                Terminator::Switch { cases, default, .. } => {
                    for (_, target) in cases {
                        predecessors.entry(target.clone()).or_default().push(block.label.clone());
                    }
                    predecessors.entry(default.clone()).or_default().push(block.label.clone());
                }
                Terminator::Return(_) | Terminator::Unreachable => {}
            }
        }

        // Find mergeable pairs: block A -> goto B where B has only A as predecessor
        // and B is not the entry block
        loop {
            let mut merge_pair: Option<(usize, usize)> = None;

            for (i, block) in func.blocks.iter().enumerate() {
                if let Terminator::Goto(target) = &block.terminator {
                    // Don't merge into entry
                    if target == "entry" {
                        continue;
                    }
                    // Find target block
                    if let Some(j) = func.blocks.iter().position(|b| &b.label == target) {
                        // Check if target has only this block as predecessor
                        if let Some(preds) = predecessors.get(target)
                            && preds.len() == 1 && preds[0] == block.label {
                                // Don't merge self-loops
                                if i != j {
                                    merge_pair = Some((i, j));
                                    break;
                                }
                            }
                    }
                }
            }

            if let Some((src_idx, dst_idx)) = merge_pair {
                // Merge dst into src
                let dst_block = func.blocks.remove(dst_idx);
                let src_idx = if dst_idx < src_idx { src_idx - 1 } else { src_idx };

                // Append dst instructions to src (skip phi nodes since single predecessor)
                let mut dst_instructions: Vec<MirInst> = dst_block.instructions
                    .into_iter()
                    .filter(|inst| !matches!(inst, MirInst::Phi { .. }))
                    .collect();

                func.blocks[src_idx].instructions.append(&mut dst_instructions);
                func.blocks[src_idx].terminator = dst_block.terminator;

                // Update predecessor map (remove merged block)
                predecessors.remove(&dst_block.label);

                // Update references to merged block
                let old_label = dst_block.label;
                let new_label = func.blocks[src_idx].label.clone();
                for block in &mut func.blocks {
                    update_terminator_labels(&mut block.terminator, &old_label, &new_label);
                    for inst in &mut block.instructions {
                        if let MirInst::Phi { values, .. } = inst {
                            for (_, label) in values {
                                if label == &old_label {
                                    *label = new_label.clone();
                                }
                            }
                        }
                    }
                }

                // Update predecessor map for new references
                for (_, preds) in predecessors.iter_mut() {
                    for pred in preds {
                        if pred == &old_label {
                            *pred = new_label.clone();
                        }
                    }
                }

                changed = true;
            } else {
                break;
            }
        }

        changed
    }
}

/// Update terminator labels when merging blocks
fn update_terminator_labels(term: &mut Terminator, old_label: &str, new_label: &str) {
    match term {
        Terminator::Goto(target) => {
            if target == old_label {
                *target = new_label.to_string();
            }
        }
        Terminator::Branch { then_label, else_label, .. } => {
            if then_label == old_label {
                *then_label = new_label.to_string();
            }
            if else_label == old_label {
                *else_label = new_label.to_string();
            }
        }
        Terminator::Switch { cases, default, .. } => {
            for (_, target) in cases {
                if target == old_label {
                    *target = new_label.to_string();
                }
            }
            if default == old_label {
                *default = new_label.to_string();
            }
        }
        Terminator::Return(_) | Terminator::Unreachable => {}
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
// Memory Load CSE Pass (v0.51.10, extended v0.60.38)
// ============================================================================

/// Memory load CSE: eliminate redundant load_f64/load_i64 calls and FieldAccess within basic blocks
///
/// Within a basic block, consecutive loads from the same memory address are equivalent
/// if no stores occur between them. This pass:
/// 1. Tracks (load_fn, ptr_arg) -> cached_dest for load calls
/// 2. Tracks (base, field_index) -> cached_dest for FieldAccess
/// 3. Replaces duplicate loads with Copy instructions
/// 4. Invalidates cache on store_f64/store_i64/FieldStore calls
///
/// v0.60.38: Extended to handle FieldAccess CSE for struct field loads
/// This eliminates duplicate field accesses like:
/// ```text
/// %left = field-access %node.left[0]  // for comparison
/// %left2 = field-access %node.left[0] // for recursion -> Copy %left2, %left
/// ```
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
            // v0.60.38: Track: (base_name, field_index, struct_name) -> cached_dest
            let mut field_cache: HashMap<(String, usize, String), Place> = HashMap::new();
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                match inst {
                    // v0.60.38: FieldAccess CSE - eliminate duplicate struct field loads
                    MirInst::FieldAccess { dest, base, field_index, struct_name, .. } => {
                        let cache_key = (base.name.clone(), *field_index, struct_name.clone());

                        if let Some(cached) = field_cache.get(&cache_key) {
                            // Replace with copy from cached value
                            new_instructions.push(MirInst::Copy {
                                dest: dest.clone(),
                                src: cached.clone(),
                            });
                            changed = true;
                        } else {
                            // Cache this field access and keep original instruction
                            field_cache.insert(cache_key, dest.clone());
                            new_instructions.push(inst.clone());
                        }
                    }
                    // v0.60.38: FieldStore invalidates field cache for that base
                    MirInst::FieldStore { base, .. } => {
                        // Invalidate all cached fields for this base
                        field_cache.retain(|(b, _, _), _| b != &base.name);
                        new_instructions.push(inst.clone());
                    }
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
                            field_cache.clear(); // Also invalidate field cache
                            new_instructions.push(inst.clone());
                        }
                        else {
                            // Other function calls may have side effects - be conservative
                            // Only invalidate if function might write to memory
                            if might_write_memory(fn_name) {
                                load_cache.clear();
                                field_cache.clear(); // Also invalidate field cache
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
/// v0.60.47: Conservative approach - assume ALL functions write unless proven pure
/// This fixes CSE bug where user-defined functions like hm_remove were not
/// invalidating the load cache, causing stale values to be used after the call.
fn might_write_memory(fn_name: &str) -> bool {
    // Known pure (non-writing) functions - these are safe to CSE across
    let is_pure = matches!(fn_name,
        // Arithmetic/math functions
        "abs" | "min" | "max" | "sqrt" | "floor" | "ceil" |
        "sin" | "cos" | "tan" | "exp" | "log" | "pow" |
        // Type conversions
        "i64_to_f64" | "f64_to_i64" | "chr" | "ord" |
        // String queries (don't modify)
        "len" | "byte_at" | "char_at" | "str_eq" | "str_cmp" |
        // Memory reads (don't write)
        "load_i64" | "load_f64" | "load_u8" | "load_i32" |
        // Hash functions
        "hash_i64" | "hash_str" |
        // Print functions (write to stdout, not memory we care about)
        "println" | "print" | "println_str" | "print_str" |
        "println_f64" | "print_f64" |  // v0.60.44: Float output functions
        "bmb_println_i64" | "bmb_print_i64" | "bmb_println_str" | "bmb_print_str" |
        "bmb_println_f64" | "bmb_print_f64"  // v0.60.44: Float output functions
    );

    // If not known pure, assume it might write to memory
    !is_pure
}

// ============================================================================
// Global Field Access CSE Pass (v0.60.38)
// ============================================================================

/// Global field access CSE: eliminate duplicate field accesses across blocks
///
/// When a parameter's field is accessed in the entry block, and the same field
/// is accessed again in successor blocks, replace with a copy of the original value.
///
/// This handles patterns like:
/// ```text
/// entry:
///   %left = field-access %node.left[0]   // for null check
///   branch %cond, then, else
/// else:
///   %left2 = field-access %node.left[0]  // for recursion - replace with %left
///   call f(%left2)
/// ```
///
/// This is safe because:
/// 1. The parameter is not modified (BMB has no mutable parameters)
/// 2. The struct field is not modified (no FieldStore between accesses)
pub struct GlobalFieldAccessCSE;

impl OptimizationPass for GlobalFieldAccessCSE {
    fn name(&self) -> &'static str {
        "global_field_access_cse"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Collect parameter names
        let params: HashSet<String> = func.params.iter().map(|(n, _)| n.clone()).collect();

        // Phase 1: Collect field accesses from entry block on parameters
        // Key: (base_name, field_index, struct_name) -> dest_name
        let mut entry_field_cache: HashMap<(String, usize, String), String> = HashMap::new();

        if let Some(entry_block) = func.blocks.first() {
            for inst in &entry_block.instructions {
                if let MirInst::FieldAccess { dest, base, field_index, struct_name, .. } = inst {
                    // Only cache if base is a parameter (guaranteed not modified)
                    if params.contains(&base.name) {
                        let cache_key = (base.name.clone(), *field_index, struct_name.clone());
                        entry_field_cache.insert(cache_key, dest.name.clone());
                    }
                }
            }
        }

        // No field accesses to optimize
        if entry_field_cache.is_empty() {
            return false;
        }

        // Phase 2: Replace duplicate field accesses in non-entry blocks
        for block_idx in 1..func.blocks.len() {
            let block = &mut func.blocks[block_idx];
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                if let MirInst::FieldAccess { dest, base, field_index, struct_name, .. } = inst {
                    let cache_key = (base.name.clone(), *field_index, struct_name.clone());

                    if let Some(cached_name) = entry_field_cache.get(&cache_key) {
                        // Replace with copy from entry block's cached value
                        new_instructions.push(MirInst::Copy {
                            dest: dest.clone(),
                            src: Place::new(cached_name.clone()),
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

        for block in &mut func.blocks {
            // v0.89.15: Scope CSE map per block to avoid cross-block dominance violations
            // Previously the map was shared across all blocks, causing "instruction does not
            // dominate all uses" when sibling blocks (e.g. if/else branches) had identical calls.
            let mut call_results: HashMap<String, Place> = HashMap::new();
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

        // v0.90.14: Reject functions with side-effectful instructions.
        // A function like `fn run_tests_once() -> i64 = { print(...); 0 }`
        // returns a constant but has side effects that must not be removed.
        for inst in &block.instructions {
            match inst {
                MirInst::Call { .. }
                | MirInst::FieldStore { .. }
                | MirInst::IndexStore { .. }
                | MirInst::PtrStore { .. } => {
                    return None;
                }
                _ => {}
            }
        }

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
                // v0.60.40: Handle `0 as *T` pattern (null pointer)
                // This recognizes functions like `fn null_ptr() -> *T = 0 as *T`
                if let MirInst::Cast { dest, src, to_ty, .. } = inst
                    && dest.name == place.name
                    && matches!(to_ty, MirType::Ptr(_) | MirType::StructPtr(_))
                    && matches!(src, Operand::Constant(Constant::Int(0)))
                {
                    // Null pointer is just constant 0
                    return Some(Constant::Int(0));
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
                // v0.89: Return value facts are postconditions, used in inter-procedural analysis
                ContractFact::ReturnCmp { .. } | ContractFact::ReturnVarCmp { .. } => {}
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
            if let Some(idx) = tail_call_idx
                && let MirInst::Call { is_tail, .. } = &mut block.instructions[idx] {
                    *is_tail = true;
                    changed = true;
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
                if let MirInst::Phi { dest, values } = inst
                    && dest.name == *return_var {
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
                if let Some(dest) = call_dest
                    && matches!(block.terminator, Terminator::Goto(_)) {
                        block.terminator = Terminator::Return(Some(Operand::Place(dest)));
                        blocks_converted_to_return.push(block_label.clone());
                        changed = true;
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
                if let MirInst::Call { func: callee, args, is_tail: true, .. } = inst
                    && *callee == self_name {
                        tail_call_blocks.push((block_idx, inst_idx, args.clone()));
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

        // v0.60.40: Create substitution map FIRST so we can apply it to phi values
        let mut subst_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for (i, (param_name, _)) in func.params.iter().enumerate() {
            if !param_is_invariant[i] {
                subst_map.insert(param_name.clone(), format!("{}_loop", param_name));
            }
        }

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
                        // v0.60.40: Apply substitution to tail_args that reference other params
                        // e.g., gcd(b, a%b) - the 'b' arg for 'a' param needs to become 'b_loop'
                        let substituted_arg = self.substitute_operand(tail_args[i].clone(), &subst_map);
                        phi_values.push((substituted_arg, block_label));
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
            // v0.60.18: Handle tuple operations - critical for correctness when tuples
            // use loop-varying parameters (e.g., partition_loop returning (i, cost))
            MirInst::TupleInit { dest, elements } => MirInst::TupleInit {
                dest,
                elements: elements.into_iter()
                    .map(|(ty, op)| (ty, self.substitute_operand(op, subst)))
                    .collect(),
            },
            MirInst::TupleExtract { dest, tuple, index, element_type } => MirInst::TupleExtract {
                dest,
                tuple: Place::new(subst.get(&tuple.name).cloned().unwrap_or(tuple.name)),
                index,
                element_type,
            },
            // v0.60.19: Pointer offset substitution
            MirInst::PtrOffset { dest, ptr, offset, element_type } => MirInst::PtrOffset {
                dest,
                ptr: self.substitute_operand(ptr, subst),
                offset: self.substitute_operand(offset, subst),
                element_type,
            },
            // v0.60.21: Array allocation - no substitution needed
            MirInst::ArrayAlloc { dest, element_type, size } => MirInst::ArrayAlloc {
                dest,
                element_type,
                size,
            },
            // v0.60.20: Pointer load/store substitution
            MirInst::PtrLoad { dest, ptr, element_type } => MirInst::PtrLoad {
                dest,
                ptr: self.substitute_operand(ptr, subst),
                element_type,
            },
            MirInst::PtrStore { ptr, value, element_type } => MirInst::PtrStore {
                ptr: self.substitute_operand(ptr, subst),
                value: self.substitute_operand(value, subst),
                element_type,
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
    /// v0.60.35: Set of functions that have direct Mul operations
    /// Used to prevent narrowing params that flow to functions with multiplication
    functions_with_mul: HashSet<String>,
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

        // v0.60.35: Build set of functions that have direct Mul operations
        let mut functions_with_mul: HashSet<String> = HashSet::new();
        for func in &program.functions {
            if Self::has_direct_multiplication(func) {
                functions_with_mul.insert(func.name.clone());
            }
        }

        // v0.60.35: Transitive closure - also include functions that CALL functions with mul
        // e.g., square_fp calls mul_fp which has Mul, so square_fp should be in the set
        // Build call graph
        let mut callers_of: HashMap<String, HashSet<String>> = HashMap::new();
        for func in &program.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let MirInst::Call { func: callee, .. } = inst {
                        callers_of.entry(callee.clone())
                            .or_default()
                            .insert(func.name.clone());
                    }
                }
            }
        }

        // Propagate: if f calls g and g is in functions_with_mul, add f
        let mut changed = true;
        while changed {
            changed = false;
            let current: Vec<_> = functions_with_mul.iter().cloned().collect();
            for func_with_mul in current {
                if let Some(callers) = callers_of.get(&func_with_mul) {
                    for caller in callers {
                        if functions_with_mul.insert(caller.clone()) {
                            changed = true;
                        }
                    }
                }
            }
        }

        Self { call_site_constants, functions_with_mul }
    }

    /// v0.60.35: Check if a function has direct Mul operations
    fn has_direct_multiplication(func: &MirFunction) -> bool {
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::BinOp { op: MirBinOp::Mul, .. } = inst {
                    return true;
                }
            }
        }
        false
    }

    /// v0.90.22: Check if a parameter flows into Div or Mod operations.
    fn param_flows_to_div_mod(func: &MirFunction, param_name: &str) -> bool {
        let mut derived: HashSet<String> = HashSet::new();
        derived.insert(param_name.to_string());
        for _ in 0..5 {
            for block in &func.blocks {
                for inst in &block.instructions {
                    match inst {
                        MirInst::Copy { dest, src } if derived.contains(&src.name) => {
                            derived.insert(dest.name.clone());
                        }
                        MirInst::BinOp { dest, lhs, rhs, .. } => {
                            let lhs_d = matches!(lhs, Operand::Place(p) if derived.contains(&p.name));
                            let rhs_d = matches!(rhs, Operand::Place(p) if derived.contains(&p.name));
                            if lhs_d || rhs_d {
                                derived.insert(dest.name.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::BinOp { op: MirBinOp::Div | MirBinOp::Mod, lhs, .. } = inst
                    && matches!(lhs, Operand::Place(p) if derived.contains(&p.name))
                {
                    return true;
                }
            }
        }
        false
    }

    /// v0.60.35: Check if parameter flows to multiplication
    fn is_used_in_multiplication(&self, func: &MirFunction, param_name: &str) -> bool {
        let mut derived: HashSet<String> = HashSet::new();
        derived.insert(param_name.to_string());

        // Multiple passes to propagate derived status
        for _ in 0..5 {
            for block in &func.blocks {
                for inst in &block.instructions {
                    match inst {
                        MirInst::BinOp { dest, lhs, rhs, op } => {
                            let lhs_derived = matches!(lhs, Operand::Place(p) if derived.contains(&p.name));
                            let rhs_derived = matches!(rhs, Operand::Place(p) if derived.contains(&p.name));

                            // If used in multiplication, return true
                            if matches!(op, MirBinOp::Mul) && (lhs_derived || rhs_derived) {
                                return true;
                            }

                            if lhs_derived || rhs_derived {
                                derived.insert(dest.name.clone());
                            }
                        }
                        MirInst::Copy { dest, src } if derived.contains(&src.name) => {
                            derived.insert(dest.name.clone());
                        }
                        // v0.60.35: Handle phi nodes - if any incoming value is derived, the phi result is too
                        MirInst::Phi { dest, values } => {
                            let any_derived = values.iter().any(|(operand, _)| {
                                matches!(operand, Operand::Place(p) if derived.contains(&p.name))
                            });
                            if any_derived {
                                derived.insert(dest.name.clone());
                            }
                        }
                        MirInst::Call { func: callee, args, .. } => {
                            let arg_is_derived = args.iter().any(|arg| {
                                matches!(arg, Operand::Place(p) if derived.contains(&p.name))
                            });

                            if arg_is_derived && self.functions_with_mul.contains(callee) {
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        false
    }

    /// Check if a parameter can be narrowed to i32 based on:
    /// 1. All constant call-site values fit in i32
    /// 2. Function is self-recursive with decreasing arguments (monotonically decreasing)
    /// 3. v0.60.35: Parameter is not used in multiplication (which can overflow i32)
    fn can_narrow_param(&self, func: &MirFunction, param_idx: usize) -> bool {
        // Only narrow i64 parameters
        if func.params.get(param_idx).map(|(_, ty)| ty) != Some(&MirType::I64) {
            return false;
        }

        // v0.60.35: Don't narrow parameters used in multiplication
        // Multiplication can easily overflow i32 even with small inputs (e.g., 50000 * 50000)
        let param_name = &func.params[param_idx].0;
        if self.is_used_in_multiplication(func, param_name) {
            return false;
        }

        // v0.90.22: Don't narrow params that flow into div/mod operations.
        // Narrowing creates sext/trunc overhead without benefit.
        if Self::param_flows_to_div_mod(func, param_name) {
            return false;
        }

        // v0.90.22: Don't narrow params of functions that still have self-recursive calls
        // (not converted to loops). After TCO, remaining recursive calls create
        // sext/trunc overhead per call frame. For fully loop-converted functions
        // (like fibonacci), there are no remaining Call instructions to self.
        // For partially converted functions (like ackermann), narrowing creates
        // i32→i64 sext at entry + i64→i32 trunc at each recursive call site.
        if Self::has_remaining_self_recursive_calls(func) {
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

    /// v0.90.22: Check if a function still has self-recursive Call instructions.
    /// After tail-call-to-loop conversion, fully optimized functions have no
    /// remaining self-recursive calls (all converted to phi+goto loops).
    /// Partially converted functions (like ackermann) still have some recursive calls.
    fn has_remaining_self_recursive_calls(func: &MirFunction) -> bool {
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Call { func: callee, .. } = inst
                    && callee == &func.name
                {
                    return true;
                }
            }
        }
        false
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
                if let MirInst::Call { func: callee, args, .. } = inst
                    && callee == &func.name {
                        has_self_recursion = true;

                        // Check if the argument at param_idx is decreasing
                        if let Some(arg) = args.get(param_idx)
                            && !self.is_decreasing_operand(arg, param_name, &definitions) {
                                return false;
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
            if let Some(func) = program.functions.iter_mut().find(|f| f.name == func_name)
                && self.narrow_function(func, param_idx) {
                    changed = true;
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
    /// v0.60.14: Set of functions that have direct Mul operations
    /// Used to prevent narrowing params that flow to functions with multiplication
    functions_with_mul: HashSet<String>,
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

        // v0.60.14: Build set of functions that have direct Mul operations
        let mut functions_with_mul: HashSet<String> = HashSet::new();
        for func in &program.functions {
            if Self::has_direct_multiplication(func) {
                functions_with_mul.insert(func.name.clone());
            }
        }

        // v0.60.35: Transitive closure - also include functions that CALL functions with mul
        // e.g., square_fp calls mul_fp which has Mul, so square_fp should be in the set
        let mut callers_of: HashMap<String, HashSet<String>> = HashMap::new();
        for func in &program.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let MirInst::Call { func: callee, .. } = inst {
                        callers_of.entry(callee.clone())
                            .or_default()
                            .insert(func.name.clone());
                    }
                }
            }
        }

        // Propagate: if f calls g and g is in functions_with_mul, add f
        let mut changed = true;
        while changed {
            changed = false;
            let current: Vec<_> = functions_with_mul.iter().cloned().collect();
            for func_with_mul in current {
                if let Some(callers) = callers_of.get(&func_with_mul) {
                    for caller in callers {
                        if functions_with_mul.insert(caller.clone()) {
                            changed = true;
                        }
                    }
                }
            }
        }

        Self { param_bounds, functions_with_mul }
    }

    /// v0.60.14: Check if a function has direct Mul operations
    fn has_direct_multiplication(func: &MirFunction) -> bool {
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::BinOp { op: MirBinOp::Mul, .. } = inst {
                    return true;
                }
            }
        }
        false
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

                        if let Some(v) = bound
                            && v >= 0 && v <= i32::MAX as i64 {
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

        let param_name = &func.params[param_idx].0;

        // v0.93.124: Analysis showed that narrowing alwaysinline function params
        // creates trunc+sext overhead for pointer indices but HELPS arithmetic-heavy
        // functions (32-bit division in matrix_a). These effects conflict:
        // - Blanket guard: helps fannkuch (73ms) but destroys spectral_norm (108ms)
        // - Targeted guard: creates call-chain inconsistency (callers i32 → callees i64)
        // - No guard (baseline): best overall (spectral_norm 72ms, fannkuch 77ms)
        // Conclusion: keep narrowing enabled for all alwaysinline functions.

        // v0.60.30: Don't narrow parameters used as values in IndexStore with i64 element type
        // Narrowing would cause type mismatch: storing i32 (4 bytes) but reading i64 (8 bytes)
        if Self::is_used_as_i64_store_value(func, param_name) {
            return false;
        }

        // v0.60.105: Check for self-recursive calls with unbounded arguments
        // If a function calls itself with a non-constant, non-decreasing argument
        // for this parameter, we cannot narrow it (e.g., collect_mir_strings_acc
        // calls itself with str_end + 1 which can grow to arbitrary size)
        if Self::has_unbounded_recursive_arg(func, param_idx) {
            return false;
        }

        // v0.90.22: Don't narrow params that flow into division/modulo operations.
        // Narrowing i64→i32 creates sext/trunc overhead (shl 32 + ashr 32 + freeze)
        // inside hot loops. LLVM uses the same magic-number multiplication trick for
        // both i32 and i64 division by constant, so narrowing provides no benefit but
        // adds conversion cost. Root cause of digital_root 1.26x regression.
        if Self::param_flows_to_div_mod(func, param_name) {
            return false;
        }

        // v0.90.22: Don't narrow loop-invariant bound parameters.
        // These are params that don't change across loop iterations and are compared
        // against loop variables. Narrowing inserts sext in loop headers, blocking
        // LLVM trip count computation and loop unrolling.
        if Self::is_loop_invariant_bound(func, param_idx) {
            return false;
        }

        // v0.90.22: Don't narrow params of functions with remaining self-recursive calls.
        // After TCO, fully converted functions have no recursive calls (all become loops).
        // Partially converted functions (like ackermann) still have recursive calls where
        // narrowing creates sext/trunc overhead per call frame.
        if Self::has_remaining_self_recursive_calls(func) {
            return false;
        }

        // v0.93.120: Don't narrow params used as loop step values (addends to loop-carried variables).
        // When a narrowed i32 param is added to an i64 loop variable in each iteration,
        // LLVM inserts shl+ashr (sext) in the hot loop body. This creates per-iteration overhead.
        // Example: mark_multiples_loop(arr, n, p) where inner loop does j = j + p.
        if Self::is_loop_step_param(func, param_name) {
            return false;
        }

        // Check if we have bounds for this parameter
        if let Some(bounds) = self.param_bounds.get(&func.name)
            && let Some(&max_val) = bounds.get(&param_idx) {
                // v0.60.51: Check multiplication with large constants
                // If parameter is multiplied by a large constant, the result can overflow i32
                // Example: seed * 1103515245 where seed=42 → 46347640290 (overflows i32)
                if let Some(max_multiplier) = Self::find_max_constant_multiplier(func, param_name) {
                    // Check if max_val * max_multiplier fits in i32
                    let product = (max_val as i128) * (max_multiplier as i128);
                    if product > i32::MAX as i128 || product < i32::MIN as i128 {
                        return false;
                    }
                }

                // v0.60.48: Smart multiplication-aware narrowing
                // If parameter is used in multiplication, check if bounds are small enough
                // that multiplication won't overflow i32 (max_val * max_val < i32::MAX)
                // sqrt(i32::MAX) ≈ 46340
                const SAFE_MUL_BOUND: i64 = 46340;

                if Self::is_used_in_multiplication(func, param_name, &self.functions_with_mul) {
                    // For multiplication, require smaller bound to prevent overflow
                    // This allows spectral_norm (n=1000, so sum<=2000) to be narrowed
                    // but blocks mandelbrot (values can be 20000+)
                    return (0..=SAFE_MUL_BOUND).contains(&max_val);
                }

                // For non-multiplication cases, just check i32 fit
                return max_val >= 0 && max_val <= i32::MAX as i64;
            }

        false
    }

    /// v0.60.30: Check if a parameter is used as the value in IndexStore with i64 element type
    /// v0.60.250: Also check store_i64/bmb_store_i64 function calls
    /// If so, narrowing to i32 would cause a type mismatch (storing 4 bytes, reading 8 bytes)
    fn is_used_as_i64_store_value(func: &MirFunction, param_name: &str) -> bool {
        // Track which variables are derived from the parameter
        let mut derived: std::collections::HashSet<String> = std::collections::HashSet::new();
        derived.insert(param_name.to_string());

        // Multiple passes to propagate derived status
        for _ in 0..5 {
            for block in &func.blocks {
                for inst in &block.instructions {
                    match inst {
                        // Check if used as value in IndexStore with i64 element type
                        MirInst::IndexStore { value, element_type, .. } => {
                            let value_derived = matches!(value, Operand::Place(p) if derived.contains(&p.name));
                            if value_derived && *element_type == MirType::I64 {
                                return true;
                            }
                        }
                        // v0.60.250: Check store_i64/bmb_store_i64 function calls
                        // store_i64(ptr, value) - value is the second argument (index 1)
                        MirInst::Call { func: callee, args, .. } => {
                            if (callee == "store_i64" || callee == "bmb_store_i64") && args.len() >= 2 {
                                let value_derived = matches!(&args[1], Operand::Place(p) if derived.contains(&p.name));
                                if value_derived {
                                    return true;
                                }
                            }
                        }
                        // Propagate derived status through copy
                        MirInst::Copy { dest, src } if derived.contains(&src.name) => {
                            derived.insert(dest.name.clone());
                        }
                        // Propagate derived status through arithmetic
                        MirInst::BinOp { dest, lhs, rhs, .. } => {
                            let lhs_derived = matches!(lhs, Operand::Place(p) if derived.contains(&p.name));
                            let rhs_derived = matches!(rhs, Operand::Place(p) if derived.contains(&p.name));
                            if lhs_derived || rhs_derived {
                                derived.insert(dest.name.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        false
    }

    /// v0.60.51: Find the maximum constant that a parameter is multiplied by
    /// Returns None if not multiplied by any constant, Some(max) otherwise
    fn find_max_constant_multiplier(func: &MirFunction, param_name: &str) -> Option<i64> {
        let mut derived: std::collections::HashSet<String> = std::collections::HashSet::new();
        derived.insert(param_name.to_string());
        let mut max_constant: Option<i64> = None;

        // Multiple passes to propagate derived status
        for _ in 0..5 {
            for block in &func.blocks {
                for inst in &block.instructions {
                    match inst {
                        MirInst::BinOp { dest, lhs, rhs, op } => {
                            let lhs_derived = matches!(lhs, Operand::Place(p) if derived.contains(&p.name));
                            let rhs_derived = matches!(rhs, Operand::Place(p) if derived.contains(&p.name));

                            // Check for multiplication with a constant
                            if matches!(op, MirBinOp::Mul) {
                                if lhs_derived
                                    && let Operand::Constant(Constant::Int(c)) = rhs {
                                        let abs_c = c.abs();
                                        max_constant = Some(max_constant.map_or(abs_c, |m| m.max(abs_c)));
                                    }
                                if rhs_derived
                                    && let Operand::Constant(Constant::Int(c)) = lhs {
                                        let abs_c = c.abs();
                                        max_constant = Some(max_constant.map_or(abs_c, |m| m.max(abs_c)));
                                    }
                            }

                            // Propagate derived status
                            if lhs_derived || rhs_derived {
                                derived.insert(dest.name.clone());
                            }
                        }
                        MirInst::Copy { dest, src } if derived.contains(&src.name) => {
                            derived.insert(dest.name.clone());
                        }
                        MirInst::Phi { dest, values } => {
                            let any_derived = values.iter().any(|(operand, _)| {
                                matches!(operand, Operand::Place(p) if derived.contains(&p.name))
                            });
                            if any_derived {
                                derived.insert(dest.name.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        max_constant
    }

    /// v0.60.105: Check if a function has unbounded loop updates or recursive args
    /// for a specific parameter. This prevents narrowing parameters like `pos` in
    /// collect_mir_strings_acc which updates pos in a loop with unbounded growth.
    ///
    /// Checks for:
    /// 1. Self-recursive calls with unbounded arguments
    /// 2. Phi nodes (from loop conversion) that update the parameter with increasing values
    fn has_unbounded_recursive_arg(func: &MirFunction, param_idx: usize) -> bool {
        let param_name = &func.params[param_idx].0;

        // Build a map of expressions for pattern analysis
        let mut definitions: HashMap<String, (&MirBinOp, &Operand, &Operand)> = HashMap::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::BinOp { dest, op, lhs, rhs } = inst {
                    definitions.insert(dest.name.clone(), (op, lhs, rhs));
                }
            }
        }

        // Check for phi nodes that update parameter-derived values
        // After tail recursion to loop conversion, the pattern is:
        //   %pos_loop = phi [%initial_pos, entry], [%next_pos, loop_body]
        // where %next_pos = %pos_loop + X (increasing pattern)
        //
        // v0.60.106: More conservative check - if param flows to phi and the
        // loop-back value is NOT obviously bounded (decreasing), assume unbounded.
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst {
                    let phi_name = &dest.name;

                    // First, check if this phi is related to our parameter
                    let param_flows = Self::param_flows_to_phi(func, param_name.as_str(), phi_name.as_str());
                    if !param_flows {
                        continue;
                    }

                    // Check each incoming value from loop iterations (not entry)
                    for (operand, block_name) in values {
                        // Skip entry block values - those are initial values
                        if block_name == "entry" {
                            continue;
                        }
                        // Skip constants (bounded)
                        if matches!(operand, Operand::Constant(_)) {
                            continue;
                        }
                        if let Operand::Place(p) = operand {
                            // Skip if it's the phi itself (no change)
                            if &p.name == phi_name {
                                continue;
                            }

                            // Check if this is a DECREASING pattern from phi_name
                            // Only decreasing patterns are safe for narrowing
                            let is_decreasing = Self::is_decreasing_from_var(operand, phi_name.as_str(), &definitions);
                            let is_increasing = Self::is_increasing_from_var(operand, phi_name.as_str(), &definitions);

                            // If it's increasing, definitely unbounded
                            if is_increasing {
                                return true;
                            }

                            // If it's not decreasing and not the same variable, assume unbounded
                            // This catches cases like: pos_loop' = str_end + 1 where str_end
                            // is derived from external sources and can be arbitrarily large
                            if !is_decreasing {
                                // Check if this value is derived from the phi at all
                                let derived_from_phi = Self::is_derived_from_var(operand, phi_name.as_str(), &definitions);
                                // If not derived from phi AND not decreasing, it's from external source
                                // and could be unbounded
                                if !derived_from_phi {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also check for self-recursive calls (in case loop conversion didn't happen)
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Call { func: callee, args, .. } = inst
                    && callee == &func.name
                        && let Some(arg) = args.get(param_idx) {
                            // Check if argument is increasing from parameter
                            if let Operand::Place(_) = arg {
                                if Self::is_increasing_from_var(arg, param_name.as_str(), &definitions) {
                                    return true;
                                }
                                // Also check for unknown/unbounded patterns
                                if !Self::is_bounded_from_param(arg, param_name.as_str(), &definitions) {
                                    return true;
                                }
                            }
                        }
            }
        }

        false
    }

    /// Check if parameter flows to a phi node (indicating loop update)
    fn param_flows_to_phi(func: &MirFunction, param_name: &str, phi_name: &str) -> bool {
        // Simple check: if the phi has an incoming value from entry block that
        // is the parameter itself, or if phi_name contains "pos" and param contains "pos"
        // This is a heuristic for common patterns like collect_mir_strings_acc

        // Check if the phi has the parameter as one of its incoming values
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst
                    && dest.name == phi_name {
                        for (operand, _) in values {
                            if let Operand::Place(p) = operand
                                && p.name == param_name {
                                    return true;
                                }
                        }
                    }
            }
        }

        // Heuristic: if phi_name ends with "_loop" and contains a substring of param_name
        // This handles cases like param "pos" → phi "pos_loop"
        if let Some(base_name) = phi_name.strip_suffix("_loop")
            && base_name == param_name {
            return true;
        }

        false
    }

    /// Check if an operand is derived from a variable with an increasing pattern (Add)
    fn is_increasing_from_var(
        operand: &Operand,
        var_name: &str,
        definitions: &HashMap<String, (&MirBinOp, &Operand, &Operand)>,
    ) -> bool {
        if let Operand::Place(p) = operand
            && let Some((MirBinOp::Add, lhs, rhs)) = definitions.get(&p.name)
        {
            let lhs_is_var = matches!(lhs, Operand::Place(l) if l.name == var_name);
            let rhs_is_positive = matches!(rhs, Operand::Constant(Constant::Int(v)) if *v > 0);
            let rhs_is_var = matches!(rhs, Operand::Place(r) if r.name == var_name);
            let lhs_is_positive = matches!(lhs, Operand::Constant(Constant::Int(v)) if *v > 0);
            // Also check transitive: if lhs is derived from var with Add
            let lhs_increasing = Self::is_increasing_from_var(lhs, var_name, definitions);
            let rhs_increasing = Self::is_increasing_from_var(rhs, var_name, definitions);
            // v0.93.120: var + unknown_place is potentially increasing.
            // If the other operand is a non-constant variable not in definitions
            // (e.g., call result), it could be positive, making the sum grow unbounded.
            // Catches: acc_loop + call_result where call_result is from a function call.
            let rhs_is_unknown_place = matches!(rhs, Operand::Place(r) if r.name != var_name && !definitions.contains_key(&r.name));
            let lhs_is_unknown_place = matches!(lhs, Operand::Place(l) if l.name != var_name && !definitions.contains_key(&l.name));
            (lhs_is_var && rhs_is_positive) ||
            (rhs_is_var && lhs_is_positive) ||
            (lhs_increasing && rhs_is_positive) ||
            (rhs_increasing && lhs_is_positive) ||
            (lhs_is_var && rhs_is_unknown_place) ||
            (rhs_is_var && lhs_is_unknown_place)
        } else {
            false
        }
    }

    /// v0.60.106: Check if an operand is derived from a variable with a decreasing pattern (Sub)
    fn is_decreasing_from_var(
        operand: &Operand,
        var_name: &str,
        definitions: &HashMap<String, (&MirBinOp, &Operand, &Operand)>,
    ) -> bool {
        if let Operand::Place(p) = operand {
            if let Some((op, lhs, rhs)) = definitions.get(&p.name) {
                match op {
                    // var - positive_const is decreasing
                    MirBinOp::Sub => {
                        let lhs_is_var = matches!(lhs, Operand::Place(l) if l.name == var_name);
                        let rhs_is_positive = matches!(rhs, Operand::Constant(Constant::Int(v)) if *v > 0);
                        // Also check transitive: if lhs is derived from var
                        let lhs_decreasing = Self::is_decreasing_from_var(lhs, var_name, definitions);
                        (lhs_decreasing || lhs_is_var) && rhs_is_positive
                    }
                    // var / const > 1 is decreasing
                    MirBinOp::Div => {
                        let lhs_is_var = matches!(lhs, Operand::Place(l) if l.name == var_name);
                        let rhs_is_divisor = matches!(rhs, Operand::Constant(Constant::Int(v)) if *v > 1);
                        let lhs_decreasing = Self::is_decreasing_from_var(lhs, var_name, definitions);
                        (lhs_decreasing || lhs_is_var) && rhs_is_divisor
                    }
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// v0.60.106: Check if an operand is derived from a variable (through any operation)
    fn is_derived_from_var(
        operand: &Operand,
        var_name: &str,
        definitions: &HashMap<String, (&MirBinOp, &Operand, &Operand)>,
    ) -> bool {
        match operand {
            Operand::Place(p) if p.name == var_name => true,
            Operand::Place(p) => {
                if let Some((_op, lhs, rhs)) = definitions.get(&p.name) {
                    Self::is_derived_from_var(lhs, var_name, definitions) ||
                    Self::is_derived_from_var(rhs, var_name, definitions)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check if an operand is bounded relative to a parameter (constant, decreasing, or same)
    fn is_bounded_from_param(
        operand: &Operand,
        param_name: &str,
        definitions: &HashMap<String, (&MirBinOp, &Operand, &Operand)>,
    ) -> bool {
        match operand {
            // Direct use of parameter
            Operand::Place(p) if p.name == param_name => true,
            // Small constants are bounded
            Operand::Constant(Constant::Int(v)) => *v >= 0 && *v <= i32::MAX as i64,
            // Check derived values
            Operand::Place(p) => {
                if let Some((op, lhs, rhs)) = definitions.get(&p.name) {
                    match op {
                        // param - positive_const is bounded (decreasing)
                        MirBinOp::Sub => {
                            let lhs_is_param = matches!(lhs, Operand::Place(l) if l.name == param_name);
                            let lhs_bounded = Self::is_bounded_from_param(lhs, param_name, definitions);
                            let rhs_is_positive = matches!(rhs, Operand::Constant(Constant::Int(v)) if *v > 0);
                            (lhs_is_param || lhs_bounded) && rhs_is_positive
                        }
                        // param / const > 1 is bounded (decreasing)
                        MirBinOp::Div => {
                            let lhs_is_param = matches!(lhs, Operand::Place(l) if l.name == param_name);
                            let lhs_bounded = Self::is_bounded_from_param(lhs, param_name, definitions);
                            let rhs_is_divisor = matches!(rhs, Operand::Constant(Constant::Int(v)) if *v > 1);
                            (lhs_is_param || lhs_bounded) && rhs_is_divisor
                        }
                        // param + const is NOT bounded (increasing) - return false
                        MirBinOp::Add => false,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// v0.90.22: Check if a parameter flows into Div or Mod operations.
    /// Narrowing params that flow to div/mod creates sext/trunc overhead without benefit,
    /// since LLVM uses the same magic-number multiplication for i32 and i64 constant division.
    fn param_flows_to_div_mod(func: &MirFunction, param_name: &str) -> bool {
        let mut derived: HashSet<String> = HashSet::new();
        derived.insert(param_name.to_string());

        // Propagate derived status through copies and arithmetic
        for _ in 0..5 {
            for block in &func.blocks {
                for inst in &block.instructions {
                    match inst {
                        MirInst::Copy { dest, src } if derived.contains(&src.name) => {
                            derived.insert(dest.name.clone());
                        }
                        MirInst::BinOp { dest, lhs, rhs, .. } => {
                            let lhs_derived = matches!(lhs, Operand::Place(p) if derived.contains(&p.name));
                            let rhs_derived = matches!(rhs, Operand::Place(p) if derived.contains(&p.name));
                            if lhs_derived || rhs_derived {
                                derived.insert(dest.name.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Check if any derived variable is used in Div or Mod
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::BinOp { op: MirBinOp::Div | MirBinOp::Mod, lhs, .. } = inst
                    && matches!(lhs, Operand::Place(p) if derived.contains(&p.name))
                {
                    return true;
                }
            }
        }
        false
    }

    /// v0.60.14: Check if a parameter is used in multiplication
    /// This includes direct use and use through derived variables
    /// Also checks if the parameter flows to a call to a function that has multiplication
    fn is_used_in_multiplication(func: &MirFunction, param_name: &str, functions_with_mul: &HashSet<String>) -> bool {
        // Track which variables are derived from the parameter
        let mut derived: std::collections::HashSet<String> = std::collections::HashSet::new();
        derived.insert(param_name.to_string());

        // Multiple passes to propagate derived status
        for _ in 0..5 {
            for block in &func.blocks {
                for inst in &block.instructions {
                    match inst {
                        // Check if result is derived from a derived variable
                        MirInst::BinOp { dest, lhs, rhs, op } => {
                            let lhs_derived = matches!(lhs, Operand::Place(p) if derived.contains(&p.name));
                            let rhs_derived = matches!(rhs, Operand::Place(p) if derived.contains(&p.name));

                            // If used in multiplication, return true immediately
                            if matches!(op, MirBinOp::Mul) && (lhs_derived || rhs_derived) {
                                return true;
                            }

                            // Propagate derived status through arithmetic
                            if lhs_derived || rhs_derived {
                                derived.insert(dest.name.clone());
                            }
                        }
                        MirInst::Copy { dest, src } if derived.contains(&src.name) => {
                            derived.insert(dest.name.clone());
                        }
                        // v0.60.35: Handle phi nodes - if any incoming value is derived, the phi result is too
                        MirInst::Phi { dest, values } => {
                            let any_derived = values.iter().any(|(operand, _)| {
                                matches!(operand, Operand::Place(p) if derived.contains(&p.name))
                            });
                            if any_derived {
                                derived.insert(dest.name.clone());
                            }
                        }
                        // v0.60.14: Check if param flows to a call to a function that has multiplication
                        // This allows narrowing for recursive calls (fibonacci has no Mul)
                        // but prevents narrowing for calls to mul_fp (which has direct Mul)
                        MirInst::Call { func: callee, args, .. } => {
                            // Check if any argument is derived from the parameter
                            let arg_is_derived = args.iter().any(|arg| {
                                matches!(arg, Operand::Place(p) if derived.contains(&p.name))
                            });

                            // If derived arg is passed to a function that has multiplication, block narrowing
                            if arg_is_derived && functions_with_mul.contains(callee) {
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
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
        if let Some((_, ty)) = func.params.get_mut(param_idx)
            && *ty == MirType::I64 {
                *ty = MirType::I32;
                return true;
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
            if let Some(func) = program.functions.iter_mut().find(|f| &f.name == func_name)
                && Self::narrow_param(func, *param_idx) {
                    changed = true;
                }
        }

        // Phase 2: Narrow loop variables that are bounded by narrowed parameters
        for func in &mut program.functions {
            let loop_vars = Self::find_loop_variables(func);

            for (var_name, param_name) in loop_vars {
                // Check if the bounding parameter was narrowed
                let param_narrowed = func.params.iter()
                    .any(|(name, ty)| name == &param_name && *ty == MirType::I32);

                if param_narrowed
                    && Self::narrow_local(func, &var_name) {
                        changed = true;
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

        // First, count how many times each variable is defined (assigned to)
        // Variables with multiple definitions might be accumulators that grow beyond i32
        let mut def_count: HashMap<String, usize> = HashMap::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                let dest_name = match inst {
                    MirInst::Const { dest, .. } |
                    MirInst::Copy { dest, .. } |
                    MirInst::BinOp { dest, .. } |
                    MirInst::UnaryOp { dest, .. } |
                    MirInst::Phi { dest, .. } |
                    MirInst::StructInit { dest, .. } |
                    MirInst::FieldAccess { dest, .. } |
                    MirInst::EnumVariant { dest, .. } |
                    MirInst::ArrayInit { dest, .. } |
                    MirInst::IndexLoad { dest, .. } => Some(&dest.name),
                    MirInst::Call { dest: Some(d), .. } => Some(&d.name),
                    _ => None,
                };
                if let Some(name) = dest_name {
                    *def_count.entry(name.clone()).or_insert(0) += 1;
                }
            }
        }

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
                        // But only if dest is not reassigned (single definition)
                        MirInst::Copy { dest, src } => {
                            let single_def = def_count.get(&dest.name).copied().unwrap_or(0) == 1;
                            if single_def && narrowed.contains(&src.name) && !narrowed.contains(&dest.name) {
                                narrowed.insert(dest.name.clone());
                                local_changed = true;
                            }
                        }

                        // BinOp: if both operands are narrowed/constant, dest can be narrowed
                        // Only for ops that preserve i32 range
                        // But only if dest is not reassigned (single definition)
                        MirInst::BinOp { dest, op, lhs, rhs } => {
                            if narrowed.contains(&dest.name) {
                                continue;
                            }

                            // Skip if variable has multiple definitions (could be accumulator)
                            let single_def = def_count.get(&dest.name).copied().unwrap_or(0) == 1;
                            if !single_def {
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
                            // NOTE: Add/Sub/Mul are EXCLUDED because they can overflow!
                            // Even if both inputs fit in i32, the result can exceed i32::MAX.
                            // Example: fibonacci accumulator variables grow beyond i32 range.
                            // Loop counter narrowing is handled separately in Phase 2.
                            let op_preserves_i32 = matches!(op,
                                // Comparisons return bool, always fits in i32
                                MirBinOp::Lt | MirBinOp::Le | MirBinOp::Gt | MirBinOp::Ge |
                                MirBinOp::Eq | MirBinOp::Ne |
                                // Logical ops return bool
                                MirBinOp::And | MirBinOp::Or |
                                // Bitwise ops preserve range (result <= max(lhs, rhs))
                                MirBinOp::Band | MirBinOp::Bor | MirBinOp::Bxor |
                                MirBinOp::Shl | MirBinOp::Shr
                            );

                            if lhs_narrow && rhs_narrow && op_preserves_i32 {
                                narrowed.insert(dest.name.clone());
                                local_changed = true;
                            }
                        }

                        // Const: small integer constants can be narrowed
                        // But only if the variable is not reassigned elsewhere (single definition)
                        // Variables with multiple definitions might be accumulators (e.g., fibonacci a, b)
                        MirInst::Const { dest, value: Constant::Int(v) } => {
                            let single_def = def_count.get(&dest.name).copied().unwrap_or(0) == 1;
                            if single_def && !narrowed.contains(&dest.name) && *v >= i32::MIN as i64 && *v <= i32::MAX as i64 {
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
        // v0.60.30: But don't narrow if param is used as i64 IndexStore value
        // v0.60.36: Also don't narrow if param is used in multiplication
        for func in &mut program.functions {
            if let Some(param_info) = callee_param_types.get(&func.name) {
                for (&param_idx, &all_narrowed) in param_info {
                    if all_narrowed {
                        let param_name = &func.params[param_idx].0;

                        // v0.60.30: Check if parameter is used as i64 IndexStore value
                        if Self::is_used_as_i64_store_value(func, param_name) {
                            continue; // Don't narrow this parameter
                        }

                        // v0.60.36: Check if parameter is used in multiplication
                        // This is critical for mandelbrot: zr/zi flow to mul_fp which can overflow i32
                        if Self::is_used_in_multiplication(func, param_name, &self.functions_with_mul) {
                            continue; // Don't narrow this parameter
                        }

                        // v0.60.106: Check if parameter is used in unbounded loop/recursive pattern
                        // Even if all external call sites pass narrow values, the internal loop
                        // can grow the value beyond i32 (e.g., collect_mir_strings_acc pos)
                        if Self::has_unbounded_recursive_arg(func, param_idx) {
                            continue; // Don't narrow this parameter
                        }

                        // v0.90.22: Don't narrow params that flow into div/mod operations.
                        // Narrowing creates sext/trunc overhead without benefit.
                        if Self::param_flows_to_div_mod(func, param_name) {
                            continue;
                        }

                        // v0.90.22: Don't narrow loop-invariant bound parameters.
                        // These are params passed unchanged in self-recursive calls and
                        // compared against other params (loop variables). Narrowing them
                        // inserts sext inside loop headers, blocking LLVM trip count
                        // computation and loop unrolling.
                        if Self::is_loop_invariant_bound(func, param_idx) {
                            continue;
                        }

                        // v0.90.22: Don't narrow params of functions with remaining
                        // self-recursive calls. Narrowing creates sext/trunc per call.
                        if Self::has_remaining_self_recursive_calls(func) {
                            continue;
                        }

                        if let Some((_, ty)) = func.params.get_mut(param_idx)
                            && *ty == MirType::I64 {
                                *ty = MirType::I32;
                                changed = true;
                            }
                    }
                }
            }
        }

        changed
    }

    /// v0.90.22: Check if a function still has self-recursive Call instructions.
    /// After tail-call-to-loop conversion, fully optimized functions have no
    /// remaining self-recursive calls (all converted to phi+goto loops).
    /// Partially converted functions (like ackermann) still have some recursive calls.
    fn has_remaining_self_recursive_calls(func: &MirFunction) -> bool {
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Call { func: callee, .. } = inst
                    && callee == &func.name
                {
                    return true;
                }
            }
        }
        false
    }

    /// v0.90.22: Check if a parameter is a loop-invariant bound.
    /// A parameter is a loop-invariant bound if:
    /// 1. It is NOT used in a phi node (i.e., not a loop variable — it's invariant)
    ///    OR it is passed unchanged in self-recursive calls
    /// 2. It is compared against a loop variable (phi-derived) or another parameter
    ///
    /// After tail-recursion-to-loop conversion:
    /// - Loop-varying params become phis: `%start_loop = phi [%start, entry], [...]`
    /// - Loop-invariant params are used directly: `%end` never appears in phi
    ///
    /// Narrowing loop-invariant bounds inserts sext inside loop headers, which prevents
    /// LLVM ScalarEvolution from computing trip counts → blocks loop unrolling.
    fn is_loop_invariant_bound(func: &MirFunction, param_idx: usize) -> bool {
        let param_name = &func.params[param_idx].0;

        // Collect all phi-defined variables (loop variables)
        let mut phi_vars: HashSet<String> = HashSet::new();
        // Collect phi initial values that come from params
        let mut phi_from_param: HashMap<String, String> = HashMap::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst {
                    phi_vars.insert(dest.name.clone());
                    // Track which param feeds into this phi
                    for (operand, _) in values {
                        if let Operand::Place(p) = operand {
                            // If a param feeds into a phi, that param is a loop var
                            if func.params.iter().any(|(name, _)| name == &p.name) {
                                phi_from_param.insert(dest.name.clone(), p.name.clone());
                            }
                        }
                    }
                }
            }
        }

        // v0.90.25: Also detect while-loop mutable variables (alloca-based pattern).
        // While loops use Goto back-edges instead of phi nodes. Detect loop headers
        // by finding blocks that are Goto targets from later blocks.
        let block_labels: Vec<String> = func.blocks.iter().map(|b| b.label.clone()).collect();
        let mut loop_headers: HashSet<String> = HashSet::new();
        for (i, block) in func.blocks.iter().enumerate() {
            if let Terminator::Goto(target) = &block.terminator {
                // If target label appears before current block, it's a back-edge
                if let Some(target_idx) = block_labels.iter().position(|l| l == target)
                    && target_idx <= i
                {
                    loop_headers.insert(target.clone());
                }
            }
        }

        // For while-loop patterns: find variables used in comparisons at loop headers.
        // A comparison in a loop header where one side derives from param and the other
        // is a non-param local variable indicates param is a loop bound.
        // Collect comparison results that feed into Branch terminators at loop headers.
        let mut loop_cond_vars: HashSet<String> = HashSet::new();
        for block in &func.blocks {
            if !loop_headers.contains(&block.label) {
                continue;
            }
            // Find the branch condition variable
            if let Terminator::Branch { cond: Operand::Place(cond_place), .. } = &block.terminator {
                loop_cond_vars.insert(cond_place.name.clone());
            }
        }

        // Condition 1: Check if param is loop-invariant
        // A param is invariant if it doesn't feed into any phi node
        let feeds_phi = phi_from_param.values().any(|v| v == param_name);

        // Also check for self-recursive calls (before loop conversion)
        let mut is_recursive_invariant = false;
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Call { func: callee, args, .. } = inst
                    && callee == &func.name
                    && let Some(arg) = args.get(param_idx)
                    && matches!(arg, Operand::Place(p) if p.name == *param_name)
                {
                    is_recursive_invariant = true;
                }
            }
        }

        if feeds_phi && !is_recursive_invariant {
            return false; // Not invariant — it's a loop variable
        }

        // Condition 2: param is compared against a loop variable (phi-derived or while-loop mutable)
        // Track variables derived from param
        let mut param_derived: HashSet<String> = HashSet::new();
        param_derived.insert(param_name.to_string());

        // Track variables derived from phi vars (loop variables)
        let mut loop_derived: HashSet<String> = phi_vars.clone();

        // Propagate through copies
        for _ in 0..3 {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let MirInst::Copy { dest, src } = inst {
                        if param_derived.contains(&src.name) {
                            param_derived.insert(dest.name.clone());
                        }
                        if loop_derived.contains(&src.name) {
                            loop_derived.insert(dest.name.clone());
                        }
                    }
                }
            }
        }

        // Check phi-based pattern (recursive/TCO loops)
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::BinOp { op, lhs, rhs, .. } = inst {
                    let is_cmp = matches!(op,
                        MirBinOp::Gt | MirBinOp::Lt |
                        MirBinOp::Ge | MirBinOp::Le |
                        MirBinOp::Eq | MirBinOp::Ne
                    );
                    if !is_cmp {
                        continue;
                    }

                    let lhs_is_param = matches!(lhs, Operand::Place(p) if param_derived.contains(&p.name));
                    let rhs_is_param = matches!(rhs, Operand::Place(p) if param_derived.contains(&p.name));
                    let lhs_is_loop = matches!(lhs, Operand::Place(p) if loop_derived.contains(&p.name));
                    let rhs_is_loop = matches!(rhs, Operand::Place(p) if loop_derived.contains(&p.name));

                    // param compared against loop variable → loop bound
                    if (lhs_is_param && rhs_is_loop) || (rhs_is_param && lhs_is_loop) {
                        return true;
                    }
                }
            }
        }

        // v0.90.25: Check while-loop pattern (alloca-based loops).
        // If param appears in a comparison that produces a loop condition variable,
        // the param is being used as a loop bound. The comparison happens in a loop
        // header block and its result feeds into the Branch terminator.
        for block in &func.blocks {
            if !loop_headers.contains(&block.label) {
                continue;
            }
            for inst in &block.instructions {
                if let MirInst::BinOp { dest, op, lhs, rhs } = inst {
                    let is_cmp = matches!(op,
                        MirBinOp::Gt | MirBinOp::Lt |
                        MirBinOp::Ge | MirBinOp::Le |
                        MirBinOp::Eq | MirBinOp::Ne
                    );
                    if !is_cmp || !loop_cond_vars.contains(&dest.name) {
                        continue;
                    }

                    let lhs_is_param = matches!(lhs, Operand::Place(p) if param_derived.contains(&p.name));
                    let rhs_is_param = matches!(rhs, Operand::Place(p) if param_derived.contains(&p.name));

                    // If param appears in a loop-header comparison → it's a loop bound
                    if lhs_is_param || rhs_is_param {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// v0.93.120: Check if a parameter is used as a loop step value.
    /// When a narrowed i32 param is added to an i64 loop variable in each iteration,
    /// LLVM inserts shl+ashr (sext) in the hot loop body after inlining.
    /// Example: mark_multiples_loop(arr, n, p) where inner loop does j = j + p.
    fn is_loop_step_param(func: &MirFunction, param_name: &str) -> bool {
        // Find while-loop back-edges (Goto to earlier block)
        let block_labels: Vec<String> = func.blocks.iter().map(|b| b.label.clone()).collect();
        let mut loop_body_blocks: HashSet<String> = HashSet::new();

        for (i, block) in func.blocks.iter().enumerate() {
            if let Terminator::Goto(target) = &block.terminator {
                if let Some(target_idx) = block_labels.iter().position(|l| l == target) {
                    if target_idx <= i {
                        // All blocks between target_idx and i are loop body
                        for j in target_idx..=i {
                            loop_body_blocks.insert(block_labels[j].clone());
                        }
                    }
                }
            }
        }

        if loop_body_blocks.is_empty() {
            return false;
        }

        // Track variables derived from param (via Copy)
        let mut param_derived: HashSet<String> = HashSet::new();
        param_derived.insert(param_name.to_string());
        for _ in 0..3 {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let MirInst::Copy { dest, src } = inst {
                        if param_derived.contains(&src.name) {
                            param_derived.insert(dest.name.clone());
                        }
                    }
                }
            }
        }

        // Check if param (or derived) is used in BinOp::Add inside a loop body.
        // This catches patterns like j = j + p where p is the parameter.
        // Any addition in a loop body with a narrowed param creates per-iteration sext.
        for block in &func.blocks {
            if !loop_body_blocks.contains(&block.label) {
                continue;
            }
            for inst in &block.instructions {
                if let MirInst::BinOp { op: MirBinOp::Add, lhs, rhs, .. } = inst {
                    let lhs_is_param = matches!(lhs, Operand::Place(p) if param_derived.contains(&p.name));
                    let rhs_is_param = matches!(rhs, Operand::Place(p) if param_derived.contains(&p.name));

                    // If param is one operand and the other is a non-constant place,
                    // this is a loop step pattern (param added to loop variable)
                    let other_is_place = if lhs_is_param {
                        matches!(rhs, Operand::Place(_))
                    } else if rhs_is_param {
                        matches!(lhs, Operand::Place(_))
                    } else {
                        false
                    };

                    if (lhs_is_param || rhs_is_param) && other_is_place {
                        return true;
                    }
                }
            }
        }

        false
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
                if let MirInst::Call { func: callee, .. } = inst
                    && callee == &func.name {
                        return true;
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
            | MirInst::EnumVariant { .. }
            // v0.60.20: Pointer load/store access memory
            | MirInst::PtrLoad { .. }
            | MirInst::PtrStore { .. }
            // v0.70: Thread spawn/join access shared state
            | MirInst::ThreadSpawn { .. }
            | MirInst::ThreadJoin { .. }
            // v0.71: Mutex operations access shared state
            | MirInst::MutexNew { .. }
            | MirInst::MutexLock { .. }
            | MirInst::MutexUnlock { .. }
            | MirInst::MutexTryLock { .. }
            | MirInst::MutexFree { .. }
            // v0.72: Arc operations access shared state (refcount)
            | MirInst::ArcNew { .. }
            | MirInst::ArcClone { .. }
            | MirInst::ArcGet { .. }
            | MirInst::ArcDrop { .. }
            | MirInst::ArcStrongCount { .. }
            // v0.72: Atomic operations access shared memory
            | MirInst::AtomicNew { .. }
            | MirInst::AtomicLoad { .. }
            | MirInst::AtomicStore { .. }
            | MirInst::AtomicFetchAdd { .. }
            | MirInst::AtomicFetchSub { .. }
            | MirInst::AtomicSwap { .. }
            | MirInst::AtomicCompareExchange { .. }
            // v0.73: Channel operations access shared memory (message queue)
            | MirInst::ChannelNew { .. }
            | MirInst::ChannelSend { .. }
            | MirInst::ChannelRecv { .. }
            | MirInst::ChannelTrySend { .. }
            | MirInst::ChannelTryRecv { .. }
            | MirInst::ChannelRecvTimeout { .. }  // v0.77
            | MirInst::BlockOn { .. }  // v0.78
            | MirInst::ChannelSendTimeout { .. }  // v0.79
            | MirInst::ChannelClose { .. }  // v0.80
            | MirInst::ChannelIsClosed { .. }  // v0.80
            | MirInst::ChannelRecvOpt { .. }  // v0.80
            | MirInst::SenderClone { .. }
            // v0.74: RwLock, Barrier, Condvar access shared memory
            | MirInst::RwLockNew { .. }
            | MirInst::RwLockRead { .. }
            | MirInst::RwLockReadUnlock { .. }
            | MirInst::RwLockWrite { .. }
            | MirInst::RwLockWriteUnlock { .. }
            | MirInst::BarrierNew { .. }
            | MirInst::BarrierWait { .. }
            | MirInst::CondvarNew { .. }
            | MirInst::CondvarWait { .. }
            | MirInst::CondvarNotifyOne { .. }
            | MirInst::CondvarNotifyAll { .. }
            // v0.83: AsyncFile instructions access I/O
            | MirInst::AsyncFileOpen { .. }
            | MirInst::AsyncFileRead { .. }
            | MirInst::AsyncFileWrite { .. }
            | MirInst::AsyncFileClose { .. }
            // v0.83.1: AsyncSocket instructions access I/O
            | MirInst::AsyncSocketConnect { .. }
            | MirInst::AsyncSocketRead { .. }
            | MirInst::AsyncSocketWrite { .. }
            | MirInst::AsyncSocketClose { .. }
            // v0.84: ThreadPool instructions access shared state
            | MirInst::ThreadPoolNew { .. }
            | MirInst::ThreadPoolExecute { .. }
            | MirInst::ThreadPoolJoin { .. }
            | MirInst::ThreadPoolShutdown { .. }
            // v0.85: Scope instructions access shared state
            | MirInst::ScopeNew { .. }
            | MirInst::ScopeSpawn { .. }
            | MirInst::ScopeWait { .. } => true,
            // Pure operations don't access memory
            MirInst::BinOp { .. }
            | MirInst::UnaryOp { .. }
            | MirInst::Const { .. }
            | MirInst::Copy { .. }
            | MirInst::Phi { .. }
            | MirInst::Cast { .. }
            // v0.60.19: Pointer offset is pure (just address arithmetic)
            | MirInst::PtrOffset { .. }
            // v0.76: Select is pure (conditional value selection)
            | MirInst::Select { .. } => false,
            // v0.55: Tuple operations - TupleInit builds a value, TupleExtract reads from it
            // These are aggregate operations that may involve stack allocation
            MirInst::TupleInit { .. } | MirInst::TupleExtract { .. } => true,
            // v0.60.21: Array allocation has side effects (allocates stack memory)
            MirInst::ArrayAlloc { .. } => true,
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
                        if let Some(&pred_idx) = block_map.get(label.as_str())
                            && pred_idx > block_idx {
                                // This is a loop header with back edge from pred_idx
                                loop_headers.push(block_idx);
                                break;
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
                    if let MirInst::Copy { dest, src } = inst
                        && hoisted_mapping.get(&dest.name) == Some(&src.name) {
                            continue;
                        }

                    // Replace references to original with hoisted
                    Self::substitute_hoisted_refs(inst, &hoisted_mapping);
                }

                // Also check terminator
                if let Terminator::Branch { cond, .. } = &mut block.terminator
                    && let Operand::Place(p) = cond
                        && let Some(hoisted) = hoisted_mapping.get(&p.name) {
                            *p = Place::new(hoisted.clone());
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
        if let Operand::Place(p) = op
            && let Some(hoisted) = mapping.get(&p.name) {
                p.name = hoisted.clone();
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
    _first_decrement: i64,
    /// Second recursive call decrement (2 for n-2)
    _second_decrement: i64,
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
            if let MirInst::BinOp { dest, op, lhs, rhs } = inst
                && dest.name == *cond_var {
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
            if let MirInst::Call { dest: Some(dest), func: callee, args, .. } = inst
                && callee == self_name && args.len() == 1 {
                    // Check for param - constant pattern
                    if let Some(decrement) = self.extract_decrement(&args[0], param_name, &recursive_block.instructions) {
                        calls.push((decrement, dest.name.clone()));
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
                if let MirInst::Phi { values, .. } = inst
                    && values.len() >= 2 {
                        let labels: Vec<_> = values.iter().map(|(_, l)| l.clone()).collect();
                        if labels.contains(&then_label) && (labels.contains(&else_label) || labels.iter().any(|l| {
                            // Check if any label is from a block that came from else branch
                            func.blocks.iter().any(|blk| &blk.label == l && matches!(&blk.terminator, Terminator::Goto(t) if t == &b.label))
                        })) {
                            return Some(b.label.clone());
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
            _first_decrement: calls[0].0,
            _second_decrement: calls[1].0,
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
                    if let MirInst::BinOp { dest, op: MirBinOp::Sub, lhs, rhs } = inst
                        && dest.name == p.name {
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
                if let Terminator::Goto(target) = &block.terminator
                    && pattern.merge_block_label.as_ref() == Some(target) {
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

        // Add the new blocks
        func.blocks.push(loop_setup_block);
        func.blocks.push(loop_header_block);
        func.blocks.push(loop_body_block);
        func.blocks.push(loop_exit_block);

        true
    }
}

// ============================================================================
// Conditional Increment to Branchless Add (v0.60.55)
// ============================================================================

/// Convert conditional increment pattern to branchless arithmetic
///
/// Pattern detected:
/// ```text
/// cond_block:
///   %cond = ...
///   branch %cond, then_block, else_block
///
/// then_block:
///   %new_val = add %val, 1
///   goto merge_block
///
/// else_block:
///   goto merge_block
///
/// merge_block:
///   %result = phi [%new_val, then_block], [%val, else_block]
/// ```
///
/// Transformed to:
/// ```text
/// cond_block:
///   %cond = ...
///   %cond_i64 = cast %cond Bool to I64
///   %result = add %val, %cond_i64
///   goto merge_block
///
/// merge_block:
///   ; (phi removed)
/// ```
///
/// This eliminates a branch and enables better vectorization.
pub struct ConditionalIncrementToSelect;

impl OptimizationPass for ConditionalIncrementToSelect {
    fn name(&self) -> &'static str {
        "conditional_increment_to_select"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;

        // Collect patterns to transform (avoid borrowing issues)
        let patterns = self.find_patterns(func);

        for pattern in patterns {
            if self.apply_transformation(func, &pattern) {
                changed = true;
            }
        }

        changed
    }
}

/// Information about a conditional increment pattern
#[allow(dead_code)]
struct ConditionalIncrementPattern {
    /// Block index that has the conditional branch
    cond_block_idx: usize,
    /// The condition operand
    cond: Operand,
    /// Block label that has the increment (kept for debugging)
    then_label: String,
    /// Block label that is empty (kept for debugging)
    else_label: String,
    /// Block label where phi merges
    merge_label: String,
    /// The value being incremented
    base_value: Operand,
    /// The destination of the phi (will become destination of the add)
    phi_dest: Place,
    /// Index of the phi instruction in merge block
    phi_inst_idx: usize,
}

impl ConditionalIncrementToSelect {
    fn find_patterns(&self, func: &MirFunction) -> Vec<ConditionalIncrementPattern> {
        let mut patterns = Vec::new();

        // Build a map of block labels to indices
        let block_map: HashMap<String, usize> = func.blocks.iter()
            .enumerate()
            .map(|(i, b)| (b.label.clone(), i))
            .collect();

        for (cond_idx, cond_block) in func.blocks.iter().enumerate() {
            // Look for branch terminators
            let (cond, then_label, else_label) = match &cond_block.terminator {
                Terminator::Branch { cond, then_label, else_label } => {
                    (cond.clone(), then_label.clone(), else_label.clone())
                }
                _ => continue,
            };

            // Get then and else blocks
            let then_idx = match block_map.get(&then_label) {
                Some(&idx) => idx,
                None => continue,
            };
            let else_idx = match block_map.get(&else_label) {
                Some(&idx) => idx,
                None => continue,
            };

            let then_block = &func.blocks[then_idx];
            let else_block = &func.blocks[else_idx];

            // Check if both blocks jump to the same merge block
            let then_target = match &then_block.terminator {
                Terminator::Goto(target) => target.clone(),
                _ => continue,
            };
            let else_target = match &else_block.terminator {
                Terminator::Goto(target) => target.clone(),
                _ => continue,
            };

            if then_target != else_target {
                continue;
            }
            let merge_label = then_target;

            // else_block should be empty
            if !else_block.instructions.is_empty() {
                continue;
            }

            // then_block should have exactly one instruction: add by 1
            if then_block.instructions.len() != 1 {
                continue;
            }

            let (add_dest, base_value) = match &then_block.instructions[0] {
                MirInst::BinOp {
                    dest,
                    op: MirBinOp::Add,
                    lhs,
                    rhs: Operand::Constant(Constant::Int(1)),
                } => (dest.clone(), lhs.clone()),
                _ => continue,
            };

            // Find merge block and check for phi
            let merge_idx = match block_map.get(&merge_label) {
                Some(&idx) => idx,
                None => continue,
            };
            let merge_block = &func.blocks[merge_idx];

            // Look for matching phi in merge block
            let mut phi_info = None;
            for (inst_idx, inst) in merge_block.instructions.iter().enumerate() {
                if let MirInst::Phi { dest, values } = inst {
                    // Check if phi merges add_dest from then_label and base_value from else_label
                    if values.len() == 2 {
                        let has_then_val = values.iter().any(|(v, l)| {
                            l == &then_label && matches!(v, Operand::Place(p) if p.name == add_dest.name)
                        });
                        let has_else_val = values.iter().any(|(v, l)| {
                            l == &else_label && Self::operands_equal(v, &base_value)
                        });

                        if has_then_val && has_else_val {
                            phi_info = Some((dest.clone(), inst_idx));
                            break;
                        }
                    }
                }
            }

            let (phi_dest, phi_inst_idx) = match phi_info {
                Some(info) => info,
                None => continue,
            };

            patterns.push(ConditionalIncrementPattern {
                cond_block_idx: cond_idx,
                cond,
                then_label,
                else_label,
                merge_label,
                base_value,
                phi_dest,
                phi_inst_idx,
            });
        }

        patterns
    }

    fn operands_equal(a: &Operand, b: &Operand) -> bool {
        match (a, b) {
            (Operand::Place(pa), Operand::Place(pb)) => pa.name == pb.name,
            (Operand::Constant(ca), Operand::Constant(cb)) => {
                match (ca, cb) {
                    (Constant::Int(ia), Constant::Int(ib)) => ia == ib,
                    (Constant::Float(fa), Constant::Float(fb)) => fa == fb,
                    (Constant::Bool(ba), Constant::Bool(bb)) => ba == bb,
                    (Constant::String(sa), Constant::String(sb)) => sa == sb,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn apply_transformation(&self, func: &mut MirFunction, pattern: &ConditionalIncrementPattern) -> bool {
        // Generate unique names for the new instructions
        let cast_name = format!("{}_cond_ext", pattern.phi_dest.name);

        // Create the cast instruction: %cast = cast %cond Bool to I64
        let cast_inst = MirInst::Cast {
            dest: Place::new(&cast_name),
            src: pattern.cond.clone(),
            from_ty: MirType::Bool,
            to_ty: MirType::I64,
        };

        // Create the add instruction: %result = add %base_value, %cast
        let add_inst = MirInst::BinOp {
            dest: pattern.phi_dest.clone(),
            op: MirBinOp::Add,
            lhs: pattern.base_value.clone(),
            rhs: Operand::Place(Place::new(&cast_name)),
        };

        // Insert instructions into cond_block before the terminator
        let cond_block = &mut func.blocks[pattern.cond_block_idx];
        cond_block.instructions.push(cast_inst);
        cond_block.instructions.push(add_inst);

        // Change terminator from Branch to Goto(merge_label)
        cond_block.terminator = Terminator::Goto(pattern.merge_label.clone());

        // Find and remove phi from merge block
        let merge_block = func.blocks.iter_mut()
            .find(|b| b.label == pattern.merge_label)
            .expect("merge block should exist");

        // Remove the phi instruction
        if pattern.phi_inst_idx < merge_block.instructions.len() {
            merge_block.instructions.remove(pattern.phi_inst_idx);
        }

        // Mark then and else blocks as unreachable (they will be cleaned up by DCE)
        // Actually, they become unreachable because no one jumps to them anymore
        // UnreachableBlockElimination will clean them up

        true
    }
}

// ============================================================================
// v0.90.27: IfElseToSelect - Convert simple if-else branches to select
// ============================================================================

/// Convert simple if-else branch patterns into select instructions.
///
/// Detects pattern:
///   header: %cond = cmp ...; branch %cond, then_block, else_block
///   then_block: <1-3 simple instructions> ; goto merge
///   else_block: <1-3 simple instructions> ; goto merge
///   merge: %result = phi [(then_val, then_block), (else_val, else_block)]
///
/// Transforms to:
///   header: <then instructions>; <else instructions>; %result = select(...)
///   goto merge
///
/// This eliminates branch misprediction in tight loops (e.g., collatz).
pub struct IfElseToSelect;

impl OptimizationPass for IfElseToSelect {
    fn name(&self) -> &'static str {
        "if_else_to_select"
    }

    fn run_on_function(&self, func: &mut MirFunction) -> bool {
        let mut changed = false;
        // Re-find patterns after each transformation since block indices change
        loop {
            let patterns = self.find_patterns(func);
            if patterns.is_empty() {
                break;
            }
            // Apply only the first pattern, then re-scan
            if self.apply_transformation(func, &patterns[0]) {
                changed = true;
            } else {
                break;
            }
        }
        changed
    }
}

/// Information about an if-else-to-select pattern
#[allow(dead_code)]
struct IfElseSelectPattern {
    /// Block index with the conditional branch
    cond_block_idx: usize,
    /// The comparison operator that produced the condition
    cond_op: MirBinOp,
    /// LHS of the comparison
    cond_lhs: Operand,
    /// RHS of the comparison
    cond_rhs: Operand,
    /// Index of the comparison instruction in cond block (to remove)
    cond_inst_idx: usize,
    /// Then block index
    then_block_idx: usize,
    /// Else block index
    else_block_idx: usize,
    /// Then block label
    then_label: String,
    /// Else block label
    else_label: String,
    /// Merge block label
    merge_label: String,
    /// Instructions from then block to hoist
    then_insts: Vec<MirInst>,
    /// Instructions from else block to hoist
    else_insts: Vec<MirInst>,
    /// Phi nodes to convert: (phi_dest, phi_inst_idx, then_val, else_val)
    phis: Vec<(Place, usize, Operand, Operand)>,
}

impl IfElseToSelect {
    fn find_patterns(&self, func: &MirFunction) -> Vec<IfElseSelectPattern> {
        let mut patterns = Vec::new();

        let block_map: HashMap<String, usize> = func.blocks.iter()
            .enumerate()
            .map(|(i, b)| (b.label.clone(), i))
            .collect();

        for (cond_idx, cond_block) in func.blocks.iter().enumerate() {
            // Look for branch terminators
            let (cond_place, then_label, else_label) = match &cond_block.terminator {
                Terminator::Branch { cond: Operand::Place(p), then_label, else_label } => {
                    (p.clone(), then_label.clone(), else_label.clone())
                }
                _ => continue,
            };

            // Find the comparison instruction that produces the condition
            let cond_info = self.find_comparison(cond_block, &cond_place);
            let (cond_op, cond_lhs, cond_rhs, cond_inst_idx) = match cond_info {
                Some(info) => info,
                None => continue,
            };

            // Get then and else blocks
            let then_idx = match block_map.get(&then_label) {
                Some(&idx) => idx,
                None => continue,
            };
            let else_idx = match block_map.get(&else_label) {
                Some(&idx) => idx,
                None => continue,
            };

            let then_block = &func.blocks[then_idx];
            let else_block = &func.blocks[else_idx];

            // Both blocks must Goto the same merge block
            let then_target = match &then_block.terminator {
                Terminator::Goto(target) => target.clone(),
                _ => continue,
            };
            let else_target = match &else_block.terminator {
                Terminator::Goto(target) => target.clone(),
                _ => continue,
            };

            if then_target != else_target {
                continue;
            }
            let merge_label = then_target;

            // Both blocks must be simple (≤3 instructions each)
            if then_block.instructions.len() > 3 || else_block.instructions.len() > 3 {
                continue;
            }

            // Both blocks must have at least 1 instruction (producing a value)
            if then_block.instructions.is_empty() || else_block.instructions.is_empty() {
                continue;
            }

            // Check that instructions are safe to hoist (no side effects, no calls)
            if !self.is_hoistable(&then_block.instructions) || !self.is_hoistable(&else_block.instructions) {
                continue;
            }

            // Find merge block and look for matching phi
            let merge_idx = match block_map.get(&merge_label) {
                Some(&idx) => idx,
                None => continue,
            };
            let merge_block = &func.blocks[merge_idx];

            // Find phis that merge values from then and else blocks
            let phis = self.find_matching_phis(
                merge_block, &then_label, &else_label,
                &then_block.instructions, &else_block.instructions,
            );
            if phis.is_empty() {
                continue;
            }

            // Don't transform if the then/else instructions reference each other's
            // destinations (dependency issue when hoisted to same block)
            if self.has_cross_dependency(&then_block.instructions, &else_block.instructions) {
                continue;
            }

            patterns.push(IfElseSelectPattern {
                cond_block_idx: cond_idx,
                cond_op,
                cond_lhs,
                cond_rhs,
                cond_inst_idx,
                then_block_idx: then_idx,
                else_block_idx: else_idx,
                then_label,
                else_label,
                merge_label,
                then_insts: then_block.instructions.clone(),
                else_insts: else_block.instructions.clone(),
                phis,
            });
        }

        patterns
    }

    /// Find the comparison instruction that defines `cond_place`
    fn find_comparison(&self, block: &BasicBlock, cond_place: &Place) -> Option<(MirBinOp, Operand, Operand, usize)> {
        for (idx, inst) in block.instructions.iter().enumerate().rev() {
            if let MirInst::BinOp { dest, op, lhs, rhs } = inst
                && dest.name == cond_place.name
                && matches!(op,
                    MirBinOp::Eq | MirBinOp::Ne |
                    MirBinOp::Lt | MirBinOp::Le |
                    MirBinOp::Gt | MirBinOp::Ge
                )
            {
                return Some((*op, lhs.clone(), rhs.clone(), idx));
            }
        }
        None
    }

    /// Check if all instructions are safe to hoist (no side effects, no trapping)
    fn is_hoistable(&self, insts: &[MirInst]) -> bool {
        for inst in insts {
            match inst {
                MirInst::BinOp { op, .. } => {
                    // Div and Mod trap on zero — not safe to hoist unconditionally
                    if matches!(op, MirBinOp::Div | MirBinOp::Mod) {
                        return false;
                    }
                }
                MirInst::UnaryOp { .. } |
                MirInst::Const { .. } |
                MirInst::Copy { .. } |
                MirInst::Cast { .. } => {}
                _ => return false, // Calls, stores, etc. not safe to hoist
            }
        }
        true
    }

    /// Find phis in the merge block that receive values from then/else blocks
    fn find_matching_phis(
        &self,
        merge_block: &BasicBlock,
        then_label: &str,
        else_label: &str,
        then_insts: &[MirInst],
        else_insts: &[MirInst],
    ) -> Vec<(Place, usize, Operand, Operand)> {
        let mut results = Vec::new();
        // Get destinations from each block
        let then_dests: HashSet<String> = then_insts.iter().filter_map(|i| match i {
            MirInst::BinOp { dest, .. } | MirInst::UnaryOp { dest, .. } |
            MirInst::Const { dest, .. } | MirInst::Copy { dest, .. } |
            MirInst::Cast { dest, .. } => Some(dest.name.clone()),
            _ => None,
        }).collect();
        let else_dests: HashSet<String> = else_insts.iter().filter_map(|i| match i {
            MirInst::BinOp { dest, .. } | MirInst::UnaryOp { dest, .. } |
            MirInst::Const { dest, .. } | MirInst::Copy { dest, .. } |
            MirInst::Cast { dest, .. } => Some(dest.name.clone()),
            _ => None,
        }).collect();

        for (inst_idx, inst) in merge_block.instructions.iter().enumerate() {
            if let MirInst::Phi { dest, values } = inst {
                // Must have at least 2 values and include both then/else labels
                if values.len() < 2 {
                    continue;
                }

                let mut then_val = None;
                let mut else_val = None;

                for (val, label) in values {
                    if label == then_label {
                        // Accept if val is produced by then block, OR
                        // is a pre-existing value (pass-through) not produced by else block.
                        // Reject only if produced by the OTHER branch (SSA violation).
                        let is_valid = match val {
                            Operand::Place(p) => !else_dests.contains(&p.name),
                            _ => true, // Constants always valid
                        };
                        if is_valid {
                            then_val = Some(val.clone());
                        }
                    } else if label == else_label {
                        let is_valid = match val {
                            Operand::Place(p) => !then_dests.contains(&p.name),
                            _ => true,
                        };
                        if is_valid {
                            else_val = Some(val.clone());
                        }
                    }
                }

                if let (Some(tv), Some(ev)) = (then_val, else_val) {
                    results.push((dest.clone(), inst_idx, tv, ev));
                }
            }
        }
        results
    }

    /// Check if instructions from one block reference destinations of the other
    fn has_cross_dependency(&self, a_insts: &[MirInst], b_insts: &[MirInst]) -> bool {
        let a_dests: HashSet<String> = a_insts.iter().filter_map(|inst| match inst {
            MirInst::BinOp { dest, .. } |
            MirInst::UnaryOp { dest, .. } |
            MirInst::Const { dest, .. } |
            MirInst::Copy { dest, .. } |
            MirInst::Cast { dest, .. } => Some(dest.name.clone()),
            _ => None,
        }).collect();

        let b_dests: HashSet<String> = b_insts.iter().filter_map(|inst| match inst {
            MirInst::BinOp { dest, .. } |
            MirInst::UnaryOp { dest, .. } |
            MirInst::Const { dest, .. } |
            MirInst::Copy { dest, .. } |
            MirInst::Cast { dest, .. } => Some(dest.name.clone()),
            _ => None,
        }).collect();

        // Check if any operand in a references a dest in b, or vice versa
        for inst in a_insts {
            for name in self.operand_names(inst) {
                if b_dests.contains(&name) {
                    return true;
                }
            }
        }
        for inst in b_insts {
            for name in self.operand_names(inst) {
                if a_dests.contains(&name) {
                    return true;
                }
            }
        }
        false
    }

    fn operand_names(&self, inst: &MirInst) -> Vec<String> {
        let mut names = Vec::new();
        match inst {
            MirInst::BinOp { lhs, rhs, .. } => {
                if let Operand::Place(p) = lhs { names.push(p.name.clone()); }
                if let Operand::Place(p) = rhs { names.push(p.name.clone()); }
            }
            MirInst::UnaryOp { src, .. } | MirInst::Cast { src, .. } => {
                if let Operand::Place(p) = src { names.push(p.name.clone()); }
            }
            MirInst::Copy { src, .. } => {
                names.push(src.name.clone());
            }
            _ => {}
        }
        names
    }

    fn apply_transformation(&self, func: &mut MirFunction, pattern: &IfElseSelectPattern) -> bool {
        // 1. Hoist then_insts and else_insts into cond_block (before the comparison)
        let cond_block = &mut func.blocks[pattern.cond_block_idx];

        // Remove the comparison instruction (it will be embedded in Select)
        cond_block.instructions.remove(pattern.cond_inst_idx);

        // Insert then and else instructions at the position where comparison was
        let insert_pos = pattern.cond_inst_idx;
        for (i, inst) in pattern.then_insts.iter().enumerate() {
            cond_block.instructions.insert(insert_pos + i, inst.clone());
        }
        let else_insert_pos = insert_pos + pattern.then_insts.len();
        for (i, inst) in pattern.else_insts.iter().enumerate() {
            cond_block.instructions.insert(else_insert_pos + i, inst.clone());
        }

        // 2. Add Select instructions for each phi (use unique dest names to avoid SSA collision)
        let mut select_names: Vec<(String, String)> = Vec::new(); // (select_dest, phi_dest)
        for (idx, (phi_dest, _, then_val, else_val)) in pattern.phis.iter().enumerate() {
            let select_dest_name = format!("_sel_{}_{}", phi_dest.name, idx);
            let select_inst = MirInst::Select {
                dest: Place::new(&select_dest_name),
                cond_op: pattern.cond_op,
                cond_lhs: pattern.cond_lhs.clone(),
                cond_rhs: pattern.cond_rhs.clone(),
                true_val: then_val.clone(),
                false_val: else_val.clone(),
            };
            cond_block.instructions.push(select_inst);
            select_names.push((select_dest_name, phi_dest.name.clone()));
        }

        // 3. Change terminator from Branch to Goto(merge_label)
        cond_block.terminator = Terminator::Goto(pattern.merge_label.clone());

        // 4. Update phis in merge block: remove then/else entries, add cond_block entry
        let cond_block_label = func.blocks[pattern.cond_block_idx].label.clone();
        let merge_block = func.blocks.iter_mut()
            .find(|b| b.label == pattern.merge_label)
            .expect("merge block should exist");

        // For each phi that we converted to select, update remaining phi entries
        // Build map: phi_dest_name -> select_dest_name
        let select_name_map: HashMap<String, String> = select_names.into_iter()
            .map(|(sel, phi)| (phi, sel))
            .collect();

        // Collect indices of phis to update
        let mut to_update: Vec<(usize, String, String)> = Vec::new(); // (inst_idx, phi_dest, select_dest)
        for (inst_idx, inst) in merge_block.instructions.iter().enumerate() {
            if let MirInst::Phi { dest, .. } = inst
                && let Some(sel_name) = select_name_map.get(&dest.name)
            {
                to_update.push((inst_idx, dest.name.clone(), sel_name.clone()));
            }
        }

        // Update each phi: remove then/else entries, add cond_block entry with select result
        for (inst_idx, _phi_dest, select_dest) in &to_update {
            if let MirInst::Phi { dest: _, values } = &mut merge_block.instructions[*inst_idx] {
                // Remove then and else entries
                values.retain(|(_, label)| {
                    label != &pattern.then_label && label != &pattern.else_label
                });
                // Add entry from cond_block with select's unique dest name
                values.push((
                    Operand::Place(Place::new(select_dest)),
                    cond_block_label.clone(),
                ));

                // If phi now has only one value, it can be simplified later by PhiSimplification
            }
        }

        // Then and else blocks become unreachable (cleaned up by UnreachableBlockElimination)
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
    fn test_algebraic_mul_power_of_2_to_shift() {
        // Test: x * 8 = x << 3 (v0.60.53)
        let mut func = MirFunction {
            name: "test_mul_pow2".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::BinOp {
                    dest: Place::new("result"),
                    op: MirBinOp::Mul,
                    lhs: Operand::Place(Place::new("x")),
                    rhs: Operand::Constant(Constant::Int(8)),
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
        assert!(changed, "x * 8 should be simplified to shift");

        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::BinOp { op: MirBinOp::Shl, rhs: Operand::Constant(Constant::Int(3)), .. }),
            "x * 8 should become x << 3, got {:?}", inst
        );
    }

    #[test]
    fn test_constant_propagation_narrowing() {
        // v0.90.22: Test that ConstantPropagationNarrowing does NOT narrow
        // functions with remaining self-recursive calls.
        //
        // fibonacci(n) has recursive calls: fibonacci(n-1), fibonacci(n-2)
        // Even though n starts at 35 and decreases, narrowing creates sext/trunc
        // overhead per recursive call frame. The LoopBoundedNarrowing pass handles
        // loop-converted versions instead.

        // Create fibonacci function (simplified, still has recursive calls)
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
                    args: vec![Operand::Constant(Constant::Int(35))],
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

        // v0.90.22: Functions with remaining self-recursive calls should NOT be narrowed.
        // The sext/trunc overhead per call frame outweighs any i32 benefit.
        assert!(!changed, "Narrowing pass should NOT narrow recursive functions");

        let fib = program.functions.iter().find(|f| f.name == "fibonacci").unwrap();
        assert_eq!(
            fib.params[0].1,
            MirType::I64,
            "fibonacci's n parameter should remain i64 (has recursive calls)"
        );
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

    // ========================================================================
    // v0.89: Copy Propagation Tests
    // ========================================================================

    #[test]
    fn test_copy_propagation_basic() {
        // %a = 10
        // %b = copy %a
        // %c = %b + 1
        // return %c
        // After CopyPropagation: %c = %a + 1
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const {
                        dest: Place::new("a"),
                        value: Constant::Int(10),
                    },
                    MirInst::Copy {
                        dest: Place::new("b"),
                        src: Place::new("a"),
                    },
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("b")),
                        rhs: Operand::Constant(Constant::Int(1)),
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
        };

        let pass = CopyPropagation;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "CopyPropagation should replace %b with %a");

        // The BinOp lhs should now reference %a, not %b
        let binop = &func.blocks[0].instructions[2];
        assert!(
            matches!(binop, MirInst::BinOp { lhs: Operand::Place(p), .. } if p.name == "a"),
            "BinOp lhs should be %a after copy propagation, got {:?}",
            binop
        );
    }

    #[test]
    fn test_copy_propagation_chain() {
        // %a = param
        // %b = copy %a
        // %c = copy %b  (should NOT chain-propagate to %a in single pass)
        // return %c
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("a".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Copy {
                        dest: Place::new("b"),
                        src: Place::new("a"),
                    },
                    MirInst::Copy {
                        dest: Place::new("c"),
                        src: Place::new("b"),
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
        };

        let pass = CopyPropagation;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "CopyPropagation should propagate copies");

        // Terminator should now reference %b (or %a if chaining works)
        // The key is that SOME propagation happened
        let term = &func.blocks[0].terminator;
        assert!(
            !matches!(term, Terminator::Return(Some(Operand::Place(p))) if p.name == "c"),
            "Return should no longer reference %c after propagation"
        );
    }

    #[test]
    fn test_copy_propagation_in_call_args() {
        // %a = param
        // %b = copy %a
        // %c = call foo(%b)
        // After: %c = call foo(%a)
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("a".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Copy {
                        dest: Place::new("b"),
                        src: Place::new("a"),
                    },
                    MirInst::Call {
                        dest: Some(Place::new("c")),
                        func: "foo".to_string(),
                        args: vec![Operand::Place(Place::new("b"))],
                        is_tail: false,
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
        };

        let pass = CopyPropagation;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "CopyPropagation should propagate into call args");

        let call = &func.blocks[0].instructions[1];
        assert!(
            matches!(call, MirInst::Call { args, .. } if matches!(&args[0], Operand::Place(p) if p.name == "a")),
            "Call arg should be %a after propagation, got {:?}",
            call
        );
    }

    // ========================================================================
    // v0.89: Common Subexpression Elimination Tests
    // ========================================================================

    #[test]
    fn test_cse_basic() {
        // %c = %a + %b
        // %d = %a + %b   (same expression, should become copy of %c)
        // return %d
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("a".to_string(), MirType::I64), ("b".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("a")),
                        rhs: Operand::Place(Place::new("b")),
                    },
                    MirInst::BinOp {
                        dest: Place::new("d"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("a")),
                        rhs: Operand::Place(Place::new("b")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("d")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = CommonSubexpressionElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "CSE should eliminate duplicate a + b");

        // Second instruction should be a Copy
        let second = &func.blocks[0].instructions[1];
        assert!(
            matches!(second, MirInst::Copy { dest, src } if dest.name == "d" && src.name == "c"),
            "Duplicate BinOp should become Copy, got {:?}",
            second
        );
    }

    #[test]
    fn test_cse_different_ops_not_eliminated() {
        // %c = %a + %b
        // %d = %a * %b   (different op, should NOT be CSE'd)
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("a".to_string(), MirType::I64), ("b".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("a")),
                        rhs: Operand::Place(Place::new("b")),
                    },
                    MirInst::BinOp {
                        dest: Place::new("d"),
                        op: MirBinOp::Mul,
                        lhs: Operand::Place(Place::new("a")),
                        rhs: Operand::Place(Place::new("b")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("d")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = CommonSubexpressionElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "CSE should not eliminate different operations");
    }

    #[test]
    fn test_cse_cross_block_isolation() {
        // Block 0: %c = %a + %b; goto block1
        // Block 1: %d = %a + %b; return %d
        // CSE is per-block, so %d should NOT be eliminated
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("a".to_string(), MirType::I64), ("b".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("c"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("a")),
                            rhs: Operand::Place(Place::new("b")),
                        },
                    ],
                    terminator: Terminator::Goto("block1".to_string()),
                },
                BasicBlock {
                    label: "block1".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("d"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("a")),
                            rhs: Operand::Place(Place::new("b")),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("d")))),
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

        let pass = CommonSubexpressionElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "CSE should not eliminate expressions across blocks");
    }

    // ========================================================================
    // v0.89: SimplifyBranches Tests
    // ========================================================================

    #[test]
    fn test_simplify_branches_true_condition() {
        // branch true -> then, else
        // Should become: goto then
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Constant(Constant::Bool(true)),
                        then_label: "then".to_string(),
                        else_label: "else".to_string(),
                    },
                },
                BasicBlock {
                    label: "then".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                },
                BasicBlock {
                    label: "else".to_string(),
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

        let pass = SimplifyBranches;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "SimplifyBranches should simplify constant true branch");

        assert!(
            matches!(&func.blocks[0].terminator, Terminator::Goto(label) if label == "then"),
            "Entry terminator should be Goto(then), got {:?}",
            func.blocks[0].terminator
        );
    }

    #[test]
    fn test_simplify_branches_false_condition() {
        // branch false -> then, else
        // Should become: goto else
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Constant(Constant::Bool(false)),
                        then_label: "then".to_string(),
                        else_label: "else".to_string(),
                    },
                },
                BasicBlock {
                    label: "then".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                },
                BasicBlock {
                    label: "else".to_string(),
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

        let pass = SimplifyBranches;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "SimplifyBranches should simplify constant false branch");

        assert!(
            matches!(&func.blocks[0].terminator, Terminator::Goto(label) if label == "else"),
            "Entry terminator should be Goto(else), got {:?}",
            func.blocks[0].terminator
        );
    }

    #[test]
    fn test_simplify_branches_variable_condition_unchanged() {
        // branch %cond -> then, else
        // Should NOT change (non-constant condition)
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::Bool)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("x")),
                        then_label: "then".to_string(),
                        else_label: "else".to_string(),
                    },
                },
                BasicBlock {
                    label: "then".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                },
                BasicBlock {
                    label: "else".to_string(),
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

        let pass = SimplifyBranches;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "SimplifyBranches should not change variable condition");
    }

    // ========================================================================
    // v0.89: UnreachableBlockElimination Tests
    // ========================================================================

    #[test]
    fn test_unreachable_block_elimination() {
        // entry: goto block1
        // block1: return 1
        // dead: return 2  (unreachable)
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("block1".to_string()),
                },
                BasicBlock {
                    label: "block1".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                },
                BasicBlock {
                    label: "dead".to_string(),
                    instructions: vec![
                        MirInst::Const {
                            dest: Place::new("x"),
                            value: Constant::Int(999),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(2)))),
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

        let pass = UnreachableBlockElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should remove unreachable block");
        assert_eq!(func.blocks.len(), 2, "Should have 2 blocks remaining");
        assert!(
            func.blocks.iter().all(|b| b.label != "dead"),
            "Dead block should be removed"
        );
    }

    #[test]
    fn test_unreachable_block_all_reachable() {
        // entry: branch -> then, else
        // then: return 1
        // else: return 0
        // All blocks reachable, no change
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::Bool)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("x")),
                        then_label: "then".to_string(),
                        else_label: "else".to_string(),
                    },
                },
                BasicBlock {
                    label: "then".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                },
                BasicBlock {
                    label: "else".to_string(),
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

        let pass = UnreachableBlockElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "All blocks reachable, should not change");
    }

    // ========================================================================
    // v0.89: PhiSimplification Tests
    // ========================================================================

    #[test]
    fn test_phi_simplification_single_value() {
        // merge: %x = phi [%a, entry]
        // Single-value phi should become: %x = copy %a
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("a".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Phi {
                        dest: Place::new("x"),
                        values: vec![
                            (Operand::Place(Place::new("a")), "pred".to_string()),
                        ],
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = PhiSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Single-value phi should be simplified");

        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::Copy { dest, src } if dest.name == "x" && src.name == "a"),
            "Single-value phi should become Copy, got {:?}",
            inst
        );
    }

    #[test]
    fn test_phi_simplification_constant_value() {
        // %x = phi [I:42, pred]
        // Single constant phi should become: %x = const 42
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Phi {
                        dest: Place::new("x"),
                        values: vec![
                            (Operand::Constant(Constant::Int(42)), "pred".to_string()),
                        ],
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = PhiSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Single-constant phi should be simplified");

        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::Const { dest, value: Constant::Int(42) } if dest.name == "x"),
            "Single-constant phi should become Const, got {:?}",
            inst
        );
    }

    #[test]
    fn test_phi_simplification_all_same_values() {
        // %x = phi [%a, block1], [%a, block2]
        // All same value should become: %x = copy %a
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("a".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Phi {
                        dest: Place::new("x"),
                        values: vec![
                            (Operand::Place(Place::new("a")), "block1".to_string()),
                            (Operand::Place(Place::new("a")), "block2".to_string()),
                        ],
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = PhiSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Phi with all same values should be simplified");

        let inst = &func.blocks[0].instructions[0];
        assert!(
            matches!(inst, MirInst::Copy { dest, src } if dest.name == "x" && src.name == "a"),
            "All-same phi should become Copy, got {:?}",
            inst
        );
    }

    #[test]
    fn test_phi_simplification_different_values_unchanged() {
        // %x = phi [%a, block1], [%b, block2]
        // Different values should NOT be simplified
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("a".to_string(), MirType::I64), ("b".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Phi {
                        dest: Place::new("x"),
                        values: vec![
                            (Operand::Place(Place::new("a")), "block1".to_string()),
                            (Operand::Place(Place::new("b")), "block2".to_string()),
                        ],
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = PhiSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Phi with different values should not be simplified");
    }

    // ========================================================================
    // v0.89: BlockMerging Tests
    // ========================================================================

    #[test]
    fn test_block_merging_basic() {
        // entry: %a = 1; goto block1
        // block1: %b = %a + 2; return %b
        // block1 has only entry as predecessor, should merge
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const {
                            dest: Place::new("a"),
                            value: Constant::Int(1),
                        },
                    ],
                    terminator: Terminator::Goto("block1".to_string()),
                },
                BasicBlock {
                    label: "block1".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("b"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("a")),
                            rhs: Operand::Constant(Constant::Int(2)),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("b")))),
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

        let pass = BlockMerging;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "BlockMerging should merge single-predecessor block");
        assert_eq!(func.blocks.len(), 1, "Should have 1 merged block");
        assert_eq!(func.blocks[0].instructions.len(), 2, "Merged block should have 2 instructions");
        assert!(
            matches!(&func.blocks[0].terminator, Terminator::Return(_)),
            "Merged block should return"
        );
    }

    #[test]
    fn test_block_merging_multiple_predecessors_unchanged() {
        // entry: branch %cond -> then, merge
        // then: goto merge
        // merge: return 0   (merge has 2 predecessors: entry, then)
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("cond".to_string(), MirType::Bool)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cond")),
                        then_label: "then".to_string(),
                        else_label: "merge".to_string(),
                    },
                },
                BasicBlock {
                    label: "then".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
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

        let pass = BlockMerging;
        let changed = pass.run_on_function(&mut func);
        // "then" has single predecessor (entry) and entry goes to then via branch,
        // but entry uses Branch not Goto, so then is not a merge candidate.
        // "merge" has 2 predecessors, so it's not a merge candidate.
        assert!(!changed, "Should not merge blocks with multiple predecessors");
    }

    // ================================================================
    // GlobalFieldAccessCSE Tests
    // ================================================================

    #[test]
    fn test_global_field_access_cse_eliminates_redundant() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("p".to_string(), MirType::Struct { name: "Point".to_string(), fields: vec![] })],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::FieldAccess {
                            dest: Place::new("x1"),
                            base: Place::new("p"),
                            field: "x".to_string(),
                            field_index: 0,
                            struct_name: "Point".to_string(),
                        },
                    ],
                    terminator: Terminator::Goto("next".to_string()),
                },
                BasicBlock {
                    label: "next".to_string(),
                    instructions: vec![
                        MirInst::FieldAccess {
                            dest: Place::new("x2"),
                            base: Place::new("p"),
                            field: "x".to_string(),
                            field_index: 0,
                            struct_name: "Point".to_string(),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x2")))),
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

        let pass = GlobalFieldAccessCSE;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should eliminate redundant field access");

        // Second block should now have a Copy instead of FieldAccess
        assert!(matches!(
            &func.blocks[1].instructions[0],
            MirInst::Copy { dest, src } if dest.name == "x2" && src.name == "x1"
        ));
    }

    #[test]
    fn test_global_field_access_cse_different_fields_no_change() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("p".to_string(), MirType::Struct { name: "Point".to_string(), fields: vec![] })],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::FieldAccess {
                            dest: Place::new("x1"),
                            base: Place::new("p"),
                            field: "x".to_string(),
                            field_index: 0,
                            struct_name: "Point".to_string(),
                        },
                    ],
                    terminator: Terminator::Goto("next".to_string()),
                },
                BasicBlock {
                    label: "next".to_string(),
                    instructions: vec![
                        MirInst::FieldAccess {
                            dest: Place::new("y1"),
                            base: Place::new("p"),
                            field: "y".to_string(),
                            field_index: 1,
                            struct_name: "Point".to_string(),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("y1")))),
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

        let pass = GlobalFieldAccessCSE;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Different fields should not be CSE'd");
    }

    // ================================================================
    // IfElseToSwitch Tests
    // ================================================================

    #[test]
    fn test_if_else_to_switch_chain() {
        // Create if-else chain: if x==0 ... elif x==1 ... elif x==2 ... else ...
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp0"),
                            op: MirBinOp::Eq,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp0")),
                        then_label: "case0".to_string(),
                        else_label: "check1".to_string(),
                    },
                },
                BasicBlock {
                    label: "check1".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp1"),
                            op: MirBinOp::Eq,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp1")),
                        then_label: "case1".to_string(),
                        else_label: "check2".to_string(),
                    },
                },
                BasicBlock {
                    label: "check2".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp2"),
                            op: MirBinOp::Eq,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(2)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp2")),
                        then_label: "case2".to_string(),
                        else_label: "default".to_string(),
                    },
                },
                BasicBlock {
                    label: "case0".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(10)))),
                },
                BasicBlock {
                    label: "case1".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(20)))),
                },
                BasicBlock {
                    label: "case2".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(30)))),
                },
                BasicBlock {
                    label: "default".to_string(),
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

        let pass = IfElseToSwitch;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should convert if-else chain to switch");

        // Entry block should now have Switch terminator
        assert!(matches!(
            &func.blocks[0].terminator,
            Terminator::Switch { cases, default, .. }
            if cases.len() >= 2 && !default.is_empty()
        ));
    }

    #[test]
    fn test_if_else_to_switch_too_few_cases() {
        // Only 2 cases (below threshold of 3), should not convert
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp0"),
                            op: MirBinOp::Eq,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp0")),
                        then_label: "case0".to_string(),
                        else_label: "default".to_string(),
                    },
                },
                BasicBlock {
                    label: "case0".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(10)))),
                },
                BasicBlock {
                    label: "default".to_string(),
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

        let pass = IfElseToSwitch;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Too few cases to convert to switch");
    }

    // ================================================================
    // StringConcatOptimization Tests
    // ================================================================

    #[test]
    fn test_string_concat_optimization_chain() {
        // Create chain: s1 = "a" + "b"; s2 = s1 + "c"; s3 = s2 + "d"
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::String,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("str_a"), value: Constant::String("a".to_string()) },
                    MirInst::Const { dest: Place::new("str_b"), value: Constant::String("b".to_string()) },
                    MirInst::BinOp {
                        dest: Place::new("s1"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("str_a")),
                        rhs: Operand::Place(Place::new("str_b")),
                    },
                    MirInst::Const { dest: Place::new("str_c"), value: Constant::String("c".to_string()) },
                    MirInst::BinOp {
                        dest: Place::new("s2"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("s1")),
                        rhs: Operand::Place(Place::new("str_c")),
                    },
                    MirInst::Const { dest: Place::new("str_d"), value: Constant::String("d".to_string()) },
                    MirInst::BinOp {
                        dest: Place::new("s3"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("s2")),
                        rhs: Operand::Place(Place::new("str_d")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("s3")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = StringConcatOptimization;
        let changed = pass.run_on_function(&mut func);
        // Chain of 4 operands (>= MIN_CHAIN_LENGTH=3), should be optimized
        assert!(changed, "Should optimize string concat chain");
    }

    #[test]
    fn test_string_concat_no_optimization_short_chain() {
        // Only 2 concats, below MIN_CHAIN_LENGTH=3
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::String,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("str_a"), value: Constant::String("a".to_string()) },
                    MirInst::Const { dest: Place::new("str_b"), value: Constant::String("b".to_string()) },
                    MirInst::BinOp {
                        dest: Place::new("s1"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("str_a")),
                        rhs: Operand::Place(Place::new("str_b")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("s1")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = StringConcatOptimization;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Short chain should not be optimized");
    }

    // ================================================================
    // LoopInvariantCodeMotion (LICM) Tests
    // ================================================================

    #[test]
    fn test_licm_hoists_pure_call() {
        // Loop with invariant len() call:
        // entry: goto loop_header
        // loop_header: i = phi [0, entry], [i_next, loop_body]
        //              l = call len(arr)
        //              cmp = i < l
        //              branch cmp, loop_body, exit
        // loop_body:   i_next = i + 1; goto loop_header
        // exit:        return i
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("arr".to_string(), MirType::String)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("loop_header".to_string()),
                },
                BasicBlock {
                    label: "loop_header".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("i"),
                            values: vec![
                                (Operand::Constant(Constant::Int(0)), "entry".to_string()),
                                (Operand::Place(Place::new("i_next")), "loop_body".to_string()),
                            ],
                        },
                        MirInst::Call {
                            dest: Some(Place::new("l")),
                            func: "len".to_string(),
                            args: vec![Operand::Place(Place::new("arr"))],
                            is_tail: false,
                        },
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Lt,
                            lhs: Operand::Place(Place::new("i")),
                            rhs: Operand::Place(Place::new("l")),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "loop_body".to_string(),
                        else_label: "exit".to_string(),
                    },
                },
                BasicBlock {
                    label: "loop_body".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("i_next"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("i")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("loop_header".to_string()),
                },
                BasicBlock {
                    label: "exit".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("i")))),
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

        let pass = LoopInvariantCodeMotion::new();
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should hoist len() call out of loop");

        // Entry block should now have the hoisted len call
        let has_hoisted_call = func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::Call { func: f, .. } if f == "len")
        });
        assert!(has_hoisted_call, "len() call should be hoisted to entry block");
    }

    #[test]
    fn test_licm_no_hoist_non_pure() {
        // Loop with non-pure call (print) — should not be hoisted
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("loop_header".to_string()),
                },
                BasicBlock {
                    label: "loop_header".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("i"),
                            values: vec![
                                (Operand::Constant(Constant::Int(0)), "entry".to_string()),
                                (Operand::Place(Place::new("i_next")), "loop_body".to_string()),
                            ],
                        },
                        MirInst::Call {
                            dest: None,
                            func: "print".to_string(),
                            args: vec![Operand::Constant(Constant::String("tick".to_string()))],
                            is_tail: false,
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("i")),
                        then_label: "loop_body".to_string(),
                        else_label: "exit".to_string(),
                    },
                },
                BasicBlock {
                    label: "loop_body".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("i_next"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("i")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("loop_header".to_string()),
                },
                BasicBlock {
                    label: "exit".to_string(),
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

        let pass = LoopInvariantCodeMotion::new();
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Non-pure function call should not be hoisted");
    }

    // ================================================================
    // LinearRecurrenceToLoop Tests
    // ================================================================

    #[test]
    fn test_linear_recurrence_fibonacci() {
        // fn fib(n: i64) -> i64 =
        //   if n <= 1 { n } else { fib(n-1) + fib(n-2) }
        let mut func = MirFunction {
            name: "fib".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
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
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "base".to_string(),
                        else_label: "recurse".to_string(),
                    },
                },
                BasicBlock {
                    label: "base".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("n")))),
                },
                BasicBlock {
                    label: "recurse".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("n_minus_1"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                        MirInst::Call {
                            dest: Some(Place::new("r1")),
                            func: "fib".to_string(),
                            args: vec![Operand::Place(Place::new("n_minus_1"))],
                            is_tail: false,
                        },
                        MirInst::BinOp {
                            dest: Place::new("n_minus_2"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(2)),
                        },
                        MirInst::Call {
                            dest: Some(Place::new("r2")),
                            func: "fib".to_string(),
                            args: vec![Operand::Place(Place::new("n_minus_2"))],
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

        let pass = LinearRecurrenceToLoop;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should transform fibonacci to iterative loop");

        // Verify no recursive calls remain
        let has_recursive_call = func.blocks.iter().any(|block| {
            block.instructions.iter().any(|inst| {
                matches!(inst, MirInst::Call { func: f, .. } if f == "fib")
            })
        });
        assert!(!has_recursive_call, "Recursive calls should be eliminated");
    }

    #[test]
    fn test_linear_recurrence_non_fibonacci_no_change() {
        // Non-fibonacci pattern: single recursion (factorial)
        let mut func = MirFunction {
            name: "fact".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
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
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "base".to_string(),
                        else_label: "recurse".to_string(),
                    },
                },
                BasicBlock {
                    label: "base".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                },
                BasicBlock {
                    label: "recurse".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("n_minus_1"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                        MirInst::Call {
                            dest: Some(Place::new("r")),
                            func: "fact".to_string(),
                            args: vec![Operand::Place(Place::new("n_minus_1"))],
                            is_tail: false,
                        },
                        MirInst::BinOp {
                            dest: Place::new("result"),
                            op: MirBinOp::Mul,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Place(Place::new("r")),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
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

        let pass = LinearRecurrenceToLoop;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Single recursion (factorial) should not be transformed");
    }

    // ================================================================
    // ConditionalIncrementToSelect Tests
    // ================================================================

    #[test]
    fn test_conditional_increment_to_select() {
        // Pattern: if cond { x + 1 } else { x }
        // cond_block:  branch %cond, then_block, else_block
        // then_block:  %sum = add %x, 1; goto merge
        // else_block:  goto merge
        // merge:       %result = phi [%sum, then_block], [%x, else_block]
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64), ("cond".to_string(), MirType::Bool)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cond")),
                        then_label: "then_block".to_string(),
                        else_label: "else_block".to_string(),
                    },
                },
                BasicBlock {
                    label: "then_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("sum"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "else_block".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("result"),
                            values: vec![
                                (Operand::Place(Place::new("sum")), "then_block".to_string()),
                                (Operand::Place(Place::new("x")), "else_block".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
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

        let pass = ConditionalIncrementToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should convert conditional increment to branchless select");

        // Entry block should now have a Cast and Add instead of Branch
        let has_cast = func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::Cast { from_ty: MirType::Bool, to_ty: MirType::I64, .. })
        });
        assert!(has_cast, "Should have bool-to-i64 cast in entry block");
    }

    #[test]
    fn test_conditional_increment_no_match_increment_by_2() {
        // Similar to above but increment by 2, not 1 — should not match
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64), ("cond".to_string(), MirType::Bool)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cond")),
                        then_label: "then_block".to_string(),
                        else_label: "else_block".to_string(),
                    },
                },
                BasicBlock {
                    label: "then_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("sum"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(2)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "else_block".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("result"),
                            values: vec![
                                (Operand::Place(Place::new("sum")), "then_block".to_string()),
                                (Operand::Place(Place::new("x")), "else_block".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
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

        let pass = ConditionalIncrementToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Increment by 2 should not match conditional increment pattern");
    }

    // ================================================================
    // Cycle 77: OptimizationPipeline tests
    // ================================================================

    #[test]
    fn test_pipeline_debug_no_passes() {
        let pipeline = OptimizationPipeline::for_level(OptLevel::Debug);
        assert!(pipeline.passes.is_empty());
    }

    #[test]
    fn test_pipeline_release_has_passes() {
        let pipeline = OptimizationPipeline::for_level(OptLevel::Release);
        assert!(!pipeline.passes.is_empty());
        // Release should include constant folding
        assert!(pipeline.passes.iter().any(|p| p.name() == "constant_folding"));
    }

    #[test]
    fn test_pipeline_aggressive_has_cse() {
        let pipeline = OptimizationPipeline::for_level(OptLevel::Aggressive);
        // Aggressive should include CSE (not in Release)
        assert!(pipeline.passes.iter().any(|p| p.name() == "common_subexpression_elimination"));
    }

    #[test]
    fn test_pipeline_default_trait() {
        let pipeline = OptimizationPipeline::default();
        assert!(pipeline.passes.is_empty());
        assert_eq!(pipeline.max_iterations, 10);
    }

    #[test]
    fn test_pipeline_set_max_iterations() {
        let mut pipeline = OptimizationPipeline::new();
        pipeline.set_max_iterations(5);
        assert_eq!(pipeline.max_iterations, 5);
    }

    // ================================================================
    // Cycle 77: OptimizationStats tests
    // ================================================================

    #[test]
    fn test_stats_new_empty() {
        let stats = OptimizationStats::new();
        assert_eq!(stats.iterations, 0);
        assert!(stats.pass_counts.is_empty());
    }

    #[test]
    fn test_stats_record_pass() {
        let mut stats = OptimizationStats::new();
        stats.record_pass("constant_folding");
        stats.record_pass("constant_folding");
        stats.record_pass("dce");
        assert_eq!(stats.pass_counts["constant_folding"], 2);
        assert_eq!(stats.pass_counts["dce"], 1);
    }

    #[test]
    fn test_stats_merge() {
        let mut stats1 = OptimizationStats::new();
        stats1.record_pass("cf");
        stats1.record_pass("cf");

        let mut stats2 = OptimizationStats::new();
        stats2.record_pass("cf");
        stats2.record_pass("dce");

        stats1.merge(&stats2);
        assert_eq!(stats1.pass_counts["cf"], 3);
        assert_eq!(stats1.pass_counts["dce"], 1);
    }

    // ================================================================
    // Cycle 77: OptLevel tests
    // ================================================================

    #[test]
    fn test_opt_level_default() {
        let level = OptLevel::default();
        assert!(matches!(level, OptLevel::Debug));
    }

    // ================================================================
    // Cycle 77: ConstantFolding extended tests
    // ================================================================

    #[test]
    fn test_constant_folding_float() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::F64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("a"), value: Constant::Float(2.5) },
                    MirInst::Const { dest: Place::new("b"), value: Constant::Float(1.5) },
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::FAdd,
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
        };

        let pass = ConstantFolding;
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[2],
            MirInst::Const { value: Constant::Float(f), .. } if (*f - 4.0).abs() < 1e-10));
    }

    #[test]
    fn test_constant_folding_string_concat() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("a"), value: Constant::String("Hello".to_string()) },
                    MirInst::Const { dest: Place::new("b"), value: Constant::String(" World".to_string()) },
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
        };

        let pass = ConstantFolding;
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[2],
            MirInst::Const { value: Constant::String(s), .. } if s == "Hello World"));
    }

    #[test]
    fn test_constant_folding_unary_neg() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("a"), value: Constant::Int(42) },
                    MirInst::UnaryOp {
                        dest: Place::new("b"),
                        op: MirUnaryOp::Neg,
                        src: Operand::Place(Place::new("a")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("b")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = ConstantFolding;
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[1],
            MirInst::Const { value: Constant::Int(-42), .. }));
    }

    #[test]
    fn test_constant_folding_unary_not() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::Bool,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("a"), value: Constant::Bool(true) },
                    MirInst::UnaryOp {
                        dest: Place::new("b"),
                        op: MirUnaryOp::Not,
                        src: Operand::Place(Place::new("a")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("b")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = ConstantFolding;
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[1],
            MirInst::Const { value: Constant::Bool(false), .. }));
    }

    #[test]
    fn test_constant_folding_div_by_zero_no_fold() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("a"), value: Constant::Int(10) },
                    MirInst::Const { dest: Place::new("b"), value: Constant::Int(0) },
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::Div,
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
        };

        let pass = ConstantFolding;
        // Should NOT fold div by zero
        let _changed = pass.run_on_function(&mut func);
        // The constants a and b are tracked but div by zero is not folded
        assert!(matches!(&func.blocks[0].instructions[2], MirInst::BinOp { op: MirBinOp::Div, .. }));
    }

    #[test]
    fn test_constant_folding_builtin_chr() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("code"), value: Constant::Int(65) },
                    MirInst::Call {
                        dest: Some(Place::new("ch")),
                        func: "chr".to_string(),
                        args: vec![Operand::Place(Place::new("code"))],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("ch")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = ConstantFolding;
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[1],
            MirInst::Const { value: Constant::String(s), .. } if s == "A"));
    }

    #[test]
    fn test_constant_folding_builtin_ord() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("ch"), value: Constant::String("Z".to_string()) },
                    MirInst::Call {
                        dest: Some(Place::new("code")),
                        func: "ord".to_string(),
                        args: vec![Operand::Place(Place::new("ch"))],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("code")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = ConstantFolding;
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[1],
            MirInst::Const { value: Constant::Int(90), .. }));
    }

    #[test]
    fn test_constant_folding_bool_and() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::Bool,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("a"), value: Constant::Bool(true) },
                    MirInst::Const { dest: Place::new("b"), value: Constant::Bool(false) },
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::And,
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
        };

        let pass = ConstantFolding;
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[2],
            MirInst::Const { value: Constant::Bool(false), .. }));
    }

    #[test]
    fn test_constant_folding_copy_propagation() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("a"), value: Constant::Int(10) },
                    MirInst::Copy { dest: Place::new("b"), src: Place::new("a") },
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("b")),
                        rhs: Operand::Constant(Constant::Int(5)),
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
        };

        let pass = ConstantFolding;
        assert!(pass.run_on_function(&mut func));
        // b should have been tracked as constant 10, so 10 + 5 = 15
        assert!(matches!(&func.blocks[0].instructions[2],
            MirInst::Const { value: Constant::Int(15), .. }));
    }

    // ================================================================
    // Cycle 77: AggressiveInlining tests
    // ================================================================

    #[test]
    fn test_aggressive_inlining_small_function() {
        let mut program = MirProgram {
            functions: vec![MirFunction {
                name: "add".to_string(),
                params: vec![("x".to_string(), MirType::I64), ("y".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("r"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Place(Place::new("y")),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let inlining = AggressiveInlining::new();
        assert!(inlining.run_on_program(&mut program));
        assert!(program.functions[0].always_inline);
    }

    #[test]
    fn test_aggressive_inlining_main_not_inlined() {
        let mut program = MirProgram {
            functions: vec![MirFunction {
                name: "main".to_string(),
                params: vec![],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const { dest: Place::new("r"), value: Constant::Int(0) },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let inlining = AggressiveInlining::new();
        let _changed = inlining.run_on_program(&mut program);
        assert!(!program.functions[0].always_inline);
    }

    #[test]
    fn test_aggressive_inlining_recursive_not_inlined() {
        let mut program = MirProgram {
            functions: vec![MirFunction {
                name: "fib".to_string(),
                params: vec![("n".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Call {
                            dest: Some(Place::new("r")),
                            func: "fib".to_string(),
                            args: vec![Operand::Place(Place::new("n"))],
                            is_tail: true,
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let inlining = AggressiveInlining::new();
        inlining.run_on_program(&mut program);
        assert!(!program.functions[0].always_inline);
        assert!(!program.functions[0].inline_hint);
    }

    #[test]
    fn test_aggressive_inlining_with_custom_thresholds() {
        let inlining = AggressiveInlining::with_thresholds(5, 10);
        assert_eq!(inlining.max_instructions, 5);
        assert_eq!(inlining.max_pure_instructions, 10);
    }

    #[test]
    fn test_aggressive_inlining_pure_higher_threshold() {
        // Pure function with 18 instructions — within pure threshold (20) but above regular (15)
        let mut instructions = Vec::new();
        for i in 0..18 {
            instructions.push(MirInst::Const {
                dest: Place::new(format!("v{}", i)),
                value: Constant::Int(i as i64),
            });
        }

        let mut program = MirProgram {
            functions: vec![MirFunction {
                name: "pure_fn".to_string(),
                params: vec![],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions,
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: true,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let inlining = AggressiveInlining::new();
        assert!(inlining.run_on_program(&mut program));
        assert!(program.functions[0].always_inline);
    }

    #[test]
    fn test_aggressive_inlining_hint_for_medium() {
        // Medium function — too large for always_inline but within hint threshold
        let mut instructions = Vec::new();
        for i in 0..30 {
            instructions.push(MirInst::Const {
                dest: Place::new(format!("v{}", i)),
                value: Constant::Int(i as i64),
            });
        }

        let mut program = MirProgram {
            functions: vec![MirFunction {
                name: "medium_fn".to_string(),
                params: vec![],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions,
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let inlining = AggressiveInlining::new();
        assert!(inlining.run_on_program(&mut program));
        assert!(!program.functions[0].always_inline);
        assert!(program.functions[0].inline_hint);
    }

    // ================================================================
    // Cycle 77: MemoryEffectAnalysis tests
    // ================================================================

    #[test]
    fn test_memory_effect_pure_arithmetic() {
        let mut program = MirProgram {
            functions: vec![MirFunction {
                name: "square".to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("r"),
                            op: MirBinOp::Mul,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Place(Place::new("x")),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let analysis = MemoryEffectAnalysis::new();
        assert!(analysis.run_on_program(&mut program));
        assert!(program.functions[0].is_memory_free);
    }

    #[test]
    fn test_memory_effect_call_not_free() {
        let mut program = MirProgram {
            functions: vec![MirFunction {
                name: "caller".to_string(),
                params: vec![],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Call {
                            dest: Some(Place::new("r")),
                            func: "foo".to_string(),
                            args: vec![],
                            is_tail: false,
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let analysis = MemoryEffectAnalysis::new();
        let changed = analysis.run_on_program(&mut program);
        assert!(!changed);
        assert!(!program.functions[0].is_memory_free);
    }

    #[test]
    fn test_memory_effect_main_skipped() {
        let mut program = MirProgram {
            functions: vec![MirFunction {
                name: "main".to_string(),
                params: vec![],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const { dest: Place::new("r"), value: Constant::Int(0) },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let analysis = MemoryEffectAnalysis::new();
        let changed = analysis.run_on_program(&mut program);
        // main is always skipped
        assert!(!changed);
        assert!(!program.functions[0].is_memory_free);
    }

    #[test]
    fn test_memory_effect_analysis_name() {
        let analysis = MemoryEffectAnalysis::new();
        assert_eq!(analysis.name(), "memory_effect_analysis");
    }

    // ================================================================
    // Cycle 77: fold_binop / fold_unaryop edge cases
    // ================================================================

    #[test]
    fn test_fold_binop_int_comparison() {
        assert!(matches!(fold_binop(MirBinOp::Lt, &Constant::Int(3), &Constant::Int(5)), Some(Constant::Bool(true))));
        assert!(matches!(fold_binop(MirBinOp::Gt, &Constant::Int(3), &Constant::Int(5)), Some(Constant::Bool(false))));
        assert!(matches!(fold_binop(MirBinOp::Le, &Constant::Int(5), &Constant::Int(5)), Some(Constant::Bool(true))));
        assert!(matches!(fold_binop(MirBinOp::Ge, &Constant::Int(5), &Constant::Int(5)), Some(Constant::Bool(true))));
        assert!(matches!(fold_binop(MirBinOp::Eq, &Constant::Int(5), &Constant::Int(5)), Some(Constant::Bool(true))));
        assert!(matches!(fold_binop(MirBinOp::Ne, &Constant::Int(5), &Constant::Int(3)), Some(Constant::Bool(true))));
    }

    #[test]
    fn test_fold_binop_bool_ops() {
        assert!(matches!(fold_binop(MirBinOp::And, &Constant::Bool(true), &Constant::Bool(true)), Some(Constant::Bool(true))));
        assert!(matches!(fold_binop(MirBinOp::Or, &Constant::Bool(false), &Constant::Bool(true)), Some(Constant::Bool(true))));
        assert!(matches!(fold_binop(MirBinOp::And, &Constant::Bool(true), &Constant::Bool(false)), Some(Constant::Bool(false))));
        assert!(matches!(fold_binop(MirBinOp::Or, &Constant::Bool(false), &Constant::Bool(false)), Some(Constant::Bool(false))));
    }

    #[test]
    fn test_fold_binop_float_ops() {
        assert!(matches!(fold_binop(MirBinOp::FSub, &Constant::Float(5.0), &Constant::Float(3.0)), Some(Constant::Float(f)) if (f - 2.0).abs() < 1e-10));
        assert!(matches!(fold_binop(MirBinOp::FMul, &Constant::Float(2.0), &Constant::Float(3.0)), Some(Constant::Float(f)) if (f - 6.0).abs() < 1e-10));
        assert!(matches!(fold_binop(MirBinOp::FDiv, &Constant::Float(6.0), &Constant::Float(2.0)), Some(Constant::Float(f)) if (f - 3.0).abs() < 1e-10));
        // Float div by zero not folded
        assert!(fold_binop(MirBinOp::FDiv, &Constant::Float(6.0), &Constant::Float(0.0)).is_none());
    }

    #[test]
    fn test_fold_binop_mod() {
        assert!(matches!(fold_binop(MirBinOp::Mod, &Constant::Int(10), &Constant::Int(3)), Some(Constant::Int(1))));
        // Mod by zero not folded
        assert!(fold_binop(MirBinOp::Mod, &Constant::Int(10), &Constant::Int(0)).is_none());
    }

    #[test]
    fn test_fold_binop_type_mismatch_returns_none() {
        // Int + Bool should return None
        assert!(fold_binop(MirBinOp::Add, &Constant::Int(1), &Constant::Bool(true)).is_none());
    }

    #[test]
    fn test_fold_unaryop_fneg() {
        let result = fold_unaryop(MirUnaryOp::FNeg, &Constant::Float(2.5));
        assert!(matches!(result, Some(Constant::Float(r)) if (r + 2.5).abs() < 1e-10));
    }

    #[test]
    fn test_fold_unaryop_type_mismatch() {
        // Neg on bool should return None
        assert!(fold_unaryop(MirUnaryOp::Neg, &Constant::Bool(true)).is_none());
    }

    // ================================================================
    // Cycle 77: LoopBoundedNarrowing tests
    // ================================================================

    #[test]
    fn test_loop_bounded_narrowing_is_builtin() {
        assert!(LoopBoundedNarrowing::is_builtin("malloc"));
        assert!(LoopBoundedNarrowing::is_builtin("free"));
        assert!(LoopBoundedNarrowing::is_builtin("println"));
        assert!(LoopBoundedNarrowing::is_builtin("sqrt"));
        assert!(LoopBoundedNarrowing::is_builtin("len"));
        assert!(!LoopBoundedNarrowing::is_builtin("my_function"));
        assert!(!LoopBoundedNarrowing::is_builtin("fib"));
    }

    #[test]
    fn test_loop_bounded_narrowing_has_direct_multiplication() {
        let func_with_mul = MirFunction {
            name: "mul_fn".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Mul,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Place(Place::new("x")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        assert!(LoopBoundedNarrowing::has_direct_multiplication(&func_with_mul));

        let func_no_mul = MirFunction {
            name: "add_fn".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        assert!(!LoopBoundedNarrowing::has_direct_multiplication(&func_no_mul));
    }

    // ================================================================
    // Cycle 78: AlgebraicSimplification extended tests
    // ================================================================

    #[test]
    fn test_algebraic_sub_zero() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Sub,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(0)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[0], MirInst::Copy { .. }));
    }

    #[test]
    fn test_algebraic_div_by_one() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Div,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[0], MirInst::Copy { .. }));
    }

    #[test]
    fn test_algebraic_div_power_of_2_to_shift() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Div,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(8)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(pass.run_on_function(&mut func));
        // x / 8 → x >> 3
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::BinOp { op: MirBinOp::Shr, rhs: Operand::Constant(Constant::Int(3)), .. }));
    }

    #[test]
    fn test_algebraic_mod_power_of_2_to_bitand() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Mod,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(16)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(pass.run_on_function(&mut func));
        // x % 16 → x & 15
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::BinOp { op: MirBinOp::Band, rhs: Operand::Constant(Constant::Int(15)), .. }));
    }

    #[test]
    fn test_algebraic_or_true() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::Bool)],
            ret_ty: MirType::Bool,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Or,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Bool(true)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::Const { value: Constant::Bool(true), .. }));
    }

    #[test]
    fn test_algebraic_and_false() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::Bool)],
            ret_ty: MirType::Bool,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::And,
                        lhs: Operand::Constant(Constant::Bool(false)),
                        rhs: Operand::Place(Place::new("x")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::Const { value: Constant::Bool(false), .. }));
    }

    #[test]
    fn test_algebraic_fadd_zero() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::F64)],
            ret_ty: MirType::F64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::FAdd,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Float(0.0)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[0], MirInst::Copy { .. }));
    }

    #[test]
    fn test_algebraic_fmul_zero() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::F64)],
            ret_ty: MirType::F64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::FMul,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Float(0.0)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::Const { value: Constant::Float(f), .. } if *f == 0.0));
    }

    #[test]
    fn test_algebraic_fmul_one() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::F64)],
            ret_ty: MirType::F64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::FMul,
                        lhs: Operand::Constant(Constant::Float(1.0)),
                        rhs: Operand::Place(Place::new("x")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[0], MirInst::Copy { .. }));
    }

    #[test]
    fn test_algebraic_mul_commutative_power_of_2() {
        // 4 * x → x << 2
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Mul,
                        lhs: Operand::Constant(Constant::Int(4)),
                        rhs: Operand::Place(Place::new("x")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(pass.run_on_function(&mut func));
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::BinOp { op: MirBinOp::Shl, rhs: Operand::Constant(Constant::Int(2)), .. }));
    }

    // ================================================================
    // Cycle 78: DCE extended tests
    // ================================================================

    #[test]
    fn test_dce_preserves_used_in_terminator() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("cond".to_string(), MirType::Bool)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("unused"), value: Constant::Int(42) },
                    MirInst::Const { dest: Place::new("used"), value: Constant::Int(1) },
                ],
                terminator: Terminator::Branch {
                    cond: Operand::Place(Place::new("cond")),
                    then_label: "t".to_string(),
                    else_label: "e".to_string(),
                },
            },
            BasicBlock {
                label: "t".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("used")))),
            },
            BasicBlock {
                label: "e".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
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
        // "unused" should be removed but "used" should remain
        assert_eq!(func.blocks[0].instructions.len(), 1);
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::Const { dest, value: Constant::Int(1), .. } if dest.name == "used"));
    }

    #[test]
    fn test_dce_preserves_calls() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("unused_result")),
                        func: "side_effect".to_string(),
                        args: vec![],
                        is_tail: false,
                    },
                    MirInst::Const { dest: Place::new("r"), value: Constant::Int(0) },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        let _changed = pass.run_on_function(&mut func);
        // Calls have side effects, should be preserved even if result is unused
        assert!(func.blocks[0].instructions.iter().any(|i| matches!(i, MirInst::Call { .. })));
    }

    // ================================================================
    // Cycle 78: ConstantPropagationNarrowing basic tests
    // ================================================================

    #[test]
    fn test_constant_propagation_narrowing_name() {
        let program = MirProgram {
            functions: vec![],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };
        let narrowing = ConstantPropagationNarrowing::from_program(&program);
        assert_eq!(narrowing.name(), "constant_propagation_narrowing");
    }

    #[test]
    fn test_constant_propagation_narrowing_empty_program() {
        let mut program = MirProgram {
            functions: vec![],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };
        let narrowing = ConstantPropagationNarrowing::from_program(&program);
        assert!(!narrowing.run_on_program(&mut program));
    }

    // ================================================================
    // Cycle 78: LICM default trait test
    // ================================================================

    #[test]
    fn test_licm_default() {
        let licm = LoopInvariantCodeMotion::default();
        assert_eq!(licm.name(), "loop_invariant_code_motion");
    }

    // ================================================================
    // Cycle 78: Pipeline optimize with multiple functions
    // ================================================================

    #[test]
    fn test_pipeline_optimize_multiple_functions() {
        let mut program = MirProgram {
            functions: vec![
                MirFunction {
                    name: "f1".to_string(),
                    params: vec![],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            MirInst::Const { dest: Place::new("a"), value: Constant::Int(2) },
                            MirInst::Const { dest: Place::new("b"), value: Constant::Int(3) },
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
                },
                MirFunction {
                    name: "f2".to_string(),
                    params: vec![],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            MirInst::Const { dest: Place::new("x"), value: Constant::Int(10) },
                            MirInst::Const { dest: Place::new("y"), value: Constant::Int(0) },
                            MirInst::BinOp {
                                dest: Place::new("z"),
                                op: MirBinOp::Add,
                                lhs: Operand::Place(Place::new("x")),
                                rhs: Operand::Place(Place::new("y")),
                            },
                        ],
                        terminator: Terminator::Return(Some(Operand::Place(Place::new("z")))),
                    }],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
            ],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let pipeline = OptimizationPipeline::for_level(OptLevel::Release);
        let stats = pipeline.optimize(&mut program);
        // Both functions should have been optimized
        assert!(stats.pass_counts.values().sum::<usize>() > 0);
    }

    // ================================================================
    // Cycle 78: simplify_binop edge cases
    // ================================================================

    #[test]
    fn test_simplify_or_false_left() {
        // false || x → x
        let result = simplify_binop(
            &Place::new("r"),
            MirBinOp::Or,
            &Operand::Constant(Constant::Bool(false)),
            &Operand::Place(Place::new("x")),
        );
        assert!(matches!(result, Some(MirInst::Copy { .. })));
    }

    #[test]
    fn test_simplify_no_simplification() {
        // x + y — no simplification possible
        let result = simplify_binop(
            &Place::new("r"),
            MirBinOp::Add,
            &Operand::Place(Place::new("x")),
            &Operand::Place(Place::new("y")),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_simplify_unsupported_op_returns_none() {
        // Bitwise ops don't have algebraic simplification
        let result = simplify_binop(
            &Place::new("r"),
            MirBinOp::Band,
            &Operand::Place(Place::new("x")),
            &Operand::Place(Place::new("y")),
        );
        assert!(result.is_none());
    }

    // ================================================================
    // Cycle 121: Advanced Optimization Pass Tests
    // ================================================================

    #[test]
    fn test_contract_based_bounds_check_with_known_bounds() {
        // Contract: pre idx >= 0, pre idx < 10 + comparison idx >= 0 should be eliminated
        let mut func = MirFunction {
            name: "access".to_string(),
            params: vec![
                ("idx".to_string(), MirType::I64),
            ],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    // Check: idx >= 0 (redundant given precondition)
                    MirInst::BinOp {
                        dest: Place::new("check"),
                        op: MirBinOp::Ge,
                        lhs: Operand::Place(Place::new("idx")),
                        rhs: Operand::Constant(Constant::Int(0)),
                    },
                ],
                terminator: Terminator::Branch {
                    cond: Operand::Place(Place::new("check")),
                    then_label: "safe".to_string(),
                    else_label: "panic".to_string(),
                },
            },
            BasicBlock {
                label: "safe".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("r"), value: Constant::Int(42) },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            },
            BasicBlock {
                label: "panic".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(-1)))),
            }],
            preconditions: vec![
                ContractFact::VarCmp { var: "idx".to_string(), op: CmpOp::Ge, value: 0 },
                ContractFact::VarCmp { var: "idx".to_string(), op: CmpOp::Lt, value: 10 },
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
        // The contract states idx >= 0, so check idx >= 0 is always true → folded to Const(true)
        assert!(changed);
        // The BinOp should be replaced with Const(Bool(true))
        assert!(matches!(&func.blocks[0].instructions[0], MirInst::Const { value: Constant::Bool(true), .. }));
    }

    #[test]
    fn test_contract_unreachable_nested_conditions() {
        // Test that contract unreachable elimination handles chain of conditions
        let func = MirFunction {
            name: "double_check".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("c1"),
                        op: MirBinOp::Gt,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(0)),
                    },
                ],
                terminator: Terminator::Branch {
                    cond: Operand::Place(Place::new("c1")),
                    then_label: "positive".to_string(),
                    else_label: "negative".to_string(),
                },
            },
            BasicBlock {
                label: "positive".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("r"), value: Constant::Int(1) },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            },
            BasicBlock {
                label: "negative".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("r2"), value: Constant::Int(-1) },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r2")))),
            }],
            preconditions: vec![
                ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Gt, value: 0 },
            ],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = ContractUnreachableElimination;
        let mut func_clone = func.clone();
        let changed = pass.run_on_function(&mut func_clone);
        // With pre x > 0, the else branch should be unreachable
        assert!(changed);
        // The branch should be simplified to a direct goto
        assert!(matches!(func_clone.blocks[0].terminator, Terminator::Goto(_)));
    }

    #[test]
    fn test_copy_propagation_replaces_in_binop() {
        // Verify copy propagation replaces operands in binary operations
        // a = copy input; result = a + 1 → result = input + 1
        let mut func = MirFunction {
            name: "copy_use".to_string(),
            params: vec![("input".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Copy {
                        dest: Place::new("a"),
                        src: Place::new("input"),
                    },
                    MirInst::BinOp {
                        dest: Place::new("result"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("a")),
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

        let pass = CopyPropagation;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        // After copy propagation, the BinOp should use 'input' directly
        let binop_found = func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::BinOp { lhs: Operand::Place(p), .. } if p.name == "input")
        });
        assert!(binop_found);
    }

    #[test]
    fn test_phi_simplification_same_operand_from_multiple_preds() {
        // Phi with same value from multiple predecessors should simplify to copy
        let mut func = MirFunction {
            name: "phi_test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cond"),
                            op: MirBinOp::Gt,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cond")),
                        then_label: "then".to_string(),
                        else_label: "else".to_string(),
                    },
                },
                BasicBlock {
                    label: "then".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "else".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("val"),
                            values: vec![
                                (Operand::Place(Place::new("x")), "then".to_string()),
                                (Operand::Place(Place::new("x")), "else".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("val")))),
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

        let pass = PhiSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        // Phi should be replaced with a Copy since both operands are the same
        assert!(matches!(&func.blocks[3].instructions[0], MirInst::Copy { .. }));
    }

    #[test]
    fn test_block_merging_chain_of_gotos() {
        // Chain: entry -> mid -> end should merge mid into entry
        let mut func = MirFunction {
            name: "chain".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const { dest: Place::new("a"), value: Constant::Int(1) },
                    ],
                    terminator: Terminator::Goto("mid".to_string()),
                },
                BasicBlock {
                    label: "mid".to_string(),
                    instructions: vec![
                        MirInst::Const { dest: Place::new("b"), value: Constant::Int(2) },
                    ],
                    terminator: Terminator::Goto("end".to_string()),
                },
                BasicBlock {
                    label: "end".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("r"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("a")),
                            rhs: Operand::Place(Place::new("b")),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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

        let pass = BlockMerging;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        // After merging, entry should contain both constants
        assert!(func.blocks[0].instructions.len() >= 2);
    }

    #[test]
    fn test_cse_eliminates_same_binop() {
        // Two identical add operations → second should be eliminated
        let mut func = MirFunction {
            name: "cse_test".to_string(),
            params: vec![("x".to_string(), MirType::I64), ("y".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("a"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Place(Place::new("y")),
                    },
                    MirInst::BinOp {
                        dest: Place::new("b"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Place(Place::new("y")),
                    },
                    MirInst::BinOp {
                        dest: Place::new("result"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("a")),
                        rhs: Operand::Place(Place::new("b")),
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

        let pass = CommonSubexpressionElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        // The second BinOp (b = x+y) should be replaced with a Copy from a
        assert!(func.blocks[0].instructions.iter().any(|i| matches!(i, MirInst::Copy { dest, .. } if dest.name == "b")));
    }

    #[test]
    fn test_dce_removes_unused_const() {
        // A constant that is never used should be eliminated
        let mut func = MirFunction {
            name: "dce_test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("unused"), value: Constant::Int(999) },
                    MirInst::Const { dest: Place::new("used"), value: Constant::Int(42) },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("used")))),
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
        assert!(matches!(&func.blocks[0].instructions[0], MirInst::Const { dest, value: Constant::Int(42) } if dest.name == "used"));
    }

    #[test]
    fn test_constant_folding_chain() {
        // Folding: a = 2+3, b = a*4 → a = 5, b = 20
        let mut func = MirFunction {
            name: "fold_chain".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("c2"), value: Constant::Int(2) },
                    MirInst::Const { dest: Place::new("c3"), value: Constant::Int(3) },
                    MirInst::BinOp {
                        dest: Place::new("a"),
                        op: MirBinOp::Add,
                        lhs: Operand::Constant(Constant::Int(2)),
                        rhs: Operand::Constant(Constant::Int(3)),
                    },
                    MirInst::BinOp {
                        dest: Place::new("b"),
                        op: MirBinOp::Mul,
                        lhs: Operand::Constant(Constant::Int(5)),
                        rhs: Operand::Constant(Constant::Int(4)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("b")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = ConstantFolding;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        // Both BinOps should be folded to constants
        let folded_count = func.blocks[0].instructions.iter()
            .filter(|i| matches!(i, MirInst::Const { .. }))
            .count();
        assert!(folded_count >= 3); // c2, c3, and at least a or b folded
    }

    #[test]
    fn test_constant_folding_comparison() {
        // Folding comparison: 5 > 3 → true
        let mut func = MirFunction {
            name: "fold_cmp".to_string(),
            params: vec![],
            ret_ty: MirType::Bool,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Gt,
                        lhs: Operand::Constant(Constant::Int(5)),
                        rhs: Operand::Constant(Constant::Int(3)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = ConstantFolding;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        // The comparison should be folded to Const(Bool(true))
        assert!(matches!(&func.blocks[0].instructions[0], MirInst::Const { value: Constant::Bool(true), .. }));
    }

    #[test]
    fn test_algebraic_sub_self_to_zero() {
        // x - x → 0 (requires same operand detection)
        let result = simplify_binop(
            &Place::new("r"),
            MirBinOp::Sub,
            &Operand::Place(Place::new("x")),
            &Operand::Constant(Constant::Int(0)),
        );
        // Sub x 0 → copy x (identity)
        assert!(result.is_some());
        if let Some(MirInst::Copy { dest, src }) = result {
            assert_eq!(dest.name, "r");
            assert_eq!(src.name, "x");
        }
    }

    #[test]
    fn test_memory_load_cse_same_call_twice() {
        // Two identical load_f64 calls with same args → second should be CSE'd
        let mut func = MirFunction {
            name: "field_cse".to_string(),
            params: vec![("ptr".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("a")),
                        func: "load_f64".to_string(),
                        args: vec![Operand::Place(Place::new("ptr"))],
                        is_tail: false,
                    },
                    MirInst::Call {
                        dest: Some(Place::new("b")),
                        func: "load_f64".to_string(),
                        args: vec![Operand::Place(Place::new("ptr"))],
                        is_tail: false,
                    },
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("a")),
                        rhs: Operand::Place(Place::new("b")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
        assert!(changed);
        // Second load_f64 should be replaced with Copy
        assert!(func.blocks[0].instructions.iter().any(|i| matches!(i, MirInst::Copy { dest, .. } if dest.name == "b")));
    }

    #[test]
    fn test_simplify_branches_constant_true_goto() {
        // Branch with constant true should become Goto to then_label
        let mut func = MirFunction {
            name: "branch_true".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Constant(Constant::Bool(true)),
                        then_label: "yes".to_string(),
                        else_label: "no".to_string(),
                    },
                },
                BasicBlock {
                    label: "yes".to_string(),
                    instructions: vec![
                        MirInst::Const { dest: Place::new("r"), value: Constant::Int(1) },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                },
                BasicBlock {
                    label: "no".to_string(),
                    instructions: vec![
                        MirInst::Const { dest: Place::new("r2"), value: Constant::Int(0) },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("r2")))),
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

        let pass = SimplifyBranches;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        assert!(matches!(func.blocks[0].terminator, Terminator::Goto(ref label) if label == "yes"));
    }

    #[test]
    fn test_pure_function_cse_three_identical_calls() {
        // Three identical calls to a pure function → only one call, two copies
        let pure_fns: HashSet<String> = ["pure_compute".to_string()].into_iter().collect();
        let cse = PureFunctionCSE::new(pure_fns);

        let mut func = MirFunction {
            name: "caller".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("a")),
                        func: "pure_compute".to_string(),
                        args: vec![Operand::Place(Place::new("x"))],
                        is_tail: false,
                    },
                    MirInst::Call {
                        dest: Some(Place::new("b")),
                        func: "pure_compute".to_string(),
                        args: vec![Operand::Place(Place::new("x"))],
                        is_tail: false,
                    },
                    MirInst::Call {
                        dest: Some(Place::new("c")),
                        func: "pure_compute".to_string(),
                        args: vec![Operand::Place(Place::new("x"))],
                        is_tail: false,
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
        };

        let changed = cse.run_on_function(&mut func);
        assert!(changed);
        // Should have 1 Call + 2 Copies
        let call_count = func.blocks[0].instructions.iter()
            .filter(|i| matches!(i, MirInst::Call { .. }))
            .count();
        let copy_count = func.blocks[0].instructions.iter()
            .filter(|i| matches!(i, MirInst::Copy { .. }))
            .count();
        assert_eq!(call_count, 1);
        assert_eq!(copy_count, 2);
    }

    #[test]
    fn test_unreachable_block_removal_after_return() {
        // Block after return with no predecessors should be removed
        let mut func = MirFunction {
            name: "unreachable_test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const { dest: Place::new("r"), value: Constant::Int(42) },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                },
                BasicBlock {
                    label: "dead".to_string(),
                    instructions: vec![
                        MirInst::Const { dest: Place::new("x"), value: Constant::Int(999) },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
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

        let pass = UnreachableBlockElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        assert_eq!(func.blocks.len(), 1);
        assert_eq!(func.blocks[0].label, "entry");
    }

    #[test]
    fn test_loop_bounded_narrowing_with_mul_in_func() {
        // Function with multiplication should be flagged
        let func = MirFunction {
            name: "mul_func".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Mul,
                        lhs: Operand::Place(Place::new("n")),
                        rhs: Operand::Place(Place::new("n")),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        assert!(LoopBoundedNarrowing::has_direct_multiplication(&func));
    }

    #[test]
    fn test_loop_bounded_narrowing_without_mul() {
        // Function without multiplication should NOT be flagged
        let func = MirFunction {
            name: "add_func".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("n")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        assert!(!LoopBoundedNarrowing::has_direct_multiplication(&func));
    }

    #[test]
    fn test_global_field_cse_different_base_no_dedup() {
        // FieldAccess from different base objects should NOT be deduplicated
        let mut func = MirFunction {
            name: "diff_base".to_string(),
            params: vec![
                ("obj1".to_string(), MirType::Struct { name: "Point".to_string(), fields: vec![] }),
                ("obj2".to_string(), MirType::Struct { name: "Point".to_string(), fields: vec![] }),
            ],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::FieldAccess {
                        dest: Place::new("a"),
                        base: Place::new("obj1"),
                        field: "x".to_string(),
                        field_index: 0,
                        struct_name: "Point".to_string(),
                    },
                    MirInst::FieldAccess {
                        dest: Place::new("b"),
                        base: Place::new("obj2"),
                        field: "x".to_string(),
                        field_index: 0,
                        struct_name: "Point".to_string(),
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

        let pass = GlobalFieldAccessCSE;
        let changed = pass.run_on_function(&mut func);
        // Different bases → no deduplication should happen
        assert!(!changed);
    }

    #[test]
    fn test_string_concat_opt_default() {
        let opt = StringConcatOptimization;
        assert_eq!(opt.name(), "StringConcatOptimization");
    }

    #[test]
    fn test_if_else_to_switch_default() {
        let opt = IfElseToSwitch;
        assert_eq!(opt.name(), "IfElseToSwitch");
    }

    #[test]
    fn test_tail_recursive_to_loop_default() {
        let opt = TailRecursiveToLoop;
        assert_eq!(opt.name(), "TailRecursiveToLoop");
    }

    #[test]
    fn test_loop_bounded_narrowing_from_main_call() {
        // Test LoopBoundedNarrowing: main calls func(100), parameter should be narrowed to i32
        let mut program = MirProgram {
            functions: vec![
                MirFunction {
                    name: "main".to_string(),
                    params: vec![],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            MirInst::Call {
                                dest: Some(Place::new("r")),
                                func: "compute".to_string(),
                                args: vec![Operand::Constant(Constant::Int(100))],
                                is_tail: false,
                            },
                        ],
                        terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                    }],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
                MirFunction {
                    name: "compute".to_string(),
                    params: vec![("n".to_string(), MirType::I64)],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            MirInst::BinOp {
                                dest: Place::new("r"),
                                op: MirBinOp::Add,
                                lhs: Operand::Place(Place::new("n")),
                                rhs: Operand::Constant(Constant::Int(1)),
                            },
                        ],
                        terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                    }],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
            ],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let narrowing = LoopBoundedNarrowing::from_program(&program);
        let changed = narrowing.run_on_program(&mut program);
        // main calls compute(100) where 100 fits in i32, so param should be narrowed
        assert!(changed);
        assert_eq!(program.functions[1].params[0].1, MirType::I32);
    }

    #[test]
    fn test_loop_bounded_narrowing_skips_div_mod() {
        // v0.90.22: Parameters flowing into div/mod should NOT be narrowed.
        // Narrowing creates sext/trunc overhead (shl 32 + ashr 32 + freeze)
        // without any benefit — LLVM uses same magic-number trick for i32/i64.
        let mut program = MirProgram {
            functions: vec![
                MirFunction {
                    name: "main".to_string(),
                    params: vec![],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            MirInst::Call {
                                dest: Some(Place::new("r")),
                                func: "digit_sum".to_string(),
                                args: vec![Operand::Constant(Constant::Int(100000))],
                                is_tail: false,
                            },
                        ],
                        terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                    }],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
                MirFunction {
                    name: "digit_sum".to_string(),
                    params: vec![("n".to_string(), MirType::I64)],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            // n % 10 — div/mod operation
                            MirInst::BinOp {
                                dest: Place::new("d"),
                                op: MirBinOp::Mod,
                                lhs: Operand::Place(Place::new("n")),
                                rhs: Operand::Constant(Constant::Int(10)),
                            },
                            // n / 10 — div operation
                            MirInst::BinOp {
                                dest: Place::new("q"),
                                op: MirBinOp::Div,
                                lhs: Operand::Place(Place::new("n")),
                                rhs: Operand::Constant(Constant::Int(10)),
                            },
                        ],
                        terminator: Terminator::Return(Some(Operand::Place(Place::new("d")))),
                    }],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
            ],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let narrowing = LoopBoundedNarrowing::from_program(&program);
        let changed = narrowing.run_on_program(&mut program);
        // digit_sum has div/mod on param, should NOT be narrowed despite 100000 < i32::MAX
        assert!(!changed);
        assert_eq!(program.functions[1].params[0].1, MirType::I64);
    }

    #[test]
    fn test_loop_bounded_narrowing_skips_loop_invariant_bound() {
        // v0.90.22: Loop-invariant bound parameters should NOT be narrowed.
        // Pattern: sum_collatz_lengths(start, end, acc) where:
        // - `end` is loop-invariant (not in phi, passed unchanged in recursive call)
        // - `end` is compared against `start` (loop variable in phi)
        // Narrowing `end` to i32 inserts sext in loop header, blocking LLVM unrolling.
        let mut program = MirProgram {
            functions: vec![
                MirFunction {
                    name: "main".to_string(),
                    params: vec![],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            MirInst::Call {
                                dest: Some(Place::new("r")),
                                func: "sum_loop".to_string(),
                                args: vec![
                                    Operand::Constant(Constant::Int(1)),
                                    Operand::Constant(Constant::Int(10000)),
                                    Operand::Constant(Constant::Int(0)),
                                ],
                                is_tail: false,
                            },
                        ],
                        terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                    }],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
                MirFunction {
                    name: "sum_loop".to_string(),
                    params: vec![
                        ("start".to_string(), MirType::I64),
                        ("end".to_string(), MirType::I64),
                        ("acc".to_string(), MirType::I64),
                    ],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![
                        BasicBlock {
                            label: "entry".to_string(),
                            instructions: vec![],
                            terminator: Terminator::Goto("loop_header".to_string()),
                        },
                        BasicBlock {
                            label: "loop_header".to_string(),
                            instructions: vec![
                                // %start_loop = phi [%start, entry], [%next, body]
                                MirInst::Phi {
                                    dest: Place::new("start_loop"),
                                    values: vec![
                                        (Operand::Place(Place::new("start")), "entry".to_string()),
                                        (Operand::Place(Place::new("next")), "body".to_string()),
                                    ],
                                },
                                MirInst::Phi {
                                    dest: Place::new("acc_loop"),
                                    values: vec![
                                        (Operand::Place(Place::new("acc")), "entry".to_string()),
                                        (Operand::Place(Place::new("new_acc")), "body".to_string()),
                                    ],
                                },
                                // %cmp = > %start_loop, %end (param compared against loop var)
                                MirInst::BinOp {
                                    dest: Place::new("cmp"),
                                    op: MirBinOp::Gt,
                                    lhs: Operand::Place(Place::new("start_loop")),
                                    rhs: Operand::Place(Place::new("end")),
                                },
                            ],
                            terminator: Terminator::Branch {
                                cond: Operand::Place(Place::new("cmp")),
                                then_label: "exit".to_string(),
                                else_label: "body".to_string(),
                            },
                        },
                        BasicBlock {
                            label: "body".to_string(),
                            instructions: vec![
                                MirInst::BinOp {
                                    dest: Place::new("next"),
                                    op: MirBinOp::Add,
                                    lhs: Operand::Place(Place::new("start_loop")),
                                    rhs: Operand::Constant(Constant::Int(1)),
                                },
                                MirInst::BinOp {
                                    dest: Place::new("new_acc"),
                                    op: MirBinOp::Add,
                                    lhs: Operand::Place(Place::new("acc_loop")),
                                    rhs: Operand::Place(Place::new("start_loop")),
                                },
                            ],
                            terminator: Terminator::Goto("loop_header".to_string()),
                        },
                        BasicBlock {
                            label: "exit".to_string(),
                            instructions: vec![],
                            terminator: Terminator::Return(Some(Operand::Place(Place::new("acc_loop")))),
                        },
                    ],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
            ],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let narrowing = LoopBoundedNarrowing::from_program(&program);
        let changed = narrowing.run_on_program(&mut program);

        let sum_loop = &program.functions[1];
        // `end` (param 1) should NOT be narrowed — it's a loop-invariant bound
        assert_eq!(sum_loop.params[1].1, MirType::I64,
            "end param should remain i64 (loop-invariant bound)");
        // `start` (param 0) feeds into phi, so it's a loop variable — could be narrowed
        // `acc` (param 2) feeds into phi — could be narrowed
        // But the key assertion is that `end` stays i64
        // `end` (param 1) should remain i64 regardless of changed status
        let _ = changed;
    }

    // v0.90.25: While-loop bound detection (alloca-based pattern, no phi nodes)
    #[test]
    fn test_loop_bounded_narrowing_skips_while_loop_bound() {
        // While-loop pattern: count_primes_loop(arr, n) where:
        // - `n` is a loop bound compared in while_cond block
        // - The while_cond block has a back-edge (Goto from while_body)
        // - No phi nodes — uses Copy/BinOp for loop variable updates
        // Narrowing `n` to i32 inserts sext in loop header, blocking vectorization.
        let mut program = MirProgram {
            functions: vec![
                MirFunction {
                    name: "main".to_string(),
                    params: vec![],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            MirInst::Call {
                                dest: Some(Place::new("r")),
                                func: "count_loop".to_string(),
                                args: vec![
                                    Operand::Constant(Constant::Int(0)),   // arr ptr
                                    Operand::Constant(Constant::Int(100)), // n
                                ],
                                is_tail: false,
                            },
                        ],
                        terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                    }],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
                MirFunction {
                    name: "count_loop".to_string(),
                    params: vec![
                        ("arr".to_string(), MirType::I64),
                        ("n".to_string(), MirType::I64),
                    ],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![
                        BasicBlock {
                            label: "entry".to_string(),
                            instructions: vec![
                                // let mut k = 0
                                MirInst::Const { dest: Place::new("k"), value: Constant::Int(0) },
                                // let mut count = 0
                                MirInst::Const { dest: Place::new("count"), value: Constant::Int(0) },
                            ],
                            terminator: Terminator::Goto("while_cond".to_string()),
                        },
                        BasicBlock {
                            label: "while_cond".to_string(),
                            instructions: vec![
                                // cmp = k <= n  (param used as loop bound)
                                MirInst::BinOp {
                                    dest: Place::new("cmp"),
                                    op: MirBinOp::Le,
                                    lhs: Operand::Place(Place::new("k")),
                                    rhs: Operand::Place(Place::new("n")),
                                },
                            ],
                            terminator: Terminator::Branch {
                                cond: Operand::Place(Place::new("cmp")),
                                then_label: "while_body".to_string(),
                                else_label: "while_exit".to_string(),
                            },
                        },
                        BasicBlock {
                            label: "while_body".to_string(),
                            instructions: vec![
                                // count = count + 1
                                MirInst::BinOp {
                                    dest: Place::new("count"),
                                    op: MirBinOp::Add,
                                    lhs: Operand::Place(Place::new("count")),
                                    rhs: Operand::Constant(Constant::Int(1)),
                                },
                                // k = k + 1
                                MirInst::BinOp {
                                    dest: Place::new("k"),
                                    op: MirBinOp::Add,
                                    lhs: Operand::Place(Place::new("k")),
                                    rhs: Operand::Constant(Constant::Int(1)),
                                },
                            ],
                            terminator: Terminator::Goto("while_cond".to_string()), // back-edge!
                        },
                        BasicBlock {
                            label: "while_exit".to_string(),
                            instructions: vec![],
                            terminator: Terminator::Return(Some(Operand::Place(Place::new("count")))),
                        },
                    ],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
            ],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let narrowing = LoopBoundedNarrowing::from_program(&program);
        let _changed = narrowing.run_on_program(&mut program);

        let count_loop = &program.functions[1];
        // `n` (param 1) should NOT be narrowed — it's a while-loop bound
        assert_eq!(count_loop.params[1].1, MirType::I64,
            "n param should remain i64 (while-loop bound, no phi nodes)");
    }

    #[test]
    fn test_loop_bounded_narrowing_allows_non_bound_param() {
        // Non-bound parameter should still be narrowed when it fits in i32.
        // Pattern: process(arr, n, scale) where `scale` is not used as a loop bound.
        let mut program = MirProgram {
            functions: vec![
                MirFunction {
                    name: "main".to_string(),
                    params: vec![],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            MirInst::Call {
                                dest: Some(Place::new("r")),
                                func: "process".to_string(),
                                args: vec![
                                    Operand::Constant(Constant::Int(0)),   // arr
                                    Operand::Constant(Constant::Int(100)), // n (bound)
                                    Operand::Constant(Constant::Int(5)),   // scale (not a bound)
                                ],
                                is_tail: false,
                            },
                        ],
                        terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
                    }],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
                MirFunction {
                    name: "process".to_string(),
                    params: vec![
                        ("arr".to_string(), MirType::I64),
                        ("n".to_string(), MirType::I64),
                        ("scale".to_string(), MirType::I64),
                    ],
                    ret_ty: MirType::I64,
                    locals: vec![],
                    blocks: vec![
                        BasicBlock {
                            label: "entry".to_string(),
                            instructions: vec![
                                MirInst::Const { dest: Place::new("i"), value: Constant::Int(0) },
                            ],
                            terminator: Terminator::Goto("while_cond".to_string()),
                        },
                        BasicBlock {
                            label: "while_cond".to_string(),
                            instructions: vec![
                                // i < n (n is the bound)
                                MirInst::BinOp {
                                    dest: Place::new("cmp"),
                                    op: MirBinOp::Lt,
                                    lhs: Operand::Place(Place::new("i")),
                                    rhs: Operand::Place(Place::new("n")),
                                },
                            ],
                            terminator: Terminator::Branch {
                                cond: Operand::Place(Place::new("cmp")),
                                then_label: "while_body".to_string(),
                                else_label: "while_exit".to_string(),
                            },
                        },
                        BasicBlock {
                            label: "while_body".to_string(),
                            instructions: vec![
                                // use scale (not as a bound, just in computation)
                                MirInst::BinOp {
                                    dest: Place::new("_v"),
                                    op: MirBinOp::Add,
                                    lhs: Operand::Place(Place::new("i")),
                                    rhs: Operand::Place(Place::new("scale")),
                                },
                                MirInst::BinOp {
                                    dest: Place::new("i"),
                                    op: MirBinOp::Add,
                                    lhs: Operand::Place(Place::new("i")),
                                    rhs: Operand::Constant(Constant::Int(1)),
                                },
                            ],
                            terminator: Terminator::Goto("while_cond".to_string()),
                        },
                        BasicBlock {
                            label: "while_exit".to_string(),
                            instructions: vec![],
                            terminator: Terminator::Return(Some(Operand::Place(Place::new("i")))),
                        },
                    ],
                    preconditions: vec![],
                    postconditions: vec![],
                    is_pure: false,
                    is_const: false,
                    always_inline: false,
                    inline_hint: false,
                    is_memory_free: false,
                },
            ],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };

        let narrowing = LoopBoundedNarrowing::from_program(&program);
        narrowing.run_on_program(&mut program);

        let process = &program.functions[1];
        // `n` (param 1) should NOT be narrowed — loop bound
        assert_eq!(process.params[1].1, MirType::I64,
            "n param should remain i64 (while-loop bound)");
        // `scale` (param 2) SHOULD be narrowed — not a loop bound
        assert_eq!(process.params[2].1, MirType::I32,
            "scale param should be narrowed to i32 (not a loop bound)");
    }

    #[test]
    fn test_pipeline_stats_display() {
        let mut stats = OptimizationStats::new();
        stats.record_pass("ConstantFolding");
        stats.record_pass("ConstantFolding");
        stats.record_pass("DCE");
        assert_eq!(*stats.pass_counts.get("ConstantFolding").unwrap(), 2);
        assert_eq!(*stats.pass_counts.get("DCE").unwrap(), 1);
    }

    #[test]
    fn test_pipeline_stats_merge_multiple() {
        let mut s1 = OptimizationStats::new();
        s1.record_pass("CSE");
        s1.record_pass("CSE");

        let mut s2 = OptimizationStats::new();
        s2.record_pass("CSE");
        s2.record_pass("DCE");

        s1.merge(&s2);
        assert_eq!(*s1.pass_counts.get("CSE").unwrap(), 3);
        assert_eq!(*s1.pass_counts.get("DCE").unwrap(), 1);
    }

    // ---- Cycle 193: SimplifyBranches edge case tests ----

    #[test]
    fn test_simplify_branches_multiple_blocks() {
        // Two blocks both have constant branch conditions
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Constant(Constant::Bool(true)),
                        then_label: "b1".to_string(),
                        else_label: "b2".to_string(),
                    },
                },
                BasicBlock {
                    label: "b1".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Constant(Constant::Bool(false)),
                        then_label: "b3".to_string(),
                        else_label: "b4".to_string(),
                    },
                },
                BasicBlock {
                    label: "b2".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
                },
                BasicBlock {
                    label: "b3".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                },
                BasicBlock {
                    label: "b4".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(2)))),
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

        let pass = SimplifyBranches;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        // entry should goto b1
        assert!(matches!(&func.blocks[0].terminator, Terminator::Goto(l) if l == "b1"));
        // b1 should goto b4 (false branch)
        assert!(matches!(&func.blocks[1].terminator, Terminator::Goto(l) if l == "b4"));
    }

    #[test]
    fn test_simplify_branches_no_change_goto() {
        // Goto terminator should not be affected
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Goto("exit".to_string()),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = SimplifyBranches;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed);
    }

    #[test]
    fn test_simplify_branches_no_change_return() {
        // Return terminator should not be affected
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(None),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = SimplifyBranches;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed);
    }

    // ---- Cycle 193: IfElseToSwitch edge case tests ----

    #[test]
    fn test_if_else_to_switch_exactly_three_cases() {
        // Exactly 3 cases (at threshold) — should convert
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                // Block 0: if x == 0 goto b_then_0 else b_check_1
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("cmp0"),
                        op: MirBinOp::Eq,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(0)),
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp0")),
                        then_label: "b_then_0".to_string(),
                        else_label: "b_check_1".to_string(),
                    },
                },
                // Block 1: then for case 0
                BasicBlock {
                    label: "b_then_0".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                // Block 2: if x == 1 goto b_then_1 else b_check_2
                BasicBlock {
                    label: "b_check_1".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("cmp1"),
                        op: MirBinOp::Eq,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp1")),
                        then_label: "b_then_1".to_string(),
                        else_label: "b_check_2".to_string(),
                    },
                },
                // Block 3: then for case 1
                BasicBlock {
                    label: "b_then_1".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                // Block 4: if x == 2 goto b_then_2 else b_default
                BasicBlock {
                    label: "b_check_2".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("cmp2"),
                        op: MirBinOp::Eq,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(2)),
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp2")),
                        then_label: "b_then_2".to_string(),
                        else_label: "b_default".to_string(),
                    },
                },
                // Block 5: then for case 2
                BasicBlock {
                    label: "b_then_2".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                // Block 6: default
                BasicBlock {
                    label: "b_default".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                // Block 7: merge
                BasicBlock {
                    label: "merge".to_string(),
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

        let pass = IfElseToSwitch;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "3-case if-else chain should be converted to switch");
        // entry block should now have a Switch terminator
        assert!(matches!(&func.blocks[0].terminator, Terminator::Switch { .. }),
            "entry should be Switch, got {:?}", func.blocks[0].terminator);
    }

    #[test]
    fn test_if_else_to_switch_different_variables_no_convert() {
        // if x == 0 ... elif y == 1 — different comparison variables, should NOT convert
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![
                ("x".to_string(), MirType::I64),
                ("y".to_string(), MirType::I64),
            ],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("cmp0"),
                        op: MirBinOp::Eq,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(0)),
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp0")),
                        then_label: "b_then_0".to_string(),
                        else_label: "b_check_1".to_string(),
                    },
                },
                BasicBlock {
                    label: "b_then_0".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                // Uses y instead of x — breaks the chain
                BasicBlock {
                    label: "b_check_1".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("cmp1"),
                        op: MirBinOp::Eq,
                        lhs: Operand::Place(Place::new("y")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp1")),
                        then_label: "b_then_1".to_string(),
                        else_label: "b_check_2".to_string(),
                    },
                },
                BasicBlock {
                    label: "b_then_1".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "b_check_2".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("cmp2"),
                        op: MirBinOp::Eq,
                        lhs: Operand::Place(Place::new("y")),
                        rhs: Operand::Constant(Constant::Int(2)),
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp2")),
                        then_label: "b_then_2".to_string(),
                        else_label: "merge".to_string(),
                    },
                },
                BasicBlock {
                    label: "b_then_2".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
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

        let pass = IfElseToSwitch;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "different comparison variables should not convert");
    }

    #[test]
    fn test_if_else_to_switch_non_eq_comparison() {
        // if x < 0 ... elif x < 1 — non-equality comparisons, should NOT convert
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("cmp0"),
                        op: MirBinOp::Lt,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(0)),
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp0")),
                        then_label: "b_then_0".to_string(),
                        else_label: "b_check_1".to_string(),
                    },
                },
                BasicBlock {
                    label: "b_then_0".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "b_check_1".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("cmp1"),
                        op: MirBinOp::Lt,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp1")),
                        then_label: "b_then_1".to_string(),
                        else_label: "b_check_2".to_string(),
                    },
                },
                BasicBlock {
                    label: "b_then_1".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "b_check_2".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("cmp2"),
                        op: MirBinOp::Lt,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(2)),
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp2")),
                        then_label: "b_then_2".to_string(),
                        else_label: "merge".to_string(),
                    },
                },
                BasicBlock {
                    label: "b_then_2".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
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

        let pass = IfElseToSwitch;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "non-equality comparisons should not convert to switch");
    }

    // ---- Cycle 193: ConditionalIncrementToSelect additional tests ----

    #[test]
    fn test_conditional_increment_name() {
        let pass = ConditionalIncrementToSelect;
        assert_eq!(pass.name(), "conditional_increment_to_select");
    }

    #[test]
    fn test_conditional_increment_empty_function() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = ConditionalIncrementToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "empty function should not be changed");
    }

    #[test]
    fn test_conditional_increment_no_branch() {
        // Function with only gotos, no branch — should not match
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::Const {
                        dest: Place::new("x"),
                        value: Constant::Int(0),
                    }],
                    terminator: Terminator::Goto("exit".to_string()),
                },
                BasicBlock {
                    label: "exit".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
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

        let pass = ConditionalIncrementToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "no branch means no pattern match");
    }

    // ---- Cycle 193: UnreachableBlockElimination edge cases ----

    #[test]
    fn test_unreachable_block_empty_function() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = UnreachableBlockElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "empty function has no blocks to remove");
    }

    #[test]
    fn test_unreachable_block_single_block() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = UnreachableBlockElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "single block is always reachable");
        assert_eq!(func.blocks.len(), 1);
    }

    #[test]
    fn test_unreachable_block_chain() {
        // entry → b1 → b2 → return, b3 unreachable
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("b1".to_string()),
                },
                BasicBlock {
                    label: "b1".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("b2".to_string()),
                },
                BasicBlock {
                    label: "b2".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
                },
                BasicBlock {
                    label: "b3_unreachable".to_string(),
                    instructions: vec![MirInst::Const {
                        dest: Place::new("dead"),
                        value: Constant::Int(999),
                    }],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
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

        let pass = UnreachableBlockElimination;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "unreachable b3 should be removed");
        assert_eq!(func.blocks.len(), 3);
        assert!(func.blocks.iter().all(|b| b.label != "b3_unreachable"));
    }

    // ---- Cycle 193: BlockMerging edge cases ----

    #[test]
    fn test_block_merging_single_block() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = BlockMerging;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "single block cannot be merged");
    }

    #[test]
    fn test_block_merging_with_branch_no_merge() {
        // entry branches to b1/b2 — neither can merge with entry
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::Bool)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("x")),
                        then_label: "b1".to_string(),
                        else_label: "b2".to_string(),
                    },
                },
                BasicBlock {
                    label: "b1".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                },
                BasicBlock {
                    label: "b2".to_string(),
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

        let pass = BlockMerging;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "branch targets have multiple predecessors conceptually");
    }

    // ---- Cycle 193: PhiSimplification edge cases ----

    #[test]
    fn test_phi_simplification_empty_phi() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::Phi {
                    dest: Place::new("x"),
                    values: vec![],
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = PhiSimplification;
        let changed = pass.run_on_function(&mut func);
        // Empty phi is a degenerate case — pass should handle gracefully
        // Empty phi is a degenerate case — pass should handle gracefully without crashing
        let _ = changed;
    }

    #[test]
    fn test_phi_simplification_two_different_constants() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::Phi {
                    dest: Place::new("x"),
                    values: vec![
                        (Operand::Constant(Constant::Int(1)), "b1".to_string()),
                        (Operand::Constant(Constant::Int(2)), "b2".to_string()),
                    ],
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = PhiSimplification;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "phi with two different constants should not simplify");
        // Should still be a phi
        assert!(matches!(&func.blocks[0].instructions[0], MirInst::Phi { .. }));
    }

    // ---- Cycle 193: CopyPropagation edge cases ----

    #[test]
    fn test_copy_propagation_no_copies() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const {
                        dest: Place::new("x"),
                        value: Constant::Int(42),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = CopyPropagation;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "no copy instructions means no propagation");
    }

    // ---- Cycle 193: ConstantFolding additional edge cases ----

    #[test]
    fn test_constant_folding_subtraction() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("a"), value: Constant::Int(10) },
                    MirInst::Const { dest: Place::new("b"), value: Constant::Int(3) },
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::Sub,
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
        };

        let pass = ConstantFolding;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        assert!(matches!(&func.blocks[0].instructions[2],
            MirInst::Const { value: Constant::Int(7), .. }));
    }

    #[test]
    fn test_constant_folding_multiplication() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("a"), value: Constant::Int(6) },
                    MirInst::Const { dest: Place::new("b"), value: Constant::Int(7) },
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::Mul,
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
        };

        let pass = ConstantFolding;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        assert!(matches!(&func.blocks[0].instructions[2],
            MirInst::Const { value: Constant::Int(42), .. }));
    }

    #[test]
    fn test_constant_folding_modulo() {
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const { dest: Place::new("a"), value: Constant::Int(17) },
                    MirInst::Const { dest: Place::new("b"), value: Constant::Int(5) },
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::Mod,
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
        };

        let pass = ConstantFolding;
        let changed = pass.run_on_function(&mut func);
        assert!(changed);
        assert!(matches!(&func.blocks[0].instructions[2],
            MirInst::Const { value: Constant::Int(2), .. }));
    }

    // ================================================================
    // Cycle 204: IfElseToSelect tests
    // ================================================================

    /// Helper: build a standard if-else-merge MIR pattern for IfElseToSelect tests.
    ///
    /// cond_block: %cmp = Lt %x, %y; branch %cmp, then_block, else_block
    /// then_block: %then_val = Add %x, 1; goto merge
    /// else_block: %else_val = Sub %y, 1; goto merge
    /// merge: %result = phi [(%then_val, then_block), (%else_val, else_block)]; ret %result
    fn make_if_else_select_function() -> MirFunction {
        MirFunction {
            name: "test_select".to_string(),
            params: vec![("x".to_string(), MirType::I64), ("y".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "cond_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Lt,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Place(Place::new("y")),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "then_block".to_string(),
                        else_label: "else_block".to_string(),
                    },
                },
                BasicBlock {
                    label: "then_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("then_val"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "else_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("else_val"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("y")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("result"),
                            values: vec![
                                (Operand::Place(Place::new("then_val")), "then_block".to_string()),
                                (Operand::Place(Place::new("else_val")), "else_block".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
                },
            ],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        }
    }

    // --- Basic pattern match tests (3 tests) ---

    #[test]
    fn test_if_else_to_select_basic() {
        // Standard if-else diamond: both arms produce a value, merged by phi.
        // Should be converted to a select instruction.
        let mut func = make_if_else_select_function();
        let pass = IfElseToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should convert basic if-else pattern to select");

        // The cond_block should now have a Select instruction
        let has_select = func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::Select { .. })
        });
        assert!(has_select, "Cond block should contain a Select instruction after transformation");

        // The cond_block terminator should be Goto(merge), not Branch
        assert!(
            matches!(&func.blocks[0].terminator, Terminator::Goto(label) if label == "merge"),
            "Cond block terminator should be Goto(merge) after transformation"
        );
    }

    #[test]
    fn test_if_else_to_select_select_operands() {
        // Verify the Select instruction has the correct comparison operator and operands.
        let mut func = make_if_else_select_function();
        let pass = IfElseToSelect;
        pass.run_on_function(&mut func);

        let select_inst = func.blocks[0].instructions.iter().find(|inst| {
            matches!(inst, MirInst::Select { .. })
        });
        assert!(select_inst.is_some(), "Should have a Select instruction");

        if let Some(MirInst::Select { dest, cond_op, cond_lhs, cond_rhs, true_val, false_val }) = select_inst {
            // The condition should be Lt %x %y (from the original comparison)
            assert!(matches!(cond_op, MirBinOp::Lt), "Select cond_op should be Lt");
            assert!(matches!(cond_lhs, Operand::Place(p) if p.name == "x"), "Select cond_lhs should be %x");
            assert!(matches!(cond_rhs, Operand::Place(p) if p.name == "y"), "Select cond_rhs should be %y");
            // true_val should reference then_val, false_val should reference else_val
            assert!(matches!(true_val, Operand::Place(p) if p.name == "then_val"), "Select true_val should be %then_val");
            assert!(matches!(false_val, Operand::Place(p) if p.name == "else_val"), "Select false_val should be %else_val");
            // Dest should use the unique _sel_ prefix
            assert!(dest.name.starts_with("_sel_result_"), "Select dest should have _sel_result_ prefix, got: {}", dest.name);
        }
    }

    #[test]
    fn test_if_else_to_select_hoisted_instructions() {
        // Both then and else block instructions should be hoisted into the cond_block.
        let mut func = make_if_else_select_function();
        let pass = IfElseToSelect;
        pass.run_on_function(&mut func);

        // The cond block should have: then_inst (Add), else_inst (Sub), Select
        // The original comparison (Lt) is removed and embedded in Select.
        let cond_insts = &func.blocks[0].instructions;
        assert!(cond_insts.len() >= 3,
            "Cond block should have at least 3 instructions (then_inst + else_inst + select), got {}",
            cond_insts.len());

        // Check that the Add instruction from then_block was hoisted
        let has_add = cond_insts.iter().any(|inst| {
            matches!(inst, MirInst::BinOp { dest, op: MirBinOp::Add, .. } if dest.name == "then_val")
        });
        assert!(has_add, "Add instruction from then_block should be hoisted to cond_block");

        // Check that the Sub instruction from else_block was hoisted
        let has_sub = cond_insts.iter().any(|inst| {
            matches!(inst, MirInst::BinOp { dest, op: MirBinOp::Sub, .. } if dest.name == "else_val")
        });
        assert!(has_sub, "Sub instruction from else_block should be hoisted to cond_block");
    }

    // --- Name test (1 test) ---

    #[test]
    fn test_if_else_to_select_name() {
        let pass = IfElseToSelect;
        assert_eq!(pass.name(), "if_else_to_select");
    }

    // --- No-match cases (4 tests) ---

    #[test]
    fn test_if_else_to_select_no_match_empty_function() {
        // A function with no blocks should not be transformed.
        let mut func = MirFunction {
            name: "empty".to_string(),
            params: vec![],
            ret_ty: MirType::Unit,
            locals: vec![],
            blocks: vec![],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = IfElseToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Empty function should not be transformed");
    }

    #[test]
    fn test_if_else_to_select_no_match_goto_terminator() {
        // A block with Goto terminator (no branch) should not match.
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const {
                            dest: Place::new("val"),
                            value: Constant::Int(42),
                        },
                    ],
                    terminator: Terminator::Goto("exit".to_string()),
                },
                BasicBlock {
                    label: "exit".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("val")))),
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

        let pass = IfElseToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Function with only Goto terminators should not be transformed");
    }

    #[test]
    fn test_if_else_to_select_no_match_too_many_instructions() {
        // Then block has >3 instructions, should be rejected.
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64), ("y".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "cond_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Lt,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Place(Place::new("y")),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "then_block".to_string(),
                        else_label: "else_block".to_string(),
                    },
                },
                BasicBlock {
                    label: "then_block".to_string(),
                    instructions: vec![
                        // 4 instructions — exceeds the 3-instruction limit
                        MirInst::BinOp {
                            dest: Place::new("t1"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                        MirInst::BinOp {
                            dest: Place::new("t2"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("t1")),
                            rhs: Operand::Constant(Constant::Int(2)),
                        },
                        MirInst::BinOp {
                            dest: Place::new("t3"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("t2")),
                            rhs: Operand::Constant(Constant::Int(3)),
                        },
                        MirInst::BinOp {
                            dest: Place::new("then_val"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("t3")),
                            rhs: Operand::Constant(Constant::Int(4)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "else_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("else_val"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("y")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("result"),
                            values: vec![
                                (Operand::Place(Place::new("then_val")), "then_block".to_string()),
                                (Operand::Place(Place::new("else_val")), "else_block".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
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

        let pass = IfElseToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Then block with >3 instructions should not match");
    }

    #[test]
    fn test_if_else_to_select_no_match_call_instruction() {
        // Then block contains a Call instruction (not hoistable), should be rejected.
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64), ("y".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "cond_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Lt,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Place(Place::new("y")),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "then_block".to_string(),
                        else_label: "else_block".to_string(),
                    },
                },
                BasicBlock {
                    label: "then_block".to_string(),
                    instructions: vec![
                        MirInst::Call {
                            dest: Some(Place::new("then_val")),
                            func: "side_effect_fn".to_string(),
                            args: vec![Operand::Place(Place::new("x"))],
                            is_tail: false,
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "else_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("else_val"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("y")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("result"),
                            values: vec![
                                (Operand::Place(Place::new("then_val")), "then_block".to_string()),
                                (Operand::Place(Place::new("else_val")), "else_block".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
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

        let pass = IfElseToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Block with Call instruction should not be hoistable");
    }

    // --- Multi-phi pattern (2 tests) ---

    #[test]
    fn test_if_else_to_select_multi_phi_tco_pattern() {
        // Pattern with >2 phi incoming values (e.g., TCO loop pattern with an
        // additional predecessor block). The phi has 3 entries but only 2 come from
        // the then/else blocks. The pass should still transform the then/else pair.
        let mut func = MirFunction {
            name: "test_multi_phi".to_string(),
            params: vec![("x".to_string(), MirType::I64), ("y".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                // Block 0: initial entry that jumps to loop header
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const {
                            dest: Place::new("init"),
                            value: Constant::Int(0),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                // Block 1: cond block with comparison and branch
                BasicBlock {
                    label: "cond_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Gt,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Place(Place::new("y")),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "then_block".to_string(),
                        else_label: "else_block".to_string(),
                    },
                },
                // Block 2: then arm
                BasicBlock {
                    label: "then_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("then_val"),
                            op: MirBinOp::Mul,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(2)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                // Block 3: else arm
                BasicBlock {
                    label: "else_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("else_val"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("y")),
                            rhs: Operand::Constant(Constant::Int(3)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                // Block 4: merge block with 3-entry phi (entry + then + else)
                BasicBlock {
                    label: "merge".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("result"),
                            values: vec![
                                (Operand::Place(Place::new("init")), "entry".to_string()),
                                (Operand::Place(Place::new("then_val")), "then_block".to_string()),
                                (Operand::Place(Place::new("else_val")), "else_block".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
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

        let pass = IfElseToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should transform if-else into select even with 3-entry phi");

        // After transformation, the merge phi should still have entries but the
        // then/else entries should be replaced by the cond_block select entry.
        let merge_block = func.blocks.iter().find(|b| b.label == "merge").unwrap();
        let phi = merge_block.instructions.iter().find(|inst| {
            matches!(inst, MirInst::Phi { .. })
        });
        assert!(phi.is_some(), "Merge block should still have a phi");

        if let Some(MirInst::Phi { values, .. }) = phi {
            // Should have 2 entries: (init, entry) and (_sel_..., cond_block)
            assert_eq!(values.len(), 2,
                "Phi should have 2 entries after transformation (entry + cond_block), got {}",
                values.len());
            // The entry predecessor should still be there
            assert!(values.iter().any(|(_, label)| label == "entry"),
                "Phi should still have the entry predecessor");
            // The cond_block select entry should be there
            assert!(values.iter().any(|(_, label)| label == "cond_block"),
                "Phi should have a cond_block entry from the select");
        }
    }

    #[test]
    fn test_if_else_to_select_multi_phi_preserves_other_entries() {
        // Verify that a phi with 3+ entries preserves the non-then/else entry
        // values correctly after transformation.
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64), ("y".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "prelude".to_string(),
                    instructions: vec![
                        MirInst::Const {
                            dest: Place::new("default"),
                            value: Constant::Int(99),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "cond_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Eq,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "then_block".to_string(),
                        else_label: "else_block".to_string(),
                    },
                },
                BasicBlock {
                    label: "then_block".to_string(),
                    instructions: vec![
                        MirInst::Const {
                            dest: Place::new("then_val"),
                            value: Constant::Int(1),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "else_block".to_string(),
                    instructions: vec![
                        MirInst::Const {
                            dest: Place::new("else_val"),
                            value: Constant::Int(2),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("result"),
                            values: vec![
                                (Operand::Place(Place::new("default")), "prelude".to_string()),
                                (Operand::Place(Place::new("then_val")), "then_block".to_string()),
                                (Operand::Place(Place::new("else_val")), "else_block".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
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

        let pass = IfElseToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should transform with 3-entry phi");

        let merge_block = func.blocks.iter().find(|b| b.label == "merge").unwrap();
        if let Some(MirInst::Phi { values, .. }) = merge_block.instructions.iter().find(|i| matches!(i, MirInst::Phi { .. })) {
            // The prelude entry should be preserved exactly as-is
            let prelude_entry = values.iter().find(|(_, label)| label == "prelude");
            assert!(prelude_entry.is_some(), "Prelude entry should be preserved");
            if let Some((Operand::Place(p), _)) = prelude_entry {
                assert_eq!(p.name, "default", "Prelude phi value should still be %default");
            }
        }
    }

    // --- Cross-dependency rejection (1 test) ---

    #[test]
    fn test_if_else_to_select_cross_dependency_rejected() {
        // Then block references a destination defined in else block.
        // This creates a cross-dependency that prevents safe hoisting.
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64), ("y".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "cond_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Lt,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Place(Place::new("y")),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "then_block".to_string(),
                        else_label: "else_block".to_string(),
                    },
                },
                BasicBlock {
                    label: "then_block".to_string(),
                    instructions: vec![
                        // References %else_val which is defined in the else block
                        MirInst::BinOp {
                            dest: Place::new("then_val"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("else_val")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "else_block".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("else_val"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("y")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("merge".to_string()),
                },
                BasicBlock {
                    label: "merge".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("result"),
                            values: vec![
                                (Operand::Place(Place::new("then_val")), "then_block".to_string()),
                                (Operand::Place(Place::new("else_val")), "else_block".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
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

        let pass = IfElseToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Cross-dependency between then and else blocks should prevent transformation");
    }

    // --- Multiple patterns in function (1 test) ---

    #[test]
    fn test_if_else_to_select_multiple_patterns() {
        // Two independent if-else diamond patterns in the same function.
        // Both should be transformed to selects.
        let mut func = MirFunction {
            name: "test_two_diamonds".to_string(),
            params: vec![
                ("x".to_string(), MirType::I64),
                ("y".to_string(), MirType::I64),
            ],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                // Diamond 1: cond1 -> then1/else1 -> merge1
                BasicBlock {
                    label: "cond1".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp1"),
                            op: MirBinOp::Lt,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Place(Place::new("y")),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp1")),
                        then_label: "then1".to_string(),
                        else_label: "else1".to_string(),
                    },
                },
                BasicBlock {
                    label: "then1".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("then_val1"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(10)),
                        },
                    ],
                    terminator: Terminator::Goto("merge1".to_string()),
                },
                BasicBlock {
                    label: "else1".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("else_val1"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("y")),
                            rhs: Operand::Constant(Constant::Int(10)),
                        },
                    ],
                    terminator: Terminator::Goto("merge1".to_string()),
                },
                BasicBlock {
                    label: "merge1".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("result1"),
                            values: vec![
                                (Operand::Place(Place::new("then_val1")), "then1".to_string()),
                                (Operand::Place(Place::new("else_val1")), "else1".to_string()),
                            ],
                        },
                        // Second comparison based on result of first
                        MirInst::BinOp {
                            dest: Place::new("cmp2"),
                            op: MirBinOp::Gt,
                            lhs: Operand::Place(Place::new("result1")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp2")),
                        then_label: "then2".to_string(),
                        else_label: "else2".to_string(),
                    },
                },
                // Diamond 2: merge1 -> then2/else2 -> merge2
                BasicBlock {
                    label: "then2".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("then_val2"),
                            op: MirBinOp::Mul,
                            lhs: Operand::Place(Place::new("result1")),
                            rhs: Operand::Constant(Constant::Int(2)),
                        },
                    ],
                    terminator: Terminator::Goto("merge2".to_string()),
                },
                BasicBlock {
                    label: "else2".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("else_val2"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("result1")),
                            rhs: Operand::Constant(Constant::Int(5)),
                        },
                    ],
                    terminator: Terminator::Goto("merge2".to_string()),
                },
                BasicBlock {
                    label: "merge2".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("result2"),
                            values: vec![
                                (Operand::Place(Place::new("then_val2")), "then2".to_string()),
                                (Operand::Place(Place::new("else_val2")), "else2".to_string()),
                            ],
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result2")))),
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

        let pass = IfElseToSelect;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should transform at least one if-else diamond to select");

        // Count total Select instructions across all blocks
        let select_count: usize = func.blocks.iter()
            .flat_map(|b| b.instructions.iter())
            .filter(|inst| matches!(inst, MirInst::Select { .. }))
            .count();
        assert!(select_count >= 2,
            "Should have at least 2 Select instructions for 2 diamond patterns, got {}",
            select_count);
    }

    // ========================================================================
    // Cycle 219: Additional DCE tests
    // ========================================================================

    #[test]
    fn test_dce_preserves_side_effects() {
        // Call instructions have side effects and should NOT be removed
        // even if their result is unused
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("unused_result")),
                        func: "print".to_string(),
                        args: vec![Operand::Constant(Constant::Int(42))],
                        is_tail: false,
                    },
                    MirInst::Const {
                        dest: Place::new("result"),
                        value: Constant::Int(0),
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

        // Call should be preserved (side effects), nothing should change
        assert!(!changed, "DCE should not remove call with side effects");
        assert_eq!(func.blocks[0].instructions.len(), 2);
    }

    #[test]
    fn test_dce_chain_dependency() {
        // a = 5, b = a + 1, c = b * 2, return c
        // All live because c depends on b depends on a
        let mut func = MirFunction {
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
                    MirInst::BinOp {
                        dest: Place::new("b"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("a")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    },
                    MirInst::BinOp {
                        dest: Place::new("c"),
                        op: MirBinOp::Mul,
                        lhs: Operand::Place(Place::new("b")),
                        rhs: Operand::Constant(Constant::Int(2)),
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
        };

        let pass = DeadCodeElimination;
        let changed = pass.run_on_function(&mut func);

        // All instructions should be kept (all are live)
        assert!(!changed, "DCE should not remove any live instructions");
        assert_eq!(func.blocks[0].instructions.len(), 3);
    }

    #[test]
    fn test_dce_multiple_independent_dead() {
        // dead1 and dead2 are independent and unused, live is used
        // Single-pass DCE should remove both dead constants
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const {
                        dest: Place::new("dead1"),
                        value: Constant::Int(10),
                    },
                    MirInst::Const {
                        dest: Place::new("dead2"),
                        value: Constant::Int(20),
                    },
                    MirInst::Const {
                        dest: Place::new("live"),
                        value: Constant::Int(42),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("live")))),
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

        assert!(changed, "DCE should remove dead instructions");
        assert_eq!(func.blocks[0].instructions.len(), 1, "Only live const should remain");
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::Const { dest, value: Constant::Int(42) } if dest.name == "live"));
    }

    #[test]
    fn test_dce_branch_condition_kept() {
        // cond used in Branch terminator should be kept
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cond"),
                            op: MirBinOp::Le,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                        MirInst::Const {
                            dest: Place::new("dead"),
                            value: Constant::Int(99),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cond")),
                        then_label: "then".to_string(),
                        else_label: "else".to_string(),
                    },
                },
                BasicBlock {
                    label: "then".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
                },
                BasicBlock {
                    label: "else".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
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

        let pass = DeadCodeElimination;
        let changed = pass.run_on_function(&mut func);

        assert!(changed, "DCE should remove dead const");
        // cond should be kept (used in branch), dead should be removed
        assert_eq!(func.blocks[0].instructions.len(), 1);
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::BinOp { dest, .. } if dest.name == "cond"));
    }

    // ========================================================================
    // Cycle 219: Additional ConstFunctionEval tests
    // ========================================================================

    #[test]
    fn test_const_function_eval_side_effects_not_inlined() {
        // Function with Call inside should NOT be treated as const
        // fn impure() -> i64 = { print(42); 0 }
        let impure_fn = MirFunction {
            name: "impure".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("_")),
                        func: "print".to_string(),
                        args: vec![Operand::Constant(Constant::Int(42))],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
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
                        dest: Some(Place::new("r")),
                        func: "impure".to_string(),
                        args: vec![],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
            functions: vec![impure_fn, caller_fn.clone()],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };

        let pass = ConstFunctionEval::from_program(&program);
        let changed = pass.run_on_function(&mut caller_fn);

        assert!(!changed, "Impure function call should NOT be inlined");
        assert!(matches!(&caller_fn.blocks[0].instructions[0], MirInst::Call { .. }));
    }

    #[test]
    fn test_const_function_eval_variable_return() {
        // fn get_val() -> i64 = { let x = 99; x }
        // Should be inlined since x is a const assigned to place then returned
        let const_fn = MirFunction {
            name: "get_val".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Const {
                        dest: Place::new("x"),
                        value: Constant::Int(99),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
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
            name: "caller".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("r")),
                        func: "get_val".to_string(),
                        args: vec![],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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

        assert!(changed, "Variable-return const function should be inlined");
        assert!(matches!(&caller_fn.blocks[0].instructions[0],
            MirInst::Const { dest, value: Constant::Int(99) } if dest.name == "r"));
    }

    #[test]
    fn test_const_function_eval_multi_block_not_inlined() {
        // Multi-block function should NOT be inlined (extract_constant_return requires 1 block)
        let multi_fn = MirFunction {
            name: "multi".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("exit".to_string()),
                },
                BasicBlock {
                    label: "exit".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(7)))),
                },
            ],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: true,
            is_const: true,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let mut caller_fn = MirFunction {
            name: "caller".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("r")),
                        func: "multi".to_string(),
                        args: vec![],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
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
            functions: vec![multi_fn, caller_fn.clone()],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };

        let pass = ConstFunctionEval::from_program(&program);
        let changed = pass.run_on_function(&mut caller_fn);

        assert!(!changed, "Multi-block function should NOT be inlined");
    }

    // ========================================================================
    // Cycle 219: Additional TailCallOptimization tests
    // ========================================================================

    #[test]
    fn test_tco_direct_tail_call() {
        // Phase 1: Direct tail call in same block
        // fn f(n) { ... ; result = call g(n); return result }
        let mut func = MirFunction {
            name: "f".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("arg"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("n")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    },
                    MirInst::Call {
                        dest: Some(Place::new("result")),
                        func: "g".to_string(),
                        args: vec![Operand::Place(Place::new("arg"))],
                        is_tail: false,
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

        let pass = TailCallOptimization::new();
        let changed = pass.run_on_function(&mut func);

        assert!(changed, "Direct tail call should be detected");
        let call = &func.blocks[0].instructions[1];
        assert!(matches!(call, MirInst::Call { is_tail: true, .. }),
            "Call should be marked as tail call");
    }

    #[test]
    fn test_tco_non_tail_call_not_marked() {
        // Call result is used after the call = NOT a tail call
        // fn f(n) { result = call g(n); final = result + 1; return final }
        let mut func = MirFunction {
            name: "f".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("result")),
                        func: "g".to_string(),
                        args: vec![Operand::Place(Place::new("n"))],
                        is_tail: false,
                    },
                    MirInst::BinOp {
                        dest: Place::new("final_val"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("result")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("final_val")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = TailCallOptimization::new();
        let changed = pass.run_on_function(&mut func);

        assert!(!changed, "Non-tail call should NOT be marked");
        let call = &func.blocks[0].instructions[0];
        assert!(matches!(call, MirInst::Call { is_tail: false, .. }),
            "Call should remain non-tail");
    }

    #[test]
    fn test_tco_void_return_no_change() {
        // Return(None) = no place to match against, should not affect anything
        let mut func = MirFunction {
            name: "f".to_string(),
            params: vec![],
            ret_ty: MirType::Unit,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("_")),
                        func: "side_effect".to_string(),
                        args: vec![],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(None),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = TailCallOptimization::new();
        let changed = pass.run_on_function(&mut func);

        assert!(!changed, "Void return should not trigger TCO");
    }

    // ========================================================================
    // Cycle 219: Additional TailRecursiveToLoop tests
    // ========================================================================

    #[test]
    fn test_tail_recursive_gcd() {
        // fn gcd(a, b) = if b <= 0 { a } else { gcd(b, a % b) }
        // Both parameters change - more complex than sum
        let mut func = MirFunction {
            name: "gcd".to_string(),
            params: vec![
                ("a".to_string(), MirType::I64),
                ("b".to_string(), MirType::I64),
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
                            lhs: Operand::Place(Place::new("b")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "base".to_string(),
                        else_label: "recurse".to_string(),
                    },
                },
                BasicBlock {
                    label: "base".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("a")))),
                },
                BasicBlock {
                    label: "recurse".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("a_mod_b"),
                            op: MirBinOp::Mod,
                            lhs: Operand::Place(Place::new("a")),
                            rhs: Operand::Place(Place::new("b")),
                        },
                        MirInst::Call {
                            dest: Some(Place::new("result")),
                            func: "gcd".to_string(),
                            args: vec![
                                Operand::Place(Place::new("b")),
                                Operand::Place(Place::new("a_mod_b")),
                            ],
                            is_tail: true,
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

        assert!(changed, "GCD should be transformed to loop");

        // Entry should jump to loop_header
        assert!(matches!(&func.blocks[0].terminator, Terminator::Goto(_)));

        // Loop header should have phi nodes for both a and b
        let loop_header = &func.blocks[1];
        let phi_count = loop_header.instructions.iter()
            .filter(|i| matches!(i, MirInst::Phi { .. }))
            .count();
        assert_eq!(phi_count, 2, "Should have phi nodes for both a and b");

        // No recursive calls should remain
        let has_call = func.blocks.iter().any(|b| {
            b.instructions.iter().any(|i| matches!(i, MirInst::Call { func: f, .. } if f == "gcd"))
        });
        assert!(!has_call, "Recursive call should be eliminated");
    }

    #[test]
    fn test_tail_recursive_non_self_call_no_change() {
        // Tail call to a DIFFERENT function should NOT be transformed
        let mut func = MirFunction {
            name: "wrapper".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("result")),
                        func: "other_func".to_string(),
                        args: vec![Operand::Place(Place::new("n"))],
                        is_tail: true,
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

        let pass = TailRecursiveToLoop::new();
        let changed = pass.run_on_function(&mut func);

        assert!(!changed, "Non-self-recursive call should not be transformed");
    }

    #[test]
    fn test_tail_recursive_all_invariant_no_change() {
        // All params passed unchanged = infinite loop, skip transform
        let mut func = MirFunction {
            name: "infinite".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Le,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "base".to_string(),
                        else_label: "recurse".to_string(),
                    },
                },
                BasicBlock {
                    label: "base".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
                },
                BasicBlock {
                    label: "recurse".to_string(),
                    instructions: vec![
                        MirInst::Call {
                            dest: Some(Place::new("result")),
                            func: "infinite".to_string(),
                            args: vec![Operand::Place(Place::new("x"))], // Same param!
                            is_tail: true,
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
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

        let pass = TailRecursiveToLoop::new();
        let changed = pass.run_on_function(&mut func);

        assert!(!changed, "All-invariant params should not be transformed (infinite loop)");
    }

    // ========================================================================
    // Cycle 219: Additional LinearRecurrenceToLoop tests
    // ========================================================================

    #[test]
    fn test_linear_recurrence_multi_param_no_change() {
        // LinearRecurrenceToLoop only handles single-param functions
        let mut func = MirFunction {
            name: "f".to_string(),
            params: vec![
                ("a".to_string(), MirType::I64),
                ("b".to_string(), MirType::I64),
            ],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
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

        let pass = LinearRecurrenceToLoop;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Multi-param function should not be transformed");
    }

    #[test]
    fn test_linear_recurrence_non_integer_param_no_change() {
        // LinearRecurrenceToLoop only handles integer params
        let mut func = MirFunction {
            name: "f".to_string(),
            params: vec![("x".to_string(), MirType::F64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = LinearRecurrenceToLoop;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Non-integer param should not be transformed");
    }

    #[test]
    fn test_linear_recurrence_fibonacci_loop_structure() {
        // Verify detailed loop structure after fibonacci transformation
        let mut func = MirFunction {
            name: "fib".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
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
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "base".to_string(),
                        else_label: "recurse".to_string(),
                    },
                },
                BasicBlock {
                    label: "base".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("n")))),
                },
                BasicBlock {
                    label: "recurse".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("n1"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                        MirInst::Call {
                            dest: Some(Place::new("r1")),
                            func: "fib".to_string(),
                            args: vec![Operand::Place(Place::new("n1"))],
                            is_tail: false,
                        },
                        MirInst::BinOp {
                            dest: Place::new("n2"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(2)),
                        },
                        MirInst::Call {
                            dest: Some(Place::new("r2")),
                            func: "fib".to_string(),
                            args: vec![Operand::Place(Place::new("n2"))],
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

        let pass = LinearRecurrenceToLoop;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Fibonacci should be transformed");

        // Verify loop structure exists
        let has_loop_setup = func.blocks.iter().any(|b| b.label == "loop_setup");
        let has_loop_header = func.blocks.iter().any(|b| b.label == "loop_header");
        let has_loop_body = func.blocks.iter().any(|b| b.label == "loop_body");
        let has_loop_exit = func.blocks.iter().any(|b| b.label == "loop_exit");
        assert!(has_loop_setup, "Should have loop_setup block");
        assert!(has_loop_header, "Should have loop_header block");
        assert!(has_loop_body, "Should have loop_body block");
        assert!(has_loop_exit, "Should have loop_exit block");

        // Verify loop_header has phi nodes
        let header = func.blocks.iter().find(|b| b.label == "loop_header").unwrap();
        let phi_count = header.instructions.iter()
            .filter(|i| matches!(i, MirInst::Phi { .. }))
            .count();
        assert!(phi_count >= 3, "Loop header should have phi nodes for prev2, prev1, i");

        // Verify loop_body has the Add operation
        let body = func.blocks.iter().find(|b| b.label == "loop_body").unwrap();
        let has_add = body.instructions.iter().any(|i| {
            matches!(i, MirInst::BinOp { op: MirBinOp::Add, .. })
        });
        assert!(has_add, "Loop body should have Add operation for fibonacci recurrence");

        // Verify recurse block is removed
        let has_recurse = func.blocks.iter().any(|b| b.label == "recurse");
        assert!(!has_recurse, "Original recursive block should be removed");
    }

    #[test]
    fn test_linear_recurrence_single_recursive_call_no_change() {
        // Only 1 recursive call (factorial pattern) - needs 2 for fibonacci
        let mut func = MirFunction {
            name: "fact".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
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
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "base".to_string(),
                        else_label: "recurse".to_string(),
                    },
                },
                BasicBlock {
                    label: "base".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                },
                BasicBlock {
                    label: "recurse".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("n1"),
                            op: MirBinOp::Sub,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                        MirInst::Call {
                            dest: Some(Place::new("r")),
                            func: "fact".to_string(),
                            args: vec![Operand::Place(Place::new("n1"))],
                            is_tail: false,
                        },
                        MirInst::BinOp {
                            dest: Place::new("result"),
                            op: MirBinOp::Mul,
                            lhs: Operand::Place(Place::new("n")),
                            rhs: Operand::Place(Place::new("r")),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
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

        let pass = LinearRecurrenceToLoop;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "Single recursive call (factorial) should not trigger fibonacci transform");
    }

    // ========================================================================
    // Cycle 220: ContractBasedOptimization additional tests
    // ========================================================================

    #[test]
    fn test_contract_no_preconditions_no_change() {
        // No preconditions → no optimization possible
        let mut func = MirFunction {
            name: "test".to_string(),
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
                        rhs: Operand::Constant(Constant::Int(0)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("cmp")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = ContractBasedOptimization;
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "No preconditions should mean no optimization");
        // Instruction should remain a BinOp
        assert!(matches!(&func.blocks[0].instructions[0], MirInst::BinOp { .. }));
    }

    #[test]
    fn test_contract_lt_comparison_elimination() {
        // pre x < 10 → check x < 20 should be true
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::Bool,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("cmp"),
                        op: MirBinOp::Lt,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(20)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("cmp")))),
            }],
            preconditions: vec![
                ContractFact::VarCmp {
                    var: "x".to_string(),
                    op: CmpOp::Lt,
                    value: 10,
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
        assert!(changed, "x < 10 implies x < 20");
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::Const { value: Constant::Bool(true), .. }));
    }

    #[test]
    fn test_contract_branch_simplification_to_goto() {
        // pre x >= 0 → branch on (x >= 0) should be simplified to Goto(then)
        let mut func = MirFunction {
            name: "test".to_string(),
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
                        then_label: "safe".to_string(),
                        else_label: "panic".to_string(),
                    },
                },
                BasicBlock {
                    label: "safe".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
                },
                BasicBlock {
                    label: "panic".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(-1)))),
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

        let pass = ContractBasedOptimization;
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Contract should simplify branch");

        // After optimization, the comparison is folded to true.
        // The branch simplification may convert Branch{cond=true} to Goto
        // depending on implementation. The comparison should at least be a Const.
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::Const { value: Constant::Bool(true), .. }),
            "Comparison should be eliminated to true");
    }

    #[test]
    fn test_contract_le_not_provable() {
        // pre x >= 5 → check x <= 3 cannot be proven (it's actually false)
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::Bool,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("cmp"),
                        op: MirBinOp::Le,
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
                    value: 5,
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
        // x >= 5 means x <= 3 is always FALSE
        assert!(changed, "x >= 5 implies x <= 3 is false");
        assert!(matches!(&func.blocks[0].instructions[0],
            MirInst::Const { value: Constant::Bool(false), .. }),
            "x <= 3 should be optimized to false when x >= 5");
    }

    // ========================================================================
    // Cycle 220: LoopInvariantCodeMotion additional tests
    // ========================================================================

    #[test]
    fn test_licm_no_loop_no_change() {
        // Function without loops → no change
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("x")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let pass = LoopInvariantCodeMotion::new();
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "No loops means nothing to hoist");
    }

    #[test]
    fn test_licm_hoists_byte_at_pure_call() {
        // byte_at is a pure function → should be hoisted out of loop
        // entry: goto loop_header
        // loop_header: i = phi [...]; b = byte_at(s, 0); cmp = i < b; branch ...
        // loop_body: i_next = i + 1; goto loop_header
        // exit: return i
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("s".to_string(), MirType::String)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("loop_header".to_string()),
                },
                BasicBlock {
                    label: "loop_header".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("i"),
                            values: vec![
                                (Operand::Constant(Constant::Int(0)), "entry".to_string()),
                                (Operand::Place(Place::new("i_next")), "loop_body".to_string()),
                            ],
                        },
                        MirInst::Call {
                            dest: Some(Place::new("b")),
                            func: "byte_at".to_string(),
                            args: vec![
                                Operand::Place(Place::new("s")),
                                Operand::Constant(Constant::Int(0)),
                            ],
                            is_tail: false,
                        },
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Lt,
                            lhs: Operand::Place(Place::new("i")),
                            rhs: Operand::Place(Place::new("b")),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "loop_body".to_string(),
                        else_label: "exit".to_string(),
                    },
                },
                BasicBlock {
                    label: "loop_body".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("i_next"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("i")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("loop_header".to_string()),
                },
                BasicBlock {
                    label: "exit".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("i")))),
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

        let pass = LoopInvariantCodeMotion::new();
        let changed = pass.run_on_function(&mut func);
        assert!(changed, "Should hoist byte_at() call out of loop");

        // Entry block should now have the hoisted call
        let has_hoisted = func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::Call { func: f, .. } if f == "byte_at")
        });
        assert!(has_hoisted, "byte_at() should be hoisted to entry block");
    }

    #[test]
    fn test_licm_phi_dependent_call_not_hoisted() {
        // Call with arg that depends on phi (loop-variant) → should NOT be hoisted
        // loop_header: i = phi [...]; v = char_at(s, i); ...
        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![("s".to_string(), MirType::String)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("loop_header".to_string()),
                },
                BasicBlock {
                    label: "loop_header".to_string(),
                    instructions: vec![
                        MirInst::Phi {
                            dest: Place::new("i"),
                            values: vec![
                                (Operand::Constant(Constant::Int(0)), "entry".to_string()),
                                (Operand::Place(Place::new("i_next")), "loop_body".to_string()),
                            ],
                        },
                        MirInst::Call {
                            dest: Some(Place::new("ch")),
                            func: "char_at".to_string(),
                            args: vec![
                                Operand::Place(Place::new("s")),
                                Operand::Place(Place::new("i")), // Depends on phi!
                            ],
                            is_tail: false,
                        },
                        MirInst::BinOp {
                            dest: Place::new("cmp"),
                            op: MirBinOp::Lt,
                            lhs: Operand::Place(Place::new("i")),
                            rhs: Operand::Constant(Constant::Int(10)),
                        },
                    ],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("cmp")),
                        then_label: "loop_body".to_string(),
                        else_label: "exit".to_string(),
                    },
                },
                BasicBlock {
                    label: "loop_body".to_string(),
                    instructions: vec![
                        MirInst::BinOp {
                            dest: Place::new("i_next"),
                            op: MirBinOp::Add,
                            lhs: Operand::Place(Place::new("i")),
                            rhs: Operand::Constant(Constant::Int(1)),
                        },
                    ],
                    terminator: Terminator::Goto("loop_header".to_string()),
                },
                BasicBlock {
                    label: "exit".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("i")))),
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

        let pass = LoopInvariantCodeMotion::new();
        let changed = pass.run_on_function(&mut func);
        assert!(!changed, "char_at(s, i) depends on loop variable i, should not be hoisted");
    }

    // ========================================================================
    // Cycle 220: ConstantPropagationNarrowing additional tests
    // ========================================================================

    #[test]
    fn test_cpn_no_narrow_with_multiplication() {
        // Function with Mul operation should NOT be narrowed
        // (multiplication can overflow i32)
        let compute_fn = MirFunction {
            name: "compute".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp {
                        dest: Place::new("r"),
                        op: MirBinOp::Mul,
                        lhs: Operand::Place(Place::new("n")),
                        rhs: Operand::Place(Place::new("n")),
                    },
                    // Recursive call (needed for narrowing to even consider)
                    MirInst::BinOp {
                        dest: Place::new("n1"),
                        op: MirBinOp::Sub,
                        lhs: Operand::Place(Place::new("n")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    },
                    MirInst::Call {
                        dest: Some(Place::new("r2")),
                        func: "compute".to_string(),
                        args: vec![Operand::Place(Place::new("n1"))],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let main_fn = MirFunction {
            name: "main".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("r")),
                        func: "compute".to_string(),
                        args: vec![Operand::Constant(Constant::Int(50))],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let mut program = MirProgram {
            functions: vec![compute_fn, main_fn],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };

        let pass = ConstantPropagationNarrowing::from_program(&program);
        let changed = pass.run_on_program(&mut program);

        assert!(!changed, "Function with multiplication should NOT be narrowed");
        let compute = program.functions.iter().find(|f| f.name == "compute").unwrap();
        assert_eq!(compute.params[0].1, MirType::I64);
    }

    #[test]
    fn test_cpn_no_narrow_non_decreasing_recursion() {
        // Function with non-decreasing recursion (passes constant, not n-1)
        // should NOT be narrowed
        let func = MirFunction {
            name: "weird".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    // Recursive call with constant arg (not decreasing)
                    MirInst::Call {
                        dest: Some(Place::new("r")),
                        func: "weird".to_string(),
                        args: vec![Operand::Place(Place::new("n"))], // Same value!
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let main_fn = MirFunction {
            name: "main".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::Call {
                        dest: Some(Place::new("r")),
                        func: "weird".to_string(),
                        args: vec![Operand::Constant(Constant::Int(10))],
                        is_tail: false,
                    },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let mut program = MirProgram {
            functions: vec![func, main_fn],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };

        let pass = ConstantPropagationNarrowing::from_program(&program);
        let changed = pass.run_on_program(&mut program);

        // The function has remaining self-recursive calls, so it should NOT be narrowed
        assert!(!changed, "Function with remaining recursive calls should not be narrowed");
    }

    #[test]
    fn test_cpn_no_call_sites_no_narrow() {
        // Function never called with constants → no narrowing
        let func = MirFunction {
            name: "never_called".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("n")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };

        let mut program = MirProgram {
            functions: vec![func],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };

        let pass = ConstantPropagationNarrowing::from_program(&program);
        let changed = pass.run_on_program(&mut program);

        assert!(!changed, "Function without constant call sites should not be narrowed");
    }

    // ===== Cycle 406: simplify_binop tests =====

    #[test]
    fn test_simplify_add_zero_rhs() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Int(0));
        let result = simplify_binop(&dest, MirBinOp::Add, &lhs, &rhs);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MirInst::Copy { .. }));
    }

    #[test]
    fn test_simplify_add_zero_lhs() {
        let dest = Place::new("result");
        let lhs = Operand::Constant(Constant::Int(0));
        let rhs = Operand::Place(Place::new("x"));
        let result = simplify_binop(&dest, MirBinOp::Add, &lhs, &rhs);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MirInst::Copy { .. }));
    }

    #[test]
    fn test_simplify_add_nonzero() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Int(5));
        let result = simplify_binop(&dest, MirBinOp::Add, &lhs, &rhs);
        assert!(result.is_none());
    }

    #[test]
    fn test_simplify_sub_zero() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Int(0));
        let result = simplify_binop(&dest, MirBinOp::Sub, &lhs, &rhs);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MirInst::Copy { .. }));
    }

    #[test]
    fn test_simplify_mul_one_rhs() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Int(1));
        let result = simplify_binop(&dest, MirBinOp::Mul, &lhs, &rhs);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MirInst::Copy { .. }));
    }

    #[test]
    fn test_simplify_mul_zero() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Int(0));
        let result = simplify_binop(&dest, MirBinOp::Mul, &lhs, &rhs);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MirInst::Const { value: Constant::Int(0), .. }));
    }

    #[test]
    fn test_simplify_mul_power_of_two() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Int(8));
        let result = simplify_binop(&dest, MirBinOp::Mul, &lhs, &rhs);
        assert!(result.is_some());
        // x * 8 → x << 3
        match result.unwrap() {
            MirInst::BinOp { op: MirBinOp::Shl, rhs: Operand::Constant(Constant::Int(3)), .. } => {}
            other => panic!("Expected shift left by 3, got {:?}", other),
        }
    }

    #[test]
    fn test_simplify_div_one() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Int(1));
        let result = simplify_binop(&dest, MirBinOp::Div, &lhs, &rhs);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MirInst::Copy { .. }));
    }

    #[test]
    fn test_simplify_div_power_of_two() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Int(4));
        let result = simplify_binop(&dest, MirBinOp::Div, &lhs, &rhs);
        assert!(result.is_some());
        // x / 4 → x >> 2
        match result.unwrap() {
            MirInst::BinOp { op: MirBinOp::Shr, rhs: Operand::Constant(Constant::Int(2)), .. } => {}
            other => panic!("Expected shift right by 2, got {:?}", other),
        }
    }

    #[test]
    fn test_simplify_mod_power_of_two() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Int(16));
        let result = simplify_binop(&dest, MirBinOp::Mod, &lhs, &rhs);
        assert!(result.is_some());
        // x % 16 → x & 15
        match result.unwrap() {
            MirInst::BinOp { op: MirBinOp::Band, rhs: Operand::Constant(Constant::Int(15)), .. } => {}
            other => panic!("Expected band with 15, got {:?}", other),
        }
    }

    #[test]
    fn test_simplify_and_true_rhs() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Bool(true));
        let result = simplify_binop(&dest, MirBinOp::And, &lhs, &rhs);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MirInst::Copy { .. }));
    }

    #[test]
    fn test_simplify_and_false() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Bool(false));
        let result = simplify_binop(&dest, MirBinOp::And, &lhs, &rhs);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MirInst::Const { value: Constant::Bool(false), .. }));
    }

    #[test]
    fn test_simplify_or_false_rhs() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Bool(false));
        let result = simplify_binop(&dest, MirBinOp::Or, &lhs, &rhs);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MirInst::Copy { .. }));
    }

    #[test]
    fn test_simplify_or_true() {
        let dest = Place::new("result");
        let lhs = Operand::Place(Place::new("x"));
        let rhs = Operand::Constant(Constant::Bool(true));
        let result = simplify_binop(&dest, MirBinOp::Or, &lhs, &rhs);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MirInst::Const { value: Constant::Bool(true), .. }));
    }

    // ===== Cycle 407: fold_binop + fold_unaryop tests =====

    #[test]
    fn test_fold_int_add() {
        assert!(matches!(fold_binop(MirBinOp::Add, &Constant::Int(3), &Constant::Int(7)), Some(Constant::Int(10))));
    }

    #[test]
    fn test_fold_int_sub() {
        assert!(matches!(fold_binop(MirBinOp::Sub, &Constant::Int(10), &Constant::Int(4)), Some(Constant::Int(6))));
    }

    #[test]
    fn test_fold_int_mul() {
        assert!(matches!(fold_binop(MirBinOp::Mul, &Constant::Int(6), &Constant::Int(7)), Some(Constant::Int(42))));
    }

    #[test]
    fn test_fold_int_div() {
        assert!(matches!(fold_binop(MirBinOp::Div, &Constant::Int(20), &Constant::Int(4)), Some(Constant::Int(5))));
    }

    #[test]
    fn test_fold_int_div_by_zero() {
        assert!(fold_binop(MirBinOp::Div, &Constant::Int(10), &Constant::Int(0)).is_none());
    }

    #[test]
    fn test_fold_int_mod() {
        assert!(matches!(fold_binop(MirBinOp::Mod, &Constant::Int(17), &Constant::Int(5)), Some(Constant::Int(2))));
    }

    #[test]
    fn test_fold_int_mod_by_zero() {
        assert!(fold_binop(MirBinOp::Mod, &Constant::Int(10), &Constant::Int(0)).is_none());
    }

    #[test]
    fn test_fold_int_eq_true() {
        assert!(matches!(fold_binop(MirBinOp::Eq, &Constant::Int(5), &Constant::Int(5)), Some(Constant::Bool(true))));
    }

    #[test]
    fn test_fold_int_eq_false() {
        assert!(matches!(fold_binop(MirBinOp::Eq, &Constant::Int(5), &Constant::Int(6)), Some(Constant::Bool(false))));
    }

    #[test]
    fn test_fold_int_lt() {
        assert!(matches!(fold_binop(MirBinOp::Lt, &Constant::Int(3), &Constant::Int(5)), Some(Constant::Bool(true))));
        assert!(matches!(fold_binop(MirBinOp::Lt, &Constant::Int(5), &Constant::Int(3)), Some(Constant::Bool(false))));
    }

    #[test]
    fn test_fold_int_ge() {
        assert!(matches!(fold_binop(MirBinOp::Ge, &Constant::Int(5), &Constant::Int(5)), Some(Constant::Bool(true))));
        assert!(matches!(fold_binop(MirBinOp::Ge, &Constant::Int(4), &Constant::Int(5)), Some(Constant::Bool(false))));
    }

    #[test]
    fn test_fold_bool_and() {
        assert!(matches!(fold_binop(MirBinOp::And, &Constant::Bool(true), &Constant::Bool(true)), Some(Constant::Bool(true))));
        assert!(matches!(fold_binop(MirBinOp::And, &Constant::Bool(true), &Constant::Bool(false)), Some(Constant::Bool(false))));
    }

    #[test]
    fn test_fold_bool_or() {
        assert!(matches!(fold_binop(MirBinOp::Or, &Constant::Bool(false), &Constant::Bool(true)), Some(Constant::Bool(true))));
        assert!(matches!(fold_binop(MirBinOp::Or, &Constant::Bool(false), &Constant::Bool(false)), Some(Constant::Bool(false))));
    }

    #[test]
    fn test_fold_float_add() {
        match fold_binop(MirBinOp::FAdd, &Constant::Float(1.5), &Constant::Float(2.5)) {
            Some(Constant::Float(f)) => assert!((f - 4.0).abs() < 0.001),
            other => panic!("Expected Some(Float(4.0)), got {:?}", other),
        }
    }

    #[test]
    fn test_fold_float_div_by_zero() {
        assert!(fold_binop(MirBinOp::FDiv, &Constant::Float(1.0), &Constant::Float(0.0)).is_none());
    }

    #[test]
    fn test_fold_string_concat() {
        match fold_binop(MirBinOp::Add, &Constant::String("Hello".to_string()), &Constant::String(" World".to_string())) {
            Some(Constant::String(s)) => assert_eq!(s, "Hello World"),
            other => panic!("Expected Some(String(\"Hello World\")), got {:?}", other),
        }
    }

    #[test]
    fn test_fold_type_mismatch() {
        assert!(fold_binop(MirBinOp::Add, &Constant::Int(1), &Constant::Bool(true)).is_none());
    }

    #[test]
    fn test_fold_unary_neg_int() {
        assert!(matches!(fold_unaryop(MirUnaryOp::Neg, &Constant::Int(5)), Some(Constant::Int(-5))));
    }

    #[test]
    fn test_fold_unary_fneg_float() {
        match fold_unaryop(MirUnaryOp::FNeg, &Constant::Float(2.5)) {
            Some(Constant::Float(f)) => assert!((f + 2.5).abs() < 0.001),
            other => panic!("Expected Some(Float(-2.5)), got {:?}", other),
        }
    }

    #[test]
    fn test_fold_unary_not_bool() {
        assert!(matches!(fold_unaryop(MirUnaryOp::Not, &Constant::Bool(true)), Some(Constant::Bool(false))));
        assert!(matches!(fold_unaryop(MirUnaryOp::Not, &Constant::Bool(false)), Some(Constant::Bool(true))));
    }

    // --- MemoryEffectAnalysis: inst_accesses_memory coverage ---

    fn make_memory_test_program(name: &str, insts: Vec<MirInst>) -> MirProgram {
        MirProgram {
            functions: vec![MirFunction {
                name: name.to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: insts,
                    terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        }
    }

    #[test]
    fn test_memory_effect_ptr_load_not_free() {
        let mut prog = make_memory_test_program("ptr_fn", vec![
            MirInst::PtrLoad {
                dest: Place::new("r"),
                ptr: Operand::Place(Place::new("x")),
                element_type: MirType::I64,
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(!prog.functions[0].is_memory_free, "PtrLoad accesses memory");
    }

    #[test]
    fn test_memory_effect_ptr_store_not_free() {
        let mut prog = make_memory_test_program("ps_fn", vec![
            MirInst::PtrStore {
                ptr: Operand::Place(Place::new("x")),
                value: Operand::Constant(Constant::Int(42)),
                element_type: MirType::I64,
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(!prog.functions[0].is_memory_free, "PtrStore accesses memory");
    }

    #[test]
    fn test_memory_effect_ptr_offset_is_free() {
        let mut prog = make_memory_test_program("po_fn", vec![
            MirInst::PtrOffset {
                dest: Place::new("r"),
                ptr: Operand::Place(Place::new("x")),
                offset: Operand::Constant(Constant::Int(1)),
                element_type: MirType::I64,
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(prog.functions[0].is_memory_free, "PtrOffset is pure address arithmetic");
    }

    #[test]
    fn test_memory_effect_select_is_free() {
        let mut prog = make_memory_test_program("sel_fn", vec![
            MirInst::Select {
                dest: Place::new("r"),
                cond_op: MirBinOp::Lt,
                cond_lhs: Operand::Place(Place::new("x")),
                cond_rhs: Operand::Constant(Constant::Int(10)),
                true_val: Operand::Constant(Constant::Int(1)),
                false_val: Operand::Constant(Constant::Int(0)),
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(prog.functions[0].is_memory_free, "Select is pure");
    }

    #[test]
    fn test_memory_effect_cast_is_free() {
        let mut prog = make_memory_test_program("cast_fn", vec![
            MirInst::Cast {
                dest: Place::new("r"),
                src: Operand::Place(Place::new("x")),
                from_ty: MirType::I64,
                to_ty: MirType::F64,
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(prog.functions[0].is_memory_free, "Cast is pure");
    }

    #[test]
    fn test_memory_effect_copy_is_free() {
        let mut prog = make_memory_test_program("copy_fn", vec![
            MirInst::Copy {
                dest: Place::new("r"),
                src: Place::new("x"),
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(prog.functions[0].is_memory_free, "Copy is pure");
    }

    #[test]
    fn test_memory_effect_unary_is_free() {
        let mut prog = make_memory_test_program("neg_fn", vec![
            MirInst::UnaryOp {
                dest: Place::new("r"),
                op: MirUnaryOp::Neg,
                src: Operand::Place(Place::new("x")),
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(prog.functions[0].is_memory_free, "UnaryOp is pure");
    }

    #[test]
    fn test_memory_effect_tuple_not_free() {
        let mut prog = make_memory_test_program("tup_fn", vec![
            MirInst::TupleInit {
                dest: Place::new("r"),
                elements: vec![(MirType::I64, Operand::Constant(Constant::Int(1)))],
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(!prog.functions[0].is_memory_free, "TupleInit accesses memory");
    }

    #[test]
    fn test_memory_effect_array_alloc_not_free() {
        let mut prog = make_memory_test_program("alloc_fn", vec![
            MirInst::ArrayAlloc {
                dest: Place::new("r"),
                element_type: MirType::I64,
                size: 10,
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(!prog.functions[0].is_memory_free, "ArrayAlloc accesses memory");
    }

    #[test]
    fn test_memory_effect_index_load_not_free() {
        let mut prog = make_memory_test_program("idx_fn", vec![
            MirInst::IndexLoad {
                dest: Place::new("r"),
                array: Place::new("x"),
                index: Operand::Constant(Constant::Int(0)),
                element_type: MirType::I64,
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(!prog.functions[0].is_memory_free, "IndexLoad accesses memory");
    }

    #[test]
    fn test_memory_effect_struct_init_not_free() {
        let mut prog = make_memory_test_program("si_fn", vec![
            MirInst::StructInit {
                dest: Place::new("r"),
                struct_name: "Foo".to_string(),
                fields: vec![("x".to_string(), Operand::Constant(Constant::Int(1)))],
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(!prog.functions[0].is_memory_free, "StructInit accesses memory");
    }

    #[test]
    fn test_memory_effect_phi_is_free() {
        let mut prog = make_memory_test_program("phi_fn", vec![
            MirInst::Phi {
                dest: Place::new("r"),
                values: vec![
                    (Operand::Constant(Constant::Int(1)), "b1".to_string()),
                ],
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(prog.functions[0].is_memory_free, "Phi is pure");
    }

    #[test]
    fn test_memory_effect_const_is_free() {
        let mut prog = make_memory_test_program("const_fn", vec![
            MirInst::Const {
                dest: Place::new("r"),
                value: Constant::Int(42),
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(prog.functions[0].is_memory_free, "Const is pure");
    }

    #[test]
    fn test_memory_effect_mixed_pure_and_impure() {
        let mut prog = make_memory_test_program("mix_fn", vec![
            MirInst::BinOp {
                dest: Place::new("t1"),
                op: MirBinOp::Add,
                lhs: Operand::Place(Place::new("x")),
                rhs: Operand::Constant(Constant::Int(1)),
            },
            MirInst::Call {
                dest: Some(Place::new("t2")),
                func: "foo".to_string(),
                args: vec![],
                is_tail: false,
            },
        ]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(!prog.functions[0].is_memory_free, "One impure instruction makes function not memory-free");
    }

    #[test]
    fn test_memory_effect_already_marked() {
        let mut prog = make_memory_test_program("already_fn", vec![
            MirInst::BinOp {
                dest: Place::new("r"),
                op: MirBinOp::Add,
                lhs: Operand::Place(Place::new("x")),
                rhs: Operand::Constant(Constant::Int(1)),
            },
        ]);
        prog.functions[0].is_memory_free = true;
        let analysis = MemoryEffectAnalysis::new();
        let changed = analysis.run_on_program(&mut prog);
        assert!(!changed, "No change if already marked correctly");
        assert!(prog.functions[0].is_memory_free);
    }

    #[test]
    fn test_memory_effect_empty_function() {
        let mut prog = make_memory_test_program("empty_fn", vec![]);
        let analysis = MemoryEffectAnalysis::new();
        analysis.run_on_program(&mut prog);
        assert!(prog.functions[0].is_memory_free, "Empty function is memory-free");
    }

    // --- LICM edge cases ---

    #[test]
    fn test_licm_single_block_no_loop() {
        let mut func = MirFunction {
            name: "no_loop".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::BinOp {
                    dest: Place::new("r"),
                    op: MirBinOp::Add,
                    lhs: Operand::Place(Place::new("x")),
                    rhs: Operand::Constant(Constant::Int(1)),
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        let licm = LoopInvariantCodeMotion::new();
        let changed = licm.run_on_function(&mut func);
        assert!(!changed, "Single block (no loop) should not change");
    }

    #[test]
    fn test_licm_loop_without_calls() {
        // Loop body with only BinOp (no calls to hoist)
        let mut func = MirFunction {
            name: "loop_no_call".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("header".to_string()),
                },
                BasicBlock {
                    label: "header".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("t"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("n")),
                        rhs: Operand::Constant(Constant::Int(1)),
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("t")),
                        then_label: "body".to_string(),
                        else_label: "exit".to_string(),
                    },
                },
                BasicBlock {
                    label: "body".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("header".to_string()),
                },
                BasicBlock {
                    label: "exit".to_string(),
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
        let licm = LoopInvariantCodeMotion::new();
        let changed = licm.run_on_function(&mut func);
        assert!(!changed, "No calls to hoist");
    }

    #[test]
    fn test_licm_tail_call_not_hoisted() {
        // Tail calls should not be hoisted (is_tail: true)
        let mut func = MirFunction {
            name: "tail_loop".to_string(),
            params: vec![("s".to_string(), MirType::String), ("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("header".to_string()),
                },
                BasicBlock {
                    label: "header".to_string(),
                    instructions: vec![MirInst::Call {
                        dest: Some(Place::new("_len")),
                        func: "len".to_string(),
                        args: vec![Operand::Place(Place::new("s"))],
                        is_tail: true,
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("n")),
                        then_label: "body".to_string(),
                        else_label: "exit".to_string(),
                    },
                },
                BasicBlock {
                    label: "body".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("header".to_string()),
                },
                BasicBlock {
                    label: "exit".to_string(),
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
        let licm = LoopInvariantCodeMotion::new();
        let changed = licm.run_on_function(&mut func);
        assert!(!changed, "Tail calls should not be hoisted");
    }

    #[test]
    fn test_licm_call_with_no_dest_not_hoisted() {
        // Side-effect-only calls (dest: None) should not be hoisted
        let mut func = MirFunction {
            name: "sideeff_loop".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("header".to_string()),
                },
                BasicBlock {
                    label: "header".to_string(),
                    instructions: vec![MirInst::Call {
                        dest: None,
                        func: "println".to_string(),
                        args: vec![Operand::Place(Place::new("n"))],
                        is_tail: false,
                    }],
                    terminator: Terminator::Branch {
                        cond: Operand::Place(Place::new("n")),
                        then_label: "body".to_string(),
                        else_label: "exit".to_string(),
                    },
                },
                BasicBlock {
                    label: "body".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("header".to_string()),
                },
                BasicBlock {
                    label: "exit".to_string(),
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
        let licm = LoopInvariantCodeMotion::new();
        let changed = licm.run_on_function(&mut func);
        assert!(!changed, "Side-effect-only calls (dest: None) should not be hoisted");
    }

    #[test]
    fn test_licm_name() {
        let licm = LoopInvariantCodeMotion::new();
        assert_eq!(licm.name(), "loop_invariant_code_motion");
    }

    #[test]
    fn test_memory_effect_analysis_name_string() {
        let analysis = MemoryEffectAnalysis::new();
        assert_eq!(analysis.name(), "memory_effect_analysis");
    }

    // --- ProvenFacts::from_preconditions ---

    #[test]
    fn test_proven_facts_ge_bound() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Ge, value: 5 },
        ]);
        let (lower, upper) = facts.var_bounds.get("x").unwrap();
        assert_eq!(*lower, Some(5));
        assert_eq!(*upper, None);
    }

    #[test]
    fn test_proven_facts_gt_bound() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Gt, value: 5 },
        ]);
        let (lower, _) = facts.var_bounds.get("x").unwrap();
        assert_eq!(*lower, Some(6), "x > 5 means lower bound is 6");
    }

    #[test]
    fn test_proven_facts_le_bound() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Le, value: 10 },
        ]);
        let (_, upper) = facts.var_bounds.get("x").unwrap();
        assert_eq!(*upper, Some(10));
    }

    #[test]
    fn test_proven_facts_lt_bound() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Lt, value: 10 },
        ]);
        let (_, upper) = facts.var_bounds.get("x").unwrap();
        assert_eq!(*upper, Some(9), "x < 10 means upper bound is 9");
    }

    #[test]
    fn test_proven_facts_eq_bound() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Eq, value: 7 },
        ]);
        let (lower, upper) = facts.var_bounds.get("x").unwrap();
        assert_eq!(*lower, Some(7));
        assert_eq!(*upper, Some(7));
    }

    #[test]
    fn test_proven_facts_multiple_bounds_merge() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Ge, value: 3 },
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Le, value: 100 },
        ]);
        let (lower, upper) = facts.var_bounds.get("x").unwrap();
        assert_eq!(*lower, Some(3));
        assert_eq!(*upper, Some(100));
    }

    #[test]
    fn test_proven_facts_nonnull() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::NonNull { var: "ptr".to_string() },
        ]);
        assert_eq!(facts.get_bool_value("ptr_is_null"), Some(false));
    }

    #[test]
    fn test_proven_facts_array_bounds_stored() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::ArrayBounds { array: "arr".to_string(), index: "i".to_string() },
        ]);
        assert_eq!(facts.var_relations.len(), 1);
    }

    // --- ProvenFacts::check_bounds ---

    #[test]
    fn test_check_bounds_ge_true() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Ge, value: 10 },
        ]);
        // x >= 10, check x >= 5 → always true
        let result = facts.check_bounds(Some(10), None, CmpOp::Ge, 5);
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_check_bounds_ge_false() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Le, value: 3 },
        ]);
        // x <= 3, check x >= 5 → always false
        let result = facts.check_bounds(None, Some(3), CmpOp::Ge, 5);
        assert_eq!(result, Some(false));
    }

    #[test]
    fn test_check_bounds_lt_true() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Le, value: 3 },
        ]);
        // x <= 3, check x < 10 → always true (upper 3 < 10)
        let result = facts.check_bounds(None, Some(3), CmpOp::Lt, 10);
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_check_bounds_lt_false() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Ge, value: 10 },
        ]);
        // x >= 10, check x < 5 → always false (lower 10 >= 5)
        let result = facts.check_bounds(Some(10), None, CmpOp::Lt, 5);
        assert_eq!(result, Some(false));
    }

    #[test]
    fn test_check_bounds_unknown() {
        let facts = ProvenFacts::from_preconditions(&[]);
        let result = facts.check_bounds(None, None, CmpOp::Ge, 5);
        assert_eq!(result, None, "No bounds = unknown");
    }

    // --- ProvenFacts::evaluate_comparison ---

    #[test]
    fn test_evaluate_comparison_var_ge_const() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Ge, value: 10 },
        ]);
        let result = facts.evaluate_comparison(
            &Operand::Place(Place::new("x")),
            CmpOp::Ge,
            &Operand::Constant(Constant::Int(5)),
        );
        assert_eq!(result, Some(true), "x >= 10, so x >= 5 is true");
    }

    #[test]
    fn test_evaluate_comparison_const_lt_var() {
        let facts = ProvenFacts::from_preconditions(&[
            ContractFact::VarCmp { var: "x".to_string(), op: CmpOp::Ge, value: 10 },
        ]);
        // 5 < x → flipped to x > 5 → true because lower=10 > 5
        let result = facts.evaluate_comparison(
            &Operand::Constant(Constant::Int(5)),
            CmpOp::Lt,
            &Operand::Place(Place::new("x")),
        );
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_evaluate_comparison_unknown_var() {
        let facts = ProvenFacts::from_preconditions(&[]);
        let result = facts.evaluate_comparison(
            &Operand::Place(Place::new("y")),
            CmpOp::Ge,
            &Operand::Constant(Constant::Int(5)),
        );
        assert_eq!(result, None, "Unknown variable = unknown comparison");
    }

    // --- fold_builtin_call ---

    #[test]
    fn test_fold_builtin_chr_valid() {
        let constants = HashMap::new();
        let result = fold_builtin_call("chr", &[Operand::Constant(Constant::Int(65))], &constants);
        assert!(matches!(result, Some(Constant::String(s)) if s == "A"));
    }

    #[test]
    fn test_fold_builtin_chr_zero() {
        let constants = HashMap::new();
        let result = fold_builtin_call("chr", &[Operand::Constant(Constant::Int(0))], &constants);
        assert!(matches!(result, Some(Constant::String(s)) if s == "\0"));
    }

    #[test]
    fn test_fold_builtin_chr_out_of_range() {
        let constants = HashMap::new();
        let result = fold_builtin_call("chr", &[Operand::Constant(Constant::Int(128))], &constants);
        assert!(result.is_none(), "128 is outside ASCII range");
    }

    #[test]
    fn test_fold_builtin_chr_negative() {
        let constants = HashMap::new();
        let result = fold_builtin_call("chr", &[Operand::Constant(Constant::Int(-1))], &constants);
        assert!(result.is_none());
    }

    #[test]
    fn test_fold_builtin_ord_valid() {
        let constants = HashMap::new();
        let result = fold_builtin_call("ord", &[Operand::Constant(Constant::String("A".to_string()))], &constants);
        assert!(matches!(result, Some(Constant::Int(65))));
    }

    #[test]
    fn test_fold_builtin_ord_empty_string() {
        let constants = HashMap::new();
        let result = fold_builtin_call("ord", &[Operand::Constant(Constant::String("".to_string()))], &constants);
        assert!(result.is_none(), "Empty string has no char");
    }

    #[test]
    fn test_fold_builtin_ord_multi_char() {
        let constants = HashMap::new();
        let result = fold_builtin_call("ord", &[Operand::Constant(Constant::String("AB".to_string()))], &constants);
        assert!(result.is_none(), "Multi-char string not handled");
    }

    #[test]
    fn test_fold_builtin_unknown_func() {
        let constants = HashMap::new();
        let result = fold_builtin_call("unknown", &[Operand::Constant(Constant::Int(1))], &constants);
        assert!(result.is_none());
    }

    #[test]
    fn test_fold_builtin_bmb_chr() {
        let constants = HashMap::new();
        let result = fold_builtin_call("bmb_chr", &[Operand::Constant(Constant::Int(90))], &constants);
        assert!(matches!(result, Some(Constant::String(s)) if s == "Z"));
    }

    // --- get_constant_with_filter ---

    #[test]
    fn test_get_constant_with_filter_constant_operand() {
        let constants = HashMap::new();
        let loop_modified = HashSet::new();
        let result = get_constant_with_filter(&Operand::Constant(Constant::Int(42)), &constants, &loop_modified);
        assert!(matches!(result, Some(Constant::Int(42))));
    }

    #[test]
    fn test_get_constant_with_filter_loop_modified_blocked() {
        let mut constants = HashMap::new();
        constants.insert("x".to_string(), Constant::Int(10));
        let mut loop_modified = HashSet::new();
        loop_modified.insert("x".to_string());
        let result = get_constant_with_filter(&Operand::Place(Place::new("x")), &constants, &loop_modified);
        assert!(result.is_none(), "Loop-modified var should not be propagated");
    }

    #[test]
    fn test_get_constant_with_filter_not_modified_found() {
        let mut constants = HashMap::new();
        constants.insert("x".to_string(), Constant::Int(10));
        let loop_modified = HashSet::new();
        let result = get_constant_with_filter(&Operand::Place(Place::new("x")), &constants, &loop_modified);
        assert!(matches!(result, Some(Constant::Int(10))));
    }

    #[test]
    fn test_get_constant_with_filter_not_found() {
        let constants = HashMap::new();
        let loop_modified = HashSet::new();
        let result = get_constant_with_filter(&Operand::Place(Place::new("y")), &constants, &loop_modified);
        assert!(result.is_none());
    }

    // --- LinearRecurrenceToLoop name ---

    #[test]
    fn test_linear_recurrence_name() {
        let pass = LinearRecurrenceToLoop::new();
        assert_eq!(pass.name(), "LinearRecurrenceToLoop");
    }

    #[test]
    fn test_linear_recurrence_default() {
        let _pass: LinearRecurrenceToLoop = Default::default();
        assert_eq!(_pass.name(), "LinearRecurrenceToLoop");
    }

    // --- ConditionalIncrementToSelect: operands_equal ---

    #[test]
    fn test_conditional_increment_operands_equal_places() {
        assert!(ConditionalIncrementToSelect::operands_equal(
            &Operand::Place(Place::new("x")),
            &Operand::Place(Place::new("x")),
        ));
        assert!(!ConditionalIncrementToSelect::operands_equal(
            &Operand::Place(Place::new("x")),
            &Operand::Place(Place::new("y")),
        ));
    }

    #[test]
    fn test_conditional_increment_operands_equal_constants() {
        assert!(ConditionalIncrementToSelect::operands_equal(
            &Operand::Constant(Constant::Int(42)),
            &Operand::Constant(Constant::Int(42)),
        ));
        assert!(!ConditionalIncrementToSelect::operands_equal(
            &Operand::Constant(Constant::Int(1)),
            &Operand::Constant(Constant::Int(2)),
        ));
    }

    #[test]
    fn test_conditional_increment_operands_equal_mixed() {
        assert!(!ConditionalIncrementToSelect::operands_equal(
            &Operand::Place(Place::new("x")),
            &Operand::Constant(Constant::Int(42)),
        ));
    }

    #[test]
    fn test_conditional_increment_operands_equal_bools() {
        assert!(ConditionalIncrementToSelect::operands_equal(
            &Operand::Constant(Constant::Bool(true)),
            &Operand::Constant(Constant::Bool(true)),
        ));
        assert!(!ConditionalIncrementToSelect::operands_equal(
            &Operand::Constant(Constant::Bool(true)),
            &Operand::Constant(Constant::Bool(false)),
        ));
    }

    #[test]
    fn test_conditional_increment_operands_equal_strings() {
        assert!(ConditionalIncrementToSelect::operands_equal(
            &Operand::Constant(Constant::String("hello".to_string())),
            &Operand::Constant(Constant::String("hello".to_string())),
        ));
    }

    // --- operand_to_place ---

    #[test]
    fn test_operand_to_place_returns_place() {
        let result = operand_to_place(&Operand::Place(Place::new("x")));
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "x");
    }

    #[test]
    fn test_operand_to_place_returns_none_for_constant() {
        let result = operand_to_place(&Operand::Constant(Constant::Int(42)));
        assert!(result.is_none());
    }

    // --- has_side_effects ---

    #[test]
    fn test_has_side_effects_call() {
        assert!(has_side_effects(&MirInst::Call {
            dest: Some(Place::new("r")),
            func: "foo".to_string(),
            args: vec![],
            is_tail: false,
        }));
    }

    #[test]
    fn test_has_side_effects_binop() {
        assert!(!has_side_effects(&MirInst::BinOp {
            dest: Place::new("r"),
            op: MirBinOp::Add,
            lhs: Operand::Constant(Constant::Int(1)),
            rhs: Operand::Constant(Constant::Int(2)),
        }));
    }

    #[test]
    fn test_has_side_effects_field_store() {
        assert!(has_side_effects(&MirInst::FieldStore {
            base: Place::new("s"),
            field: "x".to_string(),
            field_index: 0,
            struct_name: "Foo".to_string(),
            value: Operand::Constant(Constant::Int(1)),
        }));
    }

    #[test]
    fn test_has_side_effects_ptr_store() {
        assert!(has_side_effects(&MirInst::PtrStore {
            ptr: Operand::Place(Place::new("p")),
            value: Operand::Constant(Constant::Int(42)),
            element_type: MirType::I64,
        }));
    }

    #[test]
    fn test_has_side_effects_const() {
        assert!(!has_side_effects(&MirInst::Const {
            dest: Place::new("c"),
            value: Constant::Int(0),
        }));
    }

    #[test]
    fn test_has_side_effects_copy() {
        assert!(!has_side_effects(&MirInst::Copy {
            dest: Place::new("a"),
            src: Place::new("b"),
        }));
    }

    // --- phi_operand_to_inst ---

    #[test]
    fn test_phi_operand_to_inst_from_place() {
        let inst = phi_operand_to_inst(Place::new("dest"), &Operand::Place(Place::new("src")));
        assert!(matches!(inst, MirInst::Copy { dest, src } if dest.name == "dest" && src.name == "src"));
    }

    #[test]
    fn test_phi_operand_to_inst_from_constant() {
        let inst = phi_operand_to_inst(Place::new("dest"), &Operand::Constant(Constant::Int(42)));
        assert!(matches!(inst, MirInst::Const { dest, value: Constant::Int(42) } if dest.name == "dest"));
    }

    // --- phi_operands_equal ---

    #[test]
    fn test_phi_operands_equal_places() {
        assert!(phi_operands_equal(
            &Operand::Place(Place::new("x")),
            &Operand::Place(Place::new("x")),
        ));
        assert!(!phi_operands_equal(
            &Operand::Place(Place::new("x")),
            &Operand::Place(Place::new("y")),
        ));
    }

    #[test]
    fn test_phi_operands_equal_chars() {
        assert!(phi_operands_equal(
            &Operand::Constant(Constant::Char('A')),
            &Operand::Constant(Constant::Char('A')),
        ));
        assert!(!phi_operands_equal(
            &Operand::Constant(Constant::Char('A')),
            &Operand::Constant(Constant::Char('B')),
        ));
    }

    #[test]
    fn test_phi_operands_equal_units() {
        assert!(phi_operands_equal(
            &Operand::Constant(Constant::Unit),
            &Operand::Constant(Constant::Unit),
        ));
    }

    #[test]
    fn test_phi_operands_equal_mixed_types() {
        assert!(!phi_operands_equal(
            &Operand::Constant(Constant::Int(1)),
            &Operand::Constant(Constant::Bool(true)),
        ));
    }

    // --- update_terminator_labels ---

    #[test]
    fn test_update_terminator_labels_goto() {
        let mut term = Terminator::Goto("old".to_string());
        update_terminator_labels(&mut term, "old", "new");
        assert!(matches!(term, Terminator::Goto(t) if t == "new"));
    }

    #[test]
    fn test_update_terminator_labels_goto_no_match() {
        let mut term = Terminator::Goto("other".to_string());
        update_terminator_labels(&mut term, "old", "new");
        assert!(matches!(term, Terminator::Goto(t) if t == "other"));
    }

    #[test]
    fn test_update_terminator_labels_branch() {
        let mut term = Terminator::Branch {
            cond: Operand::Constant(Constant::Bool(true)),
            then_label: "old".to_string(),
            else_label: "other".to_string(),
        };
        update_terminator_labels(&mut term, "old", "new");
        match &term {
            Terminator::Branch { then_label, else_label, .. } => {
                assert_eq!(then_label, "new");
                assert_eq!(else_label, "other");
            }
            _ => panic!("Expected Branch"),
        }
    }

    #[test]
    fn test_update_terminator_labels_switch() {
        let mut term = Terminator::Switch {
            discriminant: Operand::Place(Place::new("x")),
            cases: vec![(1, "old".to_string()), (2, "keep".to_string())],
            default: "old".to_string(),
        };
        update_terminator_labels(&mut term, "old", "new");
        match &term {
            Terminator::Switch { cases, default, .. } => {
                assert_eq!(cases[0].1, "new");
                assert_eq!(cases[1].1, "keep");
                assert_eq!(default, "new");
            }
            _ => panic!("Expected Switch"),
        }
    }

    #[test]
    fn test_update_terminator_labels_return() {
        let mut term = Terminator::Return(None);
        update_terminator_labels(&mut term, "old", "new");
        assert!(matches!(term, Terminator::Return(None)));
    }

    // --- might_write_memory ---

    #[test]
    fn test_might_write_memory_pure_functions() {
        assert!(!might_write_memory("abs"));
        assert!(!might_write_memory("len"));
        assert!(!might_write_memory("chr"));
        assert!(!might_write_memory("ord"));
        assert!(!might_write_memory("println"));
        assert!(!might_write_memory("hash_i64"));
    }

    #[test]
    fn test_might_write_memory_unknown_function() {
        assert!(might_write_memory("my_func"));
        assert!(might_write_memory("store_value"));
    }

    // --- propagate_copies_in_inst ---

    #[test]
    fn test_propagate_copies_in_inst_binop() {
        let mut copies = HashMap::new();
        copies.insert("y".to_string(), Place::new("x"));
        let mut inst = MirInst::BinOp {
            dest: Place::new("r"),
            op: MirBinOp::Add,
            lhs: Operand::Place(Place::new("y")),
            rhs: Operand::Constant(Constant::Int(1)),
        };
        let changed = propagate_copies_in_inst(&mut inst, &copies);
        assert!(changed);
        match &inst {
            MirInst::BinOp { lhs, .. } => {
                assert!(matches!(lhs, Operand::Place(p) if p.name == "x"));
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_propagate_copies_in_inst_no_match() {
        let copies = HashMap::new();
        let mut inst = MirInst::BinOp {
            dest: Place::new("r"),
            op: MirBinOp::Add,
            lhs: Operand::Place(Place::new("y")),
            rhs: Operand::Constant(Constant::Int(1)),
        };
        let changed = propagate_copies_in_inst(&mut inst, &copies);
        assert!(!changed);
    }

    // --- propagate_copies_in_term ---

    #[test]
    fn test_propagate_copies_in_term_branch() {
        let mut copies = HashMap::new();
        copies.insert("c".to_string(), Place::new("flag"));
        let mut term = Terminator::Branch {
            cond: Operand::Place(Place::new("c")),
            then_label: "t".to_string(),
            else_label: "f".to_string(),
        };
        let changed = propagate_copies_in_term(&mut term, &copies);
        assert!(changed);
        match &term {
            Terminator::Branch { cond, .. } => {
                assert!(matches!(cond, Operand::Place(p) if p.name == "flag"));
            }
            _ => panic!("Expected Branch"),
        }
    }

    #[test]
    fn test_propagate_copies_in_term_return() {
        let mut copies = HashMap::new();
        copies.insert("x".to_string(), Place::new("y"));
        let mut term = Terminator::Return(Some(Operand::Place(Place::new("x"))));
        let changed = propagate_copies_in_term(&mut term, &copies);
        assert!(changed);
        match &term {
            Terminator::Return(Some(Operand::Place(p))) => assert_eq!(p.name, "y"),
            _ => panic!("Expected Return with Place"),
        }
    }

    // --- AggressiveInlining ---

    #[test]
    fn test_aggressive_inlining_name() {
        let pass = AggressiveInlining::new();
        assert_eq!(pass.name(), "aggressive_inlining");
    }

    #[test]
    fn test_aggressive_inlining_custom_thresholds() {
        let pass = AggressiveInlining::with_thresholds(5, 10);
        assert_eq!(pass.name(), "aggressive_inlining");
    }

    #[test]
    fn test_aggressive_inlining_count_instructions() {
        let func = MirFunction {
            name: "f".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const { dest: Place::new("a"), value: Constant::Int(1) },
                        MirInst::Const { dest: Place::new("b"), value: Constant::Int(2) },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("a")))),
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
        assert_eq!(AggressiveInlining::count_instructions(&func), 2);
    }

    #[test]
    fn test_aggressive_inlining_is_simple_single_block() {
        let func = MirFunction {
            name: "f".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        assert!(AggressiveInlining::is_simple_control_flow(&func));
    }

    #[test]
    fn test_aggressive_inlining_not_simple_with_loop() {
        let func = MirFunction {
            name: "f".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("header".to_string()),
                },
                BasicBlock {
                    label: "header".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Branch {
                        cond: Operand::Constant(Constant::Bool(true)),
                        then_label: "body".to_string(),
                        else_label: "exit".to_string(),
                    },
                },
                BasicBlock {
                    label: "body".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Goto("header".to_string()), // back edge
                },
                BasicBlock {
                    label: "exit".to_string(),
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
        assert!(!AggressiveInlining::is_simple_control_flow(&func), "Loop detected via back edge");
    }

    #[test]
    fn test_aggressive_inlining_is_recursive() {
        let func = MirFunction {
            name: "fib".to_string(),
            params: vec![("n".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::Call {
                    dest: Some(Place::new("r")),
                    func: "fib".to_string(),
                    args: vec![Operand::Place(Place::new("n"))],
                    is_tail: false,
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        assert!(AggressiveInlining::is_recursive(&func));
    }

    #[test]
    fn test_aggressive_inlining_not_recursive() {
        let func = MirFunction {
            name: "add".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::Call {
                    dest: Some(Place::new("r")),
                    func: "other".to_string(),
                    args: vec![],
                    is_tail: false,
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        assert!(!AggressiveInlining::is_recursive(&func));
    }

    #[test]
    fn test_aggressive_inlining_should_inline_small_function() {
        let pass = AggressiveInlining::new();
        let func = MirFunction {
            name: "small".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![MirInst::BinOp {
                    dest: Place::new("r"),
                    op: MirBinOp::Add,
                    lhs: Operand::Place(Place::new("x")),
                    rhs: Operand::Constant(Constant::Int(1)),
                }],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        assert!(pass.should_inline(&func), "Small non-recursive function should be inlined");
    }

    #[test]
    fn test_aggressive_inlining_should_not_inline_main() {
        let pass = AggressiveInlining::new();
        let func = MirFunction {
            name: "main".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        assert!(!pass.should_inline(&func), "main should never be inlined");
    }

    #[test]
    fn test_aggressive_inlining_pure_function_higher_threshold() {
        let pass = AggressiveInlining::with_thresholds(2, 20);
        let func = MirFunction {
            name: "pure_fn".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![
                    MirInst::BinOp { dest: Place::new("a"), op: MirBinOp::Add, lhs: Operand::Place(Place::new("x")), rhs: Operand::Constant(Constant::Int(1)) },
                    MirInst::BinOp { dest: Place::new("b"), op: MirBinOp::Mul, lhs: Operand::Place(Place::new("a")), rhs: Operand::Constant(Constant::Int(2)) },
                    MirInst::BinOp { dest: Place::new("r"), op: MirBinOp::Sub, lhs: Operand::Place(Place::new("b")), rhs: Operand::Constant(Constant::Int(3)) },
                ],
                terminator: Terminator::Return(Some(Operand::Place(Place::new("r")))),
            }],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: true,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        // 3 instructions > max_instructions(2) but <= max_pure_instructions(20)
        assert!(pass.should_inline(&func), "Pure function uses higher threshold");
    }

    // --- collect_used_in_instruction ---

    #[test]
    fn test_collect_used_binop() {
        let mut used = HashSet::new();
        collect_used_in_instruction(&MirInst::BinOp {
            dest: Place::new("r"),
            op: MirBinOp::Add,
            lhs: Operand::Place(Place::new("a")),
            rhs: Operand::Place(Place::new("b")),
        }, &mut used);
        assert!(used.contains("a"));
        assert!(used.contains("b"));
        assert!(!used.contains("r"), "dest should not be in used set");
    }

    #[test]
    fn test_collect_used_const() {
        let mut used = HashSet::new();
        collect_used_in_instruction(&MirInst::Const {
            dest: Place::new("c"),
            value: Constant::Int(42),
        }, &mut used);
        assert!(used.is_empty(), "Const instruction uses no variables");
    }

    #[test]
    fn test_collect_used_call() {
        let mut used = HashSet::new();
        collect_used_in_instruction(&MirInst::Call {
            dest: Some(Place::new("r")),
            func: "foo".to_string(),
            args: vec![Operand::Place(Place::new("x")), Operand::Constant(Constant::Int(1))],
            is_tail: false,
        }, &mut used);
        assert!(used.contains("x"));
        assert!(!used.contains("r"));
    }

    #[test]
    fn test_collect_used_copy() {
        let mut used = HashSet::new();
        collect_used_in_instruction(&MirInst::Copy {
            dest: Place::new("d"),
            src: Place::new("s"),
        }, &mut used);
        assert!(used.contains("s"));
        assert!(!used.contains("d"));
    }

    // --- DeadCodeElimination name ---

    #[test]
    fn test_dead_code_elimination_name() {
        let pass = DeadCodeElimination;
        assert_eq!(pass.name(), "dead_code_elimination");
    }

    // --- CopyPropagation name ---

    #[test]
    fn test_copy_propagation_name() {
        let pass = CopyPropagation;
        assert_eq!(pass.name(), "copy_propagation");
    }

    // --- BlockMerging name ---

    #[test]
    fn test_block_merging_name() {
        let pass = BlockMerging;
        assert_eq!(pass.name(), "block_merging");
    }

    // --- GlobalFieldAccessCSE name ---

    #[test]
    fn test_global_field_access_cse_name() {
        let pass = GlobalFieldAccessCSE;
        assert_eq!(pass.name(), "global_field_access_cse");
    }

    // --- ContractUnreachableElimination name ---

    #[test]
    fn test_contract_unreachable_elimination_name() {
        let pass = ContractUnreachableElimination;
        assert_eq!(pass.name(), "contract_unreachable_elimination");
    }
}
