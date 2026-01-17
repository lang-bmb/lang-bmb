//! Middle Intermediate Representation (MIR)
//!
//! MIR is a CFG-based intermediate representation that sits between
//! the high-level AST and LLVM IR. It makes control flow explicit
//! through basic blocks and terminators.
//!
//! # Optimization (v0.29)
//!
//! The `optimize` module provides optimization passes that transform
//! MIR programs to improve performance. Key optimizations include:
//! - Constant folding and propagation
//! - Dead code elimination
//! - Common subexpression elimination
//! - Contract-based optimizations (BMB-specific)

mod lower;
mod optimize;

pub use lower::lower_program;
pub use optimize::{
    OptimizationPass, OptimizationPipeline, OptimizationStats, OptLevel,
    ConstantFolding, DeadCodeElimination, SimplifyBranches,
    CopyPropagation, CommonSubexpressionElimination, ContractBasedOptimization,
    ContractUnreachableElimination, PureFunctionCSE, ConstFunctionEval,
};

use std::collections::HashMap;

/// A MIR program containing all functions
#[derive(Debug, Clone)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
    /// External function declarations (v0.13.0)
    pub extern_fns: Vec<MirExternFn>,
}

/// External function declaration (v0.13.0)
/// These are imported from external modules (WASI, libc, etc.)
#[derive(Debug, Clone)]
pub struct MirExternFn {
    /// External module name (e.g., "wasi_snapshot_preview1")
    pub module: String,
    /// Function name
    pub name: String,
    /// Parameter types
    pub params: Vec<MirType>,
    /// Return type
    pub ret_ty: MirType,
}

/// A MIR function with explicit control flow
#[derive(Debug, Clone)]
pub struct MirFunction {
    /// Function name
    pub name: String,
    /// Function parameters with their types
    pub params: Vec<(String, MirType)>,
    /// Return type
    pub ret_ty: MirType,
    /// Local variable declarations
    pub locals: Vec<(String, MirType)>,
    /// Basic blocks (first block is entry)
    pub blocks: Vec<BasicBlock>,
    /// v0.38: Contract information for optimization
    /// Preconditions proven at function entry (e.g., "x >= 0", "len > 0")
    pub preconditions: Vec<ContractFact>,
    /// Postconditions guaranteed at function exit (e.g., "ret >= 0")
    pub postconditions: Vec<ContractFact>,
    /// v0.38.3: Function is marked @pure (no side effects, deterministic)
    /// Pure functions can be optimized with CSE - duplicate calls eliminated
    pub is_pure: bool,
    /// v0.38.4: Function is marked @const (compile-time evaluatable)
    /// Const functions are pure + can be evaluated at compile time with constant args
    pub is_const: bool,
}

/// v0.38: A proven fact from a contract condition
/// Used by ContractBasedOptimization to eliminate redundant checks
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractFact {
    /// Variable comparison: var op constant (e.g., x >= 0)
    VarCmp {
        var: String,
        op: CmpOp,
        value: i64,
    },
    /// Variable-variable comparison: var1 op var2 (e.g., start <= end)
    VarVarCmp {
        lhs: String,
        op: CmpOp,
        rhs: String,
    },
    /// Array bounds: index < len(array)
    ArrayBounds {
        index: String,
        array: String,
    },
    /// Non-null guarantee
    NonNull {
        var: String,
    },
}

/// Comparison operator for contract facts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Lt,  // <
    Le,  // <=
    Gt,  // >
    Ge,  // >=
    Eq,  // ==
    Ne,  // !=
}

/// A basic block containing instructions and a terminator
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Block label (unique within function)
    pub label: String,
    /// Instructions in the block
    pub instructions: Vec<MirInst>,
    /// Block terminator (branch, return, etc.)
    pub terminator: Terminator,
}

