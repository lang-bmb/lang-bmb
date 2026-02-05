//! LLVM Code Generation using inkwell
//!
//! This module generates LLVM IR from MIR and compiles to object files.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FastMathFlags, FunctionValue, InstructionValue, PhiValue, PointerValue};
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
    Debug,
    #[default]
    Release,  // v0.60.38: Default to Release for optimized builds
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
    /// v0.60.56: Enable fast-math optimizations for floating-point operations
    fast_math: bool,
}

impl CodeGen {
    /// Create a new code generator
    pub fn new() -> Self {
        Self {
            opt_level: OptLevel::default(),
            fast_math: false,
        }
    }

    /// Create a new code generator with optimization level
    pub fn with_opt_level(opt_level: OptLevel) -> Self {
        Self { opt_level, fast_math: false }
    }

    /// Create a new code generator with optimization level and fast-math enabled
    pub fn with_fast_math(opt_level: OptLevel, fast_math: bool) -> Self {
        Self { opt_level, fast_math }
    }

    /// Compile MIR to object file
    pub fn compile(&self, program: &MirProgram, output: &Path) -> CodeGenResult<()> {
        let context = Context::create();
        // v0.60.56: Pass fast_math flag to context
        let mut ctx = LlvmContext::with_fast_math(&context, self.fast_math);

        // v0.60.15: Copy struct definitions for field access codegen
        ctx.struct_defs = program.struct_defs.clone();

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
        // v0.60.56: Pass fast_math flag to context
        let mut ctx = LlvmContext::with_fast_math(&context, self.fast_math);

        // v0.60.7: Copy struct definitions for field access codegen
        ctx.struct_defs = program.struct_defs.clone();

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
        //
        // v0.50.54: Windows GNU target segfaults in run_passes()
        // Root cause: inkwell/LLVM 21.1.8 compatibility issue on MinGW
        // Workaround: Check target triple and skip passes for Windows targets
        let is_windows_target = target_triple.as_str().to_string_lossy().contains("windows");

        if !matches!(self.opt_level, OptLevel::Debug) && !is_windows_target {
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
        } else if !matches!(self.opt_level, OptLevel::Debug) && is_windows_target {
            // v0.50.67: Windows workaround - use external opt tool for optimization
            // The inkwell run_passes() segfaults on Windows with LLVM 21.1.8 on MinGW
            // Solution: Write bitcode, run opt, read back optimized bitcode
            use std::process::Command;

            let opt_level_str = match self.opt_level {
                OptLevel::Debug => "-O0",
                OptLevel::Release => "-O2",
                OptLevel::Size => "-Os",
                OptLevel::Aggressive => "-O3",
            };

            // Write unoptimized bitcode to temp file
            let temp_bc = output.with_extension("unopt.bc");
            let opt_bc = output.with_extension("opt.bc");

            module.write_bitcode_to_path(&temp_bc);

            // Run external opt tool
            // v0.60.47: Use O3 with scalarizer to undo inefficient auto-vectorization
            // O3 enables aggressive inlining and loop unrolling, but its vectorization
            // can hurt performance for integer-heavy loops (e.g., mandelbrot).
            // The scalarizer undoes vector operations, returning to efficient scalar code.
            // Use new pass manager syntax: -passes='default<O3>,scalarizer'
            let passes_arg = match self.opt_level {
                OptLevel::Debug => "default<O0>",
                OptLevel::Release => "default<O3>,scalarizer",  // v0.60.47: Add scalarizer
                OptLevel::Size => "default<Os>",
                OptLevel::Aggressive => "default<O3>",  // Keep vectorization for aggressive
            };

            // v0.60.56: Build opt command with optional fast-math flags
            let mut opt_cmd = Command::new("opt");
            opt_cmd.args(["--passes", passes_arg]);

            // v0.60.56: Add fast-math flags when enabled
            // These enable aggressive FP optimizations (FMA, reciprocal, reassociation)
            if self.fast_math {
                opt_cmd.args([
                    "--enable-unsafe-fp-math",
                    "--enable-no-nans-fp-math",
                    "--enable-no-infs-fp-math",
                    "--enable-no-signed-zeros-fp-math",
                    "--fp-contract=fast",
                ]);
            }

            opt_cmd.arg("-o").arg(&opt_bc).arg(&temp_bc);
            let opt_result = opt_cmd.output();

            let opt_success = match opt_result {
                Ok(output_res) if output_res.status.success() => {
                    // Load optimized bitcode and write object file
                    // v0.60.46: Log the actual passes being used
                    eprintln!("Note: External opt --passes={} completed successfully", passes_arg);
                    let opt_context = Context::create();
                    match inkwell::module::Module::parse_bitcode_from_path(&opt_bc, &opt_context) {
                        Ok(opt_module) => {
                            let opt_target_machine = Target::from_triple(&target_triple)
                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                .create_target_machine(
                                    &target_triple,
                                    cpu.to_str().unwrap_or("generic"),
                                    features.to_str().unwrap_or(""),
                                    self.opt_level.into(),
                                    RelocMode::Default,
                                    CodeModel::Default,
                                )
                                .ok_or(CodeGenError::TargetMachineError)?;

                            eprintln!("Note: Writing optimized object file");
                            let result = opt_target_machine
                                .write_to_file(&opt_module, FileType::Object, output)
                                .map_err(|e| CodeGenError::ObjectFileError(e.to_string()));

                            // Cleanup temp files
                            let _ = std::fs::remove_file(&temp_bc);
                            let _ = std::fs::remove_file(&opt_bc);

                            return result;
                        }
                        Err(e) => {
                            eprintln!("Warning: Could not load optimized bitcode: {}", e);
                            false
                        }
                    }
                }
                Ok(output_res) => {
                    let stderr = String::from_utf8_lossy(&output_res.stderr);
                    eprintln!("Warning: opt tool failed (exit: {:?}): {}",
                              output_res.status.code(), stderr);
                    false
                }
                Err(e) => {
                    eprintln!("Warning: opt tool not found ({})", e);
                    false
                }
            };

            // Cleanup temp files
            let _ = std::fs::remove_file(&opt_bc);

            // v0.60.42: Try llc -O3 as fallback when opt is unavailable
            // llc performs codegen-level optimizations which are often sufficient
            // v0.60.43: Use -O3 for Release mode as well since opt is blocked
            if !opt_success {
                let llc_opt = match self.opt_level {
                    OptLevel::Debug => "-O0",
                    OptLevel::Release => "-O3",  // v0.60.43: -O3 for best codegen without opt
                    OptLevel::Size => "-Os",
                    OptLevel::Aggressive => "-O3",
                };

                let obj_from_llc = output.with_extension("llc.o");
                let llc_result = Command::new("llc")
                    .args([llc_opt, "-filetype=obj", "-o"])
                    .arg(&obj_from_llc)
                    .arg(&temp_bc)
                    .output();

                match llc_result {
                    Ok(output_res) if output_res.status.success() => {
                        eprintln!("Note: llc {} optimization successful", llc_opt);
                        // Move llc output to final destination
                        if let Err(e) = std::fs::rename(&obj_from_llc, output) {
                            // If rename fails (cross-device), try copy
                            if let Err(e2) = std::fs::copy(&obj_from_llc, output) {
                                eprintln!("Warning: Could not move llc output: {}, {}", e, e2);
                            }
                            let _ = std::fs::remove_file(&obj_from_llc);
                        }
                        let _ = std::fs::remove_file(&temp_bc);
                        return Ok(());
                    }
                    Ok(output_res) => {
                        let stderr = String::from_utf8_lossy(&output_res.stderr);
                        eprintln!("Warning: llc failed (exit: {:?}): {}",
                                  output_res.status.code(), stderr);
                    }
                    Err(e) => {
                        eprintln!("Warning: llc not found ({})", e);
                    }
                }
                let _ = std::fs::remove_file(&obj_from_llc);
            }

            let _ = std::fs::remove_file(&temp_bc);
            eprintln!("Note: Writing UNOPTIMIZED object file (fallback)");
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
    /// Used for: function parameters, explicit locals
    /// v0.51.0: PHI destinations no longer use alloca - they're true SSA values
    variables: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,

    /// v0.50.52: SSA value lookup table for temporaries
    /// This avoids alloca/load/store for intermediate values, using registers instead
    /// Dramatically improves performance by reducing memory traffic
    /// v0.51.0: Also stores PHI node results for true SSA semantics
    ssa_values: HashMap<String, BasicValueEnum<'ctx>>,

    /// v0.51.0: PHI node lookup table (local to current function)
    /// Maps PHI destination name to the PHI value for later add_incoming calls
    phi_nodes: HashMap<String, PhiValue<'ctx>>,

    /// Block lookup table (local to current function)
    blocks: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>>,

    /// v0.50.67: Cache for static string literals to avoid repeated allocations
    /// Maps string content to pointer to static BmbString struct
    static_strings: HashMap<String, PointerValue<'ctx>>,

    /// v0.51.18: Cache for raw C string pointers (maps string content to raw data pointer)
    /// Used for functions like file_exists that expect const char* instead of BMB String
    static_cstrings: HashMap<String, PointerValue<'ctx>>,

    /// Counter for unique static string names
    static_string_counter: usize,

    /// v0.50.80: Current function's return type (for type coercion in Return)
    /// Set at the start of gen_function_body, used by gen_terminator
    current_ret_type: Option<BasicTypeEnum<'ctx>>,

    /// v0.60.25: Track array variables - these should NOT be loaded from
    /// Array variables store the alloca pointer directly, not a pointer to a pointer
    array_variables: std::collections::HashSet<String>,

    /// v0.60.7: Struct type definitions for field access codegen
    /// Maps struct name -> list of (field_name, field_type)
    struct_defs: HashMap<String, Vec<(String, MirType)>>,

    /// v0.60.15: Function return types for PHI type inference
    /// Maps function name -> MIR return type
    function_return_types: HashMap<String, MirType>,

    /// v0.60.56: Enable fast-math optimizations for floating-point operations
    /// When enabled, FP operations use aggressive optimizations (FMA, reciprocal, etc.)
    fast_math: bool,

    /// v0.60.81: Track String-typed variables for proper string comparison
    /// Used to distinguish String pointers from typed pointers (*T) in BinOp
    string_variables: std::collections::HashSet<String>,

    /// v0.60.253: Track enum-typed variables for switch discriminant handling
    /// Enum values are passed as i64 (ptrtoint), but switch needs to load discriminant
    enum_variables: std::collections::HashSet<String>,
}

impl<'ctx> LlvmContext<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        Self::with_fast_math(context, false)
    }

    fn with_fast_math(context: &'ctx Context, fast_math: bool) -> Self {
        let module = context.create_module("bmb_program");
        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            functions: HashMap::new(),
            variables: HashMap::new(),
            ssa_values: HashMap::new(),
            phi_nodes: HashMap::new(),
            blocks: HashMap::new(),
            static_strings: HashMap::new(),
            static_cstrings: HashMap::new(),
            static_string_counter: 0,
            current_ret_type: None,
            struct_defs: HashMap::new(),
            function_return_types: HashMap::new(),
            array_variables: std::collections::HashSet::new(),
            fast_math,
            string_variables: std::collections::HashSet::new(),
            enum_variables: std::collections::HashSet::new(),
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

        // v0.60.43: println_f64(f64) -> void - for proper float output
        let f64_type_decl = self.context.f64_type();
        let println_f64_type = void_type.fn_type(&[f64_type_decl.into()], false);
        let println_f64_fn = self.module.add_function("bmb_println_f64", println_f64_type, None);
        self.functions.insert("println_f64".to_string(), println_f64_fn);

        // v0.60.43: print_f64(f64) -> void
        let print_f64_type = void_type.fn_type(&[f64_type_decl.into()], false);
        let print_f64_fn = self.module.add_function("bmb_print_f64", print_f64_type, None);
        self.functions.insert("print_f64".to_string(), print_f64_fn);

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

        // v0.51.15: Helper to add memory/nounwind/willreturn attributes for LICM optimization
        // "argmem: read" tells LLVM these functions only read from their argument memory
        // This is more precise than "read" which means "may read any memory"
        // CRITICAL: This enables LLVM to hoist s.len() calls out of loops because
        // it can prove that other calls in the loop don't affect the argument memory
        use inkwell::attributes::AttributeLoc;
        let memory_read_attr = self.context.create_string_attribute("memory", "argmem: read");
        let nounwind_attr = self.context.create_string_attribute("nounwind", "");
        let willreturn_attr = self.context.create_string_attribute("willreturn", "");
        // v0.51.13: Add nocapture and speculatable for better LICM
        // nocapture: the pointer argument is not stored anywhere (allows better alias analysis)
        // speculatable: the function can be speculatively executed (enables hoisting)
        let nocapture_attr = self.context.create_string_attribute("nocapture", "");
        let speculatable_attr = self.context.create_string_attribute("speculatable", "");
        // v0.51.15: nosync tells LLVM the function doesn't synchronize with other threads
        // This is CRITICAL for LICM - without it, LLVM assumes the function might have
        // side effects that prevent hoisting even if memory attributes are correct
        let nosync_attr = self.context.create_string_attribute("nosync", "");
        // v0.51.15: nofree tells LLVM the function doesn't free memory
        // This enables better GVN optimization after loop rotation
        let nofree_attr = self.context.create_string_attribute("nofree", "");

        // v0.46: chr(i64) -> ptr (returns single-char string)
        // Note: chr allocates memory, so it's not readonly
        let chr_type = ptr_type.fn_type(&[i64_type.into()], false);
        let chr_fn = self.module.add_function("bmb_chr", chr_type, None);
        chr_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        chr_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        self.functions.insert("chr".to_string(), chr_fn);
        self.function_return_types.insert("chr".to_string(), MirType::String);

        // v0.46: ord(ptr) -> i64 (takes string, returns first char code)
        let ord_type = i64_type.fn_type(&[ptr_type.into()], false);
        let ord_fn = self.module.add_function("bmb_ord", ord_type, None);
        ord_fn.add_attribute(AttributeLoc::Function, memory_read_attr);
        ord_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        ord_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        ord_fn.add_attribute(AttributeLoc::Function, nosync_attr);
        ord_fn.add_attribute(AttributeLoc::Function, speculatable_attr);
        ord_fn.add_attribute(AttributeLoc::Param(0), nocapture_attr);
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
        // v0.51.13: Enhanced attributes for LICM optimization
        // - memory(read): only reads memory, no writes
        // - nocapture on param: pointer not stored (better alias analysis)
        // - speculatable: safe to execute speculatively (enables hoisting)
        len_fn.add_attribute(AttributeLoc::Function, memory_read_attr);
        len_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        len_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        len_fn.add_attribute(AttributeLoc::Function, speculatable_attr);
        len_fn.add_attribute(AttributeLoc::Function, nosync_attr);
        len_fn.add_attribute(AttributeLoc::Function, nofree_attr);
        len_fn.add_attribute(AttributeLoc::Param(0), nocapture_attr);
        self.functions.insert("len".to_string(), len_fn);

        // v0.46: byte_at(ptr, i64) -> i64
        let byte_at_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        let byte_at_fn = self.module.add_function("byte_at", byte_at_type, None);
        // v0.51.13: Enhanced attributes for LICM optimization
        byte_at_fn.add_attribute(AttributeLoc::Function, memory_read_attr);
        byte_at_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        byte_at_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        byte_at_fn.add_attribute(AttributeLoc::Function, speculatable_attr);
        byte_at_fn.add_attribute(AttributeLoc::Function, nosync_attr);
        byte_at_fn.add_attribute(AttributeLoc::Param(0), nocapture_attr);
        self.functions.insert("byte_at".to_string(), byte_at_fn);

        // v0.50.75: char_at(ptr, i64) -> i64 (same as byte_at, for compatibility)
        let char_at_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        let char_at_fn = self.module.add_function("char_at", char_at_type, None);
        // v0.51.13: Enhanced attributes for LICM optimization
        char_at_fn.add_attribute(AttributeLoc::Function, memory_read_attr);
        char_at_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        char_at_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        char_at_fn.add_attribute(AttributeLoc::Function, speculatable_attr);
        char_at_fn.add_attribute(AttributeLoc::Function, nosync_attr);
        char_at_fn.add_attribute(AttributeLoc::Param(0), nocapture_attr);
        self.functions.insert("char_at".to_string(), char_at_fn);

        // v0.60.1: load_u8(addr: i64) -> i64 - read single byte from memory address
        // Used for byte-level memory access in brainfuck interpreter, etc.
        let load_u8_type = i64_type.fn_type(&[i64_type.into()], false);
        let load_u8_fn = self.module.add_function("load_u8", load_u8_type, None);
        load_u8_fn.add_attribute(AttributeLoc::Function, memory_read_attr);
        load_u8_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        load_u8_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        load_u8_fn.add_attribute(AttributeLoc::Function, speculatable_attr);
        load_u8_fn.add_attribute(AttributeLoc::Function, nosync_attr);
        self.functions.insert("load_u8".to_string(), load_u8_fn);

        // v0.60.1: store_u8(addr: i64, val: i64) -> void - write single byte to memory address
        let store_u8_type = void_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let store_u8_fn = self.module.add_function("store_u8", store_u8_type, None);
        store_u8_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        store_u8_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        self.functions.insert("store_u8".to_string(), store_u8_fn);

        // v0.60.58: load_i32(addr: i64) -> i64 - read 32-bit signed integer, sign-extended
        let load_i32_type = i64_type.fn_type(&[i64_type.into()], false);
        let load_i32_fn = self.module.add_function("load_i32", load_i32_type, None);
        load_i32_fn.add_attribute(AttributeLoc::Function, memory_read_attr);
        load_i32_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        load_i32_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        load_i32_fn.add_attribute(AttributeLoc::Function, speculatable_attr);
        load_i32_fn.add_attribute(AttributeLoc::Function, nosync_attr);
        self.functions.insert("load_i32".to_string(), load_i32_fn);

        // v0.60.58: store_i32(addr: i64, val: i64) -> void - write lower 32 bits to memory
        let store_i32_type = void_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let store_i32_fn = self.module.add_function("store_i32", store_i32_type, None);
        store_i32_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        store_i32_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        self.functions.insert("store_i32".to_string(), store_i32_fn);

        // v0.46: slice(ptr, i64, i64) -> ptr
        // Note: slice allocates memory, so don't mark as speculatable
        let slice_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);
        let slice_fn = self.module.add_function("slice", slice_type, None);
        slice_fn.add_attribute(AttributeLoc::Function, memory_read_attr);
        slice_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        slice_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        slice_fn.add_attribute(AttributeLoc::Function, nosync_attr);
        self.functions.insert("slice".to_string(), slice_fn);
        // v0.60.120: Register return type for string comparison tracking
        self.function_return_types.insert("slice".to_string(), MirType::String);

        // v0.46: string_eq(ptr, ptr) -> i64 (for BmbString* comparison)
        let string_eq_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let string_eq_fn = self.module.add_function("bmb_string_eq", string_eq_type, None);
        string_eq_fn.add_attribute(AttributeLoc::Function, memory_read_attr);
        string_eq_fn.add_attribute(AttributeLoc::Function, nounwind_attr);
        string_eq_fn.add_attribute(AttributeLoc::Function, willreturn_attr);
        string_eq_fn.add_attribute(AttributeLoc::Function, nosync_attr);
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

        // v0.50.75: Hashmap functions
        // hashmap_new() -> i64 (returns handle)
        let hashmap_new_type = i64_type.fn_type(&[], false);
        let hashmap_new_fn = self.module.add_function("hashmap_new", hashmap_new_type, None);
        self.functions.insert("hashmap_new".to_string(), hashmap_new_fn);

        // hashmap_insert(handle: i64, key: i64, value: i64) -> i64
        let hashmap_insert_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let hashmap_insert_fn = self.module.add_function("hashmap_insert", hashmap_insert_type, None);
        self.functions.insert("hashmap_insert".to_string(), hashmap_insert_fn);

        // hashmap_get(handle: i64, key: i64) -> i64
        let hashmap_get_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let hashmap_get_fn = self.module.add_function("hashmap_get", hashmap_get_type, None);
        self.functions.insert("hashmap_get".to_string(), hashmap_get_fn);

        // hashmap_remove(handle: i64, key: i64) -> i64
        let hashmap_remove_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let hashmap_remove_fn = self.module.add_function("hashmap_remove", hashmap_remove_type, None);
        self.functions.insert("hashmap_remove".to_string(), hashmap_remove_fn);

        // hashmap_len(handle: i64) -> i64
        let hashmap_len_type = i64_type.fn_type(&[i64_type.into()], false);
        let hashmap_len_fn = self.module.add_function("hashmap_len", hashmap_len_type, None);
        self.functions.insert("hashmap_len".to_string(), hashmap_len_fn);

        // v0.60.262: hashmap_contains(handle: i64, key: i64) -> i64
        let hashmap_contains_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let hashmap_contains_fn = self.module.add_function("hashmap_contains", hashmap_contains_type, None);
        self.functions.insert("hashmap_contains".to_string(), hashmap_contains_fn);

