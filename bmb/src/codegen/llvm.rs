//! LLVM Code Generation using inkwell
//!
//! This module generates LLVM IR from MIR and compiles to object files.

use std::collections::HashMap;
use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::passes::PassBuilderOptions;
use inkwell::OptimizationLevel;
use inkwell::{FloatPredicate, IntPredicate};
use thiserror::Error;

use crate::mir::{
    BasicBlock, Constant, MirBinOp, MirFunction, MirInst, MirProgram, MirType, MirUnaryOp,
    Operand, Place, Terminator,
};

/// Code generation error
#[derive(Debug, Error)]
pub enum CodeGenError {
    #[error("LLVM error: {0}")]
    LlvmError(String),

    #[error("Unknown function: {0}")]
    UnknownFunction(String),

    #[error("Unknown variable: {0}")]
    UnknownVariable(String),

    #[error("Unknown block: {0}")]
    UnknownBlock(String),

    #[error("Type mismatch")]
    TypeMismatch,

    #[error("Target machine creation failed")]
    TargetMachineError,

    #[error("Object file generation failed: {0}")]
    ObjectFileError(String),
}

/// Result type for code generation
pub type CodeGenResult<T> = Result<T, CodeGenError>;

/// Optimization level for code generation
#[derive(Debug, Clone, Copy, Default)]
pub enum OptLevel {
    #[default]
    Debug,
    Release,
    Size,
    Aggressive,
}

impl From<OptLevel> for OptimizationLevel {
    fn from(level: OptLevel) -> Self {
        match level {
            OptLevel::Debug => OptimizationLevel::None,
            OptLevel::Release => OptimizationLevel::Default,
            OptLevel::Size => OptimizationLevel::Less,
            OptLevel::Aggressive => OptimizationLevel::Aggressive,
        }
    }
}

/// LLVM Code Generator
pub struct CodeGen {
    opt_level: OptLevel,
}

impl CodeGen {
    /// Create a new code generator
    pub fn new() -> Self {
        Self {
            opt_level: OptLevel::default(),
        }
    }

    /// Create a new code generator with optimization level
    pub fn with_opt_level(opt_level: OptLevel) -> Self {
        Self { opt_level }
    }

    /// Compile MIR to object file
    pub fn compile(&self, program: &MirProgram, output: &Path) -> CodeGenResult<()> {
        let context = Context::create();
        let mut ctx = LlvmContext::new(&context);

        // Declare built-in functions
        ctx.declare_builtins();

        // v0.35.4: Two-pass approach for forward references
        // Pass 1: Declare all user functions
        for func in &program.functions {
            ctx.declare_function(func)?;
        }

        // Pass 2: Generate function bodies
        for func in &program.functions {
            ctx.gen_function_body(func)?;
        }

        // Write to object file
        self.write_object_file(&ctx.module, output)
    }

    /// Generate LLVM IR as string
    pub fn generate_ir(&self, program: &MirProgram) -> CodeGenResult<String> {
        let context = Context::create();
        let mut ctx = LlvmContext::new(&context);

        // Declare built-in functions
        ctx.declare_builtins();

        // v0.35.4: Two-pass approach for forward references
        // Pass 1: Declare all user functions
        for func in &program.functions {
            ctx.declare_function(func)?;
        }

        // Pass 2: Generate function bodies
        for func in &program.functions {
            ctx.gen_function_body(func)?;
        }

        Ok(ctx.module.print_to_string().to_string())
    }

    /// Write module to object file
    fn write_object_file(&self, module: &Module, output: &Path) -> CodeGenResult<()> {
        // Initialize all targets
        Target::initialize_all(&InitializationConfig::default());

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

        // Use native CPU for best performance (like Rust's -C target-cpu=native)
        let cpu = TargetMachine::get_host_cpu_name();
        let features = TargetMachine::get_host_cpu_features();

        let target_machine = target
            .create_target_machine(
                &target_triple,
                cpu.to_str().unwrap_or("x86-64"),
                features.to_str().unwrap_or(""),
                self.opt_level.into(),
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or(CodeGenError::TargetMachineError)?;

        // v0.50.12: Run LLVM optimization passes before writing object file
        // This is critical for performance - without this, the IR is unoptimized
        if !matches!(self.opt_level, OptLevel::Debug) {
            let passes = match self.opt_level {
                OptLevel::Debug => "default<O0>",
                OptLevel::Release => "default<O2>",
                OptLevel::Size => "default<Os>",
                OptLevel::Aggressive => "default<O3>",
            };

            let pass_options = PassBuilderOptions::create();
            pass_options.set_loop_vectorization(true);
            pass_options.set_loop_slp_vectorization(true);  // v0.50.14: Enable SLP vectorization
            pass_options.set_loop_unrolling(true);
            pass_options.set_merge_functions(true);

            module
                .run_passes(passes, &target_machine, pass_options)
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
        }

        target_machine
            .write_to_file(module, FileType::Object, output)
            .map_err(|e| CodeGenError::ObjectFileError(e.to_string()))
    }
}

impl Default for CodeGen {
    fn default() -> Self {
        Self::new()
    }
}

/// LLVM context for code generation
struct LlvmContext<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    /// Function lookup table
    functions: HashMap<String, FunctionValue<'ctx>>,

    /// Variable lookup table (local to current function)
    /// Stores (pointer, type) pairs for opaque pointer support
    variables: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,

    /// Block lookup table (local to current function)
    blocks: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>>,
}