/// MIR instruction (non-terminating)
#[derive(Debug, Clone)]
pub enum MirInst {
    /// Assign a constant to a place: %dest = const value
    Const {
        dest: Place,
        value: Constant,
    },
    /// Copy from one place to another: %dest = %src
    Copy {
        dest: Place,
        src: Place,
    },
    /// Binary operation: %dest = %lhs op %rhs
    BinOp {
        dest: Place,
        op: MirBinOp,
        lhs: Operand,
        rhs: Operand,
    },
    /// Unary operation: %dest = op %src
    UnaryOp {
        dest: Place,
        op: MirUnaryOp,
        src: Operand,
    },
    /// Function call: %dest = call func(args...)
    Call {
        dest: Option<Place>,
        func: String,
        args: Vec<Operand>,
    },
    /// PHI node for SSA: %dest = phi [(value1, label1), (value2, label2), ...]
    Phi {
        dest: Place,
        values: Vec<(Operand, String)>, // (value, source_block_label)
    },
    /// v0.19.0: Struct initialization: %dest = struct { field1: val1, field2: val2, ... }
    StructInit {
        dest: Place,
        struct_name: String,
        fields: Vec<(String, Operand)>, // (field_name, value)
    },
    /// v0.19.0: Field access: %dest = %base.field
    FieldAccess {
        dest: Place,
        base: Place,
        field: String,
    },
    /// v0.19.0: Field store: %base.field = %value
    FieldStore {
        base: Place,
        field: String,
        value: Operand,
    },
    /// v0.19.1: Enum variant creation: %dest = EnumName::Variant(args)
    EnumVariant {
        dest: Place,
        enum_name: String,
        variant: String,
        args: Vec<Operand>,
    },
    /// v0.19.3: Array initialization with literal elements: %dest = [elem1, elem2, ...]
    ArrayInit {
        dest: Place,
        element_type: MirType,
        elements: Vec<Operand>,
    },
    /// v0.19.3: Array index load: `%dest = %array[%index]`
    IndexLoad {
        dest: Place,
        array: Place,
        index: Operand,
    },
    /// v0.19.3: Array index store: `%array[%index] = %value`
    IndexStore {
        array: Place,
        index: Operand,
        value: Operand,
    },
}

/// Block terminator (control flow)
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Return from function: return %value or return
    Return(Option<Operand>),
    /// Unconditional jump: goto label
    Goto(String),
    /// Conditional branch: if %cond then label1 else label2
    Branch {
        cond: Operand,
        then_label: String,
        else_label: String,
    },
    /// Unreachable (for optimization)
    Unreachable,
    /// v0.19.2: Switch for pattern matching
    /// switch %discriminant { case val1 -> label1, case val2 -> label2, ... } default -> default_label
    Switch {
        discriminant: Operand,
        cases: Vec<(i64, String)>, // (value, target_label)
        default: String,
    },
}

/// An operand in MIR (either a place or constant)
#[derive(Debug, Clone)]
pub enum Operand {
    /// Reference to a place (variable/temporary)
    Place(Place),
    /// Constant value
    Constant(Constant),
}

/// A place represents a memory location (variable or temporary)
#[derive(Debug, Clone)]
pub struct Place {
    pub name: String,
}

impl Place {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Constant value
#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    /// Character constant (v0.64)
    Char(char),
    Unit,
}

/// MIR binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirBinOp {
    // Integer arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // v0.37: Wrapping integer arithmetic (no overflow panic)
    AddWrap,
    SubWrap,
    MulWrap,
    // v0.38: Checked integer arithmetic (returns Option<T>)
    AddChecked,
    SubChecked,
    MulChecked,
    // v0.38: Saturating integer arithmetic (clamps to min/max)
    AddSat,
    SubSat,
    MulSat,
    // Floating-point arithmetic
    FAdd,
    FSub,
    FMul,
    FDiv,
    // Integer comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    // Floating-point comparison
    FEq,
    FNe,
    FLt,
    FGt,
    FLe,
    FGe,
    // Logical
    And,
    Or,
    // v0.32: Shift operators
    Shl,
    Shr,
    // v0.36: Bitwise operators
    Band,
    Bor,
    Bxor,
    // v0.36: Logical implication
    Implies,
}