        // hashmap_free(handle: i64) -> void
        let hashmap_free_type = void_type.fn_type(&[i64_type.into()], false);
        let hashmap_free_fn = self.module.add_function("hashmap_free", hashmap_free_type, None);
        self.functions.insert("hashmap_free".to_string(), hashmap_free_fn);

        // v0.60.246: String-key hashmap functions (strmap_*)
        // strmap_new() -> i64 (returns handle)
        let strmap_new_type = i64_type.fn_type(&[], false);
        let strmap_new_fn = self.module.add_function("strmap_new", strmap_new_type, None);
        self.functions.insert("strmap_new".to_string(), strmap_new_fn);

        // strmap_insert(handle: i64, key: ptr, value: i64) -> i64
        let strmap_insert_type = i64_type.fn_type(&[i64_type.into(), ptr_type.into(), i64_type.into()], false);
        let strmap_insert_fn = self.module.add_function("strmap_insert", strmap_insert_type, None);
        self.functions.insert("strmap_insert".to_string(), strmap_insert_fn);

        // strmap_get(handle: i64, key: ptr) -> i64
        let strmap_get_type = i64_type.fn_type(&[i64_type.into(), ptr_type.into()], false);
        let strmap_get_fn = self.module.add_function("strmap_get", strmap_get_type, None);
        self.functions.insert("strmap_get".to_string(), strmap_get_fn);

        // strmap_contains(handle: i64, key: ptr) -> i64
        let strmap_contains_type = i64_type.fn_type(&[i64_type.into(), ptr_type.into()], false);
        let strmap_contains_fn = self.module.add_function("strmap_contains", strmap_contains_type, None);
        self.functions.insert("strmap_contains".to_string(), strmap_contains_fn);

        // strmap_size(handle: i64) -> i64
        let strmap_size_type = i64_type.fn_type(&[i64_type.into()], false);
        let strmap_size_fn = self.module.add_function("strmap_size", strmap_size_type, None);
        self.functions.insert("strmap_size".to_string(), strmap_size_fn);

        // v0.99: String conversion functions
        // char_to_string(c: i32) -> ptr (returns heap-allocated string)
        let char_to_str_type = ptr_type.fn_type(&[i32_type.into()], false);
        let char_to_str_fn = self.module.add_function("bmb_char_to_string", char_to_str_type, None);
        self.functions.insert("char_to_string".to_string(), char_to_str_fn);
        self.function_return_types.insert("char_to_string".to_string(), MirType::String);

        // int_to_string(n: i64) -> ptr
        let int_to_str_type = ptr_type.fn_type(&[i64_type.into()], false);
        let int_to_str_fn = self.module.add_function("bmb_int_to_string", int_to_str_type, None);
        self.functions.insert("int_to_string".to_string(), int_to_str_fn);
        self.function_return_types.insert("int_to_string".to_string(), MirType::String);

        // v0.46: string_from_cstr - convert C string to BmbString
        // string_from_cstr(cstr: ptr) -> ptr (returns BmbString*)
        let string_from_cstr_type = ptr_type.fn_type(&[ptr_type.into()], false);
        let string_from_cstr_fn = self.module.add_function("bmb_string_from_cstr", string_from_cstr_type, None);
        self.functions.insert("string_from_cstr".to_string(), string_from_cstr_fn);
        self.function_return_types.insert("string_from_cstr".to_string(), MirType::String);

        // v0.100: String concatenation
        // string_concat(a: ptr, b: ptr) -> ptr
        let string_concat_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let string_concat_fn = self.module.add_function("bmb_string_concat", string_concat_type, None);
        self.functions.insert("string_concat".to_string(), string_concat_fn);
        self.function_return_types.insert("string_concat".to_string(), MirType::String);

        // v0.46: StringBuilder functions
        // sb_new() -> i64 (returns handle)
        let sb_new_type = i64_type.fn_type(&[], false);
        let sb_new_fn = self.module.add_function("bmb_sb_new", sb_new_type, None);
        self.functions.insert("sb_new".to_string(), sb_new_fn);

        // v0.51.45: sb_with_capacity(capacity: i64) -> i64 (returns handle with pre-allocated capacity)
        let sb_with_capacity_type = i64_type.fn_type(&[i64_type.into()], false);
        let sb_with_capacity_fn = self.module.add_function("bmb_sb_with_capacity", sb_with_capacity_type, None);
        self.functions.insert("sb_with_capacity".to_string(), sb_with_capacity_fn);

        // sb_push(handle: i64, s: ptr) -> i64
        let sb_push_type = i64_type.fn_type(&[i64_type.into(), ptr_type.into()], false);
        let sb_push_fn = self.module.add_function("bmb_sb_push", sb_push_type, None);
        self.functions.insert("sb_push".to_string(), sb_push_fn);

        // sb_push_char(handle: i64, char_code: i64) -> i64
        let sb_push_char_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let sb_push_char_fn = self.module.add_function("bmb_sb_push_char", sb_push_char_type, None);
        self.functions.insert("sb_push_char".to_string(), sb_push_char_fn);

        // v0.50.73: sb_push_int(handle: i64, n: i64) -> i64
        let sb_push_int_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let sb_push_int_fn = self.module.add_function("bmb_sb_push_int", sb_push_int_type, None);
        self.functions.insert("sb_push_int".to_string(), sb_push_int_fn);

        // v0.50.74: sb_push_escaped(handle: i64, str: ptr) -> i64
        // Escapes and pushes entire string in one call (eliminates per-char call overhead)
        let sb_push_escaped_type = i64_type.fn_type(&[i64_type.into(), ptr_type.into()], false);
        let sb_push_escaped_fn = self.module.add_function("bmb_sb_push_escaped", sb_push_escaped_type, None);
        self.functions.insert("sb_push_escaped".to_string(), sb_push_escaped_fn);

        // sb_len(handle: i64) -> i64
        let sb_len_type = i64_type.fn_type(&[i64_type.into()], false);
        let sb_len_fn = self.module.add_function("bmb_sb_len", sb_len_type, None);
        self.functions.insert("sb_len".to_string(), sb_len_fn);

        // sb_build(handle: i64) -> ptr (returns BmbString*)
        let sb_build_type = ptr_type.fn_type(&[i64_type.into()], false);
        let sb_build_fn = self.module.add_function("bmb_sb_build", sb_build_type, None);
        self.functions.insert("sb_build".to_string(), sb_build_fn);
        self.function_return_types.insert("sb_build".to_string(), MirType::String);

        // sb_clear(handle: i64) -> i64
        let sb_clear_type = i64_type.fn_type(&[i64_type.into()], false);
        let sb_clear_fn = self.module.add_function("bmb_sb_clear", sb_clear_type, None);
        self.functions.insert("sb_clear".to_string(), sb_clear_fn);

        // sb_println(handle: i64) -> i64 (v0.60.63: print without allocation)
        let sb_println_type = i64_type.fn_type(&[i64_type.into()], false);
        let sb_println_fn = self.module.add_function("bmb_sb_println", sb_println_type, None);
        self.functions.insert("sb_println".to_string(), sb_println_fn);

        // puts_cstr(ptr: i64) -> i64 (v0.60.65: print C string from raw pointer)
        let puts_cstr_type = i64_type.fn_type(&[ptr_type.into()], false);
        let puts_cstr_fn = self.module.add_function("puts_cstr", puts_cstr_type, None);
        self.functions.insert("puts_cstr".to_string(), puts_cstr_fn);

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

        // calloc(count: i64, size: i64) -> ptr (libc function)
        // v0.60.3: Use libc calloc directly like malloc, convert ptr->i64 at call site
        let calloc_type = ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let calloc_fn = self.module.add_function("calloc", calloc_type, None);
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
        self.function_return_types.insert("read_file".to_string(), MirType::String);

        // write_file(path: ptr, content: ptr) -> i64 (returns 0 on success, -1 on error)
        let write_file_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let write_file_fn = self.module.add_function("bmb_write_file", write_file_type, None);
        self.functions.insert("write_file".to_string(), write_file_fn);

        // v0.60.80: write_file_newlines - converts | to newlines (for bootstrap compiler)
        let write_file_newlines_fn = self.module.add_function("write_file_newlines", write_file_type, None);
        self.functions.insert("write_file_newlines".to_string(), write_file_newlines_fn);

        // file_exists(path: ptr) -> i64 (returns 1 if exists, 0 otherwise)
        let file_exists_type = i64_type.fn_type(&[ptr_type.into()], false);
        let file_exists_fn = self.module.add_function("file_exists", file_exists_type, None);
        self.functions.insert("file_exists".to_string(), file_exists_fn);

        // v0.60.262: file_size(path: ptr) -> i64 (returns file size or -1 on error)
        let file_size_type = i64_type.fn_type(&[ptr_type.into()], false);
        let file_size_fn = self.module.add_function("file_size", file_size_type, None);
        self.functions.insert("file_size".to_string(), file_size_fn);

        // v0.51.18: _cstr variants for string literal optimization (zero overhead)
        let file_exists_cstr_fn = self.module.add_function("file_exists_cstr", file_exists_type, None);
        self.functions.insert("file_exists_cstr".to_string(), file_exists_cstr_fn);
        let bmb_file_exists_cstr_fn = self.module.add_function("bmb_file_exists_cstr", file_exists_type, None);
        self.functions.insert("bmb_file_exists_cstr".to_string(), bmb_file_exists_cstr_fn);

        // v0.46: Command-line argument functions for CLI Independence
        // arg_count() -> i64
        let arg_count_type = i64_type.fn_type(&[], false);
        let arg_count_fn = self.module.add_function("bmb_arg_count", arg_count_type, None);
        self.functions.insert("arg_count".to_string(), arg_count_fn);

        // get_arg(index: i64) -> ptr (returns string)
        let get_arg_type = ptr_type.fn_type(&[i64_type.into()], false);
        let get_arg_fn = self.module.add_function("bmb_get_arg", get_arg_type, None);
        self.functions.insert("get_arg".to_string(), get_arg_fn);
        self.function_return_types.insert("get_arg".to_string(), MirType::String);

        // v0.70: Threading primitives
        // bmb_spawn(func_ptr: ptr, captures: ptr) -> i64 (thread handle)
        let spawn_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        let spawn_fn = self.module.add_function("bmb_spawn", spawn_type, None);
        self.functions.insert("bmb_spawn".to_string(), spawn_fn);

        // bmb_join(handle: i64) -> i64 (result)
        let join_type = i64_type.fn_type(&[i64_type.into()], false);
        let join_fn = self.module.add_function("bmb_join", join_type, None);
        self.functions.insert("bmb_join".to_string(), join_fn);

        // v0.71: Mutex primitives
        // bmb_mutex_new(initial_value: i64) -> i64 (mutex handle)
        let mutex_new_type = i64_type.fn_type(&[i64_type.into()], false);
        let mutex_new_fn = self.module.add_function("bmb_mutex_new", mutex_new_type, None);
        self.functions.insert("bmb_mutex_new".to_string(), mutex_new_fn);

        // bmb_mutex_lock(handle: i64) -> i64 (current value)
        let mutex_lock_type = i64_type.fn_type(&[i64_type.into()], false);
        let mutex_lock_fn = self.module.add_function("bmb_mutex_lock", mutex_lock_type, None);
        self.functions.insert("bmb_mutex_lock".to_string(), mutex_lock_fn);

        // bmb_mutex_unlock(handle: i64, new_value: i64) -> void
        let mutex_unlock_type = void_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let mutex_unlock_fn = self.module.add_function("bmb_mutex_unlock", mutex_unlock_type, None);
        self.functions.insert("bmb_mutex_unlock".to_string(), mutex_unlock_fn);

        // bmb_mutex_try_lock(handle: i64) -> i64 (value if locked, 0 if failed)
        let mutex_try_lock_type = i64_type.fn_type(&[i64_type.into()], false);
        let mutex_try_lock_fn = self.module.add_function("bmb_mutex_try_lock", mutex_try_lock_type, None);
        self.functions.insert("bmb_mutex_try_lock".to_string(), mutex_try_lock_fn);

        // bmb_mutex_free(handle: i64) -> void
        let mutex_free_type = void_type.fn_type(&[i64_type.into()], false);
        let mutex_free_fn = self.module.add_function("bmb_mutex_free", mutex_free_type, None);
        self.functions.insert("bmb_mutex_free".to_string(), mutex_free_fn);

        // v0.71: Channel primitives
        // bmb_channel_new(capacity: i64, sender_out: ptr, receiver_out: ptr) -> void
        let channel_new_type = void_type.fn_type(&[i64_type.into(), ptr_type.into(), ptr_type.into()], false);
        let channel_new_fn = self.module.add_function("bmb_channel_new", channel_new_type, None);
        self.functions.insert("bmb_channel_new".to_string(), channel_new_fn);

        // bmb_channel_send(sender: i64, value: i64) -> void
        let channel_send_type = void_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let channel_send_fn = self.module.add_function("bmb_channel_send", channel_send_type, None);
        self.functions.insert("bmb_channel_send".to_string(), channel_send_fn);

        // bmb_channel_recv(receiver: i64) -> i64 (received value)
        let channel_recv_type = i64_type.fn_type(&[i64_type.into()], false);
        let channel_recv_fn = self.module.add_function("bmb_channel_recv", channel_recv_type, None);
        self.functions.insert("bmb_channel_recv".to_string(), channel_recv_fn);

        // bmb_channel_try_send(sender: i64, value: i64) -> i64 (1 if sent, 0 if full)
        let channel_try_send_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let channel_try_send_fn = self.module.add_function("bmb_channel_try_send", channel_try_send_type, None);
        self.functions.insert("bmb_channel_try_send".to_string(), channel_try_send_fn);

        // bmb_channel_try_recv(receiver: i64, value_out: ptr) -> i64 (1 if received, 0 if empty)
        let channel_try_recv_type = i64_type.fn_type(&[i64_type.into(), ptr_type.into()], false);
        let channel_try_recv_fn = self.module.add_function("bmb_channel_try_recv", channel_try_recv_type, None);
        self.functions.insert("bmb_channel_try_recv".to_string(), channel_try_recv_fn);

