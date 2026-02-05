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
pub mod proof_guided;

pub use lower::lower_program;
pub use optimize::{
    OptimizationPass, OptimizationPipeline, OptimizationStats, OptLevel,
    ConstantFolding, DeadCodeElimination, SimplifyBranches, IfElseToSwitch,
    CopyPropagation, CommonSubexpressionElimination, ContractBasedOptimization,
    ContractUnreachableElimination, PureFunctionCSE, ConstFunctionEval,
    ConstantPropagationNarrowing, LoopBoundedNarrowing, AggressiveInlining,
    LinearRecurrenceToLoop,
};
pub use proof_guided::{
    BoundsCheckElimination, NullCheckElimination, DivisionCheckElimination,
    ProofUnreachableElimination, ProvenFactSet, ProofOptimizationStats,
    run_proof_guided_optimizations, run_proof_guided_program,
};

use std::collections::HashMap;

/// A MIR program containing all functions
#[derive(Debug, Clone)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
    /// External function declarations (v0.13.0)
    pub extern_fns: Vec<MirExternFn>,
    /// v0.51.31: Struct type definitions for codegen
    /// Maps struct name -> list of (field_name, field_type)
    pub struct_defs: HashMap<String, Vec<(String, MirType)>>,
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
    /// v0.51.8: Function should be aggressively inlined
    /// Set by AggressiveInlining pass for small pure functions
    pub always_inline: bool,
    /// v0.51.52: Function should be preferentially inlined by LLVM
    /// Set by AggressiveInlining pass for medium-sized functions that don't qualify
    /// for alwaysinline but would benefit from inlining in hot loops
    pub inline_hint: bool,
    /// v0.51.11: Function does not access memory (only arithmetic/comparisons)
    /// Set by MemoryEffectAnalysis pass. Used for LLVM memory(none) attribute.
    pub is_memory_free: bool,
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
    /// v0.50.65: Added is_tail for tail call optimization
    Call {
        dest: Option<Place>,
        func: String,
        args: Vec<Operand>,
        /// If true, this call is in tail position and can use musttail
        is_tail: bool,
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
    /// v0.51.23: Added field_index for correct getelementptr codegen
    /// v0.51.31: Added struct_name for correct field type lookup in codegen
    FieldAccess {
        dest: Place,
        base: Place,
        field: String,
        /// Index of the field in struct definition (for getelementptr offset)
        field_index: usize,
        /// Name of the struct type (for field type lookup)
        struct_name: String,
    },
    /// v0.19.0: Field store: %base.field = %value
    /// v0.51.23: Added field_index for correct getelementptr codegen
    /// v0.51.31: Added struct_name for correct field type lookup in codegen
    FieldStore {
        base: Place,
        field: String,
        /// Index of the field in struct definition (for getelementptr offset)
        field_index: usize,
        /// Name of the struct type (for field type lookup)
        struct_name: String,
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
    /// v0.51.35: Added element_type for proper struct array handling
    IndexLoad {
        dest: Place,
        array: Place,
        index: Operand,
        element_type: MirType,
    },
    /// v0.19.3: Array index store: `%array[%index] = %value`
    /// v0.51.35: Added element_type for proper struct array handling
    IndexStore {
        array: Place,
        index: Operand,
        value: Operand,
        element_type: MirType,
    },
    /// v0.55: Tuple initialization with heterogeneous elements
    /// Used for native tuple returns to enable struct-based LLVM codegen
    TupleInit {
        dest: Place,
        /// Each element has its type and value
        elements: Vec<(MirType, Operand)>,
    },
    /// v0.55: Tuple field extraction by index
    /// Gets the Nth element from a tuple (compile-time constant index)
    TupleExtract {
        dest: Place,
        tuple: Place,
        index: usize,
        element_type: MirType,
    },
    /// v0.50.80: Type cast: %dest = cast %src from_ty to to_ty
    /// Generates: sext/zext/trunc/fpext/fptosi/sitofp depending on types
    Cast {
        dest: Place,
        src: Operand,
        from_ty: MirType,
        to_ty: MirType,
    },
    /// v0.60.19: Pointer offset: %dest = %ptr + %offset (scaled by element size)
    /// Generates LLVM GEP instruction for proper alias analysis
    PtrOffset {
        dest: Place,
        ptr: Operand,
        offset: Operand,
        element_type: MirType,
    },
    /// v0.60.21: Stack array allocation without initialization
    /// Used for: `let arr: [T; N];`
    /// Generates LLVM alloca for zero-overhead stack arrays
    ArrayAlloc {
        dest: Place,
        element_type: MirType,
        size: usize,
    },
    /// v0.60.20: Pointer load (dereference): %dest = *%ptr
    /// Generates native LLVM load instruction for proper alias analysis
    /// This enables LLVM to optimize pointer-based code (vectorization, LICM)
    PtrLoad {
        dest: Place,
        ptr: Operand,
        element_type: MirType,
    },
    /// v0.60.20: Pointer store: *%ptr = %value
    /// Generates native LLVM store instruction for proper alias analysis
    PtrStore {
        ptr: Operand,
        value: Operand,
        element_type: MirType,
    },

    // v0.70: Concurrency instructions

    /// Spawn a new thread: %dest = spawn(func_ptr, captures_ptr)
    /// Creates a new thread that executes the given function with captured variables.
    /// Returns a thread handle (i64) that can be used with ThreadJoin.
    ThreadSpawn {
        dest: Place,
        /// Name of the synthetic function to execute in the spawned thread
        func: String,
        /// Captured variables to pass to the spawned function
        captures: Vec<Operand>,
    },

    /// Join a thread (blocking): %dest = join(%handle)
    /// Waits for the thread to complete and returns its result.
    ThreadJoin {
        dest: Option<Place>,
        /// Thread handle returned by ThreadSpawn
        handle: Operand,
    },

    // v0.71: Mutex instructions

    /// Create a new mutex: %dest = mutex-new(%initial_value)
    MutexNew {
        dest: Place,
        initial_value: Operand,
    },

    /// Lock a mutex: %guard = mutex-lock(%mutex)
    /// Blocks until the mutex is acquired. Returns the guarded value.
    MutexLock {
        dest: Place,
        mutex: Operand,
    },

    /// Unlock a mutex: mutex-unlock(%mutex, %new_value)
    /// Releases the lock and updates the stored value.
    MutexUnlock {
        mutex: Operand,
        new_value: Operand,
    },

    /// Try to lock a mutex (non-blocking): %result = mutex-try-lock(%mutex)
    /// Returns Some(value) if lock acquired, None if contended.
    MutexTryLock {
        dest: Place,
        mutex: Operand,
    },

    /// Free a mutex: mutex-free(%mutex)
    MutexFree {
        mutex: Operand,
    },

    // v0.72: Arc instructions

    /// Create a new Arc: %dest = arc-new(%value)
    ArcNew {
        dest: Place,
        value: Operand,
    },

    /// Clone an Arc: %dest = arc-clone(%arc)
    ArcClone {
        dest: Place,
        arc: Operand,
    },

    /// Get value from Arc: %dest = arc-get(%arc)
    ArcGet {
        dest: Place,
        arc: Operand,
    },

    /// Drop an Arc (decrement refcount): arc-drop(%arc)
    ArcDrop {
        arc: Operand,
    },

    /// Get strong count: %dest = arc-strong-count(%arc)
    ArcStrongCount {
        dest: Place,
        arc: Operand,
    },

    // v0.72: Atomic instructions

    /// Create a new Atomic: %dest = atomic-new(%value)
    AtomicNew {
        dest: Place,
        value: Operand,
    },

    /// Atomic load: %dest = atomic-load(%ptr)
    AtomicLoad {
        dest: Place,
        ptr: Operand,
    },

    /// Atomic store: atomic-store(%ptr, %value)
    AtomicStore {
        ptr: Operand,
        value: Operand,
    },

    /// Atomic fetch-add: %dest = atomic-fetch-add(%ptr, %delta)
    AtomicFetchAdd {
        dest: Place,
        ptr: Operand,
        delta: Operand,
    },

    /// Atomic fetch-sub: %dest = atomic-fetch-sub(%ptr, %delta)
    AtomicFetchSub {
        dest: Place,
        ptr: Operand,
        delta: Operand,
    },

    /// Atomic swap: %dest = atomic-swap(%ptr, %new_value)
    AtomicSwap {
        dest: Place,
        ptr: Operand,
        new_value: Operand,
    },

    /// Atomic compare-exchange: %dest = atomic-cmpxchg(%ptr, %expected, %new_value)
    AtomicCompareExchange {
        dest: Place,
        ptr: Operand,
        expected: Operand,
        new_value: Operand,
    },

    // v0.73: Channel instructions

    /// Create a new channel: (sender, receiver) = channel-new(capacity)
    ChannelNew {
        sender_dest: Place,
        receiver_dest: Place,
        capacity: Operand,
    },

    /// Send a value on a channel: channel-send(sender, value)
    ChannelSend {
        sender: Operand,
        value: Operand,
    },

    /// Receive a value from a channel: %dest = channel-recv(receiver)
    ChannelRecv {
        dest: Place,
        receiver: Operand,
    },

    /// Try to send (non-blocking): %success = channel-try-send(sender, value)
    ChannelTrySend {
        dest: Place,
        sender: Operand,
        value: Operand,
    },

    /// Try to receive (non-blocking): %result = channel-try-recv(receiver)
    ChannelTryRecv {
        dest: Place,
        receiver: Operand,
    },

    /// v0.77: Receive with timeout: %dest = channel-recv-timeout(%receiver, %timeout_ms)
    /// Returns received value if successful within timeout, -1 if timeout
    ChannelRecvTimeout {
        dest: Place,
        receiver: Operand,
        timeout_ms: Operand,
    },

    /// v0.78: Block on a future: %dest = block-on(future)
    /// Runs the executor until the future completes
    BlockOn {
        dest: Place,
        future: Operand,
    },

    /// v0.79: Send with timeout: %dest = channel-send-timeout(%sender, %value, %timeout_ms)
    /// Returns 1 if sent, 0 if timeout
    ChannelSendTimeout {
        dest: Place,
        sender: Operand,
        value: Operand,
        timeout_ms: Operand,
    },

    /// Clone a sender: %dest = sender-clone(sender)
    SenderClone {
        dest: Place,
        sender: Operand,
    },

    // v0.74: RwLock instructions

    /// Create a new RwLock: %dest = rwlock-new(value)
    RwLockNew {
        dest: Place,
        initial_value: Operand,
    },

    /// Acquire read lock: %dest = rwlock-read(rwlock)
    RwLockRead {
        dest: Place,
        rwlock: Operand,
    },

    /// Release read lock: rwlock-read-unlock(rwlock)
    RwLockReadUnlock {
        rwlock: Operand,
    },

    /// Acquire write lock: %dest = rwlock-write(rwlock)
    RwLockWrite {
        dest: Place,
        rwlock: Operand,
    },

    /// Release write lock: rwlock-write-unlock(rwlock, value)
    RwLockWriteUnlock {
        rwlock: Operand,
        value: Operand,
    },

    // v0.74: Barrier instructions

    /// Create a new Barrier: %dest = barrier-new(count)
    BarrierNew {
        dest: Place,
        count: Operand,
    },

    /// Wait at barrier: %dest = barrier-wait(barrier)
    BarrierWait {
        dest: Place,
        barrier: Operand,
    },

    // v0.74: Condvar instructions

    /// Create a new Condvar: %dest = condvar-new()
    CondvarNew {
        dest: Place,
    },

    /// Wait on condvar: %dest = condvar-wait(condvar, mutex)
    CondvarWait {
        dest: Place,
        condvar: Operand,
        mutex: Operand,
    },

    /// Notify one thread: condvar-notify-one(condvar)
    CondvarNotifyOne {
        condvar: Operand,
    },

    /// Notify all threads: condvar-notify-all(condvar)
    CondvarNotifyAll {
        condvar: Operand,
    },

    // v0.76: Select instruction (ternary)
    /// Select based on comparison: %dest = select(%lhs op %rhs ? %true_val : %false_val)
    Select {
        dest: Place,
        cond_op: MirBinOp,
        cond_lhs: Operand,
        cond_rhs: Operand,
        true_val: Operand,
        false_val: Operand,
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
    /// v0.51.37: Raw pointer type for heap-allocated data
    /// Used for proper LLVM codegen with typed pointers
    Ptr(Box<MirType>),
    /// v0.55: Tuple type with heterogeneous element types
    /// Used for native tuple returns and multiple return values
    Tuple(Vec<Box<MirType>>),
}

impl MirType {
    pub fn is_integer(&self) -> bool {
        matches!(self, MirType::I32 | MirType::I64)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, MirType::F64)
    }

    /// v0.60.1: Check if this is a pointer type (Ptr, StructPtr, or String)
    /// v0.60.32: Include String since it's represented as ptr in LLVM
    pub fn is_pointer_type(&self) -> bool {
        matches!(self, MirType::Ptr(_) | MirType::StructPtr(_) | MirType::String)
    }

    /// v0.60.19: Get element type from a pointer type
    /// Returns None if not a pointer type
    pub fn pointer_element_type(&self) -> Option<MirType> {
        match self {
            MirType::Ptr(inner) => Some(*inner.clone()),
            MirType::StructPtr(name) => Some(MirType::Struct {
                name: name.clone(),
                fields: vec![], // Fields not known at this level
            }),
            _ => None,
        }
    }
}

/// Context for MIR lowering
#[derive(Debug, Clone)]
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
    /// v0.51.23: Struct definitions for field index lookup
    /// Maps struct name -> list of field names (in declaration order)
    pub struct_defs: HashMap<String, Vec<String>>,
    /// v0.51.23: Struct type of variables
    /// Maps variable name -> struct name (for field index lookup)
    pub var_struct_types: HashMap<String, String>,
    /// v0.51.31: Struct type definitions with full type info for field type lookup
    /// Maps struct name -> list of (field_name, field_type)
    pub struct_type_defs: HashMap<String, Vec<(String, MirType)>>,
    /// v0.51.31: Temporary variable types for type inference
    /// Maps temp name -> type (used for temps that aren't in locals)
    pub temp_types: HashMap<String, MirType>,
    /// v0.51.35: Array element types for proper struct array handling
    /// Maps array variable name -> element type
    pub array_element_types: HashMap<String, MirType>,
    /// v0.60.16: Loop context stack for break/continue support
    /// Each entry is (continue_label, break_label) for the enclosing loop
    pub loop_context_stack: Vec<(String, String)>,
    /// v0.70: Counter for generating unique spawn function names
    pub spawn_counter: usize,
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
            struct_defs: HashMap::new(),
            var_struct_types: HashMap::new(),
            struct_type_defs: HashMap::new(),
            temp_types: HashMap::new(),
            array_element_types: HashMap::new(),
            loop_context_stack: Vec::new(),
            spawn_counter: 0,
        }
    }

    /// v0.51.23: Look up field index for a struct field
    /// Returns 0 if struct or field not found (fallback for unknown types)
    pub fn field_index(&self, struct_name: &str, field_name: &str) -> usize {
        if let Some(fields) = self.struct_defs.get(struct_name) {
            fields.iter().position(|f| f == field_name).unwrap_or(0)
        } else {
            0
        }
    }

    /// v0.51.31: Look up field type for a struct field
    /// Returns the MirType of the field, or I64 if not found
    pub fn field_type(&self, struct_name: &str, field_name: &str) -> MirType {
        if let Some(fields) = self.struct_type_defs.get(struct_name) {
            fields.iter()
                .find(|(name, _)| name == field_name)
                .map(|(_, ty)| ty.clone())
                .unwrap_or(MirType::I64)
        } else {
            MirType::I64
        }
    }

    /// v0.51.23: Get the struct type of a place (if known)
    pub fn place_struct_type(&self, place: &Place) -> Option<String> {
        self.var_struct_types.get(&place.name).cloned()
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

    /// v0.70: Alias for push_inst for consistency with other MIR builders
    pub fn emit(&mut self, inst: MirInst) {
        self.push_inst(inst);
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
                } else if let Some(ty) = self.temp_types.get(&p.name) {
                    // v0.51.31: Check temp_types for temporaries (e.g., from FieldAccess)
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

    // v0.51.44: Show function attributes
    let mut attrs = Vec::new();
    if func.is_pure { attrs.push("pure"); }
    if func.is_const { attrs.push("const"); }
    if func.always_inline { attrs.push("alwaysinline"); }
    if func.inline_hint { attrs.push("inlinehint"); }
    if func.is_memory_free { attrs.push("memory(none)"); }

    // Function header with optional attributes
    let params_str: Vec<_> = func.params.iter()
        .map(|(name, ty)| format!("{}: {}", name, format_mir_type(ty)))
        .collect();

    if attrs.is_empty() {
        out.push_str(&format!("fn {}({}) -> {} {{\n",
            func.name,
            params_str.join(", "),
            format_mir_type(&func.ret_ty)));
    } else {
        out.push_str(&format!("fn {}({}) -> {} @{} {{\n",
            func.name,
            params_str.join(", "),
            format_mir_type(&func.ret_ty),
            attrs.join(" @")));
    }

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
        MirInst::Call { dest, func, args, is_tail } => {
            let args_str: Vec<_> = args.iter().map(format_operand).collect();
            let tail_prefix = if *is_tail { "tail " } else { "" };
            if let Some(d) = dest {
                format!("%{} = {}call {}({})", d.name, tail_prefix, func, args_str.join(", "))
            } else {
                format!("{}call {}({})", tail_prefix, func, args_str.join(", "))
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
        MirInst::FieldAccess { dest, base, field, field_index, struct_name } => {
            format!("%{} = field-access %{}.{}[{}] ({})", dest.name, base.name, field, field_index, struct_name)
        }
        MirInst::FieldStore { base, field, field_index, struct_name, value } => {
            format!("%{}.{}[{}] ({}) = {}", base.name, field, field_index, struct_name, format_operand(value))
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
        MirInst::IndexLoad { dest, array, index, element_type } => {
            format!("%{} = index-load %{}[{}] : {}", dest.name, array.name, format_operand(index), format_mir_type(element_type))
        }
        MirInst::IndexStore { array, index, value, element_type } => {
            format!("%{}[{}] = {} : {}", array.name, format_operand(index), format_operand(value), format_mir_type(element_type))
        }
        MirInst::Cast { dest, src, from_ty, to_ty } => {
            format!("%{} = cast {} {} to {}", dest.name, format_operand(src), format_mir_type(from_ty), format_mir_type(to_ty))
        }
        // v0.55: Tuple instructions
        MirInst::TupleInit { dest, elements } => {
            let elems: Vec<_> = elements.iter()
                .map(|(ty, op)| format!("{}: {}", format_mir_type(ty), format_operand(op)))
                .collect();
            format!("%{} = tuple-init ({})", dest.name, elems.join(", "))
        }
        MirInst::TupleExtract { dest, tuple, index, element_type } => {
            format!("%{} = tuple-extract %{}.{} : {}", dest.name, tuple.name, index, format_mir_type(element_type))
        }
        MirInst::PtrOffset { dest, ptr, offset, element_type } => {
            format!("%{} = ptr-offset {} + {} : *{}", dest.name, format_operand(ptr), format_operand(offset), format_mir_type(element_type))
        }
        MirInst::ArrayAlloc { dest, element_type, size } => {
            format!("%{} = array-alloc [{}; {}]", dest.name, format_mir_type(element_type), size)
        }
        MirInst::PtrLoad { dest, ptr, element_type } => {
            format!("%{} = ptr-load {} : {}", dest.name, format_operand(ptr), format_mir_type(element_type))
        }
        MirInst::PtrStore { ptr, value, element_type } => {
            format!("ptr-store {} = {} : {}", format_operand(ptr), format_operand(value), format_mir_type(element_type))
        }
        // v0.70: Threading instructions
        MirInst::ThreadSpawn { dest, func, captures } => {
            let captures_str: Vec<_> = captures.iter().map(format_operand).collect();
            format!("%{} = thread-spawn {}({})", dest.name, func, captures_str.join(", "))
        }
        MirInst::ThreadJoin { dest, handle } => {
            if let Some(d) = dest {
                format!("%{} = thread-join {}", d.name, format_operand(handle))
            } else {
                format!("thread-join {}", format_operand(handle))
            }
        }
        // v0.71: Mutex instructions
        MirInst::MutexNew { dest, initial_value } => {
            format!("%{} = mutex-new {}", dest.name, format_operand(initial_value))
        }
        MirInst::MutexLock { dest, mutex } => {
            format!("%{} = mutex-lock {}", dest.name, format_operand(mutex))
        }
        MirInst::MutexUnlock { mutex, new_value } => {
            format!("mutex-unlock {} = {}", format_operand(mutex), format_operand(new_value))
        }
        MirInst::MutexTryLock { dest, mutex } => {
            format!("%{} = mutex-try-lock {}", dest.name, format_operand(mutex))
        }
        MirInst::MutexFree { mutex } => {
            format!("mutex-free {}", format_operand(mutex))
        }
        // v0.72: Arc instructions
        MirInst::ArcNew { dest, value } => {
            format!("%{} = arc-new {}", dest.name, format_operand(value))
        }
        MirInst::ArcClone { dest, arc } => {
            format!("%{} = arc-clone {}", dest.name, format_operand(arc))
        }
        MirInst::ArcGet { dest, arc } => {
            format!("%{} = arc-get {}", dest.name, format_operand(arc))
        }
        MirInst::ArcDrop { arc } => {
            format!("arc-drop {}", format_operand(arc))
        }
        MirInst::ArcStrongCount { dest, arc } => {
            format!("%{} = arc-strong-count {}", dest.name, format_operand(arc))
        }
        // v0.72: Atomic instructions
        MirInst::AtomicNew { dest, value } => {
            format!("%{} = atomic-new {}", dest.name, format_operand(value))
        }
        MirInst::AtomicLoad { dest, ptr } => {
            format!("%{} = atomic-load {}", dest.name, format_operand(ptr))
        }
        MirInst::AtomicStore { ptr, value } => {
            format!("atomic-store {} = {}", format_operand(ptr), format_operand(value))
        }
        MirInst::AtomicFetchAdd { dest, ptr, delta } => {
            format!("%{} = atomic-fetch-add {} {}", dest.name, format_operand(ptr), format_operand(delta))
        }
        MirInst::AtomicFetchSub { dest, ptr, delta } => {
            format!("%{} = atomic-fetch-sub {} {}", dest.name, format_operand(ptr), format_operand(delta))
        }
        MirInst::AtomicSwap { dest, ptr, new_value } => {
            format!("%{} = atomic-swap {} {}", dest.name, format_operand(ptr), format_operand(new_value))
        }
        MirInst::AtomicCompareExchange { dest, ptr, expected, new_value } => {
            format!("%{} = atomic-cmpxchg {} {} {}", dest.name, format_operand(ptr), format_operand(expected), format_operand(new_value))
        }
        // v0.73: Channel instructions
        MirInst::ChannelNew { sender_dest, receiver_dest, capacity } => {
            format!("(%{}, %{}) = channel-new {}", sender_dest.name, receiver_dest.name, format_operand(capacity))
        }
        MirInst::ChannelSend { sender, value } => {
            format!("channel-send {} {}", format_operand(sender), format_operand(value))
        }
        MirInst::ChannelRecv { dest, receiver } => {
            format!("%{} = channel-recv {}", dest.name, format_operand(receiver))
        }
        MirInst::ChannelTrySend { dest, sender, value } => {
            format!("%{} = channel-try-send {} {}", dest.name, format_operand(sender), format_operand(value))
        }
        MirInst::ChannelTryRecv { dest, receiver } => {
            format!("%{} = channel-try-recv {}", dest.name, format_operand(receiver))
        }
        // v0.77: Receive with timeout
        MirInst::ChannelRecvTimeout { dest, receiver, timeout_ms } => {
            format!("%{} = channel-recv-timeout {} {}", dest.name, format_operand(receiver), format_operand(timeout_ms))
        }
        // v0.78: Block on future
        MirInst::BlockOn { dest, future } => {
            format!("%{} = block-on {}", dest.name, format_operand(future))
        }
        // v0.79: Send with timeout
        MirInst::ChannelSendTimeout { dest, sender, value, timeout_ms } => {
            format!("%{} = channel-send-timeout {} {} {}", dest.name, format_operand(sender), format_operand(value), format_operand(timeout_ms))
        }
        MirInst::SenderClone { dest, sender } => {
            format!("%{} = sender-clone {}", dest.name, format_operand(sender))
        }
        // v0.74: RwLock instructions
        MirInst::RwLockNew { dest, initial_value } => {
            format!("%{} = rwlock-new {}", dest.name, format_operand(initial_value))
        }
        MirInst::RwLockRead { dest, rwlock } => {
            format!("%{} = rwlock-read {}", dest.name, format_operand(rwlock))
        }
        MirInst::RwLockReadUnlock { rwlock } => {
            format!("rwlock-read-unlock {}", format_operand(rwlock))
        }
        MirInst::RwLockWrite { dest, rwlock } => {
            format!("%{} = rwlock-write {}", dest.name, format_operand(rwlock))
        }
        MirInst::RwLockWriteUnlock { rwlock, value } => {
            format!("rwlock-write-unlock {} {}", format_operand(rwlock), format_operand(value))
        }
        // v0.74: Barrier instructions
        MirInst::BarrierNew { dest, count } => {
            format!("%{} = barrier-new {}", dest.name, format_operand(count))
        }
        MirInst::BarrierWait { dest, barrier } => {
            format!("%{} = barrier-wait {}", dest.name, format_operand(barrier))
        }
        // v0.74: Condvar instructions
        MirInst::CondvarNew { dest } => {
            format!("%{} = condvar-new", dest.name)
        }
        MirInst::CondvarWait { dest, condvar, mutex } => {
            format!("%{} = condvar-wait {} {}", dest.name, format_operand(condvar), format_operand(mutex))
        }
        MirInst::CondvarNotifyOne { condvar } => {
            format!("condvar-notify-one {}", format_operand(condvar))
        }
        MirInst::CondvarNotifyAll { condvar } => {
            format!("condvar-notify-all {}", format_operand(condvar))
        }
        // v0.76: Select instruction
        MirInst::Select { dest, cond_op, cond_lhs, cond_rhs, true_val, false_val } => {
            format!("%{} = select {} {} {} ? {} : {}",
                dest.name,
                format_operand(cond_lhs),
                format_binop(*cond_op),
                format_operand(cond_rhs),
                format_operand(true_val),
                format_operand(false_val))
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
        // v0.51.37: Pointer type
        MirType::Ptr(inner) => format!("*{}", format_mir_type(inner)),
        // v0.55: Tuple type
        MirType::Tuple(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|e| format_mir_type(e)).collect();
            format!("({})", elems_str.join(", "))
        }
    }
}