impl MirBinOp {
    /// v0.35.4: Returns the result type of a binary operation given the operand type
    pub fn result_type(&self, operand_ty: &MirType) -> MirType {
        match self {
            // Arithmetic ops return same type as operands
            MirBinOp::Add | MirBinOp::Sub | MirBinOp::Mul | MirBinOp::Div | MirBinOp::Mod |
            // v0.37: Wrapping arithmetic also returns same type
            MirBinOp::AddWrap | MirBinOp::SubWrap | MirBinOp::MulWrap |
            // v0.38: Checked arithmetic (Option wrapper handled at type level)
            MirBinOp::AddChecked | MirBinOp::SubChecked | MirBinOp::MulChecked |
            // v0.38: Saturating arithmetic
            MirBinOp::AddSat | MirBinOp::SubSat | MirBinOp::MulSat => {
                operand_ty.clone()
            }
            // Float arithmetic returns f64
            MirBinOp::FAdd | MirBinOp::FSub | MirBinOp::FMul | MirBinOp::FDiv => MirType::F64,
            // All comparisons return bool
            MirBinOp::Eq | MirBinOp::Ne | MirBinOp::Lt | MirBinOp::Gt | MirBinOp::Le | MirBinOp::Ge |
            MirBinOp::FEq | MirBinOp::FNe | MirBinOp::FLt | MirBinOp::FGt | MirBinOp::FLe | MirBinOp::FGe => {
                MirType::Bool
            }
            // Logical ops return bool
            MirBinOp::And | MirBinOp::Or => MirType::Bool,
            // v0.32: Shift ops return same type as left operand
            MirBinOp::Shl | MirBinOp::Shr => operand_ty.clone(),
            // v0.36: Bitwise ops return same type as operands (integer)
            MirBinOp::Band | MirBinOp::Bor | MirBinOp::Bxor => operand_ty.clone(),
            // v0.36: Logical implication returns bool
            MirBinOp::Implies => MirType::Bool,
        }
    }
}

/// MIR unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirUnaryOp {
    /// Integer negation
    Neg,
    /// Floating-point negation
    FNeg,
    /// Logical not
    Not,
    /// v0.36: Bitwise not
    Bnot,
}

impl MirUnaryOp {
    /// v0.35.4: Returns the result type of a unary operation given the operand type
    pub fn result_type(&self, operand_ty: &MirType) -> MirType {
        match self {
            MirUnaryOp::Neg => operand_ty.clone(),
            MirUnaryOp::FNeg => MirType::F64,
            MirUnaryOp::Not => MirType::Bool,
            // v0.36: Bitwise not returns same type as operand (integer)
            MirUnaryOp::Bnot => operand_ty.clone(),
        }
    }
}

/// MIR type system (simplified from AST types)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirType {
    I32,
    I64,
    // v0.38: Unsigned integer types
    U32,
    U64,
    F64,
    Bool,
    String,
    /// Character type (v0.64)
    Char,
    Unit,
    /// v0.19.0: Struct type with name and field types
    Struct {
        name: String,
        fields: Vec<(String, Box<MirType>)>,
    },
    /// v0.19.0: Pointer to a struct (for references)
    StructPtr(String),
    /// v0.19.1: Enum type with name and variant types
    Enum {
        name: String,
        variants: Vec<(String, Vec<Box<MirType>>)>, // (variant_name, arg_types)
    },
    /// v0.19.3: Array type with element type and optional fixed size
    Array {
        element_type: Box<MirType>,
        size: Option<usize>, // None for dynamic arrays (slices)
    },
}

impl MirType {
    pub fn is_integer(&self) -> bool {
        matches!(self, MirType::I32 | MirType::I64)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, MirType::F64)
    }
}

/// Context for MIR lowering
#[derive(Debug)]
pub struct LoweringContext {
    /// Counter for generating unique temporary names
    temp_counter: usize,
    /// Counter for generating unique block labels
    block_counter: usize,
    /// Current basic blocks being built
    pub blocks: Vec<BasicBlock>,
    /// Current block's instructions
    current_instructions: Vec<MirInst>,
    /// Current block label
    current_label: String,
    /// Local variable types
    pub locals: HashMap<String, MirType>,
    /// Function parameter types
    pub params: HashMap<String, MirType>,
    /// v0.35.4: Function return types for Call type inference
    pub func_return_types: HashMap<String, MirType>,
}