        // v0.77: bmb_channel_recv_timeout(receiver: i64, timeout_ms: i64, value_out: ptr) -> i64 (1 if received, 0 if timeout)
        let channel_recv_timeout_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), ptr_type.into()], false);
        let channel_recv_timeout_fn = self.module.add_function("bmb_channel_recv_timeout", channel_recv_timeout_type, None);
        self.functions.insert("bmb_channel_recv_timeout".to_string(), channel_recv_timeout_fn);

        // v0.78: bmb_block_on(future_value: i64) -> i64 (runs executor, returns result)
        let block_on_type = i64_type.fn_type(&[i64_type.into()], false);
        let block_on_fn = self.module.add_function("bmb_block_on", block_on_type, None);
        self.functions.insert("bmb_block_on".to_string(), block_on_fn);

        // bmb_sender_clone(sender: i64) -> i64 (cloned sender)
        let sender_clone_type = i64_type.fn_type(&[i64_type.into()], false);
        let sender_clone_fn = self.module.add_function("bmb_sender_clone", sender_clone_type, None);
        self.functions.insert("bmb_sender_clone".to_string(), sender_clone_fn);

        // bmb_channel_free(channel: i64) -> void
        let channel_free_type = void_type.fn_type(&[i64_type.into()], false);
        let channel_free_fn = self.module.add_function("bmb_channel_free", channel_free_type, None);
        self.functions.insert("bmb_channel_free".to_string(), channel_free_fn);

        // v0.74: RwLock primitives
        // bmb_rwlock_new(initial_value: i64) -> i64 (rwlock handle)
        let rwlock_new_type = i64_type.fn_type(&[i64_type.into()], false);
        let rwlock_new_fn = self.module.add_function("bmb_rwlock_new", rwlock_new_type, None);
        self.functions.insert("bmb_rwlock_new".to_string(), rwlock_new_fn);

        // bmb_rwlock_read(handle: i64) -> i64 (current value)
        let rwlock_read_type = i64_type.fn_type(&[i64_type.into()], false);
        let rwlock_read_fn = self.module.add_function("bmb_rwlock_read", rwlock_read_type, None);
        self.functions.insert("bmb_rwlock_read".to_string(), rwlock_read_fn);

        // bmb_rwlock_read_unlock(handle: i64) -> void
        let rwlock_read_unlock_type = void_type.fn_type(&[i64_type.into()], false);
        let rwlock_read_unlock_fn = self.module.add_function("bmb_rwlock_read_unlock", rwlock_read_unlock_type, None);
        self.functions.insert("bmb_rwlock_read_unlock".to_string(), rwlock_read_unlock_fn);

        // bmb_rwlock_write(handle: i64) -> i64 (current value)
        let rwlock_write_type = i64_type.fn_type(&[i64_type.into()], false);
        let rwlock_write_fn = self.module.add_function("bmb_rwlock_write", rwlock_write_type, None);
        self.functions.insert("bmb_rwlock_write".to_string(), rwlock_write_fn);

        // bmb_rwlock_write_unlock(handle: i64, new_value: i64) -> void
        let rwlock_write_unlock_type = void_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let rwlock_write_unlock_fn = self.module.add_function("bmb_rwlock_write_unlock", rwlock_write_unlock_type, None);
        self.functions.insert("bmb_rwlock_write_unlock".to_string(), rwlock_write_unlock_fn);

        // bmb_rwlock_free(handle: i64) -> void
        let rwlock_free_type = void_type.fn_type(&[i64_type.into()], false);
        let rwlock_free_fn = self.module.add_function("bmb_rwlock_free", rwlock_free_type, None);
        self.functions.insert("bmb_rwlock_free".to_string(), rwlock_free_fn);

        // v0.74: Barrier primitives
        // bmb_barrier_new(count: i64) -> i64 (barrier handle)
        let barrier_new_type = i64_type.fn_type(&[i64_type.into()], false);
        let barrier_new_fn = self.module.add_function("bmb_barrier_new", barrier_new_type, None);
        self.functions.insert("bmb_barrier_new".to_string(), barrier_new_fn);

        // bmb_barrier_wait(handle: i64) -> i64 (1 if leader, 0 otherwise)
        let barrier_wait_type = i64_type.fn_type(&[i64_type.into()], false);
        let barrier_wait_fn = self.module.add_function("bmb_barrier_wait", barrier_wait_type, None);
        self.functions.insert("bmb_barrier_wait".to_string(), barrier_wait_fn);

        // bmb_barrier_free(handle: i64) -> void
        let barrier_free_type = void_type.fn_type(&[i64_type.into()], false);
        let barrier_free_fn = self.module.add_function("bmb_barrier_free", barrier_free_type, None);
        self.functions.insert("bmb_barrier_free".to_string(), barrier_free_fn);

        // v0.74: Condvar primitives
        // bmb_condvar_new() -> i64 (condvar handle)
        let condvar_new_type = i64_type.fn_type(&[], false);
        let condvar_new_fn = self.module.add_function("bmb_condvar_new", condvar_new_type, None);
        self.functions.insert("bmb_condvar_new".to_string(), condvar_new_fn);

        // bmb_condvar_wait(condvar: i64, mutex: i64) -> i64 (current mutex value after wakeup)
        let condvar_wait_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let condvar_wait_fn = self.module.add_function("bmb_condvar_wait", condvar_wait_type, None);
        self.functions.insert("bmb_condvar_wait".to_string(), condvar_wait_fn);

        // bmb_condvar_notify_one(condvar: i64) -> void
        let condvar_notify_one_type = void_type.fn_type(&[i64_type.into()], false);
        let condvar_notify_one_fn = self.module.add_function("bmb_condvar_notify_one", condvar_notify_one_type, None);
        self.functions.insert("bmb_condvar_notify_one".to_string(), condvar_notify_one_fn);

        // bmb_condvar_notify_all(condvar: i64) -> void
        let condvar_notify_all_type = void_type.fn_type(&[i64_type.into()], false);
        let condvar_notify_all_fn = self.module.add_function("bmb_condvar_notify_all", condvar_notify_all_type, None);
        self.functions.insert("bmb_condvar_notify_all".to_string(), condvar_notify_all_fn);

        // bmb_condvar_free(condvar: i64) -> void
        let condvar_free_type = void_type.fn_type(&[i64_type.into()], false);
        let condvar_free_fn = self.module.add_function("bmb_condvar_free", condvar_free_type, None);
        self.functions.insert("bmb_condvar_free".to_string(), condvar_free_fn);

        // v0.75: Async/Await Support (Futures)
        // __future_await(future_handle: i64) -> i64 (blocks until future completes)
        let future_await_type = i64_type.fn_type(&[i64_type.into()], false);
        let future_await_fn = self.module.add_function("__future_await", future_await_type, None);
        self.functions.insert("__future_await".to_string(), future_await_fn);
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
            // Struct/Enum/Array/Ptr types - use pointer as placeholder for now
            MirType::Struct { .. }
            | MirType::StructPtr(_)
            | MirType::Enum { .. }
            | MirType::Array { .. }
            | MirType::Ptr(_) => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            // v0.55: Tuple type - LLVM struct with heterogeneous element types
            MirType::Tuple(elems) => {
                let elem_types: Vec<BasicTypeEnum<'ctx>> = elems
                    .iter()
                    .map(|e| self.mir_type_to_llvm(e))
                    .collect();
                self.context.struct_type(&elem_types, false).into()
            }
        }
    }

    /// v0.60.7: Build LLVM struct type from struct definition
    /// Used for GEP-based field access codegen
    fn get_struct_llvm_type(&self, struct_name: &str) -> CodeGenResult<inkwell::types::StructType<'ctx>> {
        let fields = self.struct_defs.get(struct_name)
            .ok_or_else(|| CodeGenError::LlvmError(format!("Unknown struct: {}", struct_name)))?;

        let field_types: Vec<BasicTypeEnum<'ctx>> = fields
            .iter()
            .map(|(_, ty)| self.mir_type_to_llvm(ty))
            .collect();

        Ok(self.context.struct_type(&field_types, false))
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
        // v0.60.252: Use private linkage for @inline functions to avoid symbol collision
        use inkwell::module::Linkage;
        let linkage = if func.always_inline && func.name != "main" && func.name != "bmb_user_main" {
            Some(Linkage::Private)
        } else {
            None
        };
        let function = self.module.add_function(emitted_name, fn_type, linkage);

        // v0.50.76: Add function attributes for better LLVM optimization
        // - nounwind: BMB has no exceptions, so no unwinding
        // - willreturn: Most functions will eventually return
        // These help LLVM's optimization passes, especially for recursive functions
        use inkwell::attributes::{Attribute, AttributeLoc};
        // v0.60.35: Use proper enum attributes instead of string attributes
        // String attributes like "alwaysinline" are just metadata and don't trigger LLVM optimizations
        // We must use enum attributes (LLVM built-in attributes) for actual effect
        let nounwind_id = Attribute::get_named_enum_kind_id("nounwind");
        let willreturn_id = Attribute::get_named_enum_kind_id("willreturn");
        let mustprogress_id = Attribute::get_named_enum_kind_id("mustprogress");
        function.add_attribute(AttributeLoc::Function, self.context.create_enum_attribute(nounwind_id, 0));
        function.add_attribute(AttributeLoc::Function, self.context.create_enum_attribute(willreturn_id, 0));
        function.add_attribute(AttributeLoc::Function, self.context.create_enum_attribute(mustprogress_id, 0));

        // v0.51.8: Add alwaysinline for small functions to eliminate call overhead
        // This is critical for tight loops like spectral_norm's inner loop
        // v0.60.35: Fixed - use enum attribute, not string attribute
        if func.always_inline {
            let alwaysinline_id = Attribute::get_named_enum_kind_id("alwaysinline");
            function.add_attribute(AttributeLoc::Function, self.context.create_enum_attribute(alwaysinline_id, 0));
        }

        // v0.69: Add memory(none) for memory-free functions to enable LICM and constant folding
        // Functions marked @pure or detected as memory-free can be hoisted out of loops
        // and constant-folded by LLVM when called with constant arguments
        if func.is_memory_free {
            let memory_none_attr = self.context.create_string_attribute("memory", "none");
            function.add_attribute(AttributeLoc::Function, memory_none_attr);
        }

        // v0.60.56: Add fast-math attributes for FP-heavy workloads
        // These enable aggressive FP optimizations (FMA, reciprocal, reassociation)
        // WARNING: Not IEEE-754 compliant - results may differ slightly
        if self.fast_math {
            let unsafe_fp_attr = self.context.create_string_attribute("unsafe-fp-math", "true");
            let no_nans_attr = self.context.create_string_attribute("no-nans-fp-math", "true");
            let no_infs_attr = self.context.create_string_attribute("no-infs-fp-math", "true");
            let no_signed_zeros_attr = self.context.create_string_attribute("no-signed-zeros-fp-math", "true");
            let approx_func_attr = self.context.create_string_attribute("approx-func-fp-math", "true");
            function.add_attribute(AttributeLoc::Function, unsafe_fp_attr);
            function.add_attribute(AttributeLoc::Function, no_nans_attr);
            function.add_attribute(AttributeLoc::Function, no_infs_attr);
            function.add_attribute(AttributeLoc::Function, no_signed_zeros_attr);
            function.add_attribute(AttributeLoc::Function, approx_func_attr);
        }

        self.functions.insert(func.name.clone(), function);
        // v0.60.15: Store return type for PHI type inference
        self.function_return_types.insert(func.name.clone(), func.ret_ty.clone());
        Ok(())
    }

    /// v0.51.0: Generate function body using true LLVM PHI instructions
    /// Uses a three-pass approach:
    /// 1. Create PHI nodes first at the start of their blocks (LLVM requirement)
    /// 2. Generate regular instructions (skip PHI instructions)
    /// 3. Populate PHI incoming values (after all values exist)
    fn gen_function_body(&mut self, func: &MirFunction) -> CodeGenResult<()> {
        // Clear per-function state
        self.variables.clear();
        self.ssa_values.clear();
        self.phi_nodes.clear();
        self.blocks.clear();
        self.string_variables.clear();

        // v0.60.81: Track String-typed variables for proper string comparison
        // This enables bmb_string_eq to be called even when both operands are variables
        for (name, ty) in &func.params {
            if *ty == MirType::String {
                self.string_variables.insert(name.clone());
            }
        }
        for (name, ty) in &func.locals {
            if *ty == MirType::String {
                self.string_variables.insert(name.clone());
            }
        }
        // v0.60.119: Also track temporaries that receive String values from function calls
        // This fixes string comparison when comparing sliced strings (e.g., s.slice(0,3) == "fn ")
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Call { dest: Some(dest), func: callee, .. } = inst {
                    if let Some(ret_ty) = self.function_return_types.get(callee) {
                        if *ret_ty == MirType::String {
                            self.string_variables.insert(dest.name.clone());
                        }
                    }
                }
            }
        }

        // v0.50.80: Set current function's return type for type coercion in Return
        self.current_ret_type = match &func.ret_ty {
            MirType::Unit => None,
            ty => Some(self.mir_type_to_llvm(ty)),
        };

        // Get the already-declared function
        let function = *self.functions.get(&func.name)
            .ok_or_else(|| CodeGenError::UnknownFunction(func.name.clone()))?;

        // Create all basic blocks first
        for block in &func.blocks {
            let bb = self.context.append_basic_block(function, &block.label);
            self.blocks.insert(block.label.clone(), bb);
        }

        // Position at entry block for allocas
        if let Some(entry) = self.blocks.get("entry") {
            self.builder.position_at_end(*entry);
        } else if let Some(first_block) = func.blocks.first() {
            let bb = self.blocks.get(&first_block.label).unwrap();
            self.builder.position_at_end(*bb);
        } else {
            return Ok(());
        }

        // v0.51.7: Collect written places to optimize read-only parameters
        // Read-only parameters use SSA values (faster), writable ones use alloca (required)
        let written_places = self.collect_written_places(func);

        // v0.51.0: Collect PHI destinations - these are now true SSA values, not memory
        let phi_dests = self.collect_phi_destinations(func);

        // v0.51.9: Collect single-assignment locals that can use SSA instead of alloca
        let ssa_locals = self.collect_ssa_eligible_locals(func, &phi_dests);

        // Allocate parameters - v0.51.7: skip alloca for read-only params
        for (i, (name, ty)) in func.params.iter().enumerate() {
            let llvm_ty = self.mir_type_to_llvm(ty);
            let param = function.get_nth_param(i as u32).unwrap();

            // v0.60.253: Track enum-typed parameters for switch handling
            if matches!(ty, MirType::Enum { .. }) {
                self.enum_variables.insert(name.clone());
            }

            if written_places.contains(name) {
                // Parameter is modified - need alloca for memory location
                let alloca = self.builder.build_alloca(llvm_ty, name)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                self.builder.build_store(alloca, param)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                self.variables.insert(name.clone(), (alloca, llvm_ty));
            } else {
                // Read-only parameter - use SSA value directly (no alloca/load overhead)
                self.ssa_values.insert(name.clone(), param);
            }
        }

        // v0.60.54: Pre-extend i32 read-only parameters to i64 at function entry
        // This enables LLVM to use the pre-extended value instead of sign-extending
        // inside loops. The original i32 value is kept for i32-only operations.
        for (name, ty) in func.params.iter() {
            if *ty == MirType::I32 && !written_places.contains(name) {
                if let Some(param_val) = self.ssa_values.get(name).cloned() {
                    let i32_val = param_val.into_int_value();
                    let i64_val = self.builder
                        .build_int_s_extend(i32_val, self.context.i64_type(), &format!("{}_sext", name))
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    // Store under a special name that coercion can look up
                    self.ssa_values.insert(format!("{}_i64", name), i64_val.into());
                }
            }
        }

        // Allocate locals (excluding PHI destinations, _t* temporaries, and SSA-eligible locals)
        for (name, ty) in &func.locals {
            // v0.60.253: Track enum-typed locals for switch handling
            if matches!(ty, MirType::Enum { .. }) {
                self.enum_variables.insert(name.clone());
            }

            // Skip _t* temporaries - they stay as SSA values
            if name.starts_with("_t") {
                continue;
            }
            // Skip PHI destinations - they're true SSA values from PHI nodes
            if phi_dests.contains(name) {
                continue;
            }
            // v0.51.9: Skip single-assignment locals - they'll use SSA values
            if ssa_locals.contains(name) {
                continue;
            }
            let llvm_ty = self.mir_type_to_llvm(ty);
            let alloca = self.builder.build_alloca(llvm_ty, name)
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            self.variables.insert(name.clone(), (alloca, llvm_ty));
        }

        // ========== PASS 1: Create PHI nodes at the start of their blocks ==========
        // LLVM requires PHI instructions to be first in a basic block
        for block in &func.blocks {
            let bb = *self.blocks.get(&block.label).unwrap();
            self.builder.position_at_end(bb);

            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst {
                    // Infer type from first value
                    let phi_type = self.infer_phi_type(values, func)?;
                    let phi = self.builder.build_phi(phi_type, &dest.name)
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Store PHI node for later incoming value population
                    self.phi_nodes.insert(dest.name.clone(), phi);

                    // Store PHI result in ssa_values for use by other instructions
                    self.ssa_values.insert(dest.name.clone(), phi.as_basic_value());
                }
            }
        }

        // ========== PASS 2: Generate regular instructions (skip PHI) ==========
        for block in &func.blocks {
            self.gen_basic_block(block)?;
        }

        // ========== PASS 3: Populate PHI incoming values ==========
        // Now all values exist, we can add incoming edges to PHI nodes
        // IMPORTANT: For memory variables, loads must happen in predecessor blocks
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst {
                    // Get the PHI node first to know the expected type
                    let phi = *self.phi_nodes.get(&dest.name)
                        .ok_or_else(|| CodeGenError::UnknownVariable(dest.name.clone()))?;
                    let phi_type = phi.as_basic_value().get_type();

                    let mut incoming: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();

                    for (value, source_label) in values {
                        let source_bb = *self.blocks.get(source_label)
                            .ok_or_else(|| CodeGenError::UnknownBlock(source_label.clone()))?;

                        // Generate the value - handle different operand types
                        let llvm_val = match value {
                            // Constants don't generate instructions, safe to create anywhere
                            Operand::Constant(c) => self.gen_constant(c),

                            // Places need special handling
                            Operand::Place(p) => {
                                // Check if it's an SSA value (no load needed)
                                if let Some(val) = self.ssa_values.get(&p.name) {
                                    *val
                                } else if let Some((ptr, pointee_type)) = self.variables.get(&p.name) {
                                    // Memory variable - need to generate load in predecessor block
                                    // Position builder at the end of source block, before terminator
                                    let terminator = source_bb.get_terminator()
                                        .ok_or_else(|| CodeGenError::LlvmError(
                                            format!("Block {} has no terminator", source_label)))?;
                                    self.builder.position_before(&terminator);

                                    let load_name = format!("{}.phi.{}", p.name, source_label);
                                    let loaded = self.builder
                                        .build_load(*pointee_type, *ptr, &load_name)
                                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                                    loaded
                                } else {
                                    return Err(CodeGenError::UnknownVariable(p.name.clone()));
                                }
                            }
                        };

                        // v0.51.0: Type coercion for PHI incoming values
                        // The narrowing pass may create type mismatches (i32 vs i64)
                        // Add sign extension or truncation to match PHI type
                        let coerced_val = self.coerce_phi_value(llvm_val, phi_type, source_bb, source_label)?;

                        incoming.push((coerced_val, source_bb));
                    }

                    // Add all incoming values to the PHI node
                    let incoming_refs: Vec<(&dyn inkwell::values::BasicValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> =
                        incoming.iter().map(|(v, b)| (v as &dyn inkwell::values::BasicValue<'ctx>, *b)).collect();
                    phi.add_incoming(&incoming_refs);
                }
            }
        }

        Ok(())
    }

    /// v0.51.0: Collect PHI destination variable names
    /// These are true SSA values and should NOT have allocas
    fn collect_phi_destinations(&self, func: &MirFunction) -> HashSet<String> {
        let mut phi_dests = HashSet::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest, .. } = inst {
                    phi_dests.insert(dest.name.clone());
                }
            }
        }
        phi_dests
    }

    /// v0.51.7: Collect all places that are written to (instruction destinations)
    /// Used to determine which parameters need alloca (written) vs SSA (read-only)
    fn collect_written_places(&self, func: &MirFunction) -> HashSet<String> {
        let mut written = HashSet::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                // Get destination place from each instruction type
                let dest = match inst {
                    MirInst::Const { dest, .. } => Some(&dest.name),
                    MirInst::Copy { dest, .. } => Some(&dest.name),
                    MirInst::BinOp { dest, .. } => Some(&dest.name),
                    MirInst::UnaryOp { dest, .. } => Some(&dest.name),
                    MirInst::Call { dest: Some(d), .. } => Some(&d.name),
                    MirInst::Call { dest: None, .. } => None,
                    MirInst::Phi { dest, .. } => Some(&dest.name),
                    MirInst::StructInit { dest, .. } => Some(&dest.name),
                    MirInst::FieldAccess { dest, .. } => Some(&dest.name),
                    // v0.60.29: FieldStore reads the base pointer to write through it
                    // The pointer itself is not modified, only memory accessed through it
                    MirInst::FieldStore { .. } => None,
                    MirInst::EnumVariant { dest, .. } => Some(&dest.name),
                    MirInst::ArrayInit { dest, .. } => Some(&dest.name),
                    MirInst::IndexLoad { dest, .. } => Some(&dest.name),
                    // v0.60.29: IndexStore reads the array pointer to write through it
                    // The pointer itself is not modified, only memory accessed through it
                    MirInst::IndexStore { .. } => None,
                    MirInst::Cast { dest, .. } => Some(&dest.name),
                    // v0.55: Tuple instructions
                    MirInst::TupleInit { dest, .. } => Some(&dest.name),
                    MirInst::TupleExtract { dest, .. } => Some(&dest.name),
                    // v0.60.19: Pointer offset
                    MirInst::PtrOffset { dest, .. } => Some(&dest.name),
                    // v0.60.21: Array allocation
                    MirInst::ArrayAlloc { dest, .. } => Some(&dest.name),
                    // v0.60.20: Pointer load/store
                    MirInst::PtrLoad { dest, .. } => Some(&dest.name),
                    MirInst::PtrStore { .. } => None, // Modifies memory through pointer, not a named place
                    // v0.73+: Concurrency and other new instructions - handled by text-based codegen
                    _ => None,
                };
                if let Some(name) = dest {
                    written.insert(name.clone());
                }
            }
        }
        written
    }

    /// v0.51.9: Collect locals that can use SSA values instead of alloca
    /// A local can use SSA if:
    /// 1. It's written exactly once (single assignment)
    /// 2. It's not a PHI destination (those are handled separately)
    /// 3. It's not a _t* temporary (those already use SSA)
    ///
    /// Single-assignment locals written in branch blocks are safe because:
    /// - LLVM SSA values are block-scoped until used
    /// - If a local is used across blocks, MIR lowering creates a PHI node
    /// - Without a PHI, all uses must be dominated by the single write
    fn collect_ssa_eligible_locals(&self, func: &MirFunction, phi_dests: &HashSet<String>) -> HashSet<String> {
        let mut write_counts: HashMap<String, usize> = HashMap::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                let dest = match inst {
                    MirInst::Const { dest, .. } => Some(&dest.name),
                    MirInst::Copy { dest, .. } => Some(&dest.name),
                    MirInst::BinOp { dest, .. } => Some(&dest.name),
                    MirInst::UnaryOp { dest, .. } => Some(&dest.name),
                    MirInst::Call { dest: Some(d), .. } => Some(&d.name),
                    MirInst::Call { dest: None, .. } => None,
                    MirInst::Phi { .. } => None, // PHI handled separately
                    MirInst::StructInit { dest, .. } => Some(&dest.name),
                    MirInst::FieldAccess { dest, .. } => Some(&dest.name),
                    MirInst::FieldStore { .. } => None, // Field stores modify existing struct
                    MirInst::EnumVariant { dest, .. } => Some(&dest.name),
                    MirInst::ArrayInit { dest, .. } => Some(&dest.name),
                    MirInst::IndexLoad { dest, .. } => Some(&dest.name),
                    MirInst::IndexStore { .. } => None, // Index stores modify existing array
                    MirInst::Cast { dest, .. } => Some(&dest.name),
                    // v0.55: Tuple instructions
                    MirInst::TupleInit { dest, .. } => Some(&dest.name),
                    MirInst::TupleExtract { dest, .. } => Some(&dest.name),
                    // v0.60.19: Pointer offset
                    MirInst::PtrOffset { dest, .. } => Some(&dest.name),
                    // v0.60.21: Array allocation
                    MirInst::ArrayAlloc { dest, .. } => Some(&dest.name),
                    // v0.60.20: Pointer load/store
                    MirInst::PtrLoad { dest, .. } => Some(&dest.name),
                    MirInst::PtrStore { .. } => None, // Modifies memory through pointer
                    // v0.73+: Concurrency and other new instructions - handled by text-based codegen
                    _ => None,
                };

                if let Some(name) = dest {
                    *write_counts.entry(name.clone()).or_insert(0) += 1;
                }
            }
        }

        // A local is SSA-eligible if:
        // - Written exactly once (single assignment) OR
        // - Written zero times (dead local after optimization)
        // - Not a PHI destination (those are already SSA)
        // - Not a _t* temporary (those already use SSA)
        let mut ssa_eligible = HashSet::new();
        for (name, count) in &write_counts {
            if *count == 1
                && !phi_dests.contains(name)
                && !name.starts_with("_t")
            {
                ssa_eligible.insert(name.clone());
            }
        }
        // v0.51.9: Also mark dead locals (zero writes after optimization) as SSA-eligible
        // CopyPropagation + DeadCodeElimination removes Copy instructions, leaving dead locals
        for (name, _ty) in &func.locals {
            if !name.starts_with("_t") && !phi_dests.contains(name) && !write_counts.contains_key(name) {
                ssa_eligible.insert(name.clone());
            }
        }

        ssa_eligible
    }

    /// v0.51.0: Infer LLVM type for a PHI node from its incoming values
    /// Note: Uses the LARGEST type among all incoming values for robustness
    fn infer_phi_type(&self, values: &[(Operand, String)], func: &MirFunction) -> CodeGenResult<BasicTypeEnum<'ctx>> {
        let mut max_bit_width = 0u32;
        let mut result_type: Option<BasicTypeEnum<'ctx>> = None;

        for (value, _) in values {
            let ty = match value {
                Operand::Constant(c) => self.constant_type(c),
                Operand::Place(p) => {
                    // Check if it's an SSA value
                    if let Some(val) = self.ssa_values.get(&p.name) {
                        val.get_type()
                    }
                    // Check if it's a memory variable
                    else if let Some((_, ty)) = self.variables.get(&p.name) {
                        *ty
                    }
                    // Check func.locals for type
                    else if let Some((_, ty)) = func.locals.iter().find(|(n, _)| n == &p.name) {
                        self.mir_type_to_llvm(ty)
                    }
                    // Check func.params for type
                    else if let Some((_, ty)) = func.params.iter().find(|(n, _)| n == &p.name) {
                        self.mir_type_to_llvm(ty)
                    }
                    // v0.60.15: Check the defining instruction for this place
                    else if let Some(mir_ty) = self.find_place_type_from_instructions(&p.name, func) {
                        self.mir_type_to_llvm(&mir_ty)
                    }
                    // Default to i64 if type unknown
                    else {
                        self.context.i64_type().into()
                    }
                }
            };

            // Track the largest integer type
            if let BasicTypeEnum::IntType(int_ty) = ty {
                let bit_width = int_ty.get_bit_width();
                if bit_width > max_bit_width {
                    max_bit_width = bit_width;
                    result_type = Some(ty);
                }
            } else {
                // For non-integer types, use the first one found
                if result_type.is_none() {
                    result_type = Some(ty);
                }
            }
        }

        Ok(result_type.unwrap_or_else(|| self.context.i64_type().into()))
    }

    /// v0.60.15: Find the MIR type of a place by looking at its defining instruction
    /// This is used in pass 1 when SSA values haven't been generated yet
    fn find_place_type_from_instructions(&self, place_name: &str, func: &MirFunction) -> Option<MirType> {
        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    // Copy instruction: trace through to the source
                    MirInst::Copy { dest, src } if dest.name == place_name => {
                        // Recursively find the type of the source
                        return self.find_place_type_from_instructions(&src.name, func);
                    }
                    // Call instruction: use return type
                    MirInst::Call { dest: Some(dest), func: callee, .. } if dest.name == place_name => {
                        // Find the callee function's return type
                        if let Some(ret_ty) = self.function_return_types.get(callee) {
                            return Some(ret_ty.clone());
                        }
                    }
                    // Cast instruction: use target type
                    MirInst::Cast { dest, to_ty, .. } if dest.name == place_name => {
                        return Some(to_ty.clone());
                    }
                    // FieldAccess: use field type from struct
                    MirInst::FieldAccess { dest, struct_name, field_index, .. } if dest.name == place_name => {
                        if let Some(fields) = self.struct_defs.get(struct_name) {
                            if let Some((_, field_ty)) = fields.get(*field_index) {
                                return Some(field_ty.clone());
                            }
                        }
                    }
                    // BinOp: typically returns i64
                    MirInst::BinOp { dest, .. } if dest.name == place_name => {
                        return Some(MirType::I64);
                    }
                    // UnaryOp: typically returns same type as operand
                    MirInst::UnaryOp { dest, .. } if dest.name == place_name => {
                        return Some(MirType::I64);
                    }
                    // Const: use constant type
                    MirInst::Const { dest, value } if dest.name == place_name => {
                        return Some(match value {
                            Constant::Int(_) => MirType::I64,
                            Constant::Float(_) => MirType::F64,
                            Constant::Bool(_) => MirType::Bool,
                            Constant::Char(_) => MirType::I32,
                            Constant::String(_) => MirType::I64, // String is pointer as i64
                            Constant::Unit => MirType::Unit,
                        });
                    }
                    _ => {}
                }
            }
        }
        None
    }

    /// v0.51.0: Coerce a PHI incoming value to match the expected PHI type
    /// Handles type mismatches from the ConstantPropagationNarrowing pass
    /// v0.60.2: Extended to handle struct (tuple) types that are structurally equal
    fn coerce_phi_value(
        &self,
        value: BasicValueEnum<'ctx>,
        phi_type: BasicTypeEnum<'ctx>,
        source_bb: inkwell::basic_block::BasicBlock<'ctx>,
        source_label: &str,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        let value_type = value.get_type();

        // If types match, no coercion needed
        if value_type == phi_type {
            return Ok(value);
        }

        // v0.60.2: Handle struct type comparison by structure
        // LLVM anonymous struct types may have different identity but same structure
        // This is common with tuples where (i64, i64) is created in multiple places
        if let (BasicTypeEnum::StructType(from_struct), BasicTypeEnum::StructType(to_struct)) =
            (value_type, phi_type)
        {
            // Check if structs are structurally equivalent
            let from_fields = from_struct.get_field_types();
            let to_fields = to_struct.get_field_types();

            if from_fields == to_fields {
                // Structs are structurally identical - no coercion needed
                // The LLVM types may differ by identity but values are compatible
                return Ok(value);
            }
        }

        // Handle integer type coercion
        if let (BasicTypeEnum::IntType(from_int), BasicTypeEnum::IntType(to_int)) =
            (value_type, phi_type)
        {
            let from_bits = from_int.get_bit_width();
            let to_bits = to_int.get_bit_width();

            // Position builder before the terminator of source block
            // (this is where the coercion instruction should be)
            let terminator = source_bb.get_terminator()
                .ok_or_else(|| CodeGenError::LlvmError(
                    format!("Block {} has no terminator for phi coercion", source_label)))?;
            self.builder.position_before(&terminator);

            let int_val = value.into_int_value();
            let coerced = if from_bits < to_bits {
                // Sign extend to larger type
                self.builder
                    .build_int_s_extend(int_val, to_int, "phi_sext")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
            } else {
                // Truncate to smaller type
                self.builder
                    .build_int_truncate(int_val, to_int, "phi_trunc")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
            };

            return Ok(coerced.into());
        }

        // v0.60.15: Handle integer to pointer coercion
        // This occurs when a PHI expects a pointer but receives an integer (e.g., from ptrtoint/cast)
        if let (BasicTypeEnum::IntType(_), BasicTypeEnum::PointerType(to_ptr)) =
            (value_type, phi_type)
        {
            let terminator = source_bb.get_terminator()
                .ok_or_else(|| CodeGenError::LlvmError(
                    format!("Block {} has no terminator for phi coercion", source_label)))?;
            self.builder.position_before(&terminator);

            let int_val = value.into_int_value();
            let coerced = self.builder
                .build_int_to_ptr(int_val, to_ptr, "phi_inttoptr")
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

            return Ok(coerced.into());
        }

        // v0.60.15: Handle pointer to integer coercion
        // This occurs when a PHI expects an integer but receives a pointer
        if let (BasicTypeEnum::PointerType(_), BasicTypeEnum::IntType(to_int)) =
            (value_type, phi_type)
        {
            let terminator = source_bb.get_terminator()
                .ok_or_else(|| CodeGenError::LlvmError(
                    format!("Block {} has no terminator for phi coercion", source_label)))?;
            self.builder.position_before(&terminator);

            let ptr_val = value.into_pointer_value();
            let coerced = self.builder
                .build_ptr_to_int(ptr_val, to_int, "phi_ptrtoint")
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

            return Ok(coerced.into());
        }

        // For non-integer type mismatches, return error with details
        Err(CodeGenError::LlvmError(format!(
            "PHI type mismatch in block {}: value type {:?} vs phi type {:?}",
            source_label, value_type, phi_type
        )))
    }

    /// v0.51.0: Generate code for a basic block (skips PHI instructions)
    fn gen_basic_block(&mut self, block: &BasicBlock) -> CodeGenResult<()> {
        let bb = self.blocks.get(&block.label).unwrap();
        self.builder.position_at_end(*bb);

        // Generate instructions (skip PHI nodes - they were handled in pass 1)
        for inst in &block.instructions {
            if matches!(inst, MirInst::Phi { .. }) {
                continue;
            }
            self.gen_instruction(inst)?;
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
                // v0.60.122: Track String constants for proper string comparison
                // When a variable is assigned a string literal, track it for bmb_string_eq
                if matches!(value, Constant::String(_)) {
                    self.string_variables.insert(dest.name.clone());
                }
            }

            MirInst::Copy { dest, src } => {
                let value = self.load_from_place(src)?;
                self.store_to_place(dest, value)?;
                // v0.60.96: Propagate string status for proper string comparison
                // If source is a String variable, destination should also be tracked
                if self.string_variables.contains(&src.name) {
                    self.string_variables.insert(dest.name.clone());
                }
            }

            MirInst::BinOp { dest, op, lhs, rhs } => {
                let lhs_val = self.gen_operand(lhs)?;
                let rhs_val = self.gen_operand(rhs)?;
                // v0.60.33: Detect if this is a String comparison (not typed pointer)
                // String constants come from Operand::Constant(Constant::String(_))
                // v0.60.81: Also check if operands are String-typed variables
                let lhs_is_string = match lhs {
                    Operand::Constant(Constant::String(_)) => true,
                    Operand::Place(p) => self.string_variables.contains(&p.name),
                    _ => false,
                };
                let rhs_is_string = match rhs {
                    Operand::Constant(Constant::String(_)) => true,
                    Operand::Place(p) => self.string_variables.contains(&p.name),
                    _ => false,
                };
                let is_string_comparison = lhs_is_string || rhs_is_string;
                let result = self.gen_binop_with_string_hint(*op, lhs_val, rhs_val, is_string_comparison)?;
                self.store_to_place(dest, result)?;
            }

            MirInst::UnaryOp { dest, op, src } => {
                let src_val = self.gen_operand(src)?;
                let result = self.gen_unaryop(*op, src_val)?;
                self.store_to_place(dest, result)?;
            }

            MirInst::Call { dest, func, args, is_tail } => {
                // v0.35.4: Handle type conversion intrinsics specially
                // v0.50.66: Tail call optimization support via inkwell API
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
                } else if func == "ord" && args.len() == 1 {
                    // v0.50.75: Special handling for ord()
                    // If argument is already i64 (from char_at/byte_at), it's already the char code
                    // Just pass through instead of calling bmb_ord which expects a ptr
                    let arg = self.gen_operand(&args[0])?;
                    if arg.is_int_value() {
                        // Already a character code, pass through
                        if let Some(dest_place) = dest {
                            self.store_to_place(dest_place, arg)?;
                        }
                    } else {
                        // It's a pointer (string), call bmb_ord
                        let function = *self
                            .functions
                            .get(func)
                            .ok_or_else(|| CodeGenError::UnknownFunction(func.clone()))?;
                        let call_result = self.builder
                            .build_call(function, &[arg.into()], "call")
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                        if let Some(dest_place) = dest {
                            if let Some(ret_val) = call_result.try_as_basic_value().basic() {
                                self.store_to_place(dest_place, ret_val)?;
                            }
                        }
                    }
                } else if func == "load_i64" && args.len() == 1 {
                    // v0.51.2: Inline load_i64 as direct LLVM load instruction
                    // This avoids function call overhead for pointer dereference
                    let ptr_as_i64 = self.gen_operand(&args[0])?;
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let ptr = self.builder
                        .build_int_to_ptr(ptr_as_i64.into_int_value(), ptr_type, "inttoptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    let loaded = self.builder
                        .build_load(self.context.i64_type(), ptr, "load")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    if let Some(dest_place) = dest {
                        self.store_to_place(dest_place, loaded)?;
                    }
                } else if func == "store_i64" && args.len() == 2 {
                    // v0.51.2: Inline store_i64 as direct LLVM store instruction
                    // This avoids function call overhead for pointer write
                    let ptr_as_i64 = self.gen_operand(&args[0])?;
                    let value = self.gen_operand(&args[1])?;
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let ptr = self.builder
                        .build_int_to_ptr(ptr_as_i64.into_int_value(), ptr_type, "inttoptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    self.builder
                        .build_store(ptr, value.into_int_value())
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    // store_i64 returns void, but MIR may expect a value
                    if let Some(dest_place) = dest {
                        let unit_value = self.context.i64_type().const_int(0, false);
                        self.store_to_place(dest_place, unit_value.into())?;
                    }
                } else if func == "load_f64" && args.len() == 1 {
                    // v0.51.5: Inline load_f64 as direct LLVM load instruction (f64)
                    let ptr_as_i64 = self.gen_operand(&args[0])?;
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let ptr = self.builder
                        .build_int_to_ptr(ptr_as_i64.into_int_value(), ptr_type, "inttoptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    let loaded = self.builder
                        .build_load(self.context.f64_type(), ptr, "load_f64")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    if let Some(dest_place) = dest {
                        self.store_to_place(dest_place, loaded)?;
                    }
                } else if func == "store_f64" && args.len() == 2 {
                    // v0.51.5: Inline store_f64 as direct LLVM store instruction (f64)
                    let ptr_as_i64 = self.gen_operand(&args[0])?;
                    let value = self.gen_operand(&args[1])?;
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let ptr = self.builder
                        .build_int_to_ptr(ptr_as_i64.into_int_value(), ptr_type, "inttoptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    self.builder
                        .build_store(ptr, value.into_float_value())
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    // store_f64 returns void, but MIR may expect a value
                    if let Some(dest_place) = dest {
                        let unit_value = self.context.i64_type().const_int(0, false);
                        self.store_to_place(dest_place, unit_value.into())?;
                    }
                } else if func == "byte_at" && args.len() == 2 {
                    // v0.60.5: Inline byte_at as direct memory access for performance
                    // String struct layout: { ptr data, i64 len, i64 capacity }
                    // byte_at(s, idx) -> s.data[idx] as i64
                    let string_ptr = self.gen_operand(&args[0])?;
                    let index = self.gen_operand(&args[1])?;

                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let i64_type = self.context.i64_type();
                    let i8_type = self.context.i8_type();

                    // String struct type: { ptr, i64, i64 }
                    let string_struct_type = self.context.struct_type(
                        &[ptr_type.into(), i64_type.into(), i64_type.into()],
                        false
                    );

                    // GEP to get data pointer (field 0)
                    let data_ptr_ptr = self.builder
                        .build_struct_gep(string_struct_type, string_ptr.into_pointer_value(), 0, "data_ptr_ptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Load the data pointer
                    let data_ptr = self.builder
                        .build_load(ptr_type, data_ptr_ptr, "data_ptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                        .into_pointer_value();

                    // GEP to byte at data[index]
                    let byte_ptr = unsafe {
                        self.builder
                            .build_gep(i8_type, data_ptr, &[index.into_int_value()], "byte_ptr")
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    };

                    // Load byte as i8
                    let byte_val = self.builder
                        .build_load(i8_type, byte_ptr, "byte")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                        .into_int_value();

                    // Zero-extend to i64
                    let result = self.builder
                        .build_int_z_extend(byte_val, i64_type, "byte_i64")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    if let Some(dest_place) = dest {
                        self.store_to_place(dest_place, result.into())?;
                    }
                } else if func == "len" && args.len() == 1 {
                    // v0.60.5: Inline len as direct struct field access for performance
                    // String struct layout: { ptr data, i64 len, i64 capacity }
                    // len(s) -> s.len
                    let string_ptr = self.gen_operand(&args[0])?;

                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let i64_type = self.context.i64_type();

                    // String struct type: { ptr, i64, i64 }
                    let string_struct_type = self.context.struct_type(
                        &[ptr_type.into(), i64_type.into(), i64_type.into()],
                        false
                    );

                    // GEP to get len field (field 1)
                    let len_ptr = self.builder
                        .build_struct_gep(string_struct_type, string_ptr.into_pointer_value(), 1, "len_ptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Load the length
                    let len_val = self.builder
                        .build_load(i64_type, len_ptr, "len")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    if let Some(dest_place) = dest {
                        self.store_to_place(dest_place, len_val)?;
                    }
                } else if func == "load_u8" && args.len() == 1 {
                    // v0.60.6: Inline load_u8 as direct byte memory access for performance
                    // load_u8(addr) -> *((uint8_t*)addr) as i64
                    let addr = self.gen_operand(&args[0])?;

                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let i8_type = self.context.i8_type();
                    let i64_type = self.context.i64_type();

                    // Convert i64 address to pointer
                    let ptr = self.builder
                        .build_int_to_ptr(addr.into_int_value(), ptr_type, "addr_ptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Load byte as i8
                    let byte_val = self.builder
                        .build_load(i8_type, ptr, "byte")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                        .into_int_value();

                    // Zero-extend to i64
                    let result = self.builder
                        .build_int_z_extend(byte_val, i64_type, "byte_i64")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    if let Some(dest_place) = dest {
                        self.store_to_place(dest_place, result.into())?;
                    }
                } else if func == "store_u8" && args.len() == 2 {
                    // v0.60.6: Inline store_u8 as direct byte memory write for performance
                    // store_u8(addr, val) -> *((uint8_t*)addr) = val & 0xFF
                    let addr = self.gen_operand(&args[0])?;
                    let val = self.gen_operand(&args[1])?;

                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let i8_type = self.context.i8_type();

                    // Convert i64 address to pointer
                    let ptr = self.builder
                        .build_int_to_ptr(addr.into_int_value(), ptr_type, "addr_ptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Truncate i64 value to i8
                    let byte_val = self.builder
                        .build_int_truncate(val.into_int_value(), i8_type, "val_byte")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Store the byte
                    self.builder
                        .build_store(ptr, byte_val)
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // store_u8 returns void, but MIR may expect a value
                    if let Some(dest_place) = dest {
                        let unit_value = self.context.i64_type().const_int(0, false);
                        self.store_to_place(dest_place, unit_value.into())?;
                    }
                } else if func == "load_i32" && args.len() == 1 {
                    // v0.60.58: Inline load_i32 as direct 32-bit memory access
                    // load_i32(addr) -> (int64_t)(*((int32_t*)addr))
                    let addr = self.gen_operand(&args[0])?;

                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let i32_type = self.context.i32_type();
                    let i64_type = self.context.i64_type();

                    // Convert i64 address to pointer
                    let ptr = self.builder
                        .build_int_to_ptr(addr.into_int_value(), ptr_type, "addr_ptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Load the 32-bit integer
                    let int32_val = self.builder
                        .build_load(i32_type, ptr, "int32")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                        .into_int_value();

                    // Sign-extend to i64
                    let result = self.builder
                        .build_int_s_extend(int32_val, i64_type, "int64")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    if let Some(dest_place) = dest {
                        self.store_to_place(dest_place, result.into())?;
                    }
                } else if func == "store_i32" && args.len() == 2 {
                    // v0.60.58: Inline store_i32 as direct 32-bit memory write
                    // store_i32(addr, val) -> *((int32_t*)addr) = (int32_t)val
                    let addr = self.gen_operand(&args[0])?;
                    let val = self.gen_operand(&args[1])?;

                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let i32_type = self.context.i32_type();

                    // Convert i64 address to pointer
                    let ptr = self.builder
                        .build_int_to_ptr(addr.into_int_value(), ptr_type, "addr_ptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Truncate i64 value to i32
                    let int32_val = self.builder
                        .build_int_truncate(val.into_int_value(), i32_type, "val_i32")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Store the 32-bit integer
                    self.builder
                        .build_store(ptr, int32_val)
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // store_i32 returns void, but MIR may expect a value
                    if let Some(dest_place) = dest {
                        let unit_value = self.context.i64_type().const_int(0, false);
                        self.store_to_place(dest_place, unit_value.into())?;
                    }
                } else {
                    // v0.51.18: Check if function has _cstr variant for string literal optimization
                    // This avoids BMB String wrapper overhead (2.81x → 1.0x for syscall_overhead)
                    let has_cstr_variant = matches!(func.as_str(),
                        "file_exists" | "bmb_file_exists");

                    // Check if all string args are literals (for _cstr optimization)
                    // Only use _cstr variant if ALL arguments are string literal constants
                    let all_string_args_are_literals = has_cstr_variant && args.iter().all(|arg| {
                        matches!(arg, Operand::Constant(Constant::String(_)))
                    });

                    // Use _cstr variant if available and all string args are literals
                    let use_cstr = has_cstr_variant && all_string_args_are_literals;
                    let actual_func_name = if use_cstr {
                        format!("{}_cstr", func)
                    } else {
                        func.clone()
                    };

                    // v0.50.67: Get function first and copy to avoid borrow conflict
                    let function = *self
                        .functions
                        .get(&actual_func_name)
                        .or_else(|| self.functions.get(func))
                        .ok_or_else(|| CodeGenError::UnknownFunction(func.clone()))?;

                    // Get the function's parameter types for argument coercion
                    let fn_type = function.get_type();
                    let param_types: Vec<_> = fn_type.get_param_types();

                    // Generate operands with type coercion for narrowed parameters
                    // v0.51.0: The narrowing pass may change i64 params to i32
                    // v0.51.1: Handle ptr <-> i64 conversions for external calls (malloc, free, etc.)
                    let mut arg_values: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
                    for (i, arg) in args.iter().enumerate() {
                        // v0.51.18: For functions with _cstr variants, pass raw C string directly
                        let mut arg_val = if use_cstr {
                            if let Operand::Constant(Constant::String(s)) = arg {
                                self.gen_cstring_constant(s).into()
                            } else {
                                self.gen_operand(arg)?
                            }
                        } else {
                            self.gen_operand(arg)?
                        };

                        // Check if argument type matches parameter type
                        if i < param_types.len() {
                            let param_meta_type = param_types[i];
                            let arg_type = arg_val.get_type();

                            // Convert BasicMetadataTypeEnum to BasicTypeEnum for comparison
                            if let inkwell::types::BasicMetadataTypeEnum::IntType(param_int) = param_meta_type {
                                if let BasicTypeEnum::IntType(arg_int) = arg_type {
                                    let arg_bits = arg_int.get_bit_width();
                                    let param_bits = param_int.get_bit_width();

                                    if arg_bits != param_bits {
                                        let int_val = arg_val.into_int_value();
                                        arg_val = if arg_bits > param_bits {
                                            // Truncate: e.g., i64 -> i32
                                            self.builder
                                                .build_int_truncate(int_val, param_int, "arg_trunc")
                                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                                .into()
                                        } else {
                                            // Sign extend: e.g., i32 -> i64
                                            self.builder
                                                .build_int_s_extend(int_val, param_int, "arg_sext")
                                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                                .into()
                                        };
                                    }
                                }
                            } else if let inkwell::types::BasicMetadataTypeEnum::PointerType(param_ptr) = param_meta_type {
                                // v0.51.1: External function expects ptr (e.g., free(ptr))
                                // BMB uses i64 for pointers, so convert i64 -> ptr
                                if let BasicTypeEnum::IntType(_) = arg_type {
                                    let int_val = arg_val.into_int_value();
                                    arg_val = self.builder
                                        .build_int_to_ptr(int_val, param_ptr, "inttoptr")
                                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                        .into();
                                }
                            }
                        }

                        arg_values.push(arg_val.into());
                    }

                    let call_result = self.builder
                        .build_call(function, &arg_values, "call")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // v0.50.66: Enable tail call optimization when MIR marks it as tail position
                    if *is_tail {
                        call_result.set_tail_call(true);
                    }

                    if let Some(dest_place) = dest {
                        if let Some(ret_val) = call_result.try_as_basic_value().basic() {
                            // v0.51.1: Convert ptr return values to i64 for BMB's pointer representation
                            // v0.51.3: Only convert ptr->i64 when destination expects i64 (e.g., malloc)
                            //          Keep ptr as-is when destination is ptr type (e.g., String functions)
                            // v0.60.26: Improved logic to eliminate unnecessary ptrtoint/inttoptr pairs
                            let stored_val = if ret_val.is_pointer_value() {
                                // Check if destination variable expects a pointer or integer
                                let dest_expects_int = if let Some((_, dest_ty)) = self.variables.get(&dest_place.name) {
                                    dest_ty.is_int_type()
                                } else {
                                    // v0.60.26: For SSA temps, determine based on context
                                    if let Some(mir_ret_ty) = self.function_return_types.get(func) {
                                        // User function: use its declared return type
                                        !mir_ret_ty.is_pointer_type()
                                    } else {
                                        // External function
                                        // v0.60.28: Check if this is a String-returning function
                                        // These should keep ptr as ptr to avoid unnecessary ptrtoint/inttoptr
                                        let string_returning_funcs = [
                                            "sb_build", "chr", "slice", "char_to_string",
                                            "int_to_string", "string_from_cstr", "string_concat",
                                            "read_file", "get_arg"
                                        ];
                                        if string_returning_funcs.contains(&func.as_str()) {
                                            // String-returning functions: keep ptr as ptr
                                            false
                                        } else {
                                            // Other external functions (malloc, calloc, etc.)
                                            // Check if CURRENT function returns pointer type
                                            // If so, keep ptr as ptr to enable `fn f() -> *T = malloc() as *T` optimization
                                            // Otherwise, use legacy i64 conversion for backward compatibility
                                            let current_returns_ptr = self.current_ret_type
                                                .map(|ty| ty.is_pointer_type())
                                                .unwrap_or(false);
                                            !current_returns_ptr
                                        }
                                    }
                                };

                                if dest_expects_int {
                                    let ptr_val = ret_val.into_pointer_value();
                                    self.builder
                                        .build_ptr_to_int(ptr_val, self.context.i64_type(), "ptrtoint")
                                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                        .into()
                                } else {
                                    ret_val
                                }
                            } else {
                                ret_val
                            };
                            self.store_to_place(dest_place, stored_val)?;
                            // v0.60.81: Track String-typed destinations for proper string comparison
                            // Check both built-in String-returning functions and user-defined ones
                            let string_funcs = [
                                "sb_build", "chr", "slice", "char_to_string",
                                "int_to_string", "string_from_cstr", "string_concat",
                                "read_file", "get_arg", "getenv", "bmb_chr",
                                "bmb_string_slice", "bmb_string_concat", "bmb_read_file",
                                "bmb_int_to_string", "bmb_string_from_cstr", "bmb_sb_build",
                                "bmb_getenv"
                            ];
                            let is_string_func = string_funcs.contains(&func.as_str())
                                || self.function_return_types.get(func)
                                    .map(|ty| *ty == MirType::String)
                                    .unwrap_or(false);
                            if is_string_func {
                                self.string_variables.insert(dest_place.name.clone());
                            }
                        } else {
                            // v0.50.75: Handle void-returning functions with destinations
                            // MIR may assign the result of a void call to a variable (e.g., let u = println_str(...))
                            // Allocate storage with Unit type (i8) and store 0
                            let unit_type = self.context.i8_type();
                            let unit_value = unit_type.const_int(0, false);
                            self.store_to_place(dest_place, unit_value.into())?;
                        }
                    }
                }
            }

            // v0.51.0: PHI nodes are handled in the three-pass generation
            // Pass 1 creates PHI instructions, Pass 3 populates incoming values
            // This should not be reached since gen_basic_block skips PHI instructions
            MirInst::Phi { .. } => {
                // PHI nodes are now true LLVM PHI instructions (not alloca/store/load)
                // If we reach here, it's a bug in gen_basic_block
            }
            // v0.50.73: Array operations for native array support
            // v0.60.59: Optimized with memset for zero-initialized arrays
            MirInst::ArrayInit { dest, element_type, elements } => {
                // Allocate array on stack
                let array_size = elements.len() as u32;

                // Use i64 array type for most cases (BMB arrays are typically i64)
                let i64_type = self.context.i64_type();
                let array_type = i64_type.array_type(array_size.max(1));

                let array_ptr = self.builder
                    .build_alloca(array_type, &dest.name)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // v0.60.59: Check if all elements are zero constants (threshold: 64 elements)
                let all_zeros = array_size >= 64 && elements.iter().all(|e| {
                    match e {
                        Operand::Constant(Constant::Int(0)) => true,
                        Operand::Constant(Constant::Float(f)) => *f == 0.0,
                        _ => false,
                    }
                });

                if all_zeros {
                    // Use memset for zero initialization
                    let elem_size: u64 = match element_type {
                        MirType::I64 | MirType::F64 | MirType::U64 => 8,
                        MirType::I32 | MirType::U32 => 4,
                        MirType::Bool => 1,
                        _ => 8, // default to 8 for pointer-sized types
                    };
                    let total_bytes = (array_size as u64) * elem_size;

                    // Build memset intrinsic call
                    let i8_type = self.context.i8_type();
                    let i64_type = self.context.i64_type();
                    self.builder.build_memset(
                        array_ptr,
                        8, // alignment
                        i8_type.const_int(0, false),
                        i64_type.const_int(total_bytes, false),
                    ).map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                } else {
                    // Initialize elements one by one
                    // v0.60.25: Use array_type as GEP base type with two indices
                    // First index dereferences the array, second indexes into elements
                    for (i, elem) in elements.iter().enumerate() {
                        let elem_val = self.gen_operand(elem)?;
                        let elem_ptr = unsafe {
                            self.builder.build_in_bounds_gep(
                                array_type,  // Use array type, not element type
                                array_ptr,
                                &[
                                    self.context.i64_type().const_int(0, false),
                                    self.context.i64_type().const_int(i as u64, false)
                                ],
                                &format!("{}_e{}", dest.name, i)
                            )
                        }.map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                        self.builder.build_store(elem_ptr, elem_val)
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    }
                }

                // Store array pointer to destination
                // Arrays are represented as pointers in variables
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                self.variables.insert(dest.name.clone(), (array_ptr, ptr_type.into()));

                // v0.60.25: Track this as an array variable
                // Array variables should NOT be loaded from - the alloca pointer IS the array
                self.array_variables.insert(dest.name.clone());
            }

            MirInst::IndexLoad { dest, array, index, element_type } => {
                // v0.60.20: Load the array pointer value from the variable
                // We need to load the pointer value, not use the alloca address directly
                let array_val = self.load_from_place(array)?;
                let array_ptr = if array_val.is_pointer_value() {
                    array_val.into_pointer_value()
                } else {
                    // Convert i64 to pointer if needed (legacy code compatibility)
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    self.builder
                        .build_int_to_ptr(array_val.into_int_value(), ptr_type, "inttoptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                };

                // Evaluate index operand
                let index_val = self.gen_operand(index)?;

                // v0.60.20: Use actual element type for GEP and load
                let llvm_elem_type = self.mir_type_to_llvm(element_type);

                // v0.60.19: GEP to element pointer using single index (works for both arrays and pointers)
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(
                        llvm_elem_type,
                        array_ptr,
                        &[index_val.into_int_value()],
                        &format!("{}_ptr", dest.name)
                    )
                }.map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Load element
                // v0.60.35: Add alignment hint based on element type for better performance
                let loaded = self.builder
                    .build_load(llvm_elem_type, elem_ptr, &dest.name)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Set alignment based on element type size
                let alignment = match element_type {
                    MirType::I64 | MirType::U64 | MirType::F64 | MirType::Ptr(_) | MirType::StructPtr(_) | MirType::String => 8,
                    MirType::I32 | MirType::U32 | MirType::Char => 4,
                    MirType::Bool => 1,
                    _ => 8, // Default to 8 for structs and other types
                };
                if let Some(load_inst) = loaded.as_instruction_value() {
                    let _ = load_inst.set_alignment(alignment);
                }

                // Store to destination
                self.store_to_place(dest, loaded)?;
            }

            MirInst::IndexStore { array, index, value, element_type } => {
                // v0.60.20: Load the array pointer value from the variable
                // We need to load the pointer value, not use the alloca address directly
                let array_val = self.load_from_place(array)?;
                let array_ptr = if array_val.is_pointer_value() {
                    array_val.into_pointer_value()
                } else {
                    // Convert i64 to pointer if needed (legacy code compatibility)
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    self.builder
                        .build_int_to_ptr(array_val.into_int_value(), ptr_type, "inttoptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                };

                // Evaluate operands
                let index_val = self.gen_operand(index)?;
                let store_val = self.gen_operand(value)?;

                // v0.60.20: Use actual element type for GEP
                let llvm_elem_type = self.mir_type_to_llvm(element_type);

                // v0.60.19: GEP to element pointer using single index (works for both arrays and pointers)
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(
                        llvm_elem_type,
                        array_ptr,
                        &[index_val.into_int_value()],
                        &format!("{}_store_ptr", array.name)
                    )
                }.map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Store value
                // v0.60.35: Add alignment hint based on element type for better performance
                let store_inst = self.builder.build_store(elem_ptr, store_val)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Set alignment based on element type size
                let alignment = match element_type {
                    MirType::I64 | MirType::U64 | MirType::F64 | MirType::Ptr(_) | MirType::StructPtr(_) | MirType::String => 8,
                    MirType::I32 | MirType::U32 | MirType::Char => 4,
                    MirType::Bool => 1,
                    _ => 8, // Default to 8 for structs and other types
                };
                let _ = store_inst.set_alignment(alignment);
            }

            // v0.50.80: Type cast instruction
            MirInst::Cast { dest, src, from_ty, to_ty } => {
                let src_val = self.gen_operand(src)?;
                let cast_val = self.gen_cast(src_val, from_ty, to_ty)?;
                self.store_to_place(dest, cast_val)?;
            }

            // v0.55: Tuple initialization - builds LLVM struct from elements
            // v0.60.18: Handle narrowed types - operand may be i32 but tuple expects i64
            MirInst::TupleInit { dest, elements } => {
                // Create the LLVM struct type from element types
                let elem_types: Vec<BasicTypeEnum<'ctx>> = elements
                    .iter()
                    .map(|(ty, _)| self.mir_type_to_llvm(ty))
                    .collect();
                let struct_type = self.context.struct_type(&elem_types, false);

                // Build the struct value using insertvalue instructions
                let mut struct_val = struct_type.get_undef();
                for (i, (_, operand)) in elements.iter().enumerate() {
                    let elem_val = self.gen_operand(operand)?;

                    // v0.60.18: Check if type widening is needed due to ConstantPropagationNarrowing
                    // When a parameter is narrowed to i32, but the tuple element type is i64,
                    // we need to sign-extend the value before insertion
                    let expected_ty = elem_types[i];
                    let elem_val_final = if elem_val.is_int_value() && expected_ty.is_int_type() {
                        let actual_int = elem_val.into_int_value();
                        let expected_int = expected_ty.into_int_type();
                        let actual_bits = actual_int.get_type().get_bit_width();
                        let expected_bits = expected_int.get_bit_width();

                        if actual_bits < expected_bits {
                            // Need sign extension (i32 -> i64)
                            self.builder
                                .build_int_s_extend(actual_int, expected_int, &format!("{}_sext{}", dest.name, i))
                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                .into()
                        } else if actual_bits > expected_bits {
                            // Need truncation (shouldn't happen normally, but handle it)
                            self.builder
                                .build_int_truncate(actual_int, expected_int, &format!("{}_trunc{}", dest.name, i))
                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                .into()
                        } else {
                            elem_val
                        }
                    } else {
                        elem_val
                    };

                    let result = self.builder
                        .build_insert_value(struct_val, elem_val_final, i as u32, &format!("{}_elem{}", dest.name, i))
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    struct_val = result.into_struct_value();
                }

                // Store the tuple value
                self.store_to_place(dest, struct_val.into())?;
            }

            // v0.55: Tuple field extraction - extracts element from LLVM struct
            MirInst::TupleExtract { dest, tuple, index, element_type: _ } => {
                // Load the tuple value
                let tuple_val = self.load_from_place(tuple)?;

                // Extract the element using extractvalue
                let elem_val = self.builder
                    .build_extract_value(tuple_val.into_struct_value(), *index as u32, &format!("{}_extract", dest.name))
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Store the extracted value
                self.store_to_place(dest, elem_val)?;
            }

            // v0.60.19: Pointer offset - generates proper LLVM GEP for alias analysis
            MirInst::PtrOffset { dest, ptr, offset, element_type } => {
                // Get the pointer value
                let ptr_val = self.gen_operand(ptr)?;
                let offset_val = self.gen_operand(offset)?;

                // v0.60.20: Get LLVM type for the element - use actual struct type for struct pointers
                // This ensures GEP uses correct stride (e.g., 56 bytes for Body struct, not 8 bytes)
                let llvm_elem_type: BasicTypeEnum = match element_type {
                    MirType::Struct { name, .. } => {
                        self.get_struct_llvm_type(name)?.into()
                    }
                    MirType::StructPtr(name) => {
                        // StructPtr is the element type for *T where T is a struct
                        self.get_struct_llvm_type(name)?.into()
                    }
                    _ => self.mir_type_to_llvm(element_type),
                };

                // Ensure we have a pointer value
                let ptr_ptr = if ptr_val.is_pointer_value() {
                    ptr_val.into_pointer_value()
                } else {
                    // Convert i64 to pointer if needed (fallback for legacy code)
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    self.builder
                        .build_int_to_ptr(ptr_val.into_int_value(), ptr_type, &format!("{}_inttoptr", dest.name))
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                };

                // Generate GEP instruction - this enables LLVM's alias analysis
                let result_ptr = unsafe {
                    self.builder.build_gep(
                        llvm_elem_type,
                        ptr_ptr,
                        &[offset_val.into_int_value()],
                        &format!("{}_gep", dest.name),
                    )
                }.map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Store the result pointer
                self.store_to_place(dest, result_ptr.into())?;
            }

            // v0.60.21: Stack array allocation without initialization
            // Used for: `let arr: [T; N];` - zero-overhead stack arrays
            MirInst::ArrayAlloc { dest, element_type, size } => {
                let llvm_elem_type = self.mir_type_to_llvm(element_type);
                let array_type = llvm_elem_type.array_type(*size as u32);

                // Allocate array on stack (no initialization)
                let array_ptr = self.builder
                    .build_alloca(array_type, &dest.name)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Store array pointer to destination variable
                // For compatibility with IndexLoad/IndexStore, we need to store the pointer
                // in a way that load_from_place can retrieve it
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let alloca = self.builder
                    .build_alloca(ptr_type, &format!("{}_var", dest.name))
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                self.builder.build_store(alloca, array_ptr)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.variables.insert(dest.name.clone(), (alloca, ptr_type.into()));
            }

            // v0.60.20: Pointer load - load value through native pointer
            MirInst::PtrLoad { dest, ptr, element_type } => {
                // Get the pointer value
                let ptr_val = self.gen_operand(ptr)?;

                // Ensure we have a pointer value
                let ptr_ptr = if ptr_val.is_pointer_value() {
                    ptr_val.into_pointer_value()
                } else {
                    // Convert i64 to pointer if needed
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    self.builder
                        .build_int_to_ptr(ptr_val.into_int_value(), ptr_type, &format!("{}_inttoptr", dest.name))
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                };

                // Get element type for load
                let llvm_elem_type = self.mir_type_to_llvm(element_type);

                // Load the value
                // v0.60.35: Add alignment hint based on element type
                let loaded = self.builder
                    .build_load(llvm_elem_type, ptr_ptr, &dest.name)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Set alignment based on element type size
                let alignment = match element_type {
                    MirType::I64 | MirType::U64 | MirType::F64 | MirType::Ptr(_) | MirType::StructPtr(_) | MirType::String => 8,
                    MirType::I32 | MirType::U32 | MirType::Char => 4,
                    MirType::Bool => 1,
                    _ => 8, // Default to 8 for structs and other types
                };
                if let Some(load_inst) = loaded.as_instruction_value() {
                    let _ = load_inst.set_alignment(alignment);
                }

                // Store to destination
                self.store_to_place(dest, loaded)?;
            }

            // v0.60.20: Pointer store - store value through native pointer
            MirInst::PtrStore { ptr, value, element_type } => {
                // Get the pointer value
                let ptr_val = self.gen_operand(ptr)?;

                // Ensure we have a pointer value
                let ptr_ptr = if ptr_val.is_pointer_value() {
                    ptr_val.into_pointer_value()
                } else {
                    // Convert i64 to pointer if needed
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    self.builder
                        .build_int_to_ptr(ptr_val.into_int_value(), ptr_type, "ptr_inttoptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                };

                // Get value to store
                let store_val = self.gen_operand(value)?;

                // Get element type for proper alignment hints
                let _llvm_elem_type = self.mir_type_to_llvm(element_type);

                // Store the value
                // v0.60.35: Add alignment hint based on element type
                let store_inst = self.builder.build_store(ptr_ptr, store_val)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Set alignment based on element type size
                let alignment = match element_type {
                    MirType::I64 | MirType::U64 | MirType::F64 | MirType::Ptr(_) | MirType::StructPtr(_) | MirType::String => 8,
                    MirType::I32 | MirType::U32 | MirType::Char => 4,
                    MirType::Bool => 1,
                    _ => 8, // Default to 8 for structs and other types
                };
                let _ = store_inst.set_alignment(alignment);
            }

            // v0.60.7: Field access via GEP + load
            // v0.60.253: Handle empty struct_name for enum variant fields
            MirInst::FieldAccess { dest, base, field: _, field_index, struct_name } => {
                // Load the base pointer
                let base_val = self.load_from_place(base)?;

                // v0.60.15: Handle IntValue from ptrtoint/cast - convert back to pointer
                let base_ptr = if base_val.is_pointer_value() {
                    base_val.into_pointer_value()
                } else if base_val.is_int_value() {
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    self.builder
                        .build_int_to_ptr(base_val.into_int_value(), ptr_type, "inttoptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                } else {
                    return Err(CodeGenError::LlvmError(format!(
                        "FieldAccess base must be pointer or integer, got {:?}", base_val
                    )));
                };

                // v0.60.253: Handle empty struct_name (enum variant fields)
                // Enum variants use i64 array layout: [discriminant, arg0, arg1, ...]
                // Field index 0 is arg0 (at offset 1 in the array, after discriminant)
                let field_val = if struct_name.is_empty() {
                    let i64_type = self.context.i64_type();
                    // For enum variant fields, use array GEP with field_index + 1
                    // (skip discriminant at index 0)
                    let field_ptr = unsafe {
                        self.builder.build_in_bounds_gep(
                            i64_type,
                            base_ptr,
                            &[i64_type.const_int((*field_index + 1) as u64, false)],
                            "enum_field_ptr",
                        ).map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    };
                    self.builder
                        .build_load(i64_type, field_ptr, "enum_field_val")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                } else {
                    // Normal struct field access via struct GEP
                    let struct_type = self.get_struct_llvm_type(struct_name)?;
                    let field_ptr = self.builder
                        .build_struct_gep(struct_type, base_ptr, *field_index as u32, "field_ptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Get field type for load
                    let fields = self.struct_defs.get(struct_name)
                        .ok_or_else(|| CodeGenError::LlvmError(format!("Unknown struct: {}", struct_name)))?;
                    let field_llvm_type = self.mir_type_to_llvm(&fields[*field_index].1);

                    self.builder
                        .build_load(field_llvm_type, field_ptr, "field_val")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                };

                // Store to destination
                self.store_to_place(dest, field_val)?;
            }

            // v0.60.7: Field store via GEP + store
            // v0.60.15: Handle IntValue from ptrtoint/cast
            // v0.60.253: Handle empty struct_name for enum variant fields
            MirInst::FieldStore { base, field: _, field_index, struct_name, value } => {
                // Load the base pointer (struct pointer)
                let base_val = self.load_from_place(base)?;

                // v0.60.15: Handle IntValue from ptrtoint/cast - convert back to pointer
                let base_ptr = if base_val.is_pointer_value() {
                    base_val.into_pointer_value()
                } else if base_val.is_int_value() {
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    self.builder
                        .build_int_to_ptr(base_val.into_int_value(), ptr_type, "inttoptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                } else {
                    return Err(CodeGenError::LlvmError(format!(
                        "FieldStore base must be pointer or integer, got {:?}", base_val
                    )));
                };

                // Generate the value to store
                let store_val = self.gen_operand(value)?;

                // v0.60.253: Handle empty struct_name (enum variant fields)
                let field_ptr = if struct_name.is_empty() {
                    let i64_type = self.context.i64_type();
                    // For enum variant fields, use array GEP with field_index + 1
                    unsafe {
                        self.builder.build_in_bounds_gep(
                            i64_type,
                            base_ptr,
                            &[i64_type.const_int((*field_index + 1) as u64, false)],
                            "enum_field_ptr",
                        ).map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    }
                } else {
                    // Normal struct field store via struct GEP
                    let struct_type = self.get_struct_llvm_type(struct_name)?;
                    self.builder
                        .build_struct_gep(struct_type, base_ptr, *field_index as u32, "field_ptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                };

                // Store the value
                self.builder
                    .build_store(field_ptr, store_val)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            // v0.60.253: StructInit - allocate struct and initialize fields
            MirInst::StructInit { dest, struct_name, fields } => {
                // Get the struct LLVM type
                let struct_type = self.get_struct_llvm_type(struct_name)?;

                // Allocate space for the struct on the stack
                let struct_ptr = self.builder
                    .build_alloca(struct_type, &dest.name)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Initialize each field
                for (i, (_field_name, value)) in fields.iter().enumerate() {
                    // GEP to the field
                    let field_ptr = self.builder
                        .build_struct_gep(struct_type, struct_ptr, i as u32, &format!("{}_f{}", dest.name, i))
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                    // Get the value to store
                    let field_val = self.gen_operand(value)?;

                    // Store the field value
                    self.builder
                        .build_store(field_ptr, field_val)
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                }

                // Store the struct pointer to destination
                self.store_to_place(dest, struct_ptr.into())?;
            }

            // v0.60.253: EnumVariant - allocate tagged union and initialize
            MirInst::EnumVariant { dest, enum_name: _, variant, args } => {
                // Enums are represented as tagged unions:
                // - First word (i64): discriminant (hash of variant name)
                // - Following words: variant data

                let i64_type = self.context.i64_type();

                // Calculate size: discriminant + args (minimum 2 words)
                let size = (1 + args.len()).max(2);
                let byte_size = size * 8; // Each word is 8 bytes

                // Use malloc for enum allocation to support returning from functions
                // (alloca would be deallocated when function returns)
                let malloc_fn = self.module.get_function("malloc")
                    .ok_or_else(|| CodeGenError::LlvmError("malloc not found".to_string()))?;
                let call_result = self.builder
                    .build_call(
                        malloc_fn,
                        &[i64_type.const_int(byte_size as u64, false).into()],
                        &dest.name,
                    )
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                let enum_ptr = call_result.try_as_basic_value().basic()
                    .ok_or_else(|| CodeGenError::LlvmError("malloc returned void".to_string()))?
                    .into_pointer_value();

                let array_type = i64_type.array_type(size as u32);

                // Calculate discriminant - must match variant_to_discriminant in mir/lower.rs
                let discriminant: i64 = variant.chars()
                    .enumerate()
                    .fold(0i64, |acc, (i, c)| acc.wrapping_add((c as i64).wrapping_mul((i + 1) as i64)));

                // Store discriminant at index 0
                let disc_ptr = unsafe {
                    self.builder
                        .build_in_bounds_gep(array_type, enum_ptr, &[
                            i64_type.const_int(0, false),
                            i64_type.const_int(0, false),
                        ], &format!("{}_disc", dest.name))
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                };
                self.builder
                    .build_store(disc_ptr, i64_type.const_int(discriminant as u64, true))
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Store variant arguments
                for (i, arg) in args.iter().enumerate() {
                    let arg_ptr = unsafe {
                        self.builder
                            .build_in_bounds_gep(array_type, enum_ptr, &[
                                i64_type.const_int(0, false),
                                i64_type.const_int((i + 1) as u64, false),
                            ], &format!("{}_a{}", dest.name, i))
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    };

                    let arg_val = self.gen_operand(arg)?;

                    // Convert to i64 if needed (enums store everything as i64)
                    let store_val = if arg_val.is_int_value() {
                        let int_val = arg_val.into_int_value();
                        if int_val.get_type().get_bit_width() < 64 {
                            self.builder
                                .build_int_s_extend(int_val, i64_type, "arg_sext")
                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                .into()
                        } else {
                            arg_val
                        }
                    } else if arg_val.is_pointer_value() {
                        self.builder
                            .build_ptr_to_int(arg_val.into_pointer_value(), i64_type, "arg_ptrtoint")
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                            .into()
                    } else {
                        arg_val
                    };

                    self.builder
                        .build_store(arg_ptr, store_val)
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                }

                // Store the enum pointer to destination
                self.store_to_place(dest, enum_ptr.into())?;
            }

            // v0.70: Thread spawn - Phase 2 implementation with real async threading
            // For simple function call patterns (spawn { func(args) }), we:
            // 1. Generate a wrapper function that unpacks captures and calls the target
            // 2. Package arguments into a captures struct
            // 3. Call bmb_spawn with wrapper function pointer + captures
            MirInst::ThreadSpawn { dest, func, captures } => {
                let i64_type = self.context.i64_type();
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

                // Check if this is a simple function call pattern (func is a real function name)
                // vs fallback pattern (func starts with __spawn_inline_)
                let is_simple_call = !func.starts_with("__spawn_inline_");

                if is_simple_call {
                    // Phase 2: Real async threading for simple function call patterns
                    // Generate wrapper function name
                    let wrapper_name = format!("__spawn_wrapper_{}", func);

                    // Get or create the wrapper function
                    let wrapper_fn = if let Some(existing) = self.module.get_function(&wrapper_name) {
                        existing
                    } else {
                        // Create wrapper function: i64 (ptr captures)
                        let wrapper_type = i64_type.fn_type(&[ptr_type.into()], false);
                        let wrapper = self.module.add_function(&wrapper_name, wrapper_type, None);

                        // Get the target function
                        let target_fn = self.module.get_function(func)
                            .or_else(|| self.functions.get(func).copied())
                            .ok_or_else(|| CodeGenError::LlvmError(
                                format!("Target function '{}' not found for spawn", func)
                            ))?;

                        // Build wrapper body
                        let entry = self.context.append_basic_block(wrapper, "entry");
                        let saved_block = self.builder.get_insert_block();
                        self.builder.position_at_end(entry);

                        // Get captures parameter
                        let captures_param = wrapper.get_nth_param(0)
                            .ok_or_else(|| CodeGenError::LlvmError("Wrapper missing captures param".to_string()))?
                            .into_pointer_value();

                        // Load arguments from captures array
                        let captures_array_type = i64_type.array_type(captures.len().max(1) as u32);
                        let mut args: Vec<inkwell::values::BasicMetadataValueEnum> = Vec::new();
                        for i in 0..captures.len() {
                            let arg_ptr = unsafe {
                                self.builder
                                    .build_in_bounds_gep(captures_array_type, captures_param, &[
                                        i64_type.const_int(0, false),
                                        i64_type.const_int(i as u64, false),
                                    ], &format!("arg_{}_ptr", i))
                                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                            };
                            let arg_val = self.builder
                                .build_load(i64_type, arg_ptr, &format!("arg_{}", i))
                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                            args.push(arg_val.into());
                        }

                        // Call target function
                        let call_result = self.builder
                            .build_call(target_fn, &args, "spawn_call")
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                        // Return result (or 0 if void)
                        let ret_val = call_result.try_as_basic_value().basic()
                            .map(|v| {
                                if v.is_int_value() {
                                    v.into_int_value()
                                } else {
                                    i64_type.const_int(0, false)
                                }
                            })
                            .unwrap_or_else(|| i64_type.const_int(0, false));

                        self.builder.build_return(Some(&ret_val))
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                        // Restore builder position
                        if let Some(block) = saved_block {
                            self.builder.position_at_end(block);
                        }

                        wrapper
                    };

                    // Package arguments into captures array
                    let captures_alloc = if !captures.is_empty() {
                        let captures_array_type = i64_type.array_type(captures.len() as u32);
                        let alloc = self.builder
                            .build_alloca(captures_array_type, &format!("{}_captures", dest.name))
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                        for (i, cap) in captures.iter().enumerate() {
                            let cap_val = self.gen_operand(cap)?;
                            let cap_i64 = if cap_val.is_int_value() {
                                let int_val = cap_val.into_int_value();
                                if int_val.get_type().get_bit_width() < 64 {
                                    self.builder
                                        .build_int_s_extend(int_val, i64_type, "cap_sext")
                                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                } else {
                                    int_val
                                }
                            } else if cap_val.is_pointer_value() {
                                self.builder
                                    .build_ptr_to_int(cap_val.into_pointer_value(), i64_type, "cap_ptrtoint")
                                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                            } else {
                                i64_type.const_int(0, false)
                            };

                            let cap_ptr = unsafe {
                                self.builder
                                    .build_in_bounds_gep(captures_array_type, alloc, &[
                                        i64_type.const_int(0, false),
                                        i64_type.const_int(i as u64, false),
                                    ], &format!("cap_{}_ptr", i))
                                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                            };
                            self.builder.build_store(cap_ptr, cap_i64)
                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                        }
                        alloc
                    } else {
                        ptr_type.const_null()
                    };

                    // Call bmb_spawn(wrapper_fn_ptr, captures_ptr)
                    let spawn_fn = self.functions.get("bmb_spawn")
                        .ok_or_else(|| CodeGenError::LlvmError("bmb_spawn not declared".to_string()))?;

                    let wrapper_ptr = wrapper_fn.as_global_value().as_pointer_value();
                    let handle = self.builder
                        .build_call(*spawn_fn, &[wrapper_ptr.into(), captures_alloc.into()], &format!("{}_handle", dest.name))
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                        .try_as_basic_value()
                        .basic()
                        .ok_or_else(|| CodeGenError::LlvmError("bmb_spawn returned void".to_string()))?;

                    self.store_to_place(dest, handle)?;
                } else {
                    // Fallback: Phase 1 synchronous execution for complex patterns
                    // The first capture contains the already-computed result
                    let result_val = if !captures.is_empty() {
                        let cap_val = self.gen_operand(&captures[0])?;
                        if cap_val.is_int_value() {
                            let int_val = cap_val.into_int_value();
                            if int_val.get_type().get_bit_width() < 64 {
                                self.builder
                                    .build_int_s_extend(int_val, i64_type, "spawn_result_sext")
                                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                    .into()
                            } else {
                                cap_val
                            }
                        } else if cap_val.is_pointer_value() {
                            self.builder
                                .build_ptr_to_int(cap_val.into_pointer_value(), i64_type, "spawn_result_ptrtoint")
                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                                .into()
                        } else {
                            cap_val
                        }
                    } else {
                        i64_type.const_int(0, false).into()
                    };

                    self.store_to_place(dest, result_val)?;
                }
            }

            // v0.70: Thread join - calls bmb_join for real threads, or returns value for fallback
            MirInst::ThreadJoin { dest, handle } => {
                let handle_val = self.gen_operand(handle)?;
                let i64_type = self.context.i64_type();

                // Convert handle to i64 if needed
                let handle_i64 = if handle_val.is_int_value() {
                    handle_val.into_int_value()
                } else if handle_val.is_pointer_value() {
                    self.builder
                        .build_ptr_to_int(handle_val.into_pointer_value(), i64_type, "handle_ptrtoint")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                } else {
                    return Err(CodeGenError::LlvmError(
                        "ThreadJoin handle must be integer or pointer".to_string()
                    ));
                };

                // Call bmb_join(handle) for real threading
                let join_fn = self.functions.get("bmb_join")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_join not declared".to_string()))?;

                let result = self.builder
                    .build_call(*join_fn, &[handle_i64.into()], "join_result")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_join returned void".to_string()))?;

                if let Some(d) = dest {
                    self.store_to_place(d, result)?;
                }
            }

            // v0.72: Atomic operations - using LLVM atomic instructions
            MirInst::AtomicNew { dest, value } => {
                let i64_type = self.context.i64_type();
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

                // Allocate 8 bytes for the atomic value
                let malloc_fn = self.functions.get("malloc")
                    .ok_or_else(|| CodeGenError::LlvmError("malloc not declared".to_string()))?;
                let size = i64_type.const_int(8, false);
                let ptr = self.builder
                    .build_call(*malloc_fn, &[size.into()], "atomic_ptr")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("malloc returned void".to_string()))?
                    .into_pointer_value();

                // Store initial value atomically
                let init_val = self.gen_operand(value)?;
                let init_i64 = init_val.into_int_value();
                let store_inst = self.builder
                    .build_store(ptr, init_i64)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                // v0.72: Set 8-byte alignment for lock-free i64 atomics
                store_inst.set_alignment(8)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                store_inst.set_atomic_ordering(inkwell::AtomicOrdering::SequentiallyConsistent)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Convert pointer to i64 handle
                let handle = self.builder
                    .build_ptr_to_int(ptr, i64_type, "atomic_handle")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.store_to_place(dest, handle.into())?;
            }

            MirInst::AtomicLoad { dest, ptr } => {
                let i64_type = self.context.i64_type();
                let ptr_val = self.gen_operand(ptr)?;
                let ptr_i64 = ptr_val.into_int_value();

                // Convert i64 handle to pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let atomic_ptr = self.builder
                    .build_int_to_ptr(ptr_i64, ptr_type, "atomic_ptr")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Atomic load
                let loaded = self.builder
                    .build_load(i64_type, atomic_ptr, "atomic_load")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                let load_inst = loaded.as_instruction_value().unwrap();
                // v0.72: Set 8-byte alignment for lock-free i64 atomics
                load_inst.set_alignment(8)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                load_inst.set_atomic_ordering(inkwell::AtomicOrdering::SequentiallyConsistent)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.store_to_place(dest, loaded)?;
            }

            MirInst::AtomicStore { ptr, value } => {
                let ptr_val = self.gen_operand(ptr)?;
                let ptr_i64 = ptr_val.into_int_value();
                let store_val = self.gen_operand(value)?;
                let store_i64 = store_val.into_int_value();

                // Convert i64 handle to pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let atomic_ptr = self.builder
                    .build_int_to_ptr(ptr_i64, ptr_type, "atomic_ptr")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Atomic store
                let store_inst = self.builder
                    .build_store(atomic_ptr, store_i64)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                // v0.72: Set 8-byte alignment for lock-free i64 atomics
                store_inst.set_alignment(8)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                store_inst.set_atomic_ordering(inkwell::AtomicOrdering::SequentiallyConsistent)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            MirInst::AtomicFetchAdd { dest, ptr, delta } => {
                let i64_type = self.context.i64_type();
                let ptr_val = self.gen_operand(ptr)?;
                let ptr_i64 = ptr_val.into_int_value();
                let delta_val = self.gen_operand(delta)?;
                let delta_i64 = delta_val.into_int_value();

                // Convert i64 handle to pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let atomic_ptr = self.builder
                    .build_int_to_ptr(ptr_i64, ptr_type, "atomic_ptr")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Atomic fetch-add
                let old_val = self.builder
                    .build_atomicrmw(
                        inkwell::AtomicRMWBinOp::Add,
                        atomic_ptr,
                        delta_i64,
                        inkwell::AtomicOrdering::SequentiallyConsistent,
                    )
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.store_to_place(dest, old_val.into())?;
            }

            MirInst::AtomicFetchSub { dest, ptr, delta } => {
                let i64_type = self.context.i64_type();
                let ptr_val = self.gen_operand(ptr)?;
                let ptr_i64 = ptr_val.into_int_value();
                let delta_val = self.gen_operand(delta)?;
                let delta_i64 = delta_val.into_int_value();

                // Convert i64 handle to pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let atomic_ptr = self.builder
                    .build_int_to_ptr(ptr_i64, ptr_type, "atomic_ptr")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Atomic fetch-sub
                let old_val = self.builder
                    .build_atomicrmw(
                        inkwell::AtomicRMWBinOp::Sub,
                        atomic_ptr,
                        delta_i64,
                        inkwell::AtomicOrdering::SequentiallyConsistent,
                    )
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.store_to_place(dest, old_val.into())?;
            }

            MirInst::AtomicSwap { dest, ptr, new_value } => {
                let i64_type = self.context.i64_type();
                let ptr_val = self.gen_operand(ptr)?;
                let ptr_i64 = ptr_val.into_int_value();
                let new_val = self.gen_operand(new_value)?;
                let new_i64 = new_val.into_int_value();

                // Convert i64 handle to pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let atomic_ptr = self.builder
                    .build_int_to_ptr(ptr_i64, ptr_type, "atomic_ptr")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Atomic swap (exchange)
                let old_val = self.builder
                    .build_atomicrmw(
                        inkwell::AtomicRMWBinOp::Xchg,
                        atomic_ptr,
                        new_i64,
                        inkwell::AtomicOrdering::SequentiallyConsistent,
                    )
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.store_to_place(dest, old_val.into())?;
            }

            MirInst::AtomicCompareExchange { dest, ptr, expected, new_value } => {
                let i64_type = self.context.i64_type();
                let ptr_val = self.gen_operand(ptr)?;
                let ptr_i64 = ptr_val.into_int_value();
                let expected_val = self.gen_operand(expected)?;
                let expected_i64 = expected_val.into_int_value();
                let new_val = self.gen_operand(new_value)?;
                let new_i64 = new_val.into_int_value();

                // Convert i64 handle to pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let atomic_ptr = self.builder
                    .build_int_to_ptr(ptr_i64, ptr_type, "atomic_ptr")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Atomic compare-exchange
                let result = self.builder
                    .build_cmpxchg(
                        atomic_ptr,
                        expected_i64,
                        new_i64,
                        inkwell::AtomicOrdering::SequentiallyConsistent,
                        inkwell::AtomicOrdering::SequentiallyConsistent,
                    )
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Extract the old value (first element of the { i64, i1 } struct)
                let old_val = self.builder
                    .build_extract_value(result, 0, "cmpxchg_old")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.store_to_place(dest, old_val)?;
            }

            // v0.71: Mutex operations
            MirInst::MutexNew { dest, initial_value } => {
                let i64_type = self.context.i64_type();
                let value = self.gen_operand(initial_value)?;
                let value_i64 = if value.is_int_value() {
                    value.into_int_value()
                } else {
                    i64_type.const_int(0, false)
                };

                let mutex_new_fn = self.functions.get("bmb_mutex_new")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_mutex_new not declared".to_string()))?;

                let handle = self.builder
                    .build_call(*mutex_new_fn, &[value_i64.into()], "mutex_handle")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_mutex_new returned void".to_string()))?;

                self.store_to_place(dest, handle)?;
            }

            MirInst::MutexLock { dest, mutex } => {
                let mutex_val = self.gen_operand(mutex)?;
                let mutex_i64 = mutex_val.into_int_value();

                let mutex_lock_fn = self.functions.get("bmb_mutex_lock")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_mutex_lock not declared".to_string()))?;

                let value = self.builder
                    .build_call(*mutex_lock_fn, &[mutex_i64.into()], "mutex_value")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_mutex_lock returned void".to_string()))?;

                self.store_to_place(dest, value)?;
            }

            MirInst::MutexUnlock { mutex, new_value } => {
                let mutex_val = self.gen_operand(mutex)?;
                let mutex_i64 = mutex_val.into_int_value();
                let value = self.gen_operand(new_value)?;
                let value_i64 = value.into_int_value();

                let mutex_unlock_fn = self.functions.get("bmb_mutex_unlock")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_mutex_unlock not declared".to_string()))?;

                self.builder
                    .build_call(*mutex_unlock_fn, &[mutex_i64.into(), value_i64.into()], "")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            MirInst::MutexTryLock { dest, mutex } => {
                let mutex_val = self.gen_operand(mutex)?;
                let mutex_i64 = mutex_val.into_int_value();

                let mutex_try_lock_fn = self.functions.get("bmb_mutex_try_lock")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_mutex_try_lock not declared".to_string()))?;

                let result = self.builder
                    .build_call(*mutex_try_lock_fn, &[mutex_i64.into()], "try_lock_result")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_mutex_try_lock returned void".to_string()))?;

                self.store_to_place(dest, result)?;
            }

            MirInst::MutexFree { mutex } => {
                let mutex_val = self.gen_operand(mutex)?;
                let mutex_i64 = mutex_val.into_int_value();

                let mutex_free_fn = self.functions.get("bmb_mutex_free")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_mutex_free not declared".to_string()))?;

                self.builder
                    .build_call(*mutex_free_fn, &[mutex_i64.into()], "")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            // v0.71: Channel operations
            MirInst::ChannelNew { sender_dest, receiver_dest, capacity } => {
                let i64_type = self.context.i64_type();
                let cap_val = self.gen_operand(capacity)?;
                let cap_i64 = cap_val.into_int_value();

                // Allocate space for sender and receiver handles
                let sender_alloca = self.builder
                    .build_alloca(i64_type, "sender_alloca")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                let receiver_alloca = self.builder
                    .build_alloca(i64_type, "receiver_alloca")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                let channel_new_fn = self.functions.get("bmb_channel_new")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_channel_new not declared".to_string()))?;

                self.builder
                    .build_call(*channel_new_fn, &[cap_i64.into(), sender_alloca.into(), receiver_alloca.into()], "")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Load sender and receiver handles
                let sender = self.builder
                    .build_load(i64_type, sender_alloca, "sender")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                let receiver = self.builder
                    .build_load(i64_type, receiver_alloca, "receiver")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.store_to_place(sender_dest, sender)?;
                self.store_to_place(receiver_dest, receiver)?;
            }

            MirInst::ChannelSend { sender, value } => {
                let sender_val = self.gen_operand(sender)?;
                let sender_i64 = sender_val.into_int_value();
                let val = self.gen_operand(value)?;
                let val_i64 = val.into_int_value();

                let channel_send_fn = self.functions.get("bmb_channel_send")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_channel_send not declared".to_string()))?;

                self.builder
                    .build_call(*channel_send_fn, &[sender_i64.into(), val_i64.into()], "")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            MirInst::ChannelRecv { dest, receiver } => {
                let receiver_val = self.gen_operand(receiver)?;
                let receiver_i64 = receiver_val.into_int_value();

                let channel_recv_fn = self.functions.get("bmb_channel_recv")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_channel_recv not declared".to_string()))?;

                let value = self.builder
                    .build_call(*channel_recv_fn, &[receiver_i64.into()], "recv_value")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_channel_recv returned void".to_string()))?;

                self.store_to_place(dest, value)?;
            }

            MirInst::ChannelTrySend { dest, sender, value } => {
                let sender_val = self.gen_operand(sender)?;
                let sender_i64 = sender_val.into_int_value();
                let val = self.gen_operand(value)?;
                let val_i64 = val.into_int_value();

                let channel_try_send_fn = self.functions.get("bmb_channel_try_send")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_channel_try_send not declared".to_string()))?;

                let success = self.builder
                    .build_call(*channel_try_send_fn, &[sender_i64.into(), val_i64.into()], "try_send_result")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_channel_try_send returned void".to_string()))?;

                self.store_to_place(dest, success)?;
            }

            MirInst::ChannelTryRecv { dest, receiver } => {
                // v0.76: Proper non-blocking try_recv implementation
                let receiver_val = self.gen_operand(receiver)?;
                let receiver_i64 = receiver_val.into_int_value();

                let channel_try_recv_fn = self.functions.get("bmb_channel_try_recv")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_channel_try_recv not declared".to_string()))?;

                let i64_type = self.context.i64_type();
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

                // Allocate stack space for the output value
                let value_alloc = self.builder
                    .build_alloca(i64_type, "try_recv_value_alloc")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Call bmb_channel_try_recv(receiver, &value_out) -> success (1 or 0)
                let success = self.builder
                    .build_call(*channel_try_recv_fn, &[receiver_i64.into(), value_alloc.into()], "try_recv_success")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_channel_try_recv returned void".to_string()))?;

                // Load the received value (valid only if success == 1)
                let received_value = self.builder
                    .build_load(i64_type, value_alloc, "try_recv_loaded")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // If success, return value; otherwise return -1 (sentinel for "empty")
                let success_i64 = success.into_int_value();
                let is_success = self.builder
                    .build_int_compare(inkwell::IntPredicate::NE, success_i64, i64_type.const_zero(), "is_success")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                let sentinel = i64_type.const_int((-1i64) as u64, true);
                let result = self.builder
                    .build_select(is_success, received_value, sentinel.into(), "try_recv_result")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.store_to_place(dest, result)?;
            }

            // v0.77: Receive with timeout
            MirInst::ChannelRecvTimeout { dest, receiver, timeout_ms } => {
                let receiver_val = self.gen_operand(receiver)?;
                let receiver_i64 = receiver_val.into_int_value();
                let timeout_val = self.gen_operand(timeout_ms)?;
                let timeout_i64 = timeout_val.into_int_value();

                let channel_recv_timeout_fn = self.functions.get("bmb_channel_recv_timeout")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_channel_recv_timeout not declared".to_string()))?;

                let i64_type = self.context.i64_type();

                // Allocate stack space for the output value
                let value_alloc = self.builder
                    .build_alloca(i64_type, "recv_timeout_value_alloc")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // Call bmb_channel_recv_timeout(receiver, timeout_ms, &value_out) -> success (1 or 0)
                let success = self.builder
                    .build_call(*channel_recv_timeout_fn, &[receiver_i64.into(), timeout_i64.into(), value_alloc.into()], "recv_timeout_success")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_channel_recv_timeout returned void".to_string()))?;

                // Load the received value (valid only if success == 1)
                let received_value = self.builder
                    .build_load(i64_type, value_alloc, "recv_timeout_loaded")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                // If success, return value; otherwise return -1 (sentinel for "timeout")
                let success_i64 = success.into_int_value();
                let is_success = self.builder
                    .build_int_compare(inkwell::IntPredicate::NE, success_i64, i64_type.const_zero(), "is_timeout_success")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                let sentinel = i64_type.const_int((-1i64) as u64, true);
                let result = self.builder
                    .build_select(is_success, received_value, sentinel.into(), "recv_timeout_result")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.store_to_place(dest, result)?;
            }

            // v0.78: Block on a future
            MirInst::BlockOn { dest, future } => {
                let future_val = self.gen_operand(future)?;
                let future_i64 = future_val.into_int_value();

                let block_on_fn = self.functions.get("bmb_block_on")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_block_on not declared".to_string()))?;

                let result = self.builder
                    .build_call(*block_on_fn, &[future_i64.into()], "block_on_result")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_block_on returned void".to_string()))?;

                self.store_to_place(dest, result)?;
            }

            MirInst::SenderClone { dest, sender } => {
                let sender_val = self.gen_operand(sender)?;
                let sender_i64 = sender_val.into_int_value();

                let sender_clone_fn = self.functions.get("bmb_sender_clone")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_sender_clone not declared".to_string()))?;

                let cloned = self.builder
                    .build_call(*sender_clone_fn, &[sender_i64.into()], "cloned_sender")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_sender_clone returned void".to_string()))?;

                self.store_to_place(dest, cloned)?;
            }

            // v0.74: RwLock instructions
            MirInst::RwLockNew { dest, initial_value } => {
                let init_val = self.gen_operand(initial_value)?;
                let init_i64 = init_val.into_int_value();

                let rwlock_new_fn = self.functions.get("bmb_rwlock_new")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_rwlock_new not declared".to_string()))?;

                let handle = self.builder
                    .build_call(*rwlock_new_fn, &[init_i64.into()], "rwlock_handle")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_rwlock_new returned void".to_string()))?;

                self.store_to_place(dest, handle)?;
            }

            MirInst::RwLockRead { dest, rwlock } => {
                let rwlock_val = self.gen_operand(rwlock)?;
                let rwlock_i64 = rwlock_val.into_int_value();

                let rwlock_read_fn = self.functions.get("bmb_rwlock_read")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_rwlock_read not declared".to_string()))?;

                let value = self.builder
                    .build_call(*rwlock_read_fn, &[rwlock_i64.into()], "rwlock_read_value")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_rwlock_read returned void".to_string()))?;

                self.store_to_place(dest, value)?;
            }

            MirInst::RwLockReadUnlock { rwlock } => {
                let rwlock_val = self.gen_operand(rwlock)?;
                let rwlock_i64 = rwlock_val.into_int_value();

                let rwlock_read_unlock_fn = self.functions.get("bmb_rwlock_read_unlock")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_rwlock_read_unlock not declared".to_string()))?;

                self.builder
                    .build_call(*rwlock_read_unlock_fn, &[rwlock_i64.into()], "")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            MirInst::RwLockWrite { dest, rwlock } => {
                let rwlock_val = self.gen_operand(rwlock)?;
                let rwlock_i64 = rwlock_val.into_int_value();

                let rwlock_write_fn = self.functions.get("bmb_rwlock_write")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_rwlock_write not declared".to_string()))?;

                let value = self.builder
                    .build_call(*rwlock_write_fn, &[rwlock_i64.into()], "rwlock_write_value")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_rwlock_write returned void".to_string()))?;

                self.store_to_place(dest, value)?;
            }

            MirInst::RwLockWriteUnlock { rwlock, value } => {
                let rwlock_val = self.gen_operand(rwlock)?;
                let rwlock_i64 = rwlock_val.into_int_value();
                let new_val = self.gen_operand(value)?;
                let new_i64 = new_val.into_int_value();

                let rwlock_write_unlock_fn = self.functions.get("bmb_rwlock_write_unlock")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_rwlock_write_unlock not declared".to_string()))?;

                self.builder
                    .build_call(*rwlock_write_unlock_fn, &[rwlock_i64.into(), new_i64.into()], "")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            // v0.74: Barrier instructions
            MirInst::BarrierNew { dest, count } => {
                let count_val = self.gen_operand(count)?;
                let count_i64 = count_val.into_int_value();

                let barrier_new_fn = self.functions.get("bmb_barrier_new")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_barrier_new not declared".to_string()))?;

                let handle = self.builder
                    .build_call(*barrier_new_fn, &[count_i64.into()], "barrier_handle")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_barrier_new returned void".to_string()))?;

                self.store_to_place(dest, handle)?;
            }

            MirInst::BarrierWait { dest, barrier } => {
                let barrier_val = self.gen_operand(barrier)?;
                let barrier_i64 = barrier_val.into_int_value();

                let barrier_wait_fn = self.functions.get("bmb_barrier_wait")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_barrier_wait not declared".to_string()))?;

                let is_leader = self.builder
                    .build_call(*barrier_wait_fn, &[barrier_i64.into()], "barrier_is_leader")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_barrier_wait returned void".to_string()))?;

                self.store_to_place(dest, is_leader)?;
            }

            // v0.74: Condvar instructions
            MirInst::CondvarNew { dest } => {
                let condvar_new_fn = self.functions.get("bmb_condvar_new")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_condvar_new not declared".to_string()))?;

                let handle = self.builder
                    .build_call(*condvar_new_fn, &[], "condvar_handle")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_condvar_new returned void".to_string()))?;

                self.store_to_place(dest, handle)?;
            }

            MirInst::CondvarWait { dest, condvar, mutex } => {
                let condvar_val = self.gen_operand(condvar)?;
                let condvar_i64 = condvar_val.into_int_value();
                let mutex_val = self.gen_operand(mutex)?;
                let mutex_i64 = mutex_val.into_int_value();

                let condvar_wait_fn = self.functions.get("bmb_condvar_wait")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_condvar_wait not declared".to_string()))?;

                let value = self.builder
                    .build_call(*condvar_wait_fn, &[condvar_i64.into(), mutex_i64.into()], "condvar_wait_value")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_condvar_wait returned void".to_string()))?;

                self.store_to_place(dest, value)?;
            }

            MirInst::CondvarNotifyOne { condvar } => {
                let condvar_val = self.gen_operand(condvar)?;
                let condvar_i64 = condvar_val.into_int_value();

                let condvar_notify_one_fn = self.functions.get("bmb_condvar_notify_one")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_condvar_notify_one not declared".to_string()))?;

                self.builder
                    .build_call(*condvar_notify_one_fn, &[condvar_i64.into()], "")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            MirInst::CondvarNotifyAll { condvar } => {
                let condvar_val = self.gen_operand(condvar)?;
                let condvar_i64 = condvar_val.into_int_value();

                let condvar_notify_all_fn = self.functions.get("bmb_condvar_notify_all")
                    .ok_or_else(|| CodeGenError::LlvmError("bmb_condvar_notify_all not declared".to_string()))?;

                self.builder
                    .build_call(*condvar_notify_all_fn, &[condvar_i64.into()], "")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            }

            // v0.76: Select instruction
            MirInst::Select { dest, cond_op, cond_lhs, cond_rhs, true_val, false_val } => {
                let lhs_val = self.gen_operand(cond_lhs)?;
                let rhs_val = self.gen_operand(cond_rhs)?;
                let true_val_gen = self.gen_operand(true_val)?;
                let false_val_gen = self.gen_operand(false_val)?;

                // Generate comparison
                let lhs_int = lhs_val.into_int_value();
                let rhs_int = rhs_val.into_int_value();

                let cmp_pred = match cond_op {
                    MirBinOp::Eq => inkwell::IntPredicate::EQ,
                    MirBinOp::Ne => inkwell::IntPredicate::NE,
                    MirBinOp::Lt => inkwell::IntPredicate::SLT,
                    MirBinOp::Le => inkwell::IntPredicate::SLE,
                    MirBinOp::Gt => inkwell::IntPredicate::SGT,
                    MirBinOp::Ge => inkwell::IntPredicate::SGE,
                    _ => return Err(CodeGenError::LlvmError(format!("Unsupported Select condition op: {:?}", cond_op))),
                };

                let cond = self.builder
                    .build_int_compare(cmp_pred, lhs_int, rhs_int, "select_cond")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                let result = self.builder
                    .build_select(cond, true_val_gen, false_val_gen, "select_result")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

                self.store_to_place(dest, result)?;
            }

            // Other instructions not yet supported in inkwell codegen
            _ => {
                return Err(CodeGenError::LlvmError(
                    "Instruction not yet supported in inkwell codegen - use text-based codegen".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Generate code for a terminator
    /// v0.50.67: Now takes &mut self for static string generation
    /// v0.50.80: Added type coercion for Return when value type doesn't match function return type
    /// v0.60.31: Added int->ptr coercion for String-returning functions with early returns
    fn gen_terminator(&mut self, term: &Terminator) -> CodeGenResult<()> {
        match term {
            Terminator::Return(Some(op)) => {
                let value = self.gen_operand(op)?;

                // v0.50.80: Coerce return value type if needed
                // When ConstantPropagationNarrowing narrows a parameter to i32,
                // the returned value might be i32 but function returns i64
                // v0.60.31: Also handle int->ptr coercion for String-returning functions
                let coerced_value = if let Some(ret_type) = self.current_ret_type {
                    if value.is_int_value() && ret_type.is_int_type() {
                        let val_int = value.into_int_value();
                        let ret_int = ret_type.into_int_type();
                        let val_bits = val_int.get_type().get_bit_width();
                        let ret_bits = ret_int.get_bit_width();

                        if val_bits < ret_bits {
                            // Sign-extend smaller value to match return type
                            let extended = self.builder
                                .build_int_s_extend(val_int, ret_int, "ret_sext")
                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                            extended.into()
                        } else if val_bits > ret_bits {
                            // Truncate larger value (should be rare)
                            let truncated = self.builder
                                .build_int_truncate(val_int, ret_int, "ret_trunc")
                                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                            truncated.into()
                        } else {
                            value
                        }
                    } else if value.is_int_value() && ret_type.is_pointer_type() {
                        // v0.60.31: Convert int to ptr for String-returning functions
                        // This happens when TailRecursiveToLoop creates early returns
                        // that bypass the normal inttoptr conversion
                        let val_int = value.into_int_value();
                        let ret_ptr = ret_type.into_pointer_type();
                        let converted = self.builder
                            .build_int_to_ptr(val_int, ret_ptr, "ret_inttoptr")
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                        converted.into()
                    } else if value.is_pointer_value() && ret_type.is_int_type() {
                        // v0.60.31: Convert ptr to int if needed (rare case)
                        let val_ptr = value.into_pointer_value();
                        let ret_int = ret_type.into_int_type();
                        let converted = self.builder
                            .build_ptr_to_int(val_ptr, ret_int, "ret_ptrtoint")
                            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                        converted.into()
                    } else {
                        value
                    }
                } else {
                    value
                };

                self.builder.build_return(Some(&coerced_value))
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
            // v0.51.8: Full implementation for if-else chain optimization
            // v0.60.253: Handle enum discriminants by loading from pointer
            Terminator::Switch { discriminant, cases, default } => {
                let disc_val = self.gen_operand(discriminant)?;

                // Check if the discriminant is an enum type using our tracking set
                let is_enum = if let Operand::Place(p) = discriminant {
                    self.enum_variables.contains(&p.name)
                } else {
                    false
                };

                // If discriminant is a pointer (enum) or enum type passed as i64,
                // load the discriminant word from it
                let disc_int = if disc_val.is_pointer_value() {
                    let i64_type = self.context.i64_type();
                    self.builder
                        .build_load(i64_type, disc_val.into_pointer_value(), "enum_disc")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                        .into_int_value()
                } else if is_enum && disc_val.is_int_value() {
                    // Enum passed as i64 (ptrtoint) - convert back to ptr and load
                    let i64_type = self.context.i64_type();
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let enum_ptr = self.builder
                        .build_int_to_ptr(disc_val.into_int_value(), ptr_type, "enum_inttoptr")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    self.builder
                        .build_load(i64_type, enum_ptr, "enum_disc")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?
                        .into_int_value()
                } else {
                    disc_val.into_int_value()
                };

                let default_bb = self
                    .blocks
                    .get(default)
                    .ok_or_else(|| CodeGenError::UnknownBlock(default.clone()))?;

                // Build cases as (IntValue, BasicBlock) pairs
                // v0.60.262: Use the same type as disc_int for case constants to avoid type mismatch
                let disc_type = disc_int.get_type();
                let mut switch_cases: Vec<(inkwell::values::IntValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
                for (val, label) in cases {
                    let case_bb = self
                        .blocks
                        .get(label)
                        .ok_or_else(|| CodeGenError::UnknownBlock(label.clone()))?;
                    let const_int = disc_type.const_int(*val as u64, true);
                    switch_cases.push((const_int, *case_bb));
                }

                self.builder.build_switch(disc_int, *default_bb, &switch_cases)
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
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
    /// v0.50.67: Now takes &mut self to cache static string literals
    fn gen_constant(&mut self, constant: &Constant) -> BasicValueEnum<'ctx> {
        match constant {
            Constant::Int(n) => self.context.i64_type().const_int(*n as u64, true).into(),
            Constant::Float(f) => self.context.f64_type().const_float(*f).into(),
            Constant::Bool(b) => self
                .context
                .bool_type()
                .const_int(*b as u64, false)
                .into(),
            Constant::String(s) => {
                // v0.50.67: Create static BmbString struct for string literals
                // This eliminates heap allocation overhead that was causing 7x slowdown
                // in syscall_overhead benchmark (10000 iterations = 20000 malloc calls)
                //
                // BmbString struct layout: { char* data, i64 len, i64 cap }

                // Check cache first to reuse identical string literals
                if let Some(cached_ptr) = self.static_strings.get(s) {
                    return (*cached_ptr).into();
                }

                let i64_type = self.context.i64_type();
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

                // Create the BmbString struct type
                let bmb_string_type = self.context.struct_type(
                    &[ptr_type.into(), i64_type.into(), i64_type.into()],
                    false
                );

                // Create global string data
                let str_name = format!("str_data_{}", self.static_string_counter);
                let global_str = self.module.add_global(
                    self.context.i8_type().array_type((s.len() + 1) as u32),
                    None,
                    &str_name
                );

                // Create array constant for string data (including null terminator)
                let mut bytes: Vec<_> = s.bytes().map(|b| self.context.i8_type().const_int(b as u64, false)).collect();
                bytes.push(self.context.i8_type().const_int(0, false)); // null terminator
                let str_array = self.context.i8_type().const_array(&bytes);
                global_str.set_initializer(&str_array);
                global_str.set_constant(true);
                global_str.set_unnamed_addr(true);

                // Create BmbString struct constant
                let struct_name = format!("bmb_str_{}", self.static_string_counter);
                let global_struct = self.module.add_global(bmb_string_type, None, &struct_name);

                let struct_init = bmb_string_type.const_named_struct(&[
                    global_str.as_pointer_value().into(),  // data pointer
                    i64_type.const_int(s.len() as u64, false).into(),  // len
                    i64_type.const_int((s.len() + 1) as u64, false).into(),  // cap
                ]);
                global_struct.set_initializer(&struct_init);
                global_struct.set_constant(true);
                global_struct.set_unnamed_addr(true);

                self.static_string_counter += 1;

                // v0.51.18: Also cache the raw C string pointer for _cstr variants
                self.static_cstrings.insert(s.clone(), global_str.as_pointer_value());

                // Cache and return pointer to the static struct
                let struct_ptr = global_struct.as_pointer_value();
                self.static_strings.insert(s.clone(), struct_ptr);
                struct_ptr.into()
            }
            Constant::Unit => self.context.i8_type().const_int(0, false).into(),
            // v0.95: Char as i32 Unicode code point
            Constant::Char(c) => self.context.i32_type().const_int(*c as u64, false).into(),
        }
    }

    /// v0.51.18: Generate raw C string constant for functions that expect const char*
    /// Returns pointer to null-terminated string data (not BMB String struct)
    fn gen_cstring_constant(&mut self, s: &str) -> PointerValue<'ctx> {
        // Check cache first
        if let Some(cached_ptr) = self.static_cstrings.get(s) {
            return *cached_ptr;
        }

        // Create global string data (this will also populate the cache via gen_constant)
        let _ = self.gen_constant(&Constant::String(s.to_string()));

        // Return the cached raw C string pointer
        *self.static_cstrings.get(s).expect("C string should be cached after gen_constant")
    }

    /// Generate code for an operand
    /// v0.50.67: Now takes &mut self for static string generation
    fn gen_operand(&mut self, op: &Operand) -> CodeGenResult<BasicValueEnum<'ctx>> {
        match op {
            Operand::Constant(c) => Ok(self.gen_constant(c)),
            Operand::Place(p) => self.load_from_place(p),
        }
    }

    /// v0.50.80: Generate type cast instruction
    /// Generates appropriate LLVM cast based on source and destination types
    fn gen_cast(
        &self,
        src_val: BasicValueEnum<'ctx>,
        from_ty: &MirType,
        to_ty: &MirType,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        use MirType::*;

        match (from_ty, to_ty) {
            // Same type - no conversion needed
            (a, b) if a == b => Ok(src_val),

            // Integer widening (sign extend)
            (I32, I64) | (I32, U64) => {
                let int_val = src_val.into_int_value();
                let result = self.builder
                    .build_int_s_extend(int_val, self.context.i64_type(), "sext")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            (U32, I64) | (U32, U64) => {
                let int_val = src_val.into_int_value();
                let result = self.builder
                    .build_int_z_extend(int_val, self.context.i64_type(), "zext")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // Integer narrowing (truncate)
            (I64, I32) | (U64, I32) | (I64, U32) | (U64, U32) => {
                let int_val = src_val.into_int_value();
                let result = self.builder
                    .build_int_truncate(int_val, self.context.i32_type(), "trunc")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // Integer to float
            (I32, F64) | (I64, F64) => {
                let int_val = src_val.into_int_value();
                let result = self.builder
                    .build_signed_int_to_float(int_val, self.context.f64_type(), "sitofp")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            (U32, F64) | (U64, F64) => {
                let int_val = src_val.into_int_value();
                let result = self.builder
                    .build_unsigned_int_to_float(int_val, self.context.f64_type(), "uitofp")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // Float to integer
            (F64, I32) | (F64, I64) => {
                let float_val = src_val.into_float_value();
                let target_ty = if matches!(to_ty, I32) {
                    self.context.i32_type()
                } else {
                    self.context.i64_type()
                };
                let result = self.builder
                    .build_float_to_signed_int(float_val, target_ty, "fptosi")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            (F64, U32) | (F64, U64) => {
                let float_val = src_val.into_float_value();
                let target_ty = if matches!(to_ty, U32) {
                    self.context.i32_type()
                } else {
                    self.context.i64_type()
                };
                let result = self.builder
                    .build_float_to_unsigned_int(float_val, target_ty, "fptoui")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // Bool to integer
            (Bool, I32 | I64 | U32 | U64) => {
                let bool_val = src_val.into_int_value();
                let target_ty = self.mir_type_to_llvm(to_ty);
                let result = self.builder
                    .build_int_z_extend(bool_val, target_ty.into_int_type(), "bool_to_int")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // v0.60.1: Integer to pointer (inttoptr)
            // Used for `0 as *T` null pointer patterns and raw memory address casts
            // v0.60.26: If src is already a pointer (e.g., from malloc), skip inttoptr
            (I64, _) | (U64, _) if to_ty.is_pointer_type() => {
                // v0.60.26: Check if src is actually a pointer (optimization from improved Call handling)
                // MIR may say from_ty=I64, but the actual value is ptr (from malloc/calloc keeping ptr)
                if src_val.is_pointer_value() {
                    // Already a pointer, no conversion needed (ptr-to-ptr is identity)
                    return Ok(src_val);
                }
                let int_val = src_val.into_int_value();
                let ptr_type = self.mir_type_to_llvm(to_ty).into_pointer_type();
                let result = self
                    .builder
                    .build_int_to_ptr(int_val, ptr_type, "inttoptr")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // v0.60.1: Pointer to integer (ptrtoint)
            // Used for `ptr as i64` patterns for pointer arithmetic or passing to functions
            // v0.60.26: Also handle case where src is ptr but MIR says I64 (from improved Call handling)
            (_, I64) | (_, U64) if from_ty.is_pointer_type() || src_val.is_pointer_value() => {
                let ptr_val = src_val.into_pointer_value();
                let int_type = self.mir_type_to_llvm(to_ty).into_int_type();
                let result = self
                    .builder
                    .build_ptr_to_int(ptr_val, int_type, "ptrtoint")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }

            // v0.60.1: Pointer to pointer (bitcast in opaque pointer era is identity)
            (_, _) if from_ty.is_pointer_type() && to_ty.is_pointer_type() => {
                // With opaque pointers (LLVM 15+), ptr to ptr cast is identity
                Ok(src_val)
            }

            // v0.60.23: Array to pointer (array decay)
            // Used for `[T; N] as *T` patterns - the array pointer becomes an element pointer
            // The stack array's alloca pointer is the same as a pointer to its first element
            (Array { element_type, .. }, Ptr(target_elem)) if element_type.as_ref() == target_elem.as_ref() => {
                // Array decay: array pointer -> element pointer
                // With LLVM opaque pointers, the pointer value itself doesn't change
                // we just need to ensure we have a pointer value
                if src_val.is_pointer_value() {
                    Ok(src_val)
                } else {
                    // Should not happen normally since arrays are stack-allocated
                    Err(CodeGenError::LlvmError(format!(
                        "Array decay expected pointer value, got {:?}", src_val
                    )))
                }
            }

            // v0.60.254: Enum to integer (ptrtoint)
            // Enums are heap-allocated and passed as pointers, so casting to i64 is ptrtoint
            (Enum { .. }, I64) | (Enum { .. }, U64) => {
                let int_type = self.mir_type_to_llvm(to_ty).into_int_type();
                if src_val.is_pointer_value() {
                    let result = self
                        .builder
                        .build_ptr_to_int(src_val.into_pointer_value(), int_type, "enum_ptrtoint")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else if src_val.is_int_value() {
                    // Enum already passed as i64 (from another function), no conversion needed
                    Ok(src_val)
                } else {
                    Err(CodeGenError::LlvmError(format!(
                        "Enum cast expected pointer or int value, got {:?}", src_val
                    )))
                }
            }

            // Unsupported cast
            _ => Err(CodeGenError::LlvmError(format!(
                "Unsupported cast from {:?} to {:?}",
                from_ty, to_ty
            ))),
        }
    }

    /// v0.50.80: Coerce integer types in binary operations
    ///
    /// When ConstantPropagationNarrowing narrows a parameter from i64 to i32,
    /// operations involving that parameter and i64 constants would have a type
    /// mismatch.
    ///
    /// Strategy: Truncate to the SMALLER type to preserve 32-bit operations.
    /// This is safe because ConstantPropagationNarrowing only narrows parameters
    /// when all values involved fit in i32.
    ///
    /// Example for fibonacci(n):
    /// v0.60.10: Type coercion for mixed-width integer operations
    /// Smart coercion: truncate small constants to match i32 operands,
    /// but extend i32 to i64 when the larger value might be a pointer
    fn coerce_int_types(
        &self,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> CodeGenResult<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>)> {
        // Only coerce if both are integer values (not pointers, floats, etc.)
        if !lhs.is_int_value() || !rhs.is_int_value() {
            return Ok((lhs, rhs));
        }

        let lhs_int = lhs.into_int_value();
        let rhs_int = rhs.into_int_value();

        let lhs_bits = lhs_int.get_type().get_bit_width();
        let rhs_bits = rhs_int.get_type().get_bit_width();

        if lhs_bits == rhs_bits {
            // Same width, no coercion needed
            Ok((lhs_int.into(), rhs_int.into()))
        } else if lhs_bits < rhs_bits {
            // lhs is smaller (e.g., i32), rhs is larger (e.g., i64)
            // Check if rhs is a small constant that can be truncated
            if let Some(const_val) = rhs_int.get_sign_extended_constant() {
                if const_val >= i32::MIN as i64 && const_val <= i32::MAX as i64 {
                    // Safe to truncate the constant
                    let truncated = self.builder
                        .build_int_truncate(rhs_int, lhs_int.get_type(), "trunc_const")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    return Ok((lhs_int.into(), truncated.into()));
                }
            }
            // rhs is not a small constant, extend lhs to match rhs
            let extended = self.builder
                .build_int_s_extend(lhs_int, rhs_int.get_type(), "sext_lhs")
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            Ok((extended.into(), rhs_int.into()))
        } else {
            // lhs is larger (e.g., i64), rhs is smaller (e.g., i32)
            // Check if lhs is a small constant that can be truncated
            if let Some(const_val) = lhs_int.get_sign_extended_constant() {
                if const_val >= i32::MIN as i64 && const_val <= i32::MAX as i64 {
                    // Safe to truncate the constant
                    let truncated = self.builder
                        .build_int_truncate(lhs_int, rhs_int.get_type(), "trunc_const")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    return Ok((truncated.into(), rhs_int.into()));
                }
            }
            // lhs is not a small constant, extend rhs to match lhs
            let extended = self.builder
                .build_int_s_extend(rhs_int, lhs_int.get_type(), "sext_rhs")
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            Ok((lhs_int.into(), extended.into()))
        }
    }

    /// Load a value from a place
    /// v0.50.52: First checks SSA values, then falls back to memory load
    fn load_from_place(&self, place: &Place) -> CodeGenResult<BasicValueEnum<'ctx>> {
        // v0.50.52: First check if this is an SSA value (temporary)
        if let Some(value) = self.ssa_values.get(&place.name) {
            return Ok(*value);
        }

        // Fall back to memory load for params/locals/PHI destinations
        let (ptr, pointee_type) = self
            .variables
            .get(&place.name)
            .ok_or_else(|| CodeGenError::UnknownVariable(place.name.clone()))?;

        // v0.60.25: Array variables store the alloca pointer directly
        // The alloca address IS the array data - don't load from it
        // Return the pointer value directly as ptr type
        if self.array_variables.contains(&place.name) {
            return Ok((*ptr).into());
        }

        self.builder
            .build_load(*pointee_type, *ptr, &place.name)
            .map_err(|e| CodeGenError::LlvmError(e.to_string()))
    }

    /// Store a value to a place
    /// v0.50.52: Uses SSA values for temporaries that start with _t and aren't already allocated
    /// v0.51.9: Also uses SSA values for SSA-eligible locals (not pre-allocated)
    fn store_to_place(&mut self, place: &Place, value: BasicValueEnum<'ctx>) -> CodeGenResult<()> {
        // If it's an existing memory variable (param, local, PHI dest), store to memory
        if let Some((ptr, pointee_type)) = self.variables.get(&place.name) {
            // v0.60.10: Coerce value type to match destination pointee type
            // This handles narrowed locals receiving i64 constants
            let coerced_value = self.coerce_value_to_type(value, *pointee_type)?;
            self.builder.build_store(*ptr, coerced_value)
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
        } else {
            // v0.51.9: For all non-allocated places, use SSA values
            // This includes:
            // - _t* temporaries (v0.50.52)
            // - SSA-eligible locals that were skipped in allocation (v0.51.9)
            // This avoids alloca/store/load overhead - values stay in registers
            self.ssa_values.insert(place.name.clone(), value);
        }
        Ok(())
    }

    /// v0.60.10: Coerce a value to match the expected type
    /// Handles narrowing (i64 -> i32) and widening (i32 -> i64) for integer stores
    fn coerce_value_to_type(
        &self,
        value: BasicValueEnum<'ctx>,
        expected_type: inkwell::types::BasicTypeEnum<'ctx>,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        // Only coerce integer types
        if !value.is_int_value() || !expected_type.is_int_type() {
            return Ok(value);
        }

        let value_int = value.into_int_value();
        let expected_int_type = expected_type.into_int_type();

        let value_bits = value_int.get_type().get_bit_width();
        let expected_bits = expected_int_type.get_bit_width();

        if value_bits == expected_bits {
            Ok(value_int.into())
        } else if value_bits > expected_bits {
            // Truncate (e.g., i64 -> i32)
            let truncated = self.builder
                .build_int_truncate(value_int, expected_int_type, "trunc_store")
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            Ok(truncated.into())
        } else {
            // Sign-extend (e.g., i32 -> i64)
            let extended = self.builder
                .build_int_s_extend(value_int, expected_int_type, "sext_store")
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            Ok(extended.into())
        }
    }

    /// Generate a binary operation
    /// v0.50.80: Added type coercion for mixed i32/i64 operations
    fn gen_binop(
        &self,
        op: MirBinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        self.gen_binop_with_string_hint(op, lhs, rhs, false)
    }

    /// Generate a binary operation with string comparison hint
    /// v0.60.33: Added is_string_comparison to distinguish String from typed pointers
    fn gen_binop_with_string_hint(
        &self,
        op: MirBinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
        is_string_comparison: bool,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        // v0.50.80: Type coercion for mixed integer operations
        // When ConstantPropagationNarrowing narrows a parameter to i32,
        // constants may still be i64. Extend the smaller type to match.
        let (lhs, rhs) = self.coerce_int_types(lhs, rhs)?;

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
            // v0.60.124: Use fast-math flags that enable FMA but avoid reassociation
            // AllowReassoc can transform (1.0/x)*y to y/x, which hurts ILP (instruction-level parallelism)
            // because the division must wait for the load of y. Without reassociation, LLVM keeps
            // the 1.0/x separate, allowing the load of y to happen in parallel with the division.
            MirBinOp::FAdd => {
                let result = self.builder
                    .build_float_add(lhs.into_float_value(), rhs.into_float_value(), "fadd")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                if self.fast_math {
                    let inst: InstructionValue = result.as_instruction_value()
                        .ok_or_else(|| CodeGenError::LlvmError("fadd should be an instruction".to_string()))?;
                    // Use flags that enable FMA (AllowContract) but not reassociation
                    let flags = FastMathFlags::AllowContract
                        | FastMathFlags::NoNaNs
                        | FastMathFlags::NoInfs
                        | FastMathFlags::NoSignedZeros
                        | FastMathFlags::AllowReciprocal
                        | FastMathFlags::ApproxFunc;
                    let _ = inst.set_fast_math_flags(flags);
                }
                Ok(result.into())
            }
            MirBinOp::FSub => {
                let result = self.builder
                    .build_float_sub(lhs.into_float_value(), rhs.into_float_value(), "fsub")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                if self.fast_math {
                    let inst: InstructionValue = result.as_instruction_value()
                        .ok_or_else(|| CodeGenError::LlvmError("fsub should be an instruction".to_string()))?;
                    let flags = FastMathFlags::AllowContract
                        | FastMathFlags::NoNaNs
                        | FastMathFlags::NoInfs
                        | FastMathFlags::NoSignedZeros
                        | FastMathFlags::AllowReciprocal
                        | FastMathFlags::ApproxFunc;
                    let _ = inst.set_fast_math_flags(flags);
                }
                Ok(result.into())
            }
            MirBinOp::FMul => {
                let result = self.builder
                    .build_float_mul(lhs.into_float_value(), rhs.into_float_value(), "fmul")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                if self.fast_math {
                    let inst: InstructionValue = result.as_instruction_value()
                        .ok_or_else(|| CodeGenError::LlvmError("fmul should be an instruction".to_string()))?;
                    let flags = FastMathFlags::AllowContract
                        | FastMathFlags::NoNaNs
                        | FastMathFlags::NoInfs
                        | FastMathFlags::NoSignedZeros
                        | FastMathFlags::AllowReciprocal
                        | FastMathFlags::ApproxFunc;
                    let _ = inst.set_fast_math_flags(flags);
                }
                Ok(result.into())
            }
            MirBinOp::FDiv => {
                let result = self.builder
                    .build_float_div(lhs.into_float_value(), rhs.into_float_value(), "fdiv")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                if self.fast_math {
                    let inst: InstructionValue = result.as_instruction_value()
                        .ok_or_else(|| CodeGenError::LlvmError("fdiv should be an instruction".to_string()))?;
                    let flags = FastMathFlags::AllowContract
                        | FastMathFlags::NoNaNs
                        | FastMathFlags::NoInfs
                        | FastMathFlags::NoSignedZeros
                        | FastMathFlags::AllowReciprocal
                        | FastMathFlags::ApproxFunc;
                    let _ = inst.set_fast_math_flags(flags);
                }
                Ok(result.into())
            }

            // Integer/String/Pointer comparison
            MirBinOp::Eq => {
                // v0.60.33: Only use string_eq for actual String comparisons
                // Typed pointers (*T) should use direct pointer comparison
                if is_string_comparison && (lhs.is_pointer_value() || rhs.is_pointer_value()) {
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
                } else if lhs.is_pointer_value() && rhs.is_pointer_value() {
                    // v0.60.33: Direct pointer comparison for typed pointers (*T)
                    let result = self.builder
                        .build_int_compare(IntPredicate::EQ,
                            self.builder.build_ptr_to_int(lhs.into_pointer_value(), self.context.i64_type(), "lhs_ptr").map_err(|e| CodeGenError::LlvmError(e.to_string()))?,
                            self.builder.build_ptr_to_int(rhs.into_pointer_value(), self.context.i64_type(), "rhs_ptr").map_err(|e| CodeGenError::LlvmError(e.to_string()))?,
                            "ptr_eq")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else if lhs.is_pointer_value() && rhs.is_int_value() {
                    // v0.60.39: Pointer-to-int comparison (e.g., ptr == null where null is 0)
                    let result = self.builder
                        .build_int_compare(IntPredicate::EQ,
                            self.builder.build_ptr_to_int(lhs.into_pointer_value(), self.context.i64_type(), "lhs_ptr").map_err(|e| CodeGenError::LlvmError(e.to_string()))?,
                            rhs.into_int_value(),
                            "ptr_null_eq")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else if lhs.is_int_value() && rhs.is_pointer_value() {
                    // v0.60.39: Int-to-pointer comparison (e.g., null == ptr where null is 0)
                    let result = self.builder
                        .build_int_compare(IntPredicate::EQ,
                            lhs.into_int_value(),
                            self.builder.build_ptr_to_int(rhs.into_pointer_value(), self.context.i64_type(), "rhs_ptr").map_err(|e| CodeGenError::LlvmError(e.to_string()))?,
                            "null_ptr_eq")
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
                // v0.60.33: Only use string_eq for actual String comparisons
                // Typed pointers (*T) should use direct pointer comparison
                if is_string_comparison && (lhs.is_pointer_value() || rhs.is_pointer_value()) {
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
                } else if lhs.is_pointer_value() && rhs.is_pointer_value() {
                    // v0.60.33: Direct pointer comparison for typed pointers (*T)
                    let result = self.builder
                        .build_int_compare(IntPredicate::NE,
                            self.builder.build_ptr_to_int(lhs.into_pointer_value(), self.context.i64_type(), "lhs_ptr").map_err(|e| CodeGenError::LlvmError(e.to_string()))?,
                            self.builder.build_ptr_to_int(rhs.into_pointer_value(), self.context.i64_type(), "rhs_ptr").map_err(|e| CodeGenError::LlvmError(e.to_string()))?,
                            "ptr_ne")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else if lhs.is_pointer_value() && rhs.is_int_value() {
                    // v0.60.39: Pointer-to-int comparison (e.g., ptr != null where null is 0)
                    let result = self.builder
                        .build_int_compare(IntPredicate::NE,
                            self.builder.build_ptr_to_int(lhs.into_pointer_value(), self.context.i64_type(), "lhs_ptr").map_err(|e| CodeGenError::LlvmError(e.to_string()))?,
                            rhs.into_int_value(),
                            "ptr_null_ne")
                        .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                    Ok(result.into())
                } else if lhs.is_int_value() && rhs.is_pointer_value() {
                    // v0.60.39: Int-to-pointer comparison (e.g., null != ptr where null is 0)
                    let result = self.builder
                        .build_int_compare(IntPredicate::NE,
                            lhs.into_int_value(),
                            self.builder.build_ptr_to_int(rhs.into_pointer_value(), self.context.i64_type(), "rhs_ptr").map_err(|e| CodeGenError::LlvmError(e.to_string()))?,
                            "null_ptr_ne")
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
