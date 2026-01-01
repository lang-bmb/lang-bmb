//! Middle Intermediate Representation (MIR)
//!
//! MIR is a CFG-based intermediate representation that sits between
//! the high-level AST and LLVM IR. It makes control flow explicit
//! through basic blocks and terminators.

mod lower;

pub use lower::lower_program;

use std::collections::HashMap;

/// A MIR program containing all functions
#[derive(Debug, Clone)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
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
}

/// MIR type system (simplified from AST types)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirType {
    I32,
    I64,
    F64,
    Bool,
    String,
    Unit,
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
}

impl LoweringContext {
    pub fn new() -> Self {
        Self {
            temp_counter: 0,
            block_counter: 0,
            blocks: Vec::new(),
            current_instructions: Vec::new(),
            current_label: "entry".to_string(),
            locals: HashMap::new(),
            params: HashMap::new(),
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

    /// Get type of an operand
    pub fn operand_type(&self, op: &Operand) -> MirType {
        match op {
            Operand::Constant(c) => match c {
                Constant::Int(_) => MirType::I64,
                Constant::Float(_) => MirType::F64,
                Constant::Bool(_) => MirType::Bool,
                Constant::String(_) => MirType::String,
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