impl LoweringContext {
    pub fn new() -> Self {
        // v0.35.4: Initialize with built-in function return types
        let mut func_return_types = HashMap::new();
        // Math intrinsics
        func_return_types.insert("sqrt".to_string(), MirType::F64);
        func_return_types.insert("abs".to_string(), MirType::I64);
        func_return_types.insert("min".to_string(), MirType::I64);
        func_return_types.insert("max".to_string(), MirType::I64);
        // Type conversions
        func_return_types.insert("i64_to_f64".to_string(), MirType::F64);
        func_return_types.insert("f64_to_i64".to_string(), MirType::I64);
        // I/O
        func_return_types.insert("read_int".to_string(), MirType::I64);
        // Void functions return Unit
        func_return_types.insert("println".to_string(), MirType::Unit);
        func_return_types.insert("print".to_string(), MirType::Unit);
        func_return_types.insert("assert".to_string(), MirType::Unit);

        Self {
            temp_counter: 0,
            block_counter: 0,
            blocks: Vec::new(),
            current_instructions: Vec::new(),
            current_label: "entry".to_string(),
            locals: HashMap::new(),
            params: HashMap::new(),
            func_return_types,
        }
    }

    /// Generate a fresh temporary name
    pub fn fresh_temp(&mut self) -> Place {
        let name = format!("_t{}", self.temp_counter);
        self.temp_counter += 1;
        Place::new(name)
    }