impl<'ctx> LlvmContext<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("bmb_program");
        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            functions: HashMap::new(),
            variables: HashMap::new(),
            blocks: HashMap::new(),
        }
    }

    /// Declare built-in runtime functions
    fn declare_builtins(&mut self) {
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type();
        let bool_type = self.context.bool_type();

        // println(i64) -> void
        let println_type = void_type.fn_type(&[i64_type.into()], false);
        let println_fn = self.module.add_function("bmb_println_i64", println_type, None);
        self.functions.insert("println".to_string(), println_fn);

        // print(i64) -> void
        let print_type = void_type.fn_type(&[i64_type.into()], false);
        let print_fn = self.module.add_function("bmb_print_i64", print_type, None);
        self.functions.insert("print".to_string(), print_fn);

        // read_int() -> i64
        let read_int_type = i64_type.fn_type(&[], false);
        let read_int_fn = self.module.add_function("bmb_read_int", read_int_type, None);
        self.functions.insert("read_int".to_string(), read_int_fn);

        // assert(bool) -> void
        let assert_type = void_type.fn_type(&[bool_type.into()], false);
        let assert_fn = self.module.add_function("bmb_assert", assert_type, None);
        self.functions.insert("assert".to_string(), assert_fn);

        // abs(i64) -> i64
        let abs_type = i64_type.fn_type(&[i64_type.into()], false);
        let abs_fn = self.module.add_function("bmb_abs", abs_type, None);
        self.functions.insert("abs".to_string(), abs_fn);

        // min(i64, i64) -> i64
        let min_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let min_fn = self.module.add_function("bmb_min", min_type, None);
        self.functions.insert("min".to_string(), min_fn);

        // max(i64, i64) -> i64
        let max_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let max_fn = self.module.add_function("bmb_max", max_type, None);
        self.functions.insert("max".to_string(), max_fn);

        // v0.35.4: f64 math intrinsics
        let f64_type = self.context.f64_type();

        // sqrt(f64) -> f64 (LLVM intrinsic)
        let sqrt_type = f64_type.fn_type(&[f64_type.into()], false);
        let sqrt_fn = self.module.add_function("llvm.sqrt.f64", sqrt_type, None);
        self.functions.insert("sqrt".to_string(), sqrt_fn);

        // i64_to_f64(i64) -> f64
        // This is handled by sitofp instruction, but we declare it as a placeholder
        // The actual implementation is in gen_call
        let i64_to_f64_type = f64_type.fn_type(&[i64_type.into()], false);
        let i64_to_f64_fn = self.module.add_function("bmb_i64_to_f64", i64_to_f64_type, None);
        self.functions.insert("i64_to_f64".to_string(), i64_to_f64_fn);

        // f64_to_i64(f64) -> i64
        let f64_to_i64_type = i64_type.fn_type(&[f64_type.into()], false);
        let f64_to_i64_fn = self.module.add_function("bmb_f64_to_i64", f64_to_i64_type, None);
        self.functions.insert("f64_to_i64".to_string(), f64_to_i64_fn);

        // v0.97: Character functions
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        // v0.46: chr(i64) -> ptr (returns single-char string)
        let chr_type = ptr_type.fn_type(&[i64_type.into()], false);
        let chr_fn = self.module.add_function("bmb_chr", chr_type, None);
        self.functions.insert("chr".to_string(), chr_fn);

        // v0.46: ord(ptr) -> i64 (takes string, returns first char code)
        let ord_type = i64_type.fn_type(&[ptr_type.into()], false);
        let ord_fn = self.module.add_function("bmb_ord", ord_type, None);
        self.functions.insert("ord".to_string(), ord_fn);

        // v0.97: String functions
        // print_str(ptr) -> void
        let print_str_type = void_type.fn_type(&[ptr_type.into()], false);
        let print_str_fn = self.module.add_function("bmb_print_str", print_str_type, None);
        self.functions.insert("print_str".to_string(), print_str_fn);

        // println_str(ptr) -> void
        let println_str_type = void_type.fn_type(&[ptr_type.into()], false);
        let println_str_fn = self.module.add_function("bmb_println_str", println_str_type, None);
        self.functions.insert("println_str".to_string(), println_str_fn);

        // len(ptr) -> i64
        let len_type = i64_type.fn_type(&[ptr_type.into()], false);
        let len_fn = self.module.add_function("bmb_string_len", len_type, None);
        self.functions.insert("len".to_string(), len_fn);

        // v0.46: byte_at(ptr, i64) -> i64
        let byte_at_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        let byte_at_fn = self.module.add_function("byte_at", byte_at_type, None);
        self.functions.insert("byte_at".to_string(), byte_at_fn);

        // v0.46: slice(ptr, i64, i64) -> ptr
        let slice_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);
        let slice_fn = self.module.add_function("slice", slice_type, None);
        self.functions.insert("slice".to_string(), slice_fn);

        // v0.46: string_eq(ptr, ptr) -> i64 (for BmbString* comparison)
        let string_eq_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let string_eq_fn = self.module.add_function("bmb_string_eq", string_eq_type, None);
        self.functions.insert("string_eq".to_string(), string_eq_fn);

        // v0.98: Vector functions
        // vec_new() -> i64 (returns pointer as i64)
        let vec_new_type = i64_type.fn_type(&[], false);
        let vec_new_fn = self.module.add_function("bmb_vec_new", vec_new_type, None);
        self.functions.insert("vec_new".to_string(), vec_new_fn);

        // vec_with_capacity(cap: i64) -> i64
        let vec_with_cap_type = i64_type.fn_type(&[i64_type.into()], false);
        let vec_with_cap_fn = self.module.add_function("bmb_vec_with_capacity", vec_with_cap_type, None);
        self.functions.insert("vec_with_capacity".to_string(), vec_with_cap_fn);

        // vec_push(vec: i64, value: i64) -> void
        let vec_push_type = void_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let vec_push_fn = self.module.add_function("bmb_vec_push", vec_push_type, None);
        self.functions.insert("vec_push".to_string(), vec_push_fn);

        // vec_pop(vec: i64) -> i64
        let vec_pop_type = i64_type.fn_type(&[i64_type.into()], false);
        let vec_pop_fn = self.module.add_function("bmb_vec_pop", vec_pop_type, None);
        self.functions.insert("vec_pop".to_string(), vec_pop_fn);

        // vec_get(vec: i64, index: i64) -> i64
        let vec_get_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let vec_get_fn = self.module.add_function("bmb_vec_get", vec_get_type, None);
        self.functions.insert("vec_get".to_string(), vec_get_fn);

        // vec_set(vec: i64, index: i64, value: i64) -> void
        let vec_set_type = void_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let vec_set_fn = self.module.add_function("bmb_vec_set", vec_set_type, None);
        self.functions.insert("vec_set".to_string(), vec_set_fn);

        // vec_len(vec: i64) -> i64
        let vec_len_type = i64_type.fn_type(&[i64_type.into()], false);
        let vec_len_fn = self.module.add_function("bmb_vec_len", vec_len_type, None);
        self.functions.insert("vec_len".to_string(), vec_len_fn);

        // vec_cap(vec: i64) -> i64
        let vec_cap_type = i64_type.fn_type(&[i64_type.into()], false);
        let vec_cap_fn = self.module.add_function("bmb_vec_cap", vec_cap_type, None);
        self.functions.insert("vec_cap".to_string(), vec_cap_fn);

        // vec_free(vec: i64) -> void
        let vec_free_type = void_type.fn_type(&[i64_type.into()], false);
        let vec_free_fn = self.module.add_function("bmb_vec_free", vec_free_type, None);
        self.functions.insert("vec_free".to_string(), vec_free_fn);

        // vec_clear(vec: i64) -> void
        let vec_clear_type = void_type.fn_type(&[i64_type.into()], false);
        let vec_clear_fn = self.module.add_function("bmb_vec_clear", vec_clear_type, None);
        self.functions.insert("vec_clear".to_string(), vec_clear_fn);

        // v0.99: String conversion functions
        // char_to_string(c: i32) -> ptr (returns heap-allocated string)
        let char_to_str_type = ptr_type.fn_type(&[i32_type.into()], false);
        let char_to_str_fn = self.module.add_function("bmb_char_to_string", char_to_str_type, None);
        self.functions.insert("char_to_string".to_string(), char_to_str_fn);

        // int_to_string(n: i64) -> ptr
        let int_to_str_type = ptr_type.fn_type(&[i64_type.into()], false);
        let int_to_str_fn = self.module.add_function("bmb_int_to_string", int_to_str_type, None);
        self.functions.insert("int_to_string".to_string(), int_to_str_fn);

        // v0.46: string_from_cstr - convert C string to BmbString
        // string_from_cstr(cstr: ptr) -> ptr (returns BmbString*)
        let string_from_cstr_type = ptr_type.fn_type(&[ptr_type.into()], false);
        let string_from_cstr_fn = self.module.add_function("bmb_string_from_cstr", string_from_cstr_type, None);
        self.functions.insert("string_from_cstr".to_string(), string_from_cstr_fn);

        // v0.100: String concatenation
        // string_concat(a: ptr, b: ptr) -> ptr
        let string_concat_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let string_concat_fn = self.module.add_function("bmb_string_concat", string_concat_type, None);
        self.functions.insert("string_concat".to_string(), string_concat_fn);

        // v0.46: StringBuilder functions
        // sb_new() -> i64 (returns handle)
        let sb_new_type = i64_type.fn_type(&[], false);
        let sb_new_fn = self.module.add_function("bmb_sb_new", sb_new_type, None);
        self.functions.insert("sb_new".to_string(), sb_new_fn);

        // sb_push(handle: i64, s: ptr) -> i64
        let sb_push_type = i64_type.fn_type(&[i64_type.into(), ptr_type.into()], false);
        let sb_push_fn = self.module.add_function("bmb_sb_push", sb_push_type, None);
        self.functions.insert("sb_push".to_string(), sb_push_fn);

        // sb_len(handle: i64) -> i64
        let sb_len_type = i64_type.fn_type(&[i64_type.into()], false);
        let sb_len_fn = self.module.add_function("bmb_sb_len", sb_len_type, None);
        self.functions.insert("sb_len".to_string(), sb_len_fn);

        // sb_build(handle: i64) -> ptr (returns BmbString*)
        let sb_build_type = ptr_type.fn_type(&[i64_type.into()], false);
        let sb_build_fn = self.module.add_function("bmb_sb_build", sb_build_type, None);
        self.functions.insert("sb_build".to_string(), sb_build_fn);

        // sb_clear(handle: i64) -> i64
        let sb_clear_type = i64_type.fn_type(&[i64_type.into()], false);
        let sb_clear_fn = self.module.add_function("bmb_sb_clear", sb_clear_type, None);
        self.functions.insert("sb_clear".to_string(), sb_clear_fn);

        // Memory allocation (libc)
        // malloc(size: i64) -> ptr
        let malloc_type = ptr_type.fn_type(&[i64_type.into()], false);
        let malloc_fn = self.module.add_function("malloc", malloc_type, None);
        self.functions.insert("malloc".to_string(), malloc_fn);

        // realloc(ptr: ptr, size: i64) -> ptr
        let realloc_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        let realloc_fn = self.module.add_function("realloc", realloc_type, None);
        self.functions.insert("realloc".to_string(), realloc_fn);

        // free(ptr: ptr) -> void
        let free_type = void_type.fn_type(&[ptr_type.into()], false);
        let free_fn = self.module.add_function("free", free_type, None);
        self.functions.insert("free".to_string(), free_fn);

        // Memory access functions
        // store_i64(ptr: i64, value: i64) -> void
        let store_i64_type = void_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let store_i64_fn = self.module.add_function("bmb_store_i64", store_i64_type, None);
        self.functions.insert("store_i64".to_string(), store_i64_fn);

        // load_i64(ptr: i64) -> i64
        let load_i64_type = i64_type.fn_type(&[i64_type.into()], false);
        let load_i64_fn = self.module.add_function("bmb_load_i64", load_i64_type, None);
        self.functions.insert("load_i64".to_string(), load_i64_fn);

        // calloc(count: i64, size: i64) -> i64
        let calloc_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let calloc_fn = self.module.add_function("bmb_calloc", calloc_type, None);
        self.functions.insert("calloc".to_string(), calloc_fn);

        // box_new_i64(value: i64) -> i64
        let box_new_type = i64_type.fn_type(&[i64_type.into()], false);
        let box_new_fn = self.module.add_function("bmb_box_new_i64", box_new_type, None);
        self.functions.insert("box_new_i64".to_string(), box_new_fn);

        // v0.46: File I/O functions for CLI Independence
        // read_file(path: ptr) -> ptr (returns string content)
        let read_file_type = ptr_type.fn_type(&[ptr_type.into()], false);
        let read_file_fn = self.module.add_function("bmb_read_file", read_file_type, None);
        self.functions.insert("read_file".to_string(), read_file_fn);

        // write_file(path: ptr, content: ptr) -> i64 (returns 0 on success, -1 on error)
        let write_file_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let write_file_fn = self.module.add_function("bmb_write_file", write_file_type, None);
        self.functions.insert("write_file".to_string(), write_file_fn);

        // file_exists(path: ptr) -> i64 (returns 1 if exists, 0 otherwise)
        let file_exists_type = i64_type.fn_type(&[ptr_type.into()], false);
        let file_exists_fn = self.module.add_function("bmb_file_exists", file_exists_type, None);
        self.functions.insert("file_exists".to_string(), file_exists_fn);

        // v0.46: Command-line argument functions for CLI Independence
        // arg_count() -> i64
        let arg_count_type = i64_type.fn_type(&[], false);
        let arg_count_fn = self.module.add_function("bmb_arg_count", arg_count_type, None);
        self.functions.insert("arg_count".to_string(), arg_count_fn);

        // get_arg(index: i64) -> ptr (returns string)
        let get_arg_type = ptr_type.fn_type(&[i64_type.into()], false);
        let get_arg_fn = self.module.add_function("bmb_get_arg", get_arg_type, None);
        self.functions.insert("get_arg".to_string(), get_arg_fn);
    }

    /// Convert MIR type to LLVM type
    fn mir_type_to_llvm(&self, ty: &MirType) -> BasicTypeEnum<'ctx> {
        match ty {
            MirType::I32 => self.context.i32_type().into(),
            MirType::I64 => self.context.i64_type().into(),
            // v0.95: Added unsigned integer types
            MirType::U32 => self.context.i32_type().into(),
            MirType::U64 => self.context.i64_type().into(),
            MirType::F64 => self.context.f64_type().into(),
            MirType::Bool => self.context.bool_type().into(),
            // v0.95: Char represented as i32 (Unicode code point)
            MirType::Char => self.context.i32_type().into(),
            MirType::Unit => self.context.i8_type().into(), // Unit represented as i8
            // v0.35: String represented as i8 pointer
            MirType::String => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            // Struct/Enum/Array types - use i64 pointer as placeholder for now
            MirType::Struct { .. }
            | MirType::StructPtr(_)
            | MirType::Enum { .. }
            | MirType::Array { .. } => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        }
    }

    /// v0.35.4: Declare a function signature (pass 1 of two-pass approach)
    fn declare_function(&mut self, func: &MirFunction) -> CodeGenResult<()> {
        // Build function type
        let ret_type = self.mir_type_to_llvm(&func.ret_ty);
        let param_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|(_, ty)| self.mir_type_to_llvm(ty).into())
            .collect();

        let fn_type = match &func.ret_ty {
            MirType::Unit => self.context.void_type().fn_type(&param_types, false),
            _ => ret_type.fn_type(&param_types, false),
        };

        // v0.35: Rename BMB main to bmb_user_main so C runtime can provide real main()
        let emitted_name = if func.name == "main" {
            "bmb_user_main"
        } else {
            &func.name
        };

        // Create function declaration
        let function = self.module.add_function(emitted_name, fn_type, None);
        self.functions.insert(func.name.clone(), function);
        Ok(())
    }

    /// v0.35.4: Generate function body (pass 2 of two-pass approach)
    fn gen_function_body(&mut self, func: &MirFunction) -> CodeGenResult<()> {
        // Clear per-function state
        self.variables.clear();
        self.blocks.clear();

        // Get the already-declared function
        let function = *self.functions.get(&func.name)
            .ok_or_else(|| CodeGenError::UnknownFunction(func.name.clone()))?;

        // Create all basic blocks first
        for block in &func.blocks {
            let bb = self.context.append_basic_block(function, &block.label);
            self.blocks.insert(block.label.clone(), bb);
        }

        // Position at entry block
        if let Some(entry) = self.blocks.get("entry") {
            self.builder.position_at_end(*entry);
        } else if let Some(first_block) = func.blocks.first() {
            let bb = self.blocks.get(&first_block.label).unwrap();
            self.builder.position_at_end(*bb);
        } else {
            return Ok(());
        }

        // Allocate parameters
        for (i, (name, ty)) in func.params.iter().enumerate() {
            let llvm_ty = self.mir_type_to_llvm(ty);
            let alloca = self.builder.build_alloca(llvm_ty, name)
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            let param = function.get_nth_param(i as u32).unwrap();
            self.builder.build_store(alloca, param)
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            self.variables.insert(name.clone(), (alloca, llvm_ty));
        }

        // Allocate locals
        for (name, ty) in &func.locals {
            let llvm_ty = self.mir_type_to_llvm(ty);
            let alloca = self.builder.build_alloca(llvm_ty, name)
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            self.variables.insert(name.clone(), (alloca, llvm_ty));
        }

        // v0.35.3: Collect PHI stores needed for each block
        // PHI nodes are transformed into stores in predecessor blocks
        let phi_stores = self.collect_phi_stores(func);

        // v0.35.3: Pre-allocate PHI destination variables at function entry
        // This ensures all PHI destinations have allocas in the entry block
        self.allocate_phi_destinations(func)?;

        // Generate code for each block
        for block in &func.blocks {
            let stores_for_block = phi_stores.get(&block.label);
            self.gen_basic_block_with_phi(block, function, stores_for_block)?;
        }

        Ok(())
    }

    /// v0.35.3: Collect PHI stores needed for each predecessor block
    /// Returns: HashMap<block_label, Vec<(dest_place, value_operand)>>
    fn collect_phi_stores(&self, func: &MirFunction) -> HashMap<String, Vec<(Place, Operand)>> {
        let mut phi_stores: HashMap<String, Vec<(Place, Operand)>> = HashMap::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst {
                    // For each (value, source_block), record that source_block needs to store value to dest
                    for (value, source_label) in values {
                        phi_stores
                            .entry(source_label.clone())
                            .or_default()
                            .push((dest.clone(), value.clone()));
                    }
                }
            }
        }

        phi_stores
    }

    /// v0.35.3: Pre-allocate PHI destination variables at function entry
    /// This ensures all PHI destinations have allocas in the entry block, not in branch blocks
    fn allocate_phi_destinations(&mut self, func: &MirFunction) -> CodeGenResult<()> {
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst {
                    // Skip if already allocated (e.g., in locals)
                    if self.variables.contains_key(&dest.name) {
                        continue;
                    }

                    // Determine type from the first incoming value
                    // v0.46: Use constant_type() to avoid generating code during type determination
                    if let Some((first_value, _)) = values.first() {
                        let llvm_type = match first_value {
                            Operand::Constant(c) => self.constant_type(c),
                            Operand::Place(p) => {
                                if let Some((_, ty)) = self.variables.get(&p.name) {
                                    *ty
                                } else {
                                    // Default to i64 if type unknown
                                    self.context.i64_type().into()
                                }
                            }
                        };

                        let alloca = self.builder.build_alloca(llvm_type, &dest.name)
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                        self.variables.insert(dest.name.clone(), (alloca, llvm_type));
                    }
                }
            }
        }

        Ok(())
    }

    /// v0.35.3: Generate code for a basic block with PHI store support
    fn gen_basic_block_with_phi(
        &mut self,
        block: &BasicBlock,
        _function: FunctionValue<'ctx>,
        phi_stores: Option<&Vec<(Place, Operand)>>,
    ) -> CodeGenResult<()> {
        let bb = self.blocks.get(&block.label).unwrap();
        self.builder.position_at_end(*bb);

        // Generate instructions (skip PHI nodes - they're handled by stores in predecessors)
        for inst in &block.instructions {
            if matches!(inst, MirInst::Phi { .. }) {
                // PHI nodes are transformed into stores in predecessor blocks
                continue;
            }
            self.gen_instruction(inst)?;
        }

        // v0.35.3: Generate PHI stores before terminator
        // These store values for PHI nodes in successor blocks
        if let Some(stores) = phi_stores {
            for (dest, value) in stores {
                let llvm_value = self.gen_operand(value)?;
                self.store_to_place(dest, llvm_value)?;
            }
        }

        // Generate terminator
        self.gen_terminator(&block.terminator)?;

        Ok(())
    }

    /// Generate code for an instruction
    fn gen_instruction(&mut self, inst: &MirInst) -> CodeGenResult<()> {
        match inst {
            MirInst::Const { dest, value } => {
                let llvm_value = self.gen_constant(value);
                self.store_to_place(dest, llvm_value)?;
            }

            MirInst::Copy { dest, src } => {
                let value = self.load_from_place(src)?;
                self.store_to_place(dest, value)?;
            }

            MirInst::BinOp { dest, op, lhs, rhs } => {
                let lhs_val = self.gen_operand(lhs)?;
                let rhs_val = self.gen_operand(rhs)?;
                let result = self.gen_binop(*op, lhs_val, rhs_val)?;
                self.store_to_place(dest, result)?;
            }

            MirInst::UnaryOp { dest, op, src } => {
                let src_val = self.gen_operand(src)?;
                let result = self.gen_unaryop(*op, src_val)?;
                self.store_to_place(dest, result)?;
            }

            MirInst::Call { dest, func, args } => {
                // v0.35.4: Handle type conversion intrinsics specially
                if func == "i64_to_f64" && args.len() == 1 {
                    let arg = self.gen_operand(&args[0])?;
                    let result = self.builder
                        .build_signed_int_to_float(arg.into_int_value(), self.context.f64_type(), "sitofp")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    if let Some(dest_place) = dest {
                        self.store_to_place(dest_place, result.into())?;
                    }
                } else if func == "f64_to_i64" && args.len() == 1 {
                    let arg = self.gen_operand(&args[0])?;
                    let result = self.builder
                        .build_float_to_signed_int(arg.into_float_value(), self.context.i64_type(), "fptosi")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    if let Some(dest_place) = dest {
                        self.store_to_place(dest_place, result.into())?;
                    }
                } else {
                    let function = self
                        .functions
                        .get(func)
                        .ok_or_else(|| CodeGenError::UnknownFunction(func.clone()))?;

                    let arg_values: Vec<BasicMetadataValueEnum> = args
                        .iter()
                        .map(|arg| self.gen_operand(arg).map(|v| v.into()))
                        .collect::<Result<_, _>>()?;

                    let call_result = self.builder
                        .build_call(*function, &arg_values, "call")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    if let Some(dest_place) = dest {
                        if let Some(ret_val) = call_result.try_as_basic_value().basic() {
                            self.store_to_place(dest_place, ret_val)?;
                        }
                    }
                }
            }

            // v0.35.3: PHI nodes are handled by stores in predecessor blocks
            // This should not be reached since gen_basic_block_with_phi skips them
            MirInst::Phi { .. } => {
                // PHI nodes are transformed into stores in predecessor blocks
                // If we reach here, it's a bug
            }
            MirInst::StructInit { .. }
            | MirInst::FieldAccess { .. }
            | MirInst::FieldStore { .. }
            | MirInst::EnumVariant { .. }
            | MirInst::ArrayInit { .. }
            | MirInst::IndexLoad { .. }
            | MirInst::IndexStore { .. } => {
                return Err(CodeGenError::LlvmError(
                    "Struct/Enum/Array instructions not yet supported in LLVM codegen".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Generate code for a terminator
    fn gen_terminator(&self, term: &Terminator) -> CodeGenResult<()> {
        match term {
            Terminator::Return(Some(op)) => {
                let value = self.gen_operand(op)?;
                self.builder.build_return(Some(&value))
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            Terminator::Return(None) => {
                self.builder.build_return(None)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            Terminator::Goto(label) => {
                let target = self
                    .blocks
                    .get(label)
                    .ok_or_else(|| CodeGenError::UnknownBlock(label.clone()))?;
                self.builder.build_unconditional_branch(*target)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            Terminator::Branch {
                cond,
                then_label,
                else_label,
            } => {
                let cond_val = self.gen_operand(cond)?;
                let cond_int = cond_val.into_int_value();

                let then_bb = self
                    .blocks
                    .get(then_label)
                    .ok_or_else(|| CodeGenError::UnknownBlock(then_label.clone()))?;
                let else_bb = self
                    .blocks
                    .get(else_label)
                    .ok_or_else(|| CodeGenError::UnknownBlock(else_label.clone()))?;

                self.builder.build_conditional_branch(cond_int, *then_bb, *else_bb)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            Terminator::Unreachable => {
                self.builder.build_unreachable()
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            // v0.35: Switch terminator for enum matching
            Terminator::Switch { .. } => {
                return Err(CodeGenError::LlvmError(
                    "Switch terminator not yet supported in LLVM codegen".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get the LLVM type for a constant without generating code
    /// Used by allocate_phi_destinations to determine types without side effects
    fn constant_type(&self, constant: &Constant) -> BasicTypeEnum<'ctx> {
        match constant {
            Constant::Int(_) => self.context.i64_type().into(),
            Constant::Float(_) => self.context.f64_type().into(),
            Constant::Bool(_) => self.context.bool_type().into(),
            Constant::String(_) => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
            Constant::Unit => self.context.i8_type().into(),
            Constant::Char(_) => self.context.i32_type().into(),
        }
    }

    /// Generate a constant value
    fn gen_constant(&self, constant: &Constant) -> BasicValueEnum<'ctx> {
        match constant {
            Constant::Int(n) => self.context.i64_type().const_int(*n as u64, true).into(),
            Constant::Float(f) => self.context.f64_type().const_float(*f).into(),
            Constant::Bool(b) => self
                .context
                .bool_type()
                .const_int(*b as u64, false)
                .into(),
            Constant::String(s) => {
                // v0.46: Create a BmbString from the C string constant
                // First create the global C string constant
                let global = self
                    .builder
                    .build_global_string_ptr(s, "str_const")
                    .expect("Failed to build global string");
                let cstr_ptr = global.as_pointer_value();

                // Wrap with bmb_string_from_cstr to create proper BmbString*
                let string_from_cstr_fn = self.functions.get("string_from_cstr")
                    .expect("string_from_cstr not declared");
                let call_result = self.builder
                    .build_call(*string_from_cstr_fn, &[cstr_ptr.into()], "bmb_str")
                    .expect("Failed to build string_from_cstr call");
                call_result.try_as_basic_value().basic()
                    .expect("string_from_cstr should return a value")
            }
            Constant::Unit => self.context.i8_type().const_int(0, false).into(),
            // v0.95: Char as i32 Unicode code point
            Constant::Char(c) => self.context.i32_type().const_int(*c as u64, false).into(),
        }
    }

    /// Generate code for an operand
    fn gen_operand(&self, op: &Operand) -> CodeGenResult<BasicValueEnum<'ctx>> {
        match op {
            Operand::Constant(c) => Ok(self.gen_constant(c)),
            Operand::Place(p) => self.load_from_place(p),
        }
    }

    /// Load a value from a place
    fn load_from_place(&self, place: &Place) -> CodeGenResult<BasicValueEnum<'ctx>> {
        let (ptr, pointee_type) = self
            .variables
            .get(&place.name)
            .ok_or_else(|| CodeGenError::UnknownVariable(place.name.clone()))?;

        self.builder
            .build_load(*pointee_type, *ptr, &place.name)
            .map_err(|e| CodeGenError::LlvmError(e.to_string()))
    }

    /// Store a value to a place
    fn store_to_place(&mut self, place: &Place, value: BasicValueEnum<'ctx>) -> CodeGenResult<()> {
        // Get or create the variable
        let ptr = if let Some((ptr, _)) = self.variables.get(&place.name) {
            *ptr
        } else {
            // Create a new alloca for temporaries
            let ty = value.get_type();
            let alloca = self.builder.build_alloca(ty, &place.name)
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            self.variables.insert(place.name.clone(), (alloca, ty));
            alloca
        };

        self.builder.build_store(ptr, value)
            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
        Ok(())
    }

    /// Generate a binary operation
    fn gen_binop(
        &self,
        op: MirBinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        match op {
            // Integer arithmetic with nsw (no signed wrap) for better optimization
            // nsw enables more aggressive LLVM transformations
            MirBinOp::Add => {
                // v0.100: Check if operands are pointers (strings) - use string_concat
                if lhs.is_pointer_value() && rhs.is_pointer_value() {
                    let string_concat_fn = self.functions.get("string_concat")
                        .ok_or_else(|| CodeGenError::UnknownFunction("string_concat".to_string()))?;
                    let call_result = self.builder
                        .build_call(*string_concat_fn, &[lhs.into(), rhs.into()], "strcat")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    let result = call_result.try_as_basic_value().basic()
                        .ok_or_else(|| CodeGenError::LlvmError("string_concat should return a value".to_string()))?;
                    Ok(result)
                } else if lhs.is_pointer_value() || rhs.is_pointer_value() {
                    // v0.46: Pointer arithmetic - convert pointer to i64 for arithmetic
                    let lhs_int = if lhs.is_pointer_value() {
                        self.builder.build_ptr_to_int(lhs.into_pointer_value(), self.context.i64_type(), "ptr_to_int")
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    } else {
                        lhs.into_int_value()
                    };
                    let rhs_int = if rhs.is_pointer_value() {
                        self.builder.build_ptr_to_int(rhs.into_pointer_value(), self.context.i64_type(), "ptr_to_int")
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    } else {
                        rhs.into_int_value()
                    };
                    let result = self.builder
                        .build_int_nsw_add(lhs_int, rhs_int, "add")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else {
                    let result = self.builder
                        .build_int_nsw_add(lhs.into_int_value(), rhs.into_int_value(), "add")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                }
            }
            MirBinOp::Sub => {
                // v0.46: Handle pointer arithmetic for subtraction
                let lhs_int = if lhs.is_pointer_value() {
                    self.builder.build_ptr_to_int(lhs.into_pointer_value(), self.context.i64_type(), "ptr_to_int")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                } else {
                    lhs.into_int_value()
                };
                let rhs_int = if rhs.is_pointer_value() {
                    self.builder.build_ptr_to_int(rhs.into_pointer_value(), self.context.i64_type(), "ptr_to_int")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                } else {
                    rhs.into_int_value()
                };
                let result = self.builder
                    .build_int_nsw_sub(lhs_int, rhs_int, "sub")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Mul => {
                let result = self.builder
                    .build_int_nsw_mul(lhs.into_int_value(), rhs.into_int_value(), "mul")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Div => {
                let result = self.builder
                    .build_int_signed_div(lhs.into_int_value(), rhs.into_int_value(), "div")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Mod => {
                let result = self.builder
                    .build_int_signed_rem(lhs.into_int_value(), rhs.into_int_value(), "mod")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // Float arithmetic
            MirBinOp::FAdd => {
                let result = self.builder
                    .build_float_add(lhs.into_float_value(), rhs.into_float_value(), "fadd")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::FSub => {
                let result = self.builder
                    .build_float_sub(lhs.into_float_value(), rhs.into_float_value(), "fsub")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::FMul => {
                let result = self.builder
                    .build_float_mul(lhs.into_float_value(), rhs.into_float_value(), "fmul")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::FDiv => {
                let result = self.builder
                    .build_float_div(lhs.into_float_value(), rhs.into_float_value(), "fdiv")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // Integer/String comparison
            MirBinOp::Eq => {
                // v0.46: Check if either operand is a pointer (strings) - use string_eq
                if lhs.is_pointer_value() || rhs.is_pointer_value() {
                    let string_eq_fn = self.functions.get("string_eq")
                        .ok_or_else(|| CodeGenError::UnknownFunction("string_eq".to_string()))?;
                    let call_result = self.builder
                        .build_call(*string_eq_fn, &[lhs.into(), rhs.into()], "streq")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    let eq_i64 = call_result.try_as_basic_value().basic()
                        .ok_or_else(|| CodeGenError::LlvmError("string_eq should return a value".to_string()))?;
                    // Convert i64 to i1 (bool): non-zero means equal
                    let result = self.builder
                        .build_int_compare(IntPredicate::NE, eq_i64.into_int_value(), self.context.i64_type().const_zero(), "streq_bool")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else {
                    let result = self.builder
                        .build_int_compare(IntPredicate::EQ, lhs.into_int_value(), rhs.into_int_value(), "eq")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                }
            }
            MirBinOp::Ne => {
                // v0.46: Check if either operand is a pointer (strings) - use string_eq and negate
                if lhs.is_pointer_value() || rhs.is_pointer_value() {
                    let string_eq_fn = self.functions.get("string_eq")
                        .ok_or_else(|| CodeGenError::UnknownFunction("string_eq".to_string()))?;
                    let call_result = self.builder
                        .build_call(*string_eq_fn, &[lhs.into(), rhs.into()], "strne")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    let eq_i64 = call_result.try_as_basic_value().basic()
                        .ok_or_else(|| CodeGenError::LlvmError("string_eq should return a value".to_string()))?;
                    // Convert i64 to i1 (bool): zero means not equal
                    let result = self.builder
                        .build_int_compare(IntPredicate::EQ, eq_i64.into_int_value(), self.context.i64_type().const_zero(), "strne_bool")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else {
                    let result = self.builder
                        .build_int_compare(IntPredicate::NE, lhs.into_int_value(), rhs.into_int_value(), "ne")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                }
            }
            MirBinOp::Lt => {
                let result = self.builder
                    .build_int_compare(IntPredicate::SLT, lhs.into_int_value(), rhs.into_int_value(), "lt")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Gt => {
                let result = self.builder
                    .build_int_compare(IntPredicate::SGT, lhs.into_int_value(), rhs.into_int_value(), "gt")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Le => {
                let result = self.builder
                    .build_int_compare(IntPredicate::SLE, lhs.into_int_value(), rhs.into_int_value(), "le")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Ge => {
                let result = self.builder
                    .build_int_compare(IntPredicate::SGE, lhs.into_int_value(), rhs.into_int_value(), "ge")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // Float comparison
            MirBinOp::FEq => {
                let result = self.builder
                    .build_float_compare(FloatPredicate::OEQ, lhs.into_float_value(), rhs.into_float_value(), "feq")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::FNe => {
                let result = self.builder
                    .build_float_compare(FloatPredicate::ONE, lhs.into_float_value(), rhs.into_float_value(), "fne")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::FLt => {
                let result = self.builder
                    .build_float_compare(FloatPredicate::OLT, lhs.into_float_value(), rhs.into_float_value(), "flt")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::FGt => {
                let result = self.builder
                    .build_float_compare(FloatPredicate::OGT, lhs.into_float_value(), rhs.into_float_value(), "fgt")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::FLe => {
                let result = self.builder
                    .build_float_compare(FloatPredicate::OLE, lhs.into_float_value(), rhs.into_float_value(), "fle")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::FGe => {
                let result = self.builder
                    .build_float_compare(FloatPredicate::OGE, lhs.into_float_value(), rhs.into_float_value(), "fge")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // Logical
            MirBinOp::And => {
                let result = self.builder
                    .build_and(lhs.into_int_value(), rhs.into_int_value(), "and")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Or => {
                let result = self.builder
                    .build_or(lhs.into_int_value(), rhs.into_int_value(), "or")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // v0.32: Shift operators
            MirBinOp::Shl => {
                let result = self.builder
                    .build_left_shift(lhs.into_int_value(), rhs.into_int_value(), "shl")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Shr => {
                // Arithmetic right shift (sign-extending for signed integers)
                let result = self.builder
                    .build_right_shift(lhs.into_int_value(), rhs.into_int_value(), true, "shr")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // v0.95: Wrapping arithmetic (same as regular ops in LLVM, wraps on overflow)
            MirBinOp::AddWrap => {
                let result = self.builder
                    .build_int_add(lhs.into_int_value(), rhs.into_int_value(), "addwrap")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::SubWrap => {
                let result = self.builder
                    .build_int_sub(lhs.into_int_value(), rhs.into_int_value(), "subwrap")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::MulWrap => {
                let result = self.builder
                    .build_int_mul(lhs.into_int_value(), rhs.into_int_value(), "mulwrap")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // v0.95: Checked arithmetic (TODO: proper overflow detection)
            MirBinOp::AddChecked | MirBinOp::SubChecked | MirBinOp::MulChecked => {
                // For now, treat as regular ops (full implementation needs Option return)
                let result = match op {
                    MirBinOp::AddChecked => self.builder.build_int_add(lhs.into_int_value(), rhs.into_int_value(), "addchk"),
                    MirBinOp::SubChecked => self.builder.build_int_sub(lhs.into_int_value(), rhs.into_int_value(), "subchk"),
                    MirBinOp::MulChecked => self.builder.build_int_mul(lhs.into_int_value(), rhs.into_int_value(), "mulchk"),
                    _ => unreachable!(),
                }.map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // v0.95: Saturating arithmetic
            MirBinOp::AddSat | MirBinOp::SubSat | MirBinOp::MulSat => {
                // For now, treat as regular ops (full implementation needs saturation logic)
                let result = match op {
                    MirBinOp::AddSat => self.builder.build_int_add(lhs.into_int_value(), rhs.into_int_value(), "addsat"),
                    MirBinOp::SubSat => self.builder.build_int_sub(lhs.into_int_value(), rhs.into_int_value(), "subsat"),
                    MirBinOp::MulSat => self.builder.build_int_mul(lhs.into_int_value(), rhs.into_int_value(), "mulsat"),
                    _ => unreachable!(),
                }.map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // v0.95: Bitwise operations
            MirBinOp::Bxor => {
                let result = self.builder
                    .build_xor(lhs.into_int_value(), rhs.into_int_value(), "bxor")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Band => {
                let result = self.builder
                    .build_and(lhs.into_int_value(), rhs.into_int_value(), "band")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Bor => {
                let result = self.builder
                    .build_or(lhs.into_int_value(), rhs.into_int_value(), "bor")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // v0.95: Logical implication (a implies b = !a || b)
            MirBinOp::Implies => {
                let not_lhs = self.builder
                    .build_not(lhs.into_int_value(), "not_lhs")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                let result = self.builder
                    .build_or(not_lhs, rhs.into_int_value(), "implies")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
        }
    }

    /// Generate a unary operation
    fn gen_unaryop(
        &self,
        op: MirUnaryOp,
        src: BasicValueEnum<'ctx>,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        match op {
            MirUnaryOp::Neg => {
                let result = self.builder
                    .build_int_neg(src.into_int_value(), "neg")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirUnaryOp::FNeg => {
                let result = self.builder
                    .build_float_neg(src.into_float_value(), "fneg")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirUnaryOp::Not => {
                let result = self.builder
                    .build_not(src.into_int_value(), "not")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            // v0.95: Bitwise NOT
            MirUnaryOp::Bnot => {
                let result = self.builder
                    .build_not(src.into_int_value(), "bnot")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
        }
    }
}
