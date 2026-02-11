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
    /// v0.89: Return value comparison: ret op constant (e.g., ret >= 0)
    /// Used in postconditions to express guarantees about the return value
    ReturnCmp {
        op: CmpOp,
        value: i64,
    },
    /// v0.89: Return value vs variable: ret op var (e.g., ret >= x)
    ReturnVarCmp {
        op: CmpOp,
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

    /// v0.80: Close a channel: channel-close(sender)
    ChannelClose {
        sender: Operand,
    },

    /// v0.80: Check if channel is closed: %dest = channel-is-closed(receiver)
    ChannelIsClosed {
        dest: Place,
        receiver: Operand,
    },

    /// v0.80: Receive with closed awareness: %dest = channel-recv-opt(receiver)
    /// Returns Some(value) if received, None if channel closed and empty
    ChannelRecvOpt {
        dest: Place,
        receiver: Operand,
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

    // v0.83: AsyncFile operations
    /// Open file asynchronously: %dest = async-file-open(path)
    AsyncFileOpen {
        dest: Place,
        path: Operand,
    },

    /// Read file content: %dest = async-file-read(file)
    AsyncFileRead {
        dest: Place,
        file: Operand,
    },

    /// Write content to file: async-file-write(file, content)
    AsyncFileWrite {
        file: Operand,
        content: Operand,
    },

    /// Close file: async-file-close(file)
    AsyncFileClose {
        file: Operand,
    },

    // v0.83.1: AsyncSocket operations
    /// Connect to TCP server: %dest = async-socket-connect(host, port)
    AsyncSocketConnect {
        dest: Place,
        host: Operand,
        port: Operand,
    },

    /// Read from socket: %dest = async-socket-read(socket)
    AsyncSocketRead {
        dest: Place,
        socket: Operand,
    },

    /// Write to socket: async-socket-write(socket, content)
    AsyncSocketWrite {
        socket: Operand,
        content: Operand,
    },

    /// Close socket: async-socket-close(socket)
    AsyncSocketClose {
        socket: Operand,
    },

    // v0.84: ThreadPool instructions

    /// Create thread pool: thread-pool-new(size) -> pool
    ThreadPoolNew {
        dest: Place,
        size: Operand,
    },

    /// Execute task on thread pool: thread-pool-execute(pool, task)
    ThreadPoolExecute {
        pool: Operand,
        task: Operand,
    },

    /// Join thread pool (wait for all tasks and shutdown): thread-pool-join(pool)
    ThreadPoolJoin {
        pool: Operand,
    },

    /// Shutdown thread pool (request shutdown): thread-pool-shutdown(pool)
    ThreadPoolShutdown {
        pool: Operand,
    },

    // v0.85: Scope instructions for scoped threads

    /// Create scope: scope-new() -> scope
    ScopeNew {
        dest: Place,
    },

    /// Spawn scoped thread: scope-spawn(scope, task)
    ScopeSpawn {
        scope: Operand,
        task: Operand,
    },

    /// Wait for all spawned threads: scope-wait(scope)
    ScopeWait {
        scope: Operand,
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
    /// v0.88.1: Maps original variable name -> unique SSA-compatible name
    /// This ensures each let binding gets a unique definition (SSA form)
    pub var_name_map: HashMap<String, String>,
    /// v0.89.4: Tracks last let binding for Block scope extension
    /// (original_name, unique_name) - set by Let, consumed by Block
    pub last_let_binding: Option<(String, String)>,
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
            var_name_map: HashMap::new(),
            last_let_binding: None,
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
        // v0.80: Channel close operations
        MirInst::ChannelClose { sender } => {
            format!("channel-close {}", format_operand(sender))
        }
        MirInst::ChannelIsClosed { dest, receiver } => {
            format!("%{} = channel-is-closed {}", dest.name, format_operand(receiver))
        }
        MirInst::ChannelRecvOpt { dest, receiver } => {
            format!("%{} = channel-recv-opt {}", dest.name, format_operand(receiver))
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
        // v0.83: AsyncFile instructions
        MirInst::AsyncFileOpen { dest, path } => {
            format!("%{} = async-file-open {}", dest.name, format_operand(path))
        }
        MirInst::AsyncFileRead { dest, file } => {
            format!("%{} = async-file-read {}", dest.name, format_operand(file))
        }
        MirInst::AsyncFileWrite { file, content } => {
            format!("async-file-write {} {}", format_operand(file), format_operand(content))
        }
        MirInst::AsyncFileClose { file } => {
            format!("async-file-close {}", format_operand(file))
        }
        // v0.83.1: AsyncSocket instructions
        MirInst::AsyncSocketConnect { dest, host, port } => {
            format!("%{} = async-socket-connect {} {}", dest.name, format_operand(host), format_operand(port))
        }
        MirInst::AsyncSocketRead { dest, socket } => {
            format!("%{} = async-socket-read {}", dest.name, format_operand(socket))
        }
        MirInst::AsyncSocketWrite { socket, content } => {
            format!("async-socket-write {} {}", format_operand(socket), format_operand(content))
        }
        MirInst::AsyncSocketClose { socket } => {
            format!("async-socket-close {}", format_operand(socket))
        }
        // v0.84: ThreadPool instructions
        MirInst::ThreadPoolNew { dest, size } => {
            format!("%{} = thread-pool-new {}", dest.name, format_operand(size))
        }
        MirInst::ThreadPoolExecute { pool, task } => {
            format!("thread-pool-execute {} {}", format_operand(pool), format_operand(task))
        }
        MirInst::ThreadPoolJoin { pool } => {
            format!("thread-pool-join {}", format_operand(pool))
        }
        MirInst::ThreadPoolShutdown { pool } => {
            format!("thread-pool-shutdown {}", format_operand(pool))
        }
        // v0.85: Scope instructions
        MirInst::ScopeNew { dest } => {
            format!("%{} = scope-new", dest.name)
        }
        MirInst::ScopeSpawn { scope, task } => {
            format!("scope-spawn {} {}", format_operand(scope), format_operand(task))
        }
        MirInst::ScopeWait { scope } => {
            format!("scope-wait {}", format_operand(scope))
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- MirType tests ---

    #[test]
    fn test_mir_type_is_integer() {
        assert!(MirType::I32.is_integer());
        assert!(MirType::I64.is_integer());
        assert!(!MirType::F64.is_integer());
        assert!(!MirType::Bool.is_integer());
        assert!(!MirType::String.is_integer());
        assert!(!MirType::Unit.is_integer());
    }

    #[test]
    fn test_mir_type_is_float() {
        assert!(MirType::F64.is_float());
        assert!(!MirType::I64.is_float());
        assert!(!MirType::Bool.is_float());
    }

    #[test]
    fn test_mir_type_is_pointer_type() {
        assert!(MirType::Ptr(Box::new(MirType::I64)).is_pointer_type());
        assert!(MirType::StructPtr("Foo".to_string()).is_pointer_type());
        assert!(MirType::String.is_pointer_type());
        assert!(!MirType::I64.is_pointer_type());
        assert!(!MirType::Bool.is_pointer_type());
        assert!(!MirType::Unit.is_pointer_type());
    }

    #[test]
    fn test_mir_type_pointer_element_type() {
        let ptr_i64 = MirType::Ptr(Box::new(MirType::I64));
        assert_eq!(ptr_i64.pointer_element_type(), Some(MirType::I64));

        let struct_ptr = MirType::StructPtr("Point".to_string());
        match struct_ptr.pointer_element_type() {
            Some(MirType::Struct { name, .. }) => assert_eq!(name, "Point"),
            other => panic!("expected Struct, got {:?}", other),
        }

        assert_eq!(MirType::I64.pointer_element_type(), None);
        assert_eq!(MirType::Bool.pointer_element_type(), None);
    }

    // --- MirBinOp::result_type tests ---

    #[test]
    fn test_binop_arithmetic_returns_operand_type() {
        let ops = [MirBinOp::Add, MirBinOp::Sub, MirBinOp::Mul, MirBinOp::Div, MirBinOp::Mod];
        for op in &ops {
            assert_eq!(op.result_type(&MirType::I64), MirType::I64);
            assert_eq!(op.result_type(&MirType::I32), MirType::I32);
        }
    }

    #[test]
    fn test_binop_float_returns_f64() {
        let ops = [MirBinOp::FAdd, MirBinOp::FSub, MirBinOp::FMul, MirBinOp::FDiv];
        for op in &ops {
            assert_eq!(op.result_type(&MirType::F64), MirType::F64);
        }
    }

    #[test]
    fn test_binop_comparisons_return_bool() {
        let ops = [MirBinOp::Eq, MirBinOp::Ne, MirBinOp::Lt, MirBinOp::Gt, MirBinOp::Le, MirBinOp::Ge];
        for op in &ops {
            assert_eq!(op.result_type(&MirType::I64), MirType::Bool);
        }
    }

    #[test]
    fn test_binop_float_comparisons_return_bool() {
        let ops = [MirBinOp::FEq, MirBinOp::FNe, MirBinOp::FLt, MirBinOp::FGt, MirBinOp::FLe, MirBinOp::FGe];
        for op in &ops {
            assert_eq!(op.result_type(&MirType::F64), MirType::Bool);
        }
    }

    #[test]
    fn test_binop_logical_returns_bool() {
        assert_eq!(MirBinOp::And.result_type(&MirType::Bool), MirType::Bool);
        assert_eq!(MirBinOp::Or.result_type(&MirType::Bool), MirType::Bool);
        assert_eq!(MirBinOp::Implies.result_type(&MirType::Bool), MirType::Bool);
    }

    #[test]
    fn test_binop_shift_returns_operand_type() {
        assert_eq!(MirBinOp::Shl.result_type(&MirType::I64), MirType::I64);
        assert_eq!(MirBinOp::Shr.result_type(&MirType::I32), MirType::I32);
    }

    #[test]
    fn test_binop_bitwise_returns_operand_type() {
        assert_eq!(MirBinOp::Band.result_type(&MirType::I64), MirType::I64);
        assert_eq!(MirBinOp::Bor.result_type(&MirType::I32), MirType::I32);
        assert_eq!(MirBinOp::Bxor.result_type(&MirType::I64), MirType::I64);
    }

    #[test]
    fn test_binop_wrapping_returns_operand_type() {
        let ops = [MirBinOp::AddWrap, MirBinOp::SubWrap, MirBinOp::MulWrap];
        for op in &ops {
            assert_eq!(op.result_type(&MirType::I64), MirType::I64);
        }
    }

    #[test]
    fn test_binop_saturating_returns_operand_type() {
        let ops = [MirBinOp::AddSat, MirBinOp::SubSat, MirBinOp::MulSat];
        for op in &ops {
            assert_eq!(op.result_type(&MirType::I64), MirType::I64);
        }
    }

    // --- MirUnaryOp::result_type tests ---

    #[test]
    fn test_unaryop_neg_returns_operand_type() {
        assert_eq!(MirUnaryOp::Neg.result_type(&MirType::I64), MirType::I64);
        assert_eq!(MirUnaryOp::Neg.result_type(&MirType::I32), MirType::I32);
    }

    #[test]
    fn test_unaryop_fneg_returns_f64() {
        assert_eq!(MirUnaryOp::FNeg.result_type(&MirType::F64), MirType::F64);
    }

    #[test]
    fn test_unaryop_not_returns_bool() {
        assert_eq!(MirUnaryOp::Not.result_type(&MirType::Bool), MirType::Bool);
    }

    #[test]
    fn test_unaryop_bnot_returns_operand_type() {
        assert_eq!(MirUnaryOp::Bnot.result_type(&MirType::I64), MirType::I64);
    }

    // --- LoweringContext tests ---

    #[test]
    fn test_fresh_temp_generates_unique_names() {
        let mut ctx = LoweringContext::new();
        let t0 = ctx.fresh_temp();
        let t1 = ctx.fresh_temp();
        let t2 = ctx.fresh_temp();
        assert_eq!(t0.name, "_t0");
        assert_eq!(t1.name, "_t1");
        assert_eq!(t2.name, "_t2");
    }

    #[test]
    fn test_fresh_label_generates_unique_labels() {
        let mut ctx = LoweringContext::new();
        let l0 = ctx.fresh_label("then");
        let l1 = ctx.fresh_label("else");
        let l2 = ctx.fresh_label("merge");
        assert_eq!(l0, "then_0");
        assert_eq!(l1, "else_1");
        assert_eq!(l2, "merge_2");
    }

    #[test]
    fn test_operand_type_constants() {
        let ctx = LoweringContext::new();
        assert_eq!(ctx.operand_type(&Operand::Constant(Constant::Int(42))), MirType::I64);
        assert_eq!(ctx.operand_type(&Operand::Constant(Constant::Float(1.5))), MirType::F64);
        assert_eq!(ctx.operand_type(&Operand::Constant(Constant::Bool(true))), MirType::Bool);
        assert_eq!(ctx.operand_type(&Operand::Constant(Constant::String("hi".into()))), MirType::String);
        assert_eq!(ctx.operand_type(&Operand::Constant(Constant::Unit)), MirType::Unit);
    }

    #[test]
    fn test_operand_type_unknown_place_defaults_i64() {
        let ctx = LoweringContext::new();
        // Unknown place defaults to I64
        assert_eq!(ctx.operand_type(&Operand::Place(Place::new("unknown"))), MirType::I64);
    }

    // --- format_mir tests ---

    fn parse_and_lower(source: &str) -> MirProgram {
        let tokens = crate::lexer::tokenize(source).expect("tokenize failed");
        let program = crate::parser::parse("<test>", source, tokens).expect("parse failed");
        lower_program(&program)
    }

    #[test]
    fn test_format_mir_simple_function() {
        let mir = parse_and_lower("fn f() -> i64 = 42;");
        let text = format_mir(&mir);
        assert!(text.contains("fn f()"), "function header missing");
        assert!(text.contains("-> i64"), "return type missing");
    }

    #[test]
    fn test_format_mir_with_params() {
        let mir = parse_and_lower("fn add(a: i64, b: i64) -> i64 = a + b;");
        let text = format_mir(&mir);
        assert!(text.contains("fn add("), "function name missing");
        assert!(text.contains("i64"), "parameter type missing");
    }

    #[test]
    fn test_format_mir_multiple_functions() {
        let mir = parse_and_lower("fn f1() -> i64 = 1;\nfn f2() -> i64 = 2;");
        let text = format_mir(&mir);
        assert!(text.contains("fn f1("), "f1 missing");
        assert!(text.contains("fn f2("), "f2 missing");
    }

    #[test]
    fn test_format_mir_type_i64() {
        assert_eq!(format_mir_type(&MirType::I64), "i64");
    }

    #[test]
    fn test_format_mir_type_f64() {
        assert_eq!(format_mir_type(&MirType::F64), "f64");
    }

    #[test]
    fn test_format_mir_type_bool() {
        assert_eq!(format_mir_type(&MirType::Bool), "bool");
    }

    #[test]
    fn test_format_mir_type_string() {
        assert_eq!(format_mir_type(&MirType::String), "String");
    }

    #[test]
    fn test_format_mir_type_unit() {
        assert_eq!(format_mir_type(&MirType::Unit), "()");
    }

    #[test]
    fn test_format_mir_type_ptr() {
        assert_eq!(format_mir_type(&MirType::Ptr(Box::new(MirType::I64))), "*i64");
    }

    #[test]
    fn test_format_mir_type_tuple() {
        let tuple = MirType::Tuple(vec![Box::new(MirType::I64), Box::new(MirType::Bool)]);
        assert_eq!(format_mir_type(&tuple), "(i64, bool)");
    }

    // --- Place tests ---

    #[test]
    fn test_place_new() {
        let p = Place::new("x");
        assert_eq!(p.name, "x");
    }

    // --- Constant operand tests ---

    #[test]
    fn test_operand_constant_int() {
        let op = Operand::Constant(Constant::Int(42));
        match op {
            Operand::Constant(Constant::Int(v)) => assert_eq!(v, 42),
            _ => panic!("expected Int constant"),
        }
    }

    #[test]
    fn test_operand_constant_bool() {
        let op = Operand::Constant(Constant::Bool(true));
        match op {
            Operand::Constant(Constant::Bool(v)) => assert!(v),
            _ => panic!("expected Bool constant"),
        }
    }

    // --- MirProgram construction tests ---

    #[test]
    fn test_mir_program_construction_empty() {
        let prog = MirProgram {
            functions: vec![],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };
        assert!(prog.functions.is_empty());
        assert!(prog.extern_fns.is_empty());
        assert!(prog.struct_defs.is_empty());
    }

    #[test]
    fn test_mir_program_with_struct_defs() {
        let mut struct_defs = HashMap::new();
        struct_defs.insert("Point".to_string(), vec![
            ("x".to_string(), MirType::F64),
            ("y".to_string(), MirType::F64),
        ]);
        let prog = MirProgram {
            functions: vec![],
            extern_fns: vec![],
            struct_defs,
        };
        let fields = prog.struct_defs.get("Point").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].0, "x");
        assert_eq!(fields[0].1, MirType::F64);
        assert_eq!(fields[1].0, "y");
        assert_eq!(fields[1].1, MirType::F64);
    }

    #[test]
    fn test_mir_extern_fn_construction() {
        let ext = MirExternFn {
            module: "wasi_snapshot_preview1".to_string(),
            name: "fd_write".to_string(),
            params: vec![MirType::I32, MirType::I32, MirType::I32, MirType::I32],
            ret_ty: MirType::I32,
        };
        assert_eq!(ext.module, "wasi_snapshot_preview1");
        assert_eq!(ext.name, "fd_write");
        assert_eq!(ext.params.len(), 4);
        assert_eq!(ext.ret_ty, MirType::I32);
    }

    // --- MirFunction construction and attribute tests ---

    #[test]
    fn test_mir_function_construction_with_attributes() {
        let func = MirFunction {
            name: "square".to_string(),
            params: vec![("x".to_string(), MirType::I64)],
            ret_ty: MirType::I64,
            locals: vec![("tmp".to_string(), MirType::I64)],
            blocks: vec![BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(0)))),
            }],
            preconditions: vec![ContractFact::VarCmp {
                var: "x".to_string(),
                op: CmpOp::Ge,
                value: 0,
            }],
            postconditions: vec![ContractFact::ReturnCmp {
                op: CmpOp::Ge,
                value: 0,
            }],
            is_pure: true,
            is_const: false,
            always_inline: false,
            inline_hint: true,
            is_memory_free: true,
        };
        assert_eq!(func.name, "square");
        assert_eq!(func.params.len(), 1);
        assert_eq!(func.locals.len(), 1);
        assert_eq!(func.blocks.len(), 1);
        assert!(func.is_pure);
        assert!(!func.is_const);
        assert!(!func.always_inline);
        assert!(func.inline_hint);
        assert!(func.is_memory_free);
        assert_eq!(func.preconditions.len(), 1);
        assert_eq!(func.postconditions.len(), 1);
    }

    // --- BasicBlock and Terminator tests ---

    #[test]
    fn test_basic_block_with_instructions_and_return() {
        let block = BasicBlock {
            label: "entry".to_string(),
            instructions: vec![
                MirInst::Const {
                    dest: Place::new("x"),
                    value: Constant::Int(10),
                },
                MirInst::Const {
                    dest: Place::new("y"),
                    value: Constant::Int(20),
                },
                MirInst::BinOp {
                    dest: Place::new("result"),
                    op: MirBinOp::Add,
                    lhs: Operand::Place(Place::new("x")),
                    rhs: Operand::Place(Place::new("y")),
                },
            ],
            terminator: Terminator::Return(Some(Operand::Place(Place::new("result")))),
        };
        assert_eq!(block.label, "entry");
        assert_eq!(block.instructions.len(), 3);
        match &block.terminator {
            Terminator::Return(Some(Operand::Place(p))) => assert_eq!(p.name, "result"),
            _ => panic!("expected Return with Place operand"),
        }
    }

    #[test]
    fn test_terminator_goto() {
        let term = Terminator::Goto("loop_header".to_string());
        match &term {
            Terminator::Goto(label) => assert_eq!(label, "loop_header"),
            _ => panic!("expected Goto"),
        }
    }

    #[test]
    fn test_terminator_branch() {
        let term = Terminator::Branch {
            cond: Operand::Place(Place::new("cond")),
            then_label: "then_bb".to_string(),
            else_label: "else_bb".to_string(),
        };
        match &term {
            Terminator::Branch { cond, then_label, else_label } => {
                match cond {
                    Operand::Place(p) => assert_eq!(p.name, "cond"),
                    _ => panic!("expected Place operand"),
                }
                assert_eq!(then_label, "then_bb");
                assert_eq!(else_label, "else_bb");
            }
            _ => panic!("expected Branch"),
        }
    }

    #[test]
    fn test_terminator_switch() {
        let term = Terminator::Switch {
            discriminant: Operand::Place(Place::new("disc")),
            cases: vec![(0, "case_0".to_string()), (1, "case_1".to_string()), (2, "case_2".to_string())],
            default: "default_bb".to_string(),
        };
        match &term {
            Terminator::Switch { discriminant: _, cases, default } => {
                assert_eq!(cases.len(), 3);
                assert_eq!(cases[0], (0, "case_0".to_string()));
                assert_eq!(cases[1], (1, "case_1".to_string()));
                assert_eq!(cases[2], (2, "case_2".to_string()));
                assert_eq!(default, "default_bb");
            }
            _ => panic!("expected Switch"),
        }
    }

    // --- LoweringContext block management tests ---

    #[test]
    fn test_lowering_context_push_inst_and_finish_block() {
        let mut ctx = LoweringContext::new();
        ctx.push_inst(MirInst::Const {
            dest: Place::new("a"),
            value: Constant::Int(1),
        });
        ctx.emit(MirInst::Const {
            dest: Place::new("b"),
            value: Constant::Int(2),
        });
        ctx.finish_block(Terminator::Return(None));

        assert_eq!(ctx.blocks.len(), 1);
        assert_eq!(ctx.blocks[0].label, "entry");
        assert_eq!(ctx.blocks[0].instructions.len(), 2);
        match &ctx.blocks[0].terminator {
            Terminator::Return(None) => {}
            _ => panic!("expected Return(None)"),
        }
    }

    #[test]
    fn test_lowering_context_start_block_and_label() {
        let mut ctx = LoweringContext::new();
        assert_eq!(ctx.current_block_label(), "entry");

        ctx.finish_block(Terminator::Goto("next".to_string()));
        ctx.start_block("next".to_string());
        assert_eq!(ctx.current_block_label(), "next");

        ctx.push_inst(MirInst::Const {
            dest: Place::new("x"),
            value: Constant::Bool(true),
        });
        ctx.finish_block(Terminator::Return(None));

        assert_eq!(ctx.blocks.len(), 2);
        assert_eq!(ctx.blocks[0].label, "entry");
        assert_eq!(ctx.blocks[1].label, "next");
        assert_eq!(ctx.blocks[1].instructions.len(), 1);
    }

    #[test]
    fn test_lowering_context_field_index_lookup() {
        let mut ctx = LoweringContext::new();
        ctx.struct_defs.insert("Point".to_string(), vec![
            "x".to_string(), "y".to_string(), "z".to_string(),
        ]);
        assert_eq!(ctx.field_index("Point", "x"), 0);
        assert_eq!(ctx.field_index("Point", "y"), 1);
        assert_eq!(ctx.field_index("Point", "z"), 2);
        // Unknown field returns 0
        assert_eq!(ctx.field_index("Point", "w"), 0);
        // Unknown struct returns 0
        assert_eq!(ctx.field_index("Unknown", "x"), 0);
    }

    #[test]
    fn test_lowering_context_field_type_lookup() {
        let mut ctx = LoweringContext::new();
        ctx.struct_type_defs.insert("Rect".to_string(), vec![
            ("width".to_string(), MirType::F64),
            ("height".to_string(), MirType::F64),
            ("visible".to_string(), MirType::Bool),
        ]);
        assert_eq!(ctx.field_type("Rect", "width"), MirType::F64);
        assert_eq!(ctx.field_type("Rect", "height"), MirType::F64);
        assert_eq!(ctx.field_type("Rect", "visible"), MirType::Bool);
        // Unknown field returns I64 (default)
        assert_eq!(ctx.field_type("Rect", "missing"), MirType::I64);
        // Unknown struct returns I64
        assert_eq!(ctx.field_type("Unknown", "x"), MirType::I64);
    }

    #[test]
    fn test_lowering_context_operand_type_from_locals_and_params() {
        let mut ctx = LoweringContext::new();
        ctx.locals.insert("local_var".to_string(), MirType::Bool);
        ctx.params.insert("param_var".to_string(), MirType::F64);
        ctx.temp_types.insert("_t5".to_string(), MirType::String);

        assert_eq!(
            ctx.operand_type(&Operand::Place(Place::new("local_var"))),
            MirType::Bool
        );
        assert_eq!(
            ctx.operand_type(&Operand::Place(Place::new("param_var"))),
            MirType::F64
        );
        assert_eq!(
            ctx.operand_type(&Operand::Place(Place::new("_t5"))),
            MirType::String
        );
        // Char constant type
        assert_eq!(
            ctx.operand_type(&Operand::Constant(Constant::Char('A'))),
            MirType::Char
        );
    }

    #[test]
    fn test_lowering_context_place_struct_type() {
        let mut ctx = LoweringContext::new();
        ctx.var_struct_types.insert("p".to_string(), "Point".to_string());

        assert_eq!(ctx.place_struct_type(&Place::new("p")), Some("Point".to_string()));
        assert_eq!(ctx.place_struct_type(&Place::new("q")), None);
    }

    // --- ContractFact and CmpOp tests ---

    #[test]
    fn test_contract_fact_variants() {
        let var_cmp = ContractFact::VarCmp {
            var: "x".to_string(),
            op: CmpOp::Ge,
            value: 0,
        };
        let var_var_cmp = ContractFact::VarVarCmp {
            lhs: "start".to_string(),
            op: CmpOp::Le,
            rhs: "end".to_string(),
        };
        let array_bounds = ContractFact::ArrayBounds {
            index: "i".to_string(),
            array: "arr".to_string(),
        };
        let non_null = ContractFact::NonNull {
            var: "ptr".to_string(),
        };
        let ret_cmp = ContractFact::ReturnCmp {
            op: CmpOp::Gt,
            value: -1,
        };
        let ret_var_cmp = ContractFact::ReturnVarCmp {
            op: CmpOp::Ge,
            var: "input".to_string(),
        };

        // Verify equality works (derives PartialEq)
        assert_eq!(var_cmp, ContractFact::VarCmp {
            var: "x".to_string(),
            op: CmpOp::Ge,
            value: 0,
        });
        assert_ne!(var_cmp, ContractFact::VarCmp {
            var: "y".to_string(),
            op: CmpOp::Ge,
            value: 0,
        });

        // Verify all CmpOp variants exist and are distinguishable
        assert_ne!(CmpOp::Lt, CmpOp::Le);
        assert_ne!(CmpOp::Gt, CmpOp::Ge);
        assert_ne!(CmpOp::Eq, CmpOp::Ne);

        // Verify each ContractFact variant is distinct
        let facts: Vec<ContractFact> = vec![var_cmp, var_var_cmp, array_bounds, non_null, ret_cmp, ret_var_cmp];
        for i in 0..facts.len() {
            for j in (i + 1)..facts.len() {
                assert_ne!(facts[i], facts[j], "facts[{}] and facts[{}] should differ", i, j);
            }
        }
    }

    // --- format_mir_type additional coverage ---

    #[test]
    fn test_format_mir_type_i32_u32_u64_char() {
        assert_eq!(format_mir_type(&MirType::I32), "i32");
        assert_eq!(format_mir_type(&MirType::U32), "u32");
        assert_eq!(format_mir_type(&MirType::U64), "u64");
        assert_eq!(format_mir_type(&MirType::Char), "char");
    }

    #[test]
    fn test_format_mir_type_struct_and_struct_ptr() {
        let struct_ty = MirType::Struct {
            name: "Vec3".to_string(),
            fields: vec![
                ("x".to_string(), Box::new(MirType::F64)),
                ("y".to_string(), Box::new(MirType::F64)),
                ("z".to_string(), Box::new(MirType::F64)),
            ],
        };
        assert_eq!(format_mir_type(&struct_ty), "Vec3");
        assert_eq!(format_mir_type(&MirType::StructPtr("Vec3".to_string())), "&Vec3");
    }

    #[test]
    fn test_format_mir_type_enum() {
        let enum_ty = MirType::Enum {
            name: "Option".to_string(),
            variants: vec![
                ("Some".to_string(), vec![Box::new(MirType::I64)]),
                ("None".to_string(), vec![]),
            ],
        };
        assert_eq!(format_mir_type(&enum_ty), "Option");
    }

    #[test]
    fn test_format_mir_type_array_fixed_and_dynamic() {
        let fixed = MirType::Array {
            element_type: Box::new(MirType::I64),
            size: Some(10),
        };
        assert_eq!(format_mir_type(&fixed), "[i64; 10]");

        let dynamic = MirType::Array {
            element_type: Box::new(MirType::Bool),
            size: None,
        };
        assert_eq!(format_mir_type(&dynamic), "[bool]");
    }

    // --- Instruction and terminator formatting tests ---

    #[test]
    fn test_format_const_instruction() {
        let inst = MirInst::Const {
            dest: Place::new("x"),
            value: Constant::Int(42),
        };
        let text = format_mir_inst(&inst);
        assert_eq!(text, "%x = const I:42");
    }

    #[test]
    fn test_format_copy_instruction() {
        let inst = MirInst::Copy {
            dest: Place::new("dst"),
            src: Place::new("src"),
        };
        assert_eq!(format_mir_inst(&inst), "%dst = copy %src");
    }

    #[test]
    fn test_format_binop_instruction() {
        let inst = MirInst::BinOp {
            dest: Place::new("sum"),
            op: MirBinOp::Add,
            lhs: Operand::Place(Place::new("a")),
            rhs: Operand::Constant(Constant::Int(1)),
        };
        assert_eq!(format_mir_inst(&inst), "%sum = + %a, I:1");
    }

    #[test]
    fn test_format_unaryop_instruction() {
        let inst = MirInst::UnaryOp {
            dest: Place::new("neg_x"),
            op: MirUnaryOp::Neg,
            src: Operand::Place(Place::new("x")),
        };
        assert_eq!(format_mir_inst(&inst), "%neg_x = neg %x");
    }

    #[test]
    fn test_format_call_instruction_with_and_without_dest() {
        let call_with_dest = MirInst::Call {
            dest: Some(Place::new("result")),
            func: "compute".to_string(),
            args: vec![
                Operand::Place(Place::new("a")),
                Operand::Constant(Constant::Int(5)),
            ],
            is_tail: false,
        };
        assert_eq!(format_mir_inst(&call_with_dest), "%result = call compute(%a, I:5)");

        let call_no_dest = MirInst::Call {
            dest: None,
            func: "println".to_string(),
            args: vec![Operand::Constant(Constant::String("hello".to_string()))],
            is_tail: false,
        };
        assert_eq!(format_mir_inst(&call_no_dest), "call println(S:\"hello\")");

        let tail_call = MirInst::Call {
            dest: Some(Place::new("r")),
            func: "recurse".to_string(),
            args: vec![Operand::Place(Place::new("n"))],
            is_tail: true,
        };
        assert_eq!(format_mir_inst(&tail_call), "%r = tail call recurse(%n)");
    }

    #[test]
    fn test_format_phi_instruction() {
        let inst = MirInst::Phi {
            dest: Place::new("merged"),
            values: vec![
                (Operand::Constant(Constant::Int(1)), "then_bb".to_string()),
                (Operand::Constant(Constant::Int(2)), "else_bb".to_string()),
            ],
        };
        assert_eq!(format_mir_inst(&inst), "%merged = phi [I:1, then_bb], [I:2, else_bb]");
    }

    #[test]
    fn test_format_terminator_return_none_and_some() {
        assert_eq!(format_terminator(&Terminator::Return(None)), "return");
        assert_eq!(
            format_terminator(&Terminator::Return(Some(Operand::Constant(Constant::Int(0))))),
            "return I:0"
        );
        assert_eq!(
            format_terminator(&Terminator::Return(Some(Operand::Place(Place::new("res"))))),
            "return %res"
        );
    }

    #[test]
    fn test_format_terminator_goto() {
        assert_eq!(
            format_terminator(&Terminator::Goto("loop_body".to_string())),
            "goto loop_body"
        );
    }

    #[test]
    fn test_format_terminator_branch() {
        let term = Terminator::Branch {
            cond: Operand::Place(Place::new("flag")),
            then_label: "bb_true".to_string(),
            else_label: "bb_false".to_string(),
        };
        assert_eq!(format_terminator(&term), "branch %flag, bb_true, bb_false");
    }

    #[test]
    fn test_format_terminator_unreachable() {
        assert_eq!(format_terminator(&Terminator::Unreachable), "unreachable");
    }

    #[test]
    fn test_format_terminator_switch() {
        let term = Terminator::Switch {
            discriminant: Operand::Place(Place::new("tag")),
            cases: vec![(0, "case_a".to_string()), (1, "case_b".to_string())],
            default: "default_bb".to_string(),
        };
        assert_eq!(
            format_terminator(&term),
            "switch %tag, [0 -> case_a, 1 -> case_b], default_bb"
        );
    }

    // --- Constant formatting tests ---

    #[test]
    fn test_format_constant_all_types() {
        assert_eq!(format_constant(&Constant::Int(99)), "I:99");
        assert_eq!(format_constant(&Constant::Int(-5)), "I:-5");
        assert_eq!(format_constant(&Constant::Float(1.23)), "F:1.23");
        assert_eq!(format_constant(&Constant::Bool(true)), "B:1");
        assert_eq!(format_constant(&Constant::Bool(false)), "B:0");
        assert_eq!(format_constant(&Constant::String("test".to_string())), "S:\"test\"");
        assert_eq!(format_constant(&Constant::Char('Z')), "C:'Z'");
        assert_eq!(format_constant(&Constant::Unit), "U");
    }

    // --- format_mir function attributes test ---

    #[test]
    fn test_format_mir_function_with_attributes() {
        let func = MirFunction {
            name: "pure_fn".to_string(),
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
            is_pure: true,
            is_const: true,
            always_inline: false,
            inline_hint: false,
            is_memory_free: true,
        };
        let prog = MirProgram {
            functions: vec![func],
            extern_fns: vec![],
            struct_defs: HashMap::new(),
        };
        let text = format_mir(&prog);
        assert!(text.contains("@pure"), "expected @pure attribute, got: {}", text);
        assert!(text.contains("@const"), "expected @const attribute, got: {}", text);
        assert!(text.contains("@memory(none)"), "expected @memory(none) attribute, got: {}", text);
        assert!(!text.contains("@alwaysinline"), "should not have @alwaysinline");
        assert!(!text.contains("@inlinehint"), "should not have @inlinehint");
        assert!(text.contains("fn pure_fn(n: i64) -> i64"), "function header missing");
        assert!(text.contains("return %n"), "return missing");
    }

    // --- Checked arithmetic result type test ---

    #[test]
    fn test_binop_checked_returns_operand_type() {
        let ops = [MirBinOp::AddChecked, MirBinOp::SubChecked, MirBinOp::MulChecked];
        for op in &ops {
            assert_eq!(op.result_type(&MirType::I64), MirType::I64);
            assert_eq!(op.result_type(&MirType::I32), MirType::I32);
        }
    }

    // --- MirType unsigned integer tests ---

    #[test]
    fn test_mir_type_unsigned_not_integer() {
        // U32 and U64 are NOT classified as integer by is_integer() (only I32/I64 are)
        assert!(!MirType::U32.is_integer());
        assert!(!MirType::U64.is_integer());
        assert!(!MirType::U32.is_float());
        assert!(!MirType::U64.is_float());
    }
}