    /// Generate a fresh block label
    pub fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.block_counter);
        self.block_counter += 1;
        label
    }

    /// Add an instruction to the current block
    pub fn push_inst(&mut self, inst: MirInst) {
        self.current_instructions.push(inst);
    }

    /// Finish the current block with a terminator
    pub fn finish_block(&mut self, terminator: Terminator) {
        let block = BasicBlock {
            label: self.current_label.clone(),
            instructions: std::mem::take(&mut self.current_instructions),
            terminator,
        };
        self.blocks.push(block);
    }

    /// Start a new block
    pub fn start_block(&mut self, label: String) {
        self.current_label = label;
        self.current_instructions = Vec::new();
    }

    /// Get the current block label
    pub fn current_block_label(&self) -> &str {
        &self.current_label
    }

    /// Get type of an operand
    pub fn operand_type(&self, op: &Operand) -> MirType {
        match op {
            Operand::Constant(c) => match c {
                Constant::Int(_) => MirType::I64,
                Constant::Float(_) => MirType::F64,
                Constant::Bool(_) => MirType::Bool,
                Constant::String(_) => MirType::String,
                // v0.64: Character type
                Constant::Char(_) => MirType::Char,
                Constant::Unit => MirType::Unit,
            },
            Operand::Place(p) => {
                if let Some(ty) = self.locals.get(&p.name) {
                    ty.clone()
                } else if let Some(ty) = self.params.get(&p.name) {
                    ty.clone()
                } else {
                    // Temporary - infer from usage or default to i64
                    MirType::I64
                }
            }
        }
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MIR Text Formatting (v0.21.2)
// Formats MIR to text format matching Bootstrap compiler output
// ============================================================================

/// Format a MIR program to text format
pub fn format_mir(program: &MirProgram) -> String {
    let mut output = String::new();

    for (i, func) in program.functions.iter().enumerate() {
        if i > 0 {
            output.push_str("\n\n");
        }
        output.push_str(&format_mir_function(func));
    }

    output
}

/// Format a single MIR function
fn format_mir_function(func: &MirFunction) -> String {
    let mut out = String::new();

    // Function header
    let params_str: Vec<_> = func.params.iter()
        .map(|(name, ty)| format!("{}: {}", name, format_mir_type(ty)))
        .collect();
    out.push_str(&format!("fn {}({}) -> {} {{\n",
        func.name,
        params_str.join(", "),
        format_mir_type(&func.ret_ty)));

    // Blocks
    for block in &func.blocks {
        out.push_str(&format!("{}:\n", block.label));

        // Instructions
        for inst in &block.instructions {
            out.push_str(&format!("  {}\n", format_mir_inst(inst)));
        }

        // Terminator
        out.push_str(&format!("  {}\n", format_terminator(&block.terminator)));
    }

    out.push_str("}\n");
    out
}

/// Format a MIR instruction
fn format_mir_inst(inst: &MirInst) -> String {
    match inst {
        MirInst::Const { dest, value } => {
            format!("%{} = const {}", dest.name, format_constant(value))
        }
        MirInst::Copy { dest, src } => {
            format!("%{} = copy %{}", dest.name, src.name)
        }
        MirInst::BinOp { dest, op, lhs, rhs } => {
            format!("%{} = {} {}, {}",
                dest.name,
                format_binop(*op),
                format_operand(lhs),
                format_operand(rhs))
        }
        MirInst::UnaryOp { dest, op, src } => {
            format!("%{} = {} {}", dest.name, format_unaryop(*op), format_operand(src))
        }
        MirInst::Call { dest, func, args } => {
            let args_str: Vec<_> = args.iter().map(format_operand).collect();
            if let Some(d) = dest {
                format!("%{} = call {}({})", d.name, func, args_str.join(", "))
            } else {
                format!("call {}({})", func, args_str.join(", "))
            }
        }
        MirInst::Phi { dest, values } => {
            let vals: Vec<_> = values.iter()
                .map(|(v, lbl)| format!("[{}, {}]", format_operand(v), lbl))
                .collect();
            format!("%{} = phi {}", dest.name, vals.join(", "))
        }
        MirInst::StructInit { dest, struct_name, fields } => {
            let fields_str: Vec<_> = fields.iter()
                .map(|(name, val)| format!("{}: {}", name, format_operand(val)))
                .collect();
            format!("%{} = struct-init {} {{ {} }}", dest.name, struct_name, fields_str.join(", "))
        }
        MirInst::FieldAccess { dest, base, field } => {
            format!("%{} = field-access %{}.{}", dest.name, base.name, field)
        }
        MirInst::FieldStore { base, field, value } => {
            format!("%{}.{} = {}", base.name, field, format_operand(value))
        }
        MirInst::EnumVariant { dest, enum_name, variant, args } => {
            if args.is_empty() {
                format!("%{} = enum-variant {}::{} 0", dest.name, enum_name, variant)
            } else {
                let args_str: Vec<_> = args.iter().map(format_operand).collect();
                format!("%{} = enum-variant {}::{} 1 {}", dest.name, enum_name, variant, args_str.join(", "))
            }
        }
        MirInst::ArrayInit { dest, element_type: _, elements } => {
            let elems: Vec<_> = elements.iter().map(format_operand).collect();
            format!("%{} = array-init [{}]", dest.name, elems.join(", "))
        }
        MirInst::IndexLoad { dest, array, index } => {
            format!("%{} = index-load %{}[{}]", dest.name, array.name, format_operand(index))
        }
        MirInst::IndexStore { array, index, value } => {
            format!("%{}[{}] = {}", array.name, format_operand(index), format_operand(value))
        }
    }
}

/// Format a terminator
fn format_terminator(term: &Terminator) -> String {
    match term {
        Terminator::Return(None) => "return".to_string(),
        Terminator::Return(Some(op)) => format!("return {}", format_operand(op)),
        Terminator::Goto(label) => format!("goto {}", label),
        Terminator::Branch { cond, then_label, else_label } => {
            format!("branch {}, {}, {}", format_operand(cond), then_label, else_label)
        }
        Terminator::Unreachable => "unreachable".to_string(),
        Terminator::Switch { discriminant, cases, default } => {
            let cases_str: Vec<_> = cases.iter()
                .map(|(val, lbl)| format!("{} -> {}", val, lbl))
                .collect();
            format!("switch {}, [{}], {}", format_operand(discriminant), cases_str.join(", "), default)
        }
    }
}

/// Format an operand
fn format_operand(op: &Operand) -> String {
    match op {
        Operand::Place(p) => format!("%{}", p.name),
        Operand::Constant(c) => format_constant(c),
    }
}

/// Format a constant value
fn format_constant(c: &Constant) -> String {
    match c {
        Constant::Int(n) => format!("I:{}", n),
        Constant::Float(f) => format!("F:{}", f),
        Constant::Bool(b) => format!("B:{}", if *b { 1 } else { 0 }),
        Constant::String(s) => format!("S:\"{}\"", s),
        // v0.64: Character constant
        Constant::Char(c) => format!("C:'{}'", c.escape_default()),
        Constant::Unit => "U".to_string(),
    }
}

/// Format a binary operator
fn format_binop(op: MirBinOp) -> String {
    match op {
        MirBinOp::Add => "+",
        MirBinOp::Sub => "-",
        MirBinOp::Mul => "*",
        MirBinOp::Div => "/",
        MirBinOp::Mod => "%",
        // v0.37: Wrapping arithmetic
        MirBinOp::AddWrap => "+%",
        MirBinOp::SubWrap => "-%",
        MirBinOp::MulWrap => "*%",
        // v0.38: Checked arithmetic
        MirBinOp::AddChecked => "+?",
        MirBinOp::SubChecked => "-?",
        MirBinOp::MulChecked => "*?",
        // v0.38: Saturating arithmetic
        MirBinOp::AddSat => "+|",
        MirBinOp::SubSat => "-|",
        MirBinOp::MulSat => "*|",
        MirBinOp::FAdd => "+.",
        MirBinOp::FSub => "-.",
        MirBinOp::FMul => "*.",
        MirBinOp::FDiv => "/.",
        MirBinOp::Eq => "==",
        MirBinOp::Ne => "!=",
        MirBinOp::Lt => "<",
        MirBinOp::Gt => ">",
        MirBinOp::Le => "<=",
        MirBinOp::Ge => ">=",
        MirBinOp::FEq => "==.",
        MirBinOp::FNe => "!=.",
        MirBinOp::FLt => "<.",
        MirBinOp::FGt => ">.",
        MirBinOp::FLe => "<=.",
        MirBinOp::FGe => ">=.",
        MirBinOp::And => "and",
        MirBinOp::Or => "or",
        // v0.32: Shift operators
        MirBinOp::Shl => "<<",
        MirBinOp::Shr => ">>",
        // v0.36: Bitwise operators
        MirBinOp::Band => "band",
        MirBinOp::Bor => "bor",
        MirBinOp::Bxor => "bxor",
        // v0.36: Logical implication
        MirBinOp::Implies => "implies",
    }.to_string()
}

/// Format a unary operator
fn format_unaryop(op: MirUnaryOp) -> String {
    match op {
        MirUnaryOp::Neg => "neg",
        MirUnaryOp::FNeg => "fneg",
        MirUnaryOp::Not => "not",
        // v0.36: Bitwise not
        MirUnaryOp::Bnot => "bnot",
    }.to_string()
}

/// Format a MIR type
fn format_mir_type(ty: &MirType) -> String {
    match ty {
        MirType::I32 => "i32".to_string(),
        MirType::I64 => "i64".to_string(),
        // v0.38: Unsigned types
        MirType::U32 => "u32".to_string(),
        MirType::U64 => "u64".to_string(),
        MirType::F64 => "f64".to_string(),
        MirType::Bool => "bool".to_string(),
        MirType::String => "String".to_string(),
        // v0.64: Character type
        MirType::Char => "char".to_string(),
        MirType::Unit => "()".to_string(),
        MirType::Struct { name, .. } => name.clone(),
        MirType::StructPtr(name) => format!("&{}", name),
        MirType::Enum { name, .. } => name.clone(),
        MirType::Array { element_type, size } => {
            if let Some(s) = size {
                format!("[{}; {}]", format_mir_type(element_type), s)
            } else {
                format!("[{}]", format_mir_type(element_type))
            }
        }
    }
}
