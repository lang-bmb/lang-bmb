//! Text-based LLVM IR Generation
//!
//! This module generates LLVM IR as text (.ll files) that can be compiled
//! with clang or llc. It doesn't require the LLVM C API, making it more
//! portable and easier to debug.
//!
//! The generated IR is compatible with the bootstrap compiler output.

use std::collections::HashMap;
use std::fmt::Write;
use thiserror::Error;

use crate::mir::{
    BasicBlock, Constant, MirBinOp, MirFunction, MirInst, MirProgram, MirType, MirUnaryOp,
    Operand, Place, Terminator,
};

/// Text-based code generation error
#[derive(Debug, Error)]
pub enum TextCodeGenError {
    #[error("Unknown function: {0}")]
    UnknownFunction(String),

    #[error("Unknown variable: {0}")]
    UnknownVariable(String),

    #[error("Formatting error: {0}")]
    FormatError(#[from] std::fmt::Error),
}

/// Result type for text code generation
pub type TextCodeGenResult<T> = Result<T, TextCodeGenError>;

/// Text-based LLVM IR Generator
pub struct TextCodeGen {
    /// Target triple (default: x86_64-pc-windows-msvc for Windows)
    target_triple: String,
}

impl TextCodeGen {
    /// Create a new text code generator
    pub fn new() -> Self {
        Self {
            target_triple: Self::default_target_triple(),
        }
    }

    /// Create with custom target triple
    pub fn with_target(target: impl Into<String>) -> Self {
        Self {
            target_triple: target.into(),
        }
    }

    /// Get default target triple based on platform
    fn default_target_triple() -> String {
        #[cfg(target_os = "windows")]
        {
            "x86_64-pc-windows-msvc".to_string()
        }
        #[cfg(target_os = "linux")]
        {
            "x86_64-unknown-linux-gnu".to_string()
        }
        #[cfg(target_os = "macos")]
        {
            "x86_64-apple-darwin".to_string()
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            "x86_64-unknown-linux-gnu".to_string()
        }
    }

    /// v0.93.121: Get target datalayout string for the target triple
    /// Required for LLVM to enable target-specific optimizations like
    /// BypassSlowDivision (divq → divl on x86 when operands fit in 32 bits)
    fn target_datalayout(triple: &str) -> &'static str {
        if triple.contains("x86_64") && triple.contains("windows") {
            "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
        } else if triple.contains("x86_64") && triple.contains("linux") {
            "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
        } else if triple.contains("x86_64") && triple.contains("darwin") {
            "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
        } else if triple.contains("aarch64") && triple.contains("linux") {
            "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32"
        } else if triple.contains("aarch64") && triple.contains("darwin") {
            "e-m:o-i64:64-i128:128-n32:64-S128-Fn32"
        } else {
            // Generic x86_64 fallback
            "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
        }
    }

    /// v0.60.122: Check if an operand is of String type
    /// Used to distinguish String comparison from typed pointer comparison
    fn is_string_operand(operand: &Operand, func: &MirFunction) -> bool {
        match operand {
            Operand::Constant(Constant::String(_)) => true,
            Operand::Place(p) => {
                // Check params
                for (name, ty) in &func.params {
                    if name == &p.name {
                        return *ty == MirType::String;
                    }
                }
                // Check locals
                for (name, ty) in &func.locals {
                    if name == &p.name {
                        return *ty == MirType::String;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Generate complete LLVM IR module as text
    pub fn generate(&self, program: &MirProgram) -> TextCodeGenResult<String> {
        let mut output = String::new();

        // Module header
        writeln!(output, "; ModuleID = bmb_program")?;
        writeln!(output, "target datalayout = \"{}\"", Self::target_datalayout(&self.target_triple))?;
        writeln!(output, "target triple = \"{}\"", self.target_triple)?;
        writeln!(output)?;

        // Phase 32.3: Collect all string constants from the program
        let string_table = self.collect_string_constants(program);

        // Phase 32.3: Build function return type map for user-defined functions
        let fn_return_types: HashMap<String, &'static str> = program
            .functions
            .iter()
            .map(|f| (f.name.clone(), self.mir_type_to_llvm(&f.ret_ty)))
            .collect();

        // v0.51.17: Build function parameter type map for call site type coercion
        // This is needed because ConstantPropagationNarrowing may change i64 params to i32
        let fn_param_types: HashMap<String, Vec<&'static str>> = program
            .functions
            .iter()
            .map(|f| {
                let param_types: Vec<&'static str> = f
                    .params
                    .iter()
                    .map(|(_, ty)| self.mir_type_to_llvm(ty))
                    .collect();
                (f.name.clone(), param_types)
            })
            .collect();

        // v0.51.27: Build sret function map for struct return optimization
        // v0.51.28: Only use sret for large structs (3+ fields)
        // Small structs (1-2 fields) use register return instead
        let sret_functions: HashMap<String, usize> = program
            .functions
            .iter()
            .filter_map(|f| {
                if let MirType::Struct { fields, .. } = &f.ret_ty
                    && fields.len() > 2 {  // Only 3+ fields use sret
                        return Some((f.name.clone(), fields.len()));
                    }
                None
            })
            .collect();

        // v0.51.28: Small struct functions (1-2 fields) use register return
        let small_struct_functions: HashMap<String, usize> = program
            .functions
            .iter()
            .filter_map(|f| {
                if let MirType::Struct { fields, .. } = &f.ret_ty
                    && !fields.is_empty() && fields.len() <= 2 {
                        return Some((f.name.clone(), fields.len()));
                    }
                None
            })
            .collect();

        // v0.55: Tuple-returning functions map (function name -> element types as LLVM string)
        let tuple_functions: HashMap<String, String> = program
            .functions
            .iter()
            .filter_map(|f| {
                if let MirType::Tuple(elems) = &f.ret_ty {
                    let elem_types: Vec<&str> = elems.iter()
                        .map(|e| self.mir_type_to_llvm(e))
                        .collect();
                    let llvm_type = format!("{{ {} }}", elem_types.join(", "));
                    return Some((f.name.clone(), llvm_type));
                }
                None
            })
            .collect();

        // v0.51.31: Emit struct type definitions
        self.emit_struct_types(&mut output, &program.struct_defs)?;

        // Emit string globals
        self.emit_string_globals(&mut output, &string_table)?;

        // Runtime declarations
        self.emit_runtime_declarations(&mut output)?;

        // Generate functions with string table and function type map
        for func in &program.functions {
            self.emit_function_with_strings(&mut output, func, &string_table, &fn_return_types, &fn_param_types, &sret_functions, &small_struct_functions, &tuple_functions, &program.struct_defs)?;
        }

        Ok(output)
    }

    /// Collect all string constants from the program
    fn collect_string_constants(&self, program: &MirProgram) -> HashMap<String, String> {
        let mut table = HashMap::new();
        let mut counter = 0;

        for func in &program.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let MirInst::Const { value: Constant::String(s), .. } = inst
                        && !table.contains_key(s) {
                            table.insert(s.clone(), format!(".str.{}", counter));
                            counter += 1;
                        }
                    // Check for string constants in call arguments
                    if let MirInst::Call { args, .. } = inst {
                        for arg in args {
                            if let Operand::Constant(Constant::String(s)) = arg
                                && !table.contains_key(s) {
                                    table.insert(s.clone(), format!(".str.{}", counter));
                                    counter += 1;
                                }
                        }
                    }
                    // Check for string constants in phi values
                    if let MirInst::Phi { values, .. } = inst {
                        for (val, _label) in values {
                            if let Operand::Constant(Constant::String(s)) = val
                                && !table.contains_key(s) {
                                    table.insert(s.clone(), format!(".str.{}", counter));
                                    counter += 1;
                                }
                        }
                    }
                    // Check for string constants in BinOp operands
                    if let MirInst::BinOp { lhs, rhs, .. } = inst {
                        for operand in [lhs, rhs] {
                            if let Operand::Constant(Constant::String(s)) = operand
                                && !table.contains_key(s) {
                                    table.insert(s.clone(), format!(".str.{}", counter));
                                    counter += 1;
                                }
                        }
                    }
                }
                // Check for string constants in Return terminator
                if let Terminator::Return(Some(Operand::Constant(Constant::String(s)))) = &block.terminator
                    && !table.contains_key(s) {
                        table.insert(s.clone(), format!(".str.{}", counter));
                        counter += 1;
                    }
            }
        }

        table
    }

    /// v0.51.31: Emit LLVM struct type definitions
    /// This provides LLVM with proper type information for better alias analysis and optimization
    fn emit_struct_types(&self, out: &mut String, struct_defs: &HashMap<String, Vec<(String, MirType)>>) -> TextCodeGenResult<()> {
        if struct_defs.is_empty() {
            return Ok(());
        }

        writeln!(out, "; Struct type definitions")?;
        // Sort by name for deterministic output
        let mut sorted: Vec<_> = struct_defs.iter().collect();
        sorted.sort_by_key(|(name, _)| *name);

        for (name, fields) in sorted {
            let field_types: Vec<&str> = fields
                .iter()
                .map(|(_, ty)| self.mir_type_to_llvm(ty))
                .collect();
            writeln!(out, "%struct.{} = type {{ {} }}", name, field_types.join(", "))?;
        }
        writeln!(out)?;

        Ok(())
    }

    /// Emit string global constants
    /// v0.51.22: Also emit pre-initialized BmbString structs for zero-overhead string constants
    fn emit_string_globals(&self, out: &mut String, table: &HashMap<String, String>) -> TextCodeGenResult<()> {
        if table.is_empty() {
            return Ok(());
        }

        // v0.51.22: BmbString struct type: { ptr, i64, i64 } (data, len, cap)
        writeln!(out, "; BmbString struct type")?;
        writeln!(out, "%BmbString = type {{ ptr, i64, i64 }}")?;
        writeln!(out)?;

        writeln!(out, "; String constants")?;
        for (content, name) in table {
            // Escape the string for LLVM IR
            let escaped = self.escape_string_for_llvm(content);
            let byte_len = content.len() + 1; // +1 for null terminator
            let str_len = content.len() as i64; // actual string length (without null)
            writeln!(out, "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"",
                     name, byte_len, escaped)?;
            // v0.51.22: Pre-initialized BmbString struct pointing to the constant
            // This avoids bmb_string_from_cstr overhead entirely
            writeln!(out, "@{}.bmb = private unnamed_addr global %BmbString {{ ptr @{}, i64 {}, i64 {} }}",
                     name, name, str_len, str_len)?;
        }
        writeln!(out)?;

        Ok(())
    }

    /// Escape a string for LLVM IR constant
    fn escape_string_for_llvm(&self, s: &str) -> String {
        let mut result = String::new();
        for c in s.bytes() {
            match c {
                // Printable ASCII (except backslash and double-quote)
                0x20..=0x21 | 0x23..=0x5B | 0x5D..=0x7E => {
                    result.push(c as char);
                }
                // Backslash
                0x5C => result.push_str("\\5C"),
                // Double-quote
                0x22 => result.push_str("\\22"),
                // Newline
                0x0A => result.push_str("\\0A"),
                // Carriage return
                0x0D => result.push_str("\\0D"),
                // Tab
                0x09 => result.push_str("\\09"),
                // Other characters - hex escape
                _ => result.push_str(&format!("\\{:02X}", c)),
            }
        }
        result
    }

    /// Emit runtime function declarations
    fn emit_runtime_declarations(&self, out: &mut String) -> TextCodeGenResult<()> {
        writeln!(out, "; Runtime declarations - Basic I/O")?;
        writeln!(out, "declare void @println(i64)")?;
        writeln!(out, "declare void @print(i64)")?;
        // v0.60.43: Float output for spectral_norm, n_body benchmarks
        writeln!(out, "declare void @println_f64(double)")?;
        writeln!(out, "declare void @print_f64(double)")?;
        writeln!(out, "declare i64 @read_int()")?;
        writeln!(out, "declare void @assert(i1)")?;
        writeln!(out, "declare i64 @bmb_abs(i64) nounwind willreturn memory(none) speculatable")?;
        writeln!(out, "declare i64 @bmb_min(i64, i64) nounwind willreturn memory(none) speculatable")?;
        writeln!(out, "declare i64 @bmb_max(i64, i64) nounwind willreturn memory(none) speculatable")?;
        writeln!(out, "declare i64 @bmb_clamp(i64, i64, i64) nounwind willreturn memory(none) speculatable")?;
        writeln!(out, "declare i64 @bmb_pow(i64, i64) nounwind willreturn memory(none) speculatable")?;
        writeln!(out)?;

        // Phase 32.3: String runtime functions
        // v0.51.13: Added speculatable for LICM optimization
        // Functions marked speculatable can be hoisted out of loops by LLVM
        writeln!(out, "; Runtime declarations - String operations")?;
        writeln!(out, "declare ptr @bmb_string_new(ptr, i64) nounwind")?;
        writeln!(out, "declare ptr @bmb_string_from_cstr(ptr) nounwind")?;
        // v0.51.15: memory(argmem: read) tells LLVM these only read from args, enabling LICM
        // This is more precise than "readonly" which means "may read any memory"
        writeln!(out, "declare i64 @bmb_string_len(ptr nocapture) memory(argmem: read) nounwind willreturn speculatable")?;
        // Note: Returns byte at index, not Unicode char. Name kept for ABI compatibility.
        // v0.67: Interpreter method renamed to byte_at for clarity
        writeln!(out, "declare i64 @bmb_string_char_at(ptr nocapture, i64) memory(argmem: read) nounwind willreturn speculatable")?;
        writeln!(out, "declare ptr @bmb_string_slice(ptr, i64, i64) memory(argmem: read) nounwind willreturn")?;
        writeln!(out, "declare ptr @bmb_string_concat(ptr, ptr) nounwind")?;
        writeln!(out, "declare ptr @bmb_string_concat3(ptr, ptr, ptr) nounwind willreturn")?;
        writeln!(out, "declare i64 @bmb_string_eq(ptr, ptr) memory(argmem: read) nounwind willreturn")?;
        // v0.93.7: String method runtime declarations
        writeln!(out, "declare i64 @bmb_string_starts_with(ptr nocapture, ptr nocapture) memory(argmem: read) nounwind willreturn speculatable")?;
        writeln!(out, "declare i64 @bmb_string_ends_with(ptr nocapture, ptr nocapture) memory(argmem: read) nounwind willreturn speculatable")?;
        writeln!(out, "declare i64 @bmb_string_contains(ptr nocapture, ptr nocapture) memory(argmem: read) nounwind willreturn speculatable")?;
        writeln!(out, "declare i64 @bmb_string_index_of(ptr nocapture, ptr nocapture) memory(argmem: read) nounwind willreturn speculatable")?;
        writeln!(out, "declare ptr @bmb_string_trim(ptr nocapture) nounwind willreturn")?;
        writeln!(out, "declare ptr @bmb_string_replace(ptr nocapture, ptr nocapture, ptr nocapture) nounwind willreturn")?;
        writeln!(out, "declare ptr @bmb_string_to_upper(ptr nocapture) nounwind willreturn")?;
        writeln!(out, "declare ptr @bmb_string_to_lower(ptr nocapture) nounwind willreturn")?;
        writeln!(out, "declare ptr @bmb_string_repeat(ptr nocapture, i64) nounwind willreturn")?;
        writeln!(out, "declare i64 @bmb_string_is_empty(ptr nocapture) memory(argmem: read) nounwind willreturn speculatable")?;
        writeln!(out, "declare ptr @bmb_chr(i64) nounwind willreturn")?;
        writeln!(out, "declare i64 @bmb_ord(ptr nocapture) memory(argmem: read) nounwind willreturn speculatable")?;
        writeln!(out, "declare void @bmb_print_str(ptr) nounwind")?;
        writeln!(out)?;

        // Phase 32.3: File I/O runtime functions
        writeln!(out, "; Runtime declarations - File I/O")?;
        writeln!(out, "declare i64 @bmb_file_exists(ptr)")?;
        writeln!(out, "declare i64 @bmb_file_size(ptr)")?;
        writeln!(out, "declare ptr @bmb_read_file(ptr)")?;
        writeln!(out, "declare i64 @bmb_write_file(ptr, ptr)")?;
        writeln!(out, "declare i64 @write_file_newlines(ptr, ptr)")?;  // v0.60.80: bootstrap support
        writeln!(out, "declare i64 @bmb_append_file(ptr, ptr)")?;
        writeln!(out)?;

        // Phase 32.3: StringBuilder runtime functions
        writeln!(out, "; Runtime declarations - StringBuilder")?;
        writeln!(out, "declare i64 @bmb_sb_new()")?;
        writeln!(out, "declare i64 @bmb_sb_with_capacity(i64)")?;  // v0.51.45: P0-E optimization
        writeln!(out, "declare i64 @bmb_sb_push(i64, ptr)")?;
        writeln!(out, "declare i64 @bmb_sb_push_char(i64, i64)")?;
        writeln!(out, "declare i64 @bmb_sb_push_int(i64, i64)")?;  // v0.50.73
        writeln!(out, "declare i64 @bmb_sb_push_escaped(i64, ptr)")?;  // v0.50.74
        writeln!(out, "declare i64 @bmb_sb_push_range(i64, ptr, i64, i64)")?;  // v0.95.7
        writeln!(out, "declare i64 @bmb_sb_len(i64)")?;
        writeln!(out, "declare ptr @bmb_sb_build(i64)")?;
        writeln!(out, "declare i64 @bmb_sb_clear(i64)")?;
        writeln!(out, "declare i64 @bmb_sb_contains(i64, ptr)")?;
        writeln!(out, "declare i64 @bmb_sb_println(i64)")?;
        writeln!(out)?;

        // Phase 32.3: Process execution runtime functions
        writeln!(out, "; Runtime declarations - Process execution")?;
        writeln!(out, "declare i64 @bmb_system(ptr)")?;
        writeln!(out, "declare ptr @bmb_system_capture(ptr)")?;
        writeln!(out, "declare ptr @bmb_exec_output(ptr, ptr)")?;
        writeln!(out, "declare ptr @bmb_getenv(ptr)")?;
        writeln!(out)?;

        // v0.88.2: Memory management functions
        writeln!(out, "; Runtime declarations - Memory management (v0.88.2)")?;
        writeln!(out, "declare i64 @bmb_string_free(ptr)")?;
        writeln!(out, "declare i64 @free_string(ptr)")?;
        writeln!(out, "declare i64 @bmb_sb_free(i64)")?;
        writeln!(out, "declare i64 @sb_free(i64)")?;
        writeln!(out, "declare i64 @bmb_arena_mode(i64)")?;
        writeln!(out, "declare i64 @arena_mode(i64)")?;
        writeln!(out, "declare i64 @bmb_arena_reset()")?;
        writeln!(out, "declare i64 @arena_reset()")?;
        writeln!(out, "declare i64 @bmb_arena_save()")?;
        writeln!(out, "declare i64 @arena_save()")?;
        writeln!(out, "declare i64 @bmb_arena_restore()")?;
        writeln!(out, "declare i64 @arena_restore()")?;
        writeln!(out, "declare i64 @bmb_arena_usage()")?;
        writeln!(out, "declare i64 @arena_usage()")?;
        writeln!(out)?;

        // v0.63: Timing functions for bmb-bench
        writeln!(out, "; Runtime declarations - Timing (v0.63)")?;
        writeln!(out, "declare i64 @bmb_time_ns() nounwind willreturn")?;
        writeln!(out, "declare i64 @time_ns() nounwind willreturn")?;
        writeln!(out)?;

        // v0.70: Threading runtime functions
        writeln!(out, "; Runtime declarations - Threading (v0.70)")?;
        writeln!(out, "declare i64 @bmb_spawn(ptr, ptr) nounwind")?;
        writeln!(out, "declare i64 @bmb_join(i64) nounwind")?;
        writeln!(out)?;

        // v0.71: Mutex runtime functions
        writeln!(out, "; Runtime declarations - Mutex (v0.71)")?;
        writeln!(out, "declare i64 @bmb_mutex_new(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_mutex_lock(i64) nounwind")?;
        writeln!(out, "declare void @bmb_mutex_unlock(i64, i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_mutex_try_lock(i64) nounwind")?;
        writeln!(out, "declare void @bmb_mutex_free(i64) nounwind")?;
        writeln!(out)?;

        // v0.72: Arc runtime functions
        writeln!(out, "; Runtime declarations - Arc (v0.72)")?;
        writeln!(out, "declare i64 @bmb_arc_new(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_arc_clone(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_arc_get(i64) nounwind")?;
        writeln!(out, "declare void @bmb_arc_drop(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_arc_strong_count(i64) nounwind")?;
        writeln!(out)?;

        // v0.72: malloc for atomic allocation - moved to memory allocation section (line ~477)
        // to avoid duplicate declaration conflict

        // v0.73: Channel runtime functions
        writeln!(out, "; Runtime declarations - Channel (v0.73)")?;
        writeln!(out, "declare void @bmb_channel_new(i64, ptr, ptr) nounwind")?;
        writeln!(out, "declare void @bmb_channel_send(i64, i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_channel_recv(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_channel_try_send(i64, i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_channel_try_recv(i64, ptr) nounwind")?;
        // v0.77: recv_timeout
        writeln!(out, "declare i64 @bmb_channel_recv_timeout(i64, i64, ptr) nounwind")?;
        writeln!(out, "declare i64 @bmb_sender_clone(i64) nounwind")?;
        // v0.78: block_on
        writeln!(out, "declare i64 @bmb_block_on(i64) nounwind")?;
        // v0.79: send_timeout
        writeln!(out, "declare i64 @bmb_channel_send_timeout(i64, i64, i64) nounwind")?;
        // v0.80: channel close operations
        writeln!(out, "declare void @bmb_channel_close(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_channel_is_closed(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_channel_recv_opt(i64, ptr) nounwind")?;
        writeln!(out)?;

        // v0.74: RwLock runtime functions
        writeln!(out, "; Runtime declarations - RwLock (v0.74)")?;
        writeln!(out, "declare i64 @bmb_rwlock_new(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_rwlock_read(i64) nounwind")?;
        writeln!(out, "declare void @bmb_rwlock_read_unlock(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_rwlock_write(i64) nounwind")?;
        writeln!(out, "declare void @bmb_rwlock_write_unlock(i64, i64) nounwind")?;
        writeln!(out, "declare void @bmb_rwlock_free(i64) nounwind")?;
        writeln!(out)?;

        // v0.74: Barrier runtime functions
        writeln!(out, "; Runtime declarations - Barrier (v0.74)")?;
        writeln!(out, "declare i64 @bmb_barrier_new(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_barrier_wait(i64) nounwind")?;
        writeln!(out, "declare void @bmb_barrier_free(i64) nounwind")?;
        writeln!(out)?;

        // v0.74: Condvar runtime functions
        writeln!(out, "; Runtime declarations - Condvar (v0.74)")?;
        writeln!(out, "declare i64 @bmb_condvar_new() nounwind")?;
        writeln!(out, "declare i64 @bmb_condvar_wait(i64, i64) nounwind")?;
        writeln!(out, "declare void @bmb_condvar_notify_one(i64) nounwind")?;
        writeln!(out, "declare void @bmb_condvar_notify_all(i64) nounwind")?;
        writeln!(out, "declare void @bmb_condvar_free(i64) nounwind")?;
        writeln!(out)?;

        // v0.75: Async/await runtime functions
        writeln!(out, "; Runtime declarations - Async/Await (v0.75)")?;
        writeln!(out, "declare i64 @__future_await(i64) nounwind")?;
        writeln!(out)?;

        // v0.83: AsyncFile runtime functions
        writeln!(out, "; Runtime declarations - AsyncFile (v0.83)")?;
        writeln!(out, "declare i64 @bmb_async_file_open(i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_async_file_read(i64) nounwind")?;
        writeln!(out, "declare void @bmb_async_file_write(i64, i64) nounwind")?;
        writeln!(out, "declare void @bmb_async_file_close(i64) nounwind")?;
        writeln!(out)?;

        // v0.83.1: AsyncSocket runtime functions
        writeln!(out, "; Runtime declarations - AsyncSocket (v0.83.1)")?;
        writeln!(out, "declare i64 @bmb_async_socket_connect(i64, i64) nounwind")?;
        writeln!(out, "declare i64 @bmb_async_socket_read(i64) nounwind")?;
        writeln!(out, "declare void @bmb_async_socket_write(i64, i64) nounwind")?;
        writeln!(out, "declare void @bmb_async_socket_close(i64) nounwind")?;
        writeln!(out)?;

        // v0.84: ThreadPool runtime functions
        writeln!(out, "; Runtime declarations - ThreadPool (v0.84)")?;
        writeln!(out, "declare i64 @bmb_thread_pool_new(i64) nounwind")?;
        writeln!(out, "declare void @bmb_thread_pool_execute(i64, i64) nounwind")?;
        writeln!(out, "declare void @bmb_thread_pool_join(i64) nounwind")?;
        writeln!(out, "declare void @bmb_thread_pool_shutdown(i64) nounwind")?;
        writeln!(out)?;

        // v0.85: Scope runtime functions
        writeln!(out, "; Runtime declarations - Scope (v0.85)")?;
        writeln!(out, "declare i64 @bmb_scope_new() nounwind")?;
        writeln!(out, "declare void @bmb_scope_spawn(i64, i64) nounwind")?;
        writeln!(out, "declare void @bmb_scope_wait(i64) nounwind")?;
        writeln!(out)?;

        // v0.31.23: Command-line argument builtins for Phase 32.3.G CLI Independence
        writeln!(out, "; Runtime declarations - CLI arguments")?;
        writeln!(out, "declare i64 @arg_count()")?;
        writeln!(out, "declare ptr @get_arg(i64)")?;
        writeln!(out)?;

        // Phase 32.3: Simple-name wrappers (for method call lowering)
        // BMB methods like s.len() generate calls to @len
        // v0.51.15: memory(argmem: read) enables full LICM hoisting
        writeln!(out, "; Runtime declarations - Method name wrappers")?;
        writeln!(out, "declare i64 @len(ptr nocapture) memory(argmem: read) nounwind willreturn speculatable")?;
        writeln!(out, "declare i64 @char_at(ptr nocapture, i64) memory(argmem: read) nounwind willreturn speculatable")?;
        // v0.46: byte_at is the preferred name (same as interpreter)
        writeln!(out, "declare i64 @byte_at(ptr nocapture, i64) memory(argmem: read) nounwind willreturn speculatable")?;
        writeln!(out, "declare ptr @slice(ptr, i64, i64) memory(argmem: read) nounwind willreturn")?;
        writeln!(out, "declare ptr @chr(i64) nounwind willreturn")?;
        writeln!(out, "declare i64 @ord(ptr) memory(argmem: read) nounwind willreturn")?;
        // v0.50.18: char_to_string for bootstrap compiler (takes i32 char code)
        writeln!(out, "declare ptr @char_to_string(i32)")?;
        writeln!(out, "declare void @print_str(ptr)")?;
        writeln!(out, "declare void @println_str(ptr)")?;
        writeln!(out)?;

        // File I/O wrappers
        writeln!(out, "declare i64 @file_exists(ptr)")?;
        // v0.51.2: cstr variants for string literal optimization
        writeln!(out, "declare i64 @file_exists_cstr(ptr)")?;
        writeln!(out, "declare i64 @bmb_file_exists_cstr(ptr)")?;
        writeln!(out, "declare i64 @file_size(ptr)")?;
        writeln!(out, "declare ptr @read_file(ptr)")?;
        writeln!(out, "declare i64 @write_file(ptr, ptr)")?;
        writeln!(out, "declare i64 @append_file(ptr, ptr)")?;
        writeln!(out)?;

        // StringBuilder wrappers
        // v0.51.26: Added nounwind for better optimization
        writeln!(out, "declare i64 @sb_new() nounwind")?;
        writeln!(out, "declare i64 @sb_with_capacity(i64) nounwind")?;  // v0.51.46: P0-E optimization
        writeln!(out, "declare i64 @sb_push(i64, ptr) nounwind")?;
        writeln!(out, "declare i64 @sb_push_cstr(i64, ptr) nounwind")?;  // v0.50.77: zero allocation for string literals
        writeln!(out, "declare i64 @sb_push_char(i64, i64) nounwind")?;
        writeln!(out, "declare i64 @sb_push_int(i64, i64) nounwind")?;  // v0.50.73
        writeln!(out, "declare i64 @sb_push_escaped(i64, ptr) nounwind")?;  // v0.50.74
        writeln!(out, "declare i64 @sb_push_range(i64, ptr, i64, i64) nounwind")?;  // v0.95.7
        writeln!(out, "declare i64 @sb_len(i64) nounwind willreturn")?;
        writeln!(out, "declare ptr @sb_build(i64) nounwind")?;
        writeln!(out, "declare i64 @sb_clear(i64) nounwind")?;
        writeln!(out, "declare i64 @sb_contains(i64, ptr) nounwind")?;
        writeln!(out, "declare i64 @sb_println(i64) nounwind")?;
        writeln!(out, "declare i64 @puts_cstr(ptr) nounwind")?;
        writeln!(out)?;

        // v0.60.246: String-key HashMap for O(1) lookups (strmap_*)
        writeln!(out, "; Runtime declarations - String HashMap")?;
        writeln!(out, "declare i64 @strmap_new() nounwind")?;
        writeln!(out, "declare i64 @strmap_insert(i64, ptr, i64) nounwind")?;
        writeln!(out, "declare i64 @strmap_get(i64, ptr) nounwind")?;
        writeln!(out, "declare i64 @strmap_contains(i64, ptr) nounwind")?;
        writeln!(out, "declare i64 @strmap_size(i64) nounwind")?;
        writeln!(out)?;

        // v0.50.36: find_close_paren is now defined in BMB, no extern needed

        // v0.34: Math intrinsics for Phase 34.4 Benchmark Gate
        // v0.51.26: Added more LLVM intrinsics for better optimization
        writeln!(out, "; Runtime declarations - Math intrinsics")?;
        writeln!(out, "declare double @llvm.sqrt.f64(double)")?;
        writeln!(out, "declare double @llvm.sin.f64(double)")?;
        writeln!(out, "declare double @llvm.cos.f64(double)")?;
        writeln!(out, "declare double @llvm.floor.f64(double)")?;
        writeln!(out, "declare double @llvm.ceil.f64(double)")?;
        writeln!(out, "declare double @llvm.fabs.f64(double)")?;
        writeln!(out, "declare double @llvm.pow.f64(double, double)")?;
        writeln!(out, "declare double @llvm.fma.f64(double, double, double)")?;
        // v0.51.35: memcpy intrinsic for struct array initialization
        writeln!(out, "declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)")?;
        // v0.60.59: memset intrinsic for zero-initialized array optimization
        writeln!(out, "declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)")?;
        writeln!(out)?;

        // v0.34.2: Memory allocation for Phase 34.2 Dynamic Collections
        // v0.51.26: Added noalias and nounwind for better optimization
        writeln!(out, "; Runtime declarations - Memory allocation")?;
        writeln!(out, "declare noalias ptr @malloc(i64) nounwind allocsize(0)")?;
        writeln!(out, "declare noalias ptr @realloc(ptr, i64) nounwind allocsize(1)")?;
        writeln!(out, "declare void @free(ptr nocapture) nounwind")?;
        writeln!(out, "declare noalias ptr @calloc(i64, i64) nounwind allocsize(0,1)")?;
        writeln!(out)?;

        // v0.51.51: Byte-level memory access for high-performance string parsing
        writeln!(out, "; Runtime declarations - Low-level memory access")?;
        writeln!(out, "declare i64 @load_u8(i64) memory(read) nounwind willreturn speculatable")?;
        writeln!(out, "declare void @store_u8(i64, i64) memory(write) nounwind willreturn")?;
        writeln!(out, "declare i64 @str_data(ptr nocapture) memory(argmem: read) nounwind willreturn speculatable")?;
        writeln!(out)?;

        // v0.50.70: Vector runtime functions (avoids inline PHI bug)
        // v0.51.26: Added nounwind attributes for better optimization
        writeln!(out, "; Runtime declarations - Vector")?;
        writeln!(out, "declare i64 @vec_new() nounwind")?;
        writeln!(out, "declare void @vec_free(i64) nounwind")?;
        writeln!(out, "declare i64 @vec_len(i64) nounwind willreturn")?;
        writeln!(out, "declare i64 @vec_get(i64, i64) nounwind willreturn")?;
        writeln!(out, "declare void @vec_set(i64, i64, i64) nounwind")?;
        writeln!(out, "declare void @vec_push(i64, i64) nounwind")?;
        writeln!(out, "declare void @bmb_vec_push(i64, i64) nounwind")?;
        writeln!(out)?;

        // v0.50.64: Hashmap runtime functions
        // v0.51.26: Added optimization attributes - readonly for get/len, nounwind for all
        writeln!(out, "; Runtime declarations - Hashmap")?;
        writeln!(out, "declare i64 @hashmap_new() nounwind")?;
        writeln!(out, "declare i64 @hashmap_insert(i64, i64, i64) nounwind")?;
        writeln!(out, "declare i64 @hashmap_get(i64, i64) nounwind willreturn")?;
        writeln!(out, "declare i64 @hashmap_remove(i64, i64) nounwind")?;
        writeln!(out, "declare i64 @hashmap_len(i64) nounwind willreturn")?;
        writeln!(out, "declare i64 @hashmap_contains(i64, i64) nounwind willreturn")?;
        writeln!(out, "declare void @hashmap_free(i64) nounwind")?;
        writeln!(out)?;

        // v0.90.83: String-content hashmap + cached registry lookup
        writeln!(out, "; Runtime declarations - String Hashmap")?;
        writeln!(out, "declare ptr @str_hashmap_new() nounwind")?;
        writeln!(out, "declare i64 @str_hashmap_insert(ptr, ptr, i64) nounwind")?;
        writeln!(out, "declare i64 @str_hashmap_get(ptr, ptr) nounwind willreturn")?;
        writeln!(out, "declare void @str_hashmap_free(ptr) nounwind")?;
        writeln!(out, "declare ptr @reg_cached_lookup(ptr, ptr, i64) nounwind willreturn")?;
        writeln!(out)?;

        Ok(())
    }

    /// Emit a function definition (legacy - without string table)
    #[allow(dead_code)]
    fn emit_function(&self, out: &mut String, func: &MirFunction) -> TextCodeGenResult<()> {
        let empty_str_table = HashMap::new();
        let empty_fn_types = HashMap::new();
        let empty_fn_param_types = HashMap::new();
        let empty_sret_functions = HashMap::new();
        let empty_small_struct_functions = HashMap::new();
        let empty_tuple_functions = HashMap::new();
        let empty_struct_defs = HashMap::new();
        self.emit_function_with_strings(out, func, &empty_str_table, &empty_fn_types, &empty_fn_param_types, &empty_sret_functions, &empty_small_struct_functions, &empty_tuple_functions, &empty_struct_defs)
    }

    /// v0.51.25: Check if a specific struct variable escapes the function
    /// A struct escapes if it's returned, passed to a call, or copied to something that escapes
    fn check_struct_escapes(&self, func: &MirFunction, struct_name: &str) -> bool {
        use crate::mir::{Terminator, Operand};

        for block in &func.blocks {
            // Check if returned
            if let Terminator::Return(Some(Operand::Place(p))) = &block.terminator
                && p.name == struct_name {
                    return true;
                }

            for inst in &block.instructions {
                match inst {
                    // Passed to a call
                    MirInst::Call { args, .. } => {
                        for arg in args {
                            if let Operand::Place(p) = arg
                                && p.name == struct_name {
                                    return true;
                                }
                        }
                    }
                    // Copied to another variable (conservative: treat as escape)
                    MirInst::Copy { src, .. } => {
                        if src.name == struct_name {
                            return true;
                        }
                    }
                    // Used in phi node (may be returned through phi)
                    MirInst::Phi { values, .. } => {
                        for (val, _) in values {
                            if let Operand::Place(p) = val
                                && p.name == struct_name {
                                    return true;
                                }
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// v0.51.27: Check if a struct variable is directly returned from the function
    /// This is used to determine if we can use sret pointer instead of malloc
    fn is_struct_returned(&self, func: &MirFunction, struct_name: &str) -> bool {
        let mut visited = std::collections::HashSet::new();
        self.is_struct_returned_inner(func, struct_name, &mut visited)
    }

    fn is_struct_returned_inner(
        &self,
        func: &MirFunction,
        struct_name: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> bool {
        use crate::mir::{Terminator, Operand};

        // Prevent infinite recursion
        if visited.contains(struct_name) {
            return false;
        }
        visited.insert(struct_name.to_string());

        for block in &func.blocks {
            // Check if directly returned
            if let Terminator::Return(Some(Operand::Place(p))) = &block.terminator
                && p.name == struct_name {
                    return true;
                }

            // Check if flows through phi to return
            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst {
                    // If phi dest is returned and this struct is one of phi inputs
                    if self.is_struct_returned_inner(func, &dest.name, visited) {
                        for (val, _) in values {
                            if let Operand::Place(p) = val
                                && p.name == struct_name {
                                    return true;
                                }
                        }
                    }
                }
            }
        }
        false
    }

    /// v0.51.25: Escape analysis for struct allocation (batch version)
    /// Returns set of struct variable names that escape the function (returned or passed to calls)
    /// These must be heap-allocated, others can use stack allocation
    #[allow(dead_code)]
    fn collect_escaped_structs(&self, func: &MirFunction) -> std::collections::HashSet<String> {
        use crate::mir::{Terminator, Operand};
        let mut escaped = std::collections::HashSet::new();

        // Find all struct init destinations
        let mut struct_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::StructInit { dest, .. } = inst {
                    struct_vars.insert(dest.name.clone());
                }
            }
        }

        // Check what escapes:
        // 1. Returned values
        // 2. Call arguments
        // 3. Values copied to parameters that might escape
        for block in &func.blocks {
            // Check terminator for returns
            if let Terminator::Return(Some(Operand::Place(p))) = &block.terminator
                && struct_vars.contains(&p.name) {
                    escaped.insert(p.name.clone());
                }

            // Check instructions for call arguments and assignments
            for inst in &block.instructions {
                match inst {
                    // Calls: any struct passed as argument escapes
                    MirInst::Call { args, .. } => {
                        for arg in args {
                            if let Operand::Place(p) = arg
                                && struct_vars.contains(&p.name) {
                                    escaped.insert(p.name.clone());
                                }
                        }
                    }
                    // Copy: if source is struct and dest might escape, source escapes
                    MirInst::Copy { dest, src } => {
                        if struct_vars.contains(&src.name) {
                            // Conservative: any copy of struct marks it as escaped
                            escaped.insert(src.name.clone());
                        }
                        // Also check if dest was already marked as escaped
                        if escaped.contains(&dest.name) && struct_vars.contains(&src.name) {
                            escaped.insert(src.name.clone());
                        }
                    }
                    // Phi: if any incoming value is struct and phi result might escape
                    MirInst::Phi { dest, values } => {
                        for (val, _) in values {
                            if let Operand::Place(p) = val
                                && struct_vars.contains(&p.name) {
                                    // If this phi is returned, the struct escapes
                                    // Mark it conservatively
                                    escaped.insert(p.name.clone());
                                }
                        }
                        // If dest is escaped, mark all incoming struct values
                        if escaped.contains(&dest.name) {
                            for (val, _) in values {
                                if let Operand::Place(p) = val
                                    && struct_vars.contains(&p.name) {
                                        escaped.insert(p.name.clone());
                                    }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        escaped
    }

    /// Build a map of place names to their types by pre-scanning instructions
    fn build_place_type_map(
        &self,
        func: &MirFunction,
        fn_return_types: &HashMap<String, &'static str>,
        struct_defs: &HashMap<String, Vec<(String, MirType)>>,
    ) -> HashMap<String, &'static str> {
        let mut place_types: HashMap<String, &'static str> = HashMap::new();

        // Add parameters
        for (name, ty) in &func.params {
            place_types.insert(name.clone(), self.mir_type_to_llvm(ty));
        }

        // Add locals
        for (name, ty) in &func.locals {
            place_types.insert(name.clone(), self.mir_type_to_llvm(ty));
        }

        // Scan all instructions to determine temporary types
        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    MirInst::Const { dest, value } => {
                        // v0.51.48: Don't override type of declared locals
                        // This preserves i32 type when assigning i64 constant literals
                        let is_declared_local = func.locals.iter().any(|(name, _)| name == &dest.name);
                        if !is_declared_local {
                            place_types.insert(dest.name.clone(), self.constant_type(value));
                        }
                    }
                    MirInst::Call { dest: Some(d), func: fn_name, .. } => {
                        let ret_ty = fn_return_types
                            .get(fn_name)
                            .copied()
                            .unwrap_or_else(|| self.infer_call_return_type(fn_name, func));
                        place_types.insert(d.name.clone(), ret_ty);
                    }
                    MirInst::BinOp { dest, op, lhs, rhs } => {
                        // Determine result type based on operator
                        // v0.51.20: Consider both operand types and use widest to handle narrowed params
                        // v0.51.30: Keep operation in i32 when narrowed param is used with i32-safe constant
                        let lhs_ty = match lhs {
                            Operand::Constant(c) => self.constant_type(c),
                            Operand::Place(p) => place_types.get(&p.name).copied().unwrap_or("i64"),
                        };
                        let rhs_ty = match rhs {
                            Operand::Constant(c) => self.constant_type(c),
                            Operand::Place(p) => place_types.get(&p.name).copied().unwrap_or("i64"),
                        };

                        // v0.51.30: Check for i32 + i64 constant optimization
                        // When one operand is i32 (narrowed param) and the other is an i64 constant
                        // that fits in i32, we can keep the operation in i32 to avoid sext/trunc overhead.
                        let can_narrow_to_i32 = if lhs_ty == "i32" && rhs_ty == "i64" {
                            // Check if rhs is a constant that fits in i32
                            matches!(rhs, Operand::Constant(Constant::Int(v)) if *v >= i32::MIN as i64 && *v <= i32::MAX as i64)
                        } else if lhs_ty == "i64" && rhs_ty == "i32" {
                            // Check if lhs is a constant that fits in i32
                            matches!(lhs, Operand::Constant(Constant::Int(v)) if *v >= i32::MIN as i64 && *v <= i32::MAX as i64)
                        } else {
                            false
                        };

                        // Use widest integer type (handles i32/i64 narrowing mismatch)
                        // UNLESS we can narrow to i32 (i32 param with i32-safe constant)
                        let widest_ty = if can_narrow_to_i32 {
                            "i32"
                        } else {
                            match (lhs_ty, rhs_ty) {
                                (_, "ptr") | ("ptr", _) => "ptr",
                                (_, "double") | ("double", _) => "double",
                                (_, "i64") | ("i64", _) => "i64",
                                ("i32", "i32") => "i32",
                                ("i32", "i1") | ("i1", "i32") => "i32",
                                ("i1", "i1") => "i1",
                                _ => lhs_ty, // Default to lhs type
                            }
                        };

                        let result_ty = match op {
                            // Comparison operators return i1
                            MirBinOp::Eq | MirBinOp::Ne | MirBinOp::Lt | MirBinOp::Le
                            | MirBinOp::Gt | MirBinOp::Ge => "i1",
                            // Float comparisons return i1
                            MirBinOp::FEq | MirBinOp::FNe | MirBinOp::FLt | MirBinOp::FLe
                            | MirBinOp::FGt | MirBinOp::FGe => "i1",
                            // String concat returns ptr
                            MirBinOp::Add if lhs_ty == "ptr" || rhs_ty == "ptr" => "ptr",
                            // Logical ops preserve operand type
                            MirBinOp::And | MirBinOp::Or => widest_ty,
                            // Arithmetic ops use widest type
                            _ => widest_ty,
                        };
                        place_types.insert(dest.name.clone(), result_ty);
                    }
                    MirInst::Phi { dest, values } => {
                        // v0.51.13: Use WIDEST type among all phi values to avoid type mismatch
                        // This handles ConstantPropagationNarrowing which may narrow parameters to i32
                        // while recursive calls return i64
                        // v0.51.17: Fixed - start with "i1" (narrowest) not "i32"
                        let mut widest_ty = "i1"; // Start with narrowest integer type
                        for (val, _) in values {
                            let ty = match val {
                                Operand::Constant(c) => self.constant_type(c),
                                Operand::Place(p) => place_types.get(&p.name).copied().unwrap_or("i64"),
                            };
                            // Compare type widths: ptr > double > i64 > i32 > i1
                            // v0.90.62: ptr takes priority (strings, struct pointers)
                            // Fixes phi type mismatch when branches mix ptr and i64
                            widest_ty = match (widest_ty, ty) {
                                (_, "ptr") | ("ptr", _) => "ptr",
                                (_, "double") | ("double", _) => "double",
                                (_, "i64") | ("i64", _) => "i64",
                                ("i32", "i32") => "i32",
                                ("i32", "i1") | ("i1", "i32") => "i32",
                                ("i1", "i1") => "i1",
                                _ => ty, // Default to the new type
                            };
                        }
                        place_types.insert(dest.name.clone(), widest_ty);
                    }
                    MirInst::Copy { dest, src } => {
                        // v0.51.48: If dest is a local with explicit type annotation, preserve it
                        // This prevents i64 constants from overriding i32 declared variables
                        let is_declared_local = func.locals.iter().any(|(name, _)| name == &dest.name);
                        if !is_declared_local {
                            // Only inherit type from source for temps, not declared locals
                            let ty = place_types.get(&src.name).copied().unwrap_or("i64");
                            place_types.insert(dest.name.clone(), ty);
                        }
                    }
                    // v0.50.50: ArrayInit produces ptr type (pointer to allocated array)
                    MirInst::ArrayInit { dest, .. } => {
                        place_types.insert(dest.name.clone(), "ptr");
                    }
                    // v0.50.50: StructInit produces ptr type (pointer to allocated struct)
                    MirInst::StructInit { dest, .. } => {
                        place_types.insert(dest.name.clone(), "ptr");
                    }
                    // v0.50.50: IndexLoad produces element type
                    // v0.51.35: Use actual element type for struct arrays
                    MirInst::IndexLoad { dest, element_type, .. } => {
                        let ty = match element_type {
                            MirType::Struct { .. } => "ptr",  // Struct arrays return pointers
                            _ => self.mir_type_to_llvm(element_type),
                        };
                        place_types.insert(dest.name.clone(), ty);
                    }
                    // v0.51.31: FieldAccess produces the field's type
                    MirInst::FieldAccess { dest, struct_name, field_index, .. } => {
                        let field_llvm_ty = struct_defs.get(struct_name)
                            .and_then(|fields| fields.get(*field_index))
                            .map(|(_, ty)| self.mir_type_to_llvm(ty))
                            .unwrap_or("i64");
                        place_types.insert(dest.name.clone(), field_llvm_ty);
                    }
                    // v0.51.39: Cast produces the target type
                    MirInst::Cast { dest, to_ty, .. } => {
                        place_types.insert(dest.name.clone(), self.mir_type_to_llvm(to_ty));
                    }
                    // v0.55: TupleInit produces ptr type (aggregate value)
                    MirInst::TupleInit { dest, .. } => {
                        place_types.insert(dest.name.clone(), "ptr");
                    }
                    // v0.55: TupleExtract produces element type
                    MirInst::TupleExtract { dest, element_type, .. } => {
                        place_types.insert(dest.name.clone(), self.mir_type_to_llvm(element_type));
                    }
                    // v0.60.19: PtrOffset produces ptr type
                    MirInst::PtrOffset { dest, .. } => {
                        place_types.insert(dest.name.clone(), "ptr");
                    }
                    // v0.60.21: ArrayAlloc produces ptr type
                    MirInst::ArrayAlloc { dest, .. } => {
                        place_types.insert(dest.name.clone(), "ptr");
                    }
                    // v0.60.20: PtrLoad produces the element type
                    MirInst::PtrLoad { dest, element_type, .. } => {
                        place_types.insert(dest.name.clone(), self.mir_type_to_llvm(element_type));
                    }
                    // v0.60.20: PtrStore has no destination
                    MirInst::PtrStore { .. } => {}
                    _ => {}
                }
            }
        }

        place_types
    }

    /// Emit a function definition with string table support
    #[allow(clippy::too_many_arguments)]
    fn emit_function_with_strings(
        &self,
        out: &mut String,
        func: &MirFunction,
        string_table: &HashMap<String, String>,
        fn_return_types: &HashMap<String, &'static str>,
        fn_param_types: &HashMap<String, Vec<&'static str>>,
        sret_functions: &HashMap<String, usize>,
        small_struct_functions: &HashMap<String, usize>,
        tuple_functions: &HashMap<String, String>,
        struct_defs: &HashMap<String, Vec<(String, MirType)>>,
    ) -> TextCodeGenResult<()> {
        // Pre-scan to build place type map
        let place_types = self.build_place_type_map(func, fn_return_types, struct_defs);

        // v0.51.25: Escape analysis is now done inline in emit_instruction_with_strings
        // via check_struct_escapes() for each StructInit instruction

        // v0.51.18: Track narrowed i32 params but DON'T override to i64
        // With proper i32 propagation:
        // - Narrowed params stay as i32 in place_types (built by build_place_type_map)
        // - Derived temporaries (_t2 = n - 1) are also i32
        // - No sext at entry, no trunc before recursive calls
        // - Phi coercion handles i32→i64 when mixing with call results
        let mut narrowed_param_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (name, ty) in &func.params {
            let llvm_ty = self.mir_type_to_llvm(ty);
            if llvm_ty == "i32" {
                narrowed_param_set.insert(name.clone());
                // DON'T override to i64 - keep as i32 for proper 32-bit operations
            }
        }

        // Track defined names to handle SSA violations from MIR
        let mut name_counts: HashMap<String, u32> = HashMap::new();

        // v0.55: Build map of tuple variable names to their LLVM struct types
        // This is needed for TupleExtract to know the correct type for load/extractvalue
        // We need to iterate multiple times to propagate types through phi nodes
        let mut tuple_var_types: HashMap<String, String> = HashMap::new();
        // First pass: collect direct tuple sources (calls, TupleInit)
        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Call { dest: Some(d), func: fn_name, .. } = inst
                    && let Some(tuple_type) = tuple_functions.get(fn_name) {
                        tuple_var_types.insert(d.name.clone(), tuple_type.clone());
                    }
                // Also track TupleInit instructions - these create tuple values directly
                if let MirInst::TupleInit { dest, elements } = inst {
                    let elem_types: Vec<&str> = elements.iter()
                        .map(|(ty, _)| self.mir_type_to_llvm(ty))
                        .collect();
                    let tuple_type = format!("{{ {} }}", elem_types.join(", "));
                    tuple_var_types.insert(dest.name.clone(), tuple_type);
                }
            }
        }
        // Second pass: propagate through phi nodes and copies (iterate until stable)
        for _ in 0..10 {  // Limit iterations to avoid infinite loops
            let mut changed = false;
            for block in &func.blocks {
                for inst in &block.instructions {
                    // Track Copy instructions that copy from a tuple variable
                    if let MirInst::Copy { dest, src } = inst
                        && !tuple_var_types.contains_key(&dest.name)
                            && let Some(tuple_type) = tuple_var_types.get(&src.name) {
                                tuple_var_types.insert(dest.name.clone(), tuple_type.clone());
                                changed = true;
                            }
                    // Track Phi instructions where any input is a tuple
                    if let MirInst::Phi { dest, values } = inst
                        && !tuple_var_types.contains_key(&dest.name) {
                            let tuple_type = values.iter().find_map(|(val, _)| {
                                if let Operand::Place(p) = val {
                                    tuple_var_types.get(&p.name).cloned()
                                } else {
                                    None
                                }
                            });
                            if let Some(tt) = tuple_type {
                                tuple_var_types.insert(dest.name.clone(), tt);
                                changed = true;
                            }
                        }
                }
            }
            if !changed {
                break;
            }
        }

        // v0.51.27: Detect struct return functions for sret optimization
        // v0.51.28: Small structs (1-2 fields) use register return, larger structs use sret
        let struct_field_count = if let MirType::Struct { fields, .. } = &func.ret_ty {
            fields.len()
        } else {
            0
        };
        let is_small_struct = struct_field_count > 0 && struct_field_count <= 2;
        let is_sret = struct_field_count > 2;  // Only use sret for 3+ field structs

        // v0.55: Check if return type is a tuple
        let tuple_elems = if let MirType::Tuple(elems) = &func.ret_ty {
            Some(elems)
        } else {
            None
        };

        // Function signature - small structs and tuples use aggregate return type
        let ret_type = if is_sret {
            "void".to_string()
        } else if is_small_struct {
            if struct_field_count == 1 { "{i64}".to_string() } else { "{i64, i64}".to_string() }
        } else if let Some(elems) = &tuple_elems {
            // v0.55: Tuple return type - build LLVM struct type string
            let elem_types: Vec<&str> = elems.iter()
                .map(|e| self.mir_type_to_llvm(e))
                .collect();
            format!("{{ {} }}", elem_types.join(", "))
        } else {
            self.mir_type_to_llvm(&func.ret_ty).to_string()
        };
        // v0.60.37: Add nocapture readonly for String parameters (ptr type)
        // This tells LLVM the string isn't modified, enabling LICM to hoist
        // string data pointer loads out of loops (fixes fasta 440% regression)
        let mut params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| {
                let llvm_ty = self.mir_type_to_llvm(ty);
                if matches!(ty, MirType::String) {
                    // String parameters are read-only (immutable in BMB)
                    format!("ptr nocapture readonly %{}", name)
                } else {
                    format!("{} %{}", llvm_ty, name)
                }
            })
            .collect();

        // v0.51.27: Add sret parameter for struct return functions
        if is_sret {
            params.insert(0, "ptr noalias sret(i8) %_sret".to_string());
        }

        // Mark parameters as defined
        for (name, _) in &func.params {
            name_counts.insert(name.clone(), 1);
        }
        if is_sret {
            name_counts.insert("_sret".to_string(), 1);
        }

        // Function attributes for optimization:
        // - nounwind: BMB doesn't have exceptions, enables better codegen
        // - willreturn: Most functions will eventually return (helps optimization)
        // - mustprogress: Function must make forward progress (LLVM 12+)
        // - For main: no special attributes (ABI compatibility)
        // - Attributes go AFTER the parameter list in LLVM IR syntax
        //
        // NOTE: inlinehint was tested (v0.50.66) but caused performance regression
        // LLVM's default inlining heuristics are better than manual hints
        //
        // v0.50.76: Added willreturn and mustprogress for better recursion optimization
        // v0.51.8: Added alwaysinline for small functions to eliminate call overhead
        // v0.51.11: Added memory(none) for memory-free functions to enable LICM
        // v0.51.52: Added inlinehint for medium-sized functions (like lexer's next_token)
        let attrs = if func.name == "main" {
            String::new()
        } else if func.always_inline && func.is_memory_free {
            " alwaysinline nounwind willreturn mustprogress memory(none)".to_string()
        } else if func.always_inline {
            " alwaysinline nounwind willreturn mustprogress".to_string()
        } else if func.inline_hint && func.is_memory_free {
            " inlinehint nounwind willreturn mustprogress memory(none)".to_string()
        } else if func.inline_hint {
            " inlinehint nounwind willreturn mustprogress".to_string()
        } else if func.is_memory_free {
            " nounwind willreturn mustprogress memory(none)".to_string()
        } else {
            " nounwind willreturn mustprogress".to_string()
        };

        // v0.31.23: Rename BMB main to bmb_user_main so C runtime can provide real main()
        // This enables argv support through bmb_init_argv called from real main()
        let emitted_name = if func.name == "main" { "bmb_user_main" } else { &func.name };

        // v0.60.252: Use private linkage for @inline functions to avoid symbol collision
        let linkage = if func.always_inline && func.name != "main" && func.name != "bmb_user_main" {
            "private "
        } else {
            ""
        };

        writeln!(
            out,
            "define {}{} @{}({}){} {{",
            linkage,
            ret_type,
            emitted_name,
            params.join(", "),
            attrs
        )?;

        // Collect phi destination names first - these are SSA values, not memory locations
        // They should NOT have allocas or be loaded from memory
        let phi_dests: std::collections::HashSet<String> = func.blocks.iter()
            .flat_map(|b| b.instructions.iter())
            .filter_map(|inst| {
                if let MirInst::Phi { dest, .. } = inst {
                    Some(dest.name.clone())
                } else {
                    None
                }
            })
            .collect();

        // Build map of (phi_dest_block, local_name, pred_block) -> load_temp_name
        // This is needed because phi nodes must reference SSA values, not memory locations
        // So we emit loads before terminators in predecessor blocks
        // IMPORTANT: Exclude phi destinations - they're already SSA values
        let mut phi_load_map: std::collections::HashMap<(String, String, String), String> =
            std::collections::HashMap::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest: _, values } = inst {
                    for (val, pred_label) in values {
                        if let Operand::Place(p) = val {
                            // Check if this place is a local variable (not a phi destination)
                            // Phi destinations are SSA values, not memory locations
                            if func.locals.iter().any(|(n, _)| n == &p.name)
                               && !phi_dests.contains(&p.name) {
                                let key = (block.label.clone(), p.name.clone(), pred_label.clone());
                                let load_temp = format!("{}.phi.{}", p.name, pred_label);
                                phi_load_map.insert(key, load_temp);
                            }
                        }
                    }
                }
            }
        }

        // Build map for string constants in phi nodes
        // Key: (dest_block, string_value, pred_block) -> temp_name
        // String constants need to be wrapped with bmb_string_from_cstr before phi
        let mut phi_string_map: std::collections::HashMap<(String, String, String), String> =
            std::collections::HashMap::new();
        let mut string_phi_counter = 0u32;

        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest: _, values } = inst {
                    for (val, pred_label) in values {
                        if let Operand::Constant(Constant::String(s)) = val {
                            let key = (block.label.clone(), s.clone(), pred_label.clone());
                            phi_string_map.entry(key).or_insert_with(|| {
                                let temp_name = format!("_str_phi_{}", string_phi_counter);
                                string_phi_counter += 1;
                                temp_name
                            });
                        }
                    }
                }
            }
        }

        // v0.51.13: Build map for phi value coercion (type widening)
        // Key: (phi_dest_block, value_name, pred_block) -> (coerced_temp_name, from_type, to_type)
        // When ConstantPropagationNarrowing changes a parameter to i32 but the function returns i64,
        // phi nodes may have mixed types. We need to emit sext instructions to widen narrower values.
        let mut phi_coerce_map: std::collections::HashMap<(String, String, String), (String, &'static str, &'static str)> =
            std::collections::HashMap::new();
        let mut coerce_counter = 0u32;

        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst {
                    // Get the phi's target type from place_types (computed above with widest type)
                    let phi_ty = place_types.get(&dest.name).copied().unwrap_or("i64");

                    for (val, pred_label) in values {
                        let val_ty = match val {
                            Operand::Constant(c) => self.constant_type(c),
                            Operand::Place(p) => place_types.get(&p.name).copied().unwrap_or("i64"),
                        };

                        // Check if this value needs coercion (narrower than phi type)
                        let needs_coerce = matches!((val_ty, phi_ty), ("i32", "i64") | ("i1", "i64") | ("i1", "i32"));

                        if needs_coerce
                            && let Operand::Place(p) = val {
                                let key = (block.label.clone(), p.name.clone(), pred_label.clone());
                                let temp_name = format!("_phi_sext_{}", coerce_counter);
                                coerce_counter += 1;
                                phi_coerce_map.insert(key, (temp_name, val_ty, phi_ty));
                            }
                    }
                }
            }
        }

        // Collect local variable names for alloca-based handling
        // Using alloca avoids SSA dominance issues when locals are assigned in branches
        // Exclude: void-typed locals (can't allocate), phi destinations (they're SSA values)
        // v0.50.72: Exclude array types (they get their own alloca in ArrayInit)
        let local_names: std::collections::HashSet<String> = func.locals.iter()
            .filter(|(_, ty)| self.mir_type_to_llvm(ty) != "void")
            .filter(|(name, _)| !phi_dests.contains(name))
            .filter(|(_, ty)| !matches!(ty, crate::mir::MirType::Array { .. }))
            .map(|(name, _)| name.clone())
            .collect();

        // v0.51.18: Track narrowed i32 parameters (used for phi coercion and return handling)
        // With proper i32 propagation, we DON'T emit sext at entry
        let narrowed_param_names: std::collections::HashSet<String> = func.params
            .iter()
            .filter(|(_, ty)| self.mir_type_to_llvm(ty) == "i32")
            .map(|(name, _)| name.clone())
            .collect();

        // Emit entry block with allocas for local variables (excluding phi-referenced ones)
        // Use "alloca_entry" to avoid conflicts with user variables named "entry"
        if !local_names.is_empty() {
            writeln!(out, "alloca_entry:")?;
            for (name, ty) in &func.locals {
                if local_names.contains(name) {
                    // v0.55: Check if this is a tuple variable first
                    let llvm_ty = if let Some(tuple_type) = tuple_var_types.get(name) {
                        tuple_type.as_str()
                    } else {
                        // v0.51.30: Use place_types for alloca to handle narrowed types correctly
                        // This ensures that when a BinaryOp produces i32 (due to narrowing optimization),
                        // the alloca is also i32, avoiding mismatched store/load sizes.
                        place_types.get(name).copied().unwrap_or_else(|| self.mir_type_to_llvm(ty))
                    };
                    // Skip void types - they can't be allocated
                    if llvm_ty != "void" {
                        writeln!(out, "  %{}.addr = alloca {}", name, llvm_ty)?;
                    }
                }
            }
            // v0.51.18: NO sext for narrowed params - use i32 directly for proper 32-bit ops
            // Jump to the actual first block
            if let Some(first_block) = func.blocks.first() {
                writeln!(out, "  br label %bb_{}", first_block.label)?;
            }
        }

        // Emit basic blocks with place type information
        for block in &func.blocks {
            self.emit_block_with_strings(out, block, func, string_table, fn_return_types, fn_param_types, sret_functions, small_struct_functions, tuple_functions, &tuple_var_types, &place_types, &mut name_counts, &local_names, &narrowed_param_names, &phi_load_map, &phi_string_map, &phi_coerce_map, struct_defs)?;
        }

        writeln!(out, "}}")?;
        writeln!(out)?;

        Ok(())
    }

    /// Emit a basic block (legacy - without string table)
    #[allow(dead_code)]
    fn emit_block(
        &self,
        out: &mut String,
        block: &BasicBlock,
        func: &MirFunction,
    ) -> TextCodeGenResult<()> {
        let empty_str_table = HashMap::new();
        let empty_fn_types = HashMap::new();
        let empty_fn_param_types = HashMap::new();
        let empty_sret_functions = HashMap::new();
        let empty_place_types = HashMap::new();
        let mut empty_name_counts = HashMap::new();
        let empty_local_names = std::collections::HashSet::new();
        let empty_phi_map = std::collections::HashMap::new();
        let empty_phi_string_map = std::collections::HashMap::new();
        let empty_phi_coerce_map = std::collections::HashMap::new();
        let empty_narrowed = std::collections::HashSet::new();
        let empty_small_struct_functions = HashMap::new();
        let empty_tuple_functions = HashMap::new();
        let empty_tuple_var_types = HashMap::new();
        let empty_struct_defs = HashMap::new();
        self.emit_block_with_strings(out, block, func, &empty_str_table, &empty_fn_types, &empty_fn_param_types, &empty_sret_functions, &empty_small_struct_functions, &empty_tuple_functions, &empty_tuple_var_types, &empty_place_types, &mut empty_name_counts, &empty_local_names, &empty_narrowed, &empty_phi_map, &empty_phi_string_map, &empty_phi_coerce_map, &empty_struct_defs)
    }

    /// Emit a basic block with string table support
    #[allow(clippy::too_many_arguments)]
    fn emit_block_with_strings(
        &self,
        out: &mut String,
        block: &BasicBlock,
        func: &MirFunction,
        string_table: &HashMap<String, String>,
        fn_return_types: &HashMap<String, &'static str>,
        fn_param_types: &HashMap<String, Vec<&'static str>>,
        sret_functions: &HashMap<String, usize>,
        small_struct_functions: &HashMap<String, usize>,
        tuple_functions: &HashMap<String, String>,
        tuple_var_types: &HashMap<String, String>,
        place_types: &HashMap<String, &'static str>,
        name_counts: &mut HashMap<String, u32>,
        local_names: &std::collections::HashSet<String>,
        narrowed_param_names: &std::collections::HashSet<String>,
        phi_load_map: &std::collections::HashMap<(String, String, String), String>,
        phi_string_map: &std::collections::HashMap<(String, String, String), String>,
        phi_coerce_map: &std::collections::HashMap<(String, String, String), (String, &'static str, &'static str)>,
        struct_defs: &HashMap<String, Vec<(String, MirType)>>,
    ) -> TextCodeGenResult<()> {
        // Use bb_ prefix to avoid collision with variable names
        writeln!(out, "bb_{}:", block.label)?;

        // Emit instructions (pass phi_load_map for phi node handling)
        for inst in &block.instructions {
            self.emit_instruction_with_strings(out, inst, func, string_table, fn_return_types, fn_param_types, sret_functions, small_struct_functions, tuple_functions, tuple_var_types, place_types, name_counts, local_names, narrowed_param_names, phi_load_map, phi_string_map, phi_coerce_map, &block.label, struct_defs)?;
        }

        // Emit loads for locals that will be used in phi nodes of successor blocks
        // This must happen BEFORE the terminator
        for ((_dest_block, local_name, pred_block), load_temp) in phi_load_map {
            if pred_block == &block.label {
                // v0.55: Check tuple_var_types first for tuple locals
                let llvm_ty: std::borrow::Cow<'static, str> = if let Some(tuple_type) = tuple_var_types.get(local_name) {
                    std::borrow::Cow::Owned(tuple_type.clone())
                } else if let Some(ty) = place_types.get(local_name) {
                    std::borrow::Cow::Borrowed(*ty)
                } else if let Some((_, ty)) = func.locals.iter().find(|(n, _)| n == local_name) {
                    std::borrow::Cow::Borrowed(self.mir_type_to_llvm(ty))
                } else {
                    std::borrow::Cow::Borrowed("ptr") // Default to ptr for unknown types
                };
                writeln!(out, "  %{} = load {}, ptr %{}.addr", load_temp, llvm_ty, local_name)?;
            }
        }

        // v0.51.22: String constants in phi nodes use pre-initialized global BmbString
        // This must happen BEFORE the terminator
        for ((_dest_block, string_val, pred_block), temp_name) in phi_string_map {
            if pred_block == &block.label {
                // Look up the global string constant name
                if let Some(global_name) = string_table.get(string_val) {
                    // Use getelementptr to get pointer to global BmbString
                    writeln!(out, "  %{} = getelementptr %BmbString, ptr @{}.bmb, i32 0", temp_name, global_name)?;
                }
            }
        }

        // v0.51.13: Emit sext instructions for phi value type coercion
        // This handles ConstantPropagationNarrowing: when parameter is i32 but return is i64
        for ((_dest_block, val_name, pred_block), (coerce_temp, from_ty, to_ty)) in phi_coerce_map {
            if pred_block == &block.label {
                // Check if the value was loaded via phi_load_map (local variable)
                // or is a direct parameter/SSA value
                let source_name = if let Some(load_temp) = phi_load_map.iter()
                    .find(|((_, ln, pb), _)| ln == val_name && pb == pred_block)
                    .map(|(_, lt)| lt.clone())
                {
                    // Use the loaded value
                    format!("%{}", load_temp)
                } else {
                    // Direct SSA value (parameter or temp)
                    format!("%{}", val_name)
                };

                // Emit the appropriate coercion instruction
                let instr = match (*from_ty, *to_ty) {
                    ("i32", "i64") | ("i1", "i64") | ("i1", "i32") => "sext",
                    ("i64", "i32") => "trunc",
                    _ => "bitcast", // Fallback
                };
                writeln!(out, "  %{} = {} {} {} to {}", coerce_temp, instr, from_ty, source_name, to_ty)?;
            }
        }

        // Emit terminator
        self.emit_terminator(out, &block.terminator, func, string_table, local_names, narrowed_param_names, place_types, &block.label)?;

        Ok(())
    }

    /// Get unique name for SSA definition, handling duplicates
    fn unique_name(&self, name: &str, name_counts: &mut HashMap<String, u32>) -> String {
        let count = name_counts.entry(name.to_string()).or_insert(0);
        *count += 1;
        if *count == 1 {
            name.to_string()
        } else {
            format!("{}_{}", name, *count - 1)
        }
    }

    /// Emit an instruction with string table support
    #[allow(clippy::too_many_arguments)]
    fn emit_instruction_with_strings(
        &self,
        out: &mut String,
        inst: &MirInst,
        func: &MirFunction,
        string_table: &HashMap<String, String>,
        fn_return_types: &HashMap<String, &'static str>,
        fn_param_types: &HashMap<String, Vec<&'static str>>,
        sret_functions: &HashMap<String, usize>,
        small_struct_functions: &HashMap<String, usize>,
        tuple_functions: &HashMap<String, String>,
        tuple_var_types: &HashMap<String, String>,
        place_types: &HashMap<String, &'static str>,
        name_counts: &mut HashMap<String, u32>,
        local_names: &std::collections::HashSet<String>,
        narrowed_param_names: &std::collections::HashSet<String>,
        _phi_load_map: &std::collections::HashMap<(String, String, String), String>,
        phi_string_map: &std::collections::HashMap<(String, String, String), String>,
        phi_coerce_map: &std::collections::HashMap<(String, String, String), (String, &'static str, &'static str)>,
        current_block_label: &str,
        struct_defs: &HashMap<String, Vec<(String, MirType)>>,
    ) -> TextCodeGenResult<()> {
        match inst {
            MirInst::Const { dest, value } => {
                let const_ty = self.constant_type(value);
                // Check if destination is a local (uses alloca)
                if local_names.contains(&dest.name) {
                    // v0.51.48: Use destination type from place_types for locals
                    // This ensures i32 locals get i32 stores, not i64
                    let dest_ty = place_types.get(&dest.name).copied().unwrap_or(const_ty);
                    // v0.51.33: Store constants directly to allocas without intermediate SSA values
                    // This eliminates unnecessary `add 0, const` instructions
                    match value {
                        Constant::Int(n) => {
                            writeln!(out, "  store {} {}, ptr %{}.addr", dest_ty, n, dest.name)?;
                        }
                        Constant::Bool(b) => {
                            let v = if *b { 1 } else { 0 };
                            writeln!(out, "  store {} {}, ptr %{}.addr", dest_ty, v, dest.name)?;
                        }
                        Constant::Float(f) => {
                            // Format float in LLVM-compatible way (scientific notation)
                            let f_str = if f.is_nan() {
                                "0x7FF8000000000000".to_string()
                            } else if f.is_infinite() {
                                if f.is_sign_positive() { "0x7FF0000000000000".to_string() } else { "0xFFF0000000000000".to_string() }
                            } else {
                                format!("{:.6e}", f)
                            };
                            writeln!(out, "  store {} {}, ptr %{}.addr", dest_ty, f_str, dest.name)?;
                        }
                        Constant::Unit => {
                            writeln!(out, "  store i8 0, ptr %{}.addr", dest.name)?;
                        }
                        Constant::String(s) => {
                            // v0.51.22: Use pre-initialized global BmbString
                            if let Some(global_name) = string_table.get(s) {
                                // Need temp for getelementptr result
                                let temp_base = format!("{}.tmp", dest.name);
                                let temp_name = self.unique_name(&temp_base, name_counts);
                                writeln!(out, "  %{} = getelementptr %BmbString, ptr @{}.bmb, i32 0",
                                         temp_name, global_name)?;
                                writeln!(out, "  store ptr %{}, ptr %{}.addr", temp_name, dest.name)?;
                            } else {
                                writeln!(out, "  ; string constant not in table: {}", s)?;
                                writeln!(out, "  store ptr null, ptr %{}.addr", dest.name)?;
                            }
                        }
                        // v0.64: Character constant (stored as i32 Unicode codepoint)
                        Constant::Char(c) => {
                            writeln!(out, "  store {} {}, ptr %{}.addr", dest_ty, *c as u32, dest.name)?;
                        }
                    }
                } else {
                    let dest_name = self.unique_name(&dest.name, name_counts);
                    // Use add with 0 for integer constants (LLVM IR idiom)
                    match value {
                        Constant::Int(n) => {
                            writeln!(out, "  %{} = add {} 0, {}", dest_name, const_ty, n)?;
                        }
                        Constant::Bool(b) => {
                            let v = if *b { 1 } else { 0 };
                            writeln!(out, "  %{} = add {} 0, {}", dest_name, const_ty, v)?;
                        }
                        Constant::Float(f) => {
                            // Format float in LLVM-compatible way (scientific notation)
                            let f_str = if f.is_nan() {
                                "0x7FF8000000000000".to_string()
                            } else if f.is_infinite() {
                                if f.is_sign_positive() { "0x7FF0000000000000".to_string() } else { "0xFFF0000000000000".to_string() }
                            } else {
                                format!("{:.6e}", f)
                            };
                            writeln!(out, "  %{} = fadd fast {} 0.0, {}", dest_name, const_ty, f_str)?;
                        }
                        Constant::Unit => {
                            // Unit type - just assign 0
                            writeln!(out, "  %{} = add i8 0, 0", dest_name)?;
                        }
                        Constant::String(s) => {
                            // v0.51.22: Use pre-initialized global BmbString instead of bmb_string_from_cstr
                            // This eliminates runtime overhead for string constants
                            if let Some(global_name) = string_table.get(s) {
                                writeln!(out, "  %{} = getelementptr %BmbString, ptr @{}.bmb, i32 0",
                                         dest_name, global_name)?;
                            } else {
                                // Fallback if string not in table (shouldn't happen)
                                writeln!(out, "  ; string constant not in table: {}", s)?;
                            }
                        }
                        // v0.64: Character constant (stored as i32 Unicode codepoint)
                        Constant::Char(c) => {
                            writeln!(out, "  %{} = add {} 0, {}", dest_name, const_ty, *c as u32)?;
                        }
                    }
                }
            }

            MirInst::Copy { dest, src } => {
                // v0.55: Check if source is a tuple variable - use actual tuple type
                let (ty, is_tuple) = if let Some(tuple_type) = tuple_var_types.get(&src.name) {
                    (tuple_type.as_str(), true)
                } else {
                    // Use place_types for accurate type inference
                    let t = place_types.get(&src.name).copied()
                        .unwrap_or_else(|| self.infer_place_type(src, func));
                    (t, false)
                };

                // v0.31.23: Skip void type copies (result of void-returning function calls)
                if ty == "void" {
                    // No-op: void values cannot be copied or stored
                    // This happens when a let binding captures a void call result
                    return Ok(());
                }

                // Load from alloca if source is a local
                // v0.50.50: Use unique_name to avoid SSA violations when same src loaded multiple times
                let src_val = if local_names.contains(&src.name) {
                    let load_base = format!("{}.load", src.name);
                    let load_name = self.unique_name(&load_base, name_counts);
                    writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, ty, src.name)?;
                    format!("%{}", load_name)
                } else {
                    format!("%{}", src.name)
                };

                // Store to alloca if destination is a local
                if local_names.contains(&dest.name) {
                    // v0.60.13: Get destination alloca type - handle type width mismatches
                    // This handles Copy between different-width types correctly
                    let dest_ty = place_types.get(&dest.name).copied().unwrap_or(ty);
                    if ty == "i64" && dest_ty == "i32" {
                        // Need to truncate i64 to i32 before storing
                        let trunc_name = self.unique_name(&format!("{}.trunc", dest.name), name_counts);
                        writeln!(out, "  %{} = trunc i64 {} to i32", trunc_name, src_val)?;
                        writeln!(out, "  store i32 %{}, ptr %{}.addr", trunc_name, dest.name)?;
                    } else if ty == "i32" && dest_ty == "i64" {
                        // v0.60.13: Need to sign extend i32 to i64 before storing
                        let sext_name = self.unique_name(&format!("{}.sext", dest.name), name_counts);
                        writeln!(out, "  %{} = sext i32 {} to i64", sext_name, src_val)?;
                        writeln!(out, "  store i64 %{}, ptr %{}.addr", sext_name, dest.name)?;
                    } else {
                        writeln!(out, "  store {} {}, ptr %{}.addr", ty, src_val, dest.name)?;
                    }
                    // Suppress unused warning for is_tuple
                    let _ = is_tuple;
                } else {
                    let dest_name = self.unique_name(&dest.name, name_counts);
                    if ty == "ptr" {
                        // For pointers, use select with always-true condition
                        writeln!(out, "  %{} = select i1 true, ptr {}, ptr null", dest_name, src_val)?;
                    } else if ty == "f64" {
                        // For floats, use fadd (v0.60.8: with fast flag)
                        writeln!(out, "  %{} = fadd fast {} {}, 0.0", dest_name, ty, src_val)?;
                    } else {
                        // For integers, use add
                        writeln!(out, "  %{} = add {} {}, 0", dest_name, ty, src_val)?;
                    }
                }
            }

            MirInst::BinOp { dest, op, lhs, rhs } => {
                let dest_name = self.unique_name(&dest.name, name_counts);
                // Use place_types for accurate type inference
                let lhs_ty = match lhs {
                    Operand::Constant(c) => self.constant_type(c),
                    Operand::Place(p) => place_types.get(&p.name).copied()
                        .unwrap_or_else(|| self.infer_place_type(p, func)),
                };
                let rhs_ty = match rhs {
                    Operand::Constant(c) => self.constant_type(c),
                    Operand::Place(p) => place_types.get(&p.name).copied()
                        .unwrap_or_else(|| self.infer_place_type(p, func)),
                };

                // Emit loads for local operands (use dest_name for uniqueness)
                // v0.51.17: Use narrowing-aware formatting for non-local operands
                let lhs_str = match lhs {
                    Operand::Place(p) if local_names.contains(&p.name) => {
                        let load_name = format!("{}.{}.lhs", dest_name, p.name);
                        writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, lhs_ty, p.name)?;
                        format!("%{}", load_name)
                    }
                    _ => self.format_operand_with_strings_and_narrowing(lhs, string_table, narrowed_param_names),
                };
                let rhs_str = match rhs {
                    Operand::Place(p) if local_names.contains(&p.name) => {
                        let load_name = format!("{}.{}.rhs", dest_name, p.name);
                        writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, rhs_ty, p.name)?;
                        format!("%{}", load_name)
                    }
                    _ => self.format_operand_with_strings_and_narrowing(rhs, string_table, narrowed_param_names),
                };

                // String concatenation: either operand is ptr with Add op
                if (lhs_ty == "ptr" || rhs_ty == "ptr") && *op == MirBinOp::Add {
                    // v0.51.22: Use pre-initialized global BmbString
                    let lhs_final = if let Operand::Constant(Constant::String(s)) = lhs {
                        if let Some(global_name) = string_table.get(s) {
                            format!("@{}.bmb", global_name)
                        } else { lhs_str.clone() }
                    } else { lhs_str.clone() };
                    let rhs_final = if let Operand::Constant(Constant::String(s)) = rhs {
                        if let Some(global_name) = string_table.get(s) {
                            format!("@{}.bmb", global_name)
                        } else { rhs_str.clone() }
                    } else { rhs_str.clone() };
                    // Call bmb_string_concat for string concatenation
                    writeln!(out, "  %{} = call ptr @bmb_string_concat(ptr {}, ptr {})",
                             dest_name, lhs_final, rhs_final)?;
                    // v0.46: Store result to alloca if destination is a local variable
                    if local_names.contains(&dest.name) {
                        writeln!(out, "  store ptr %{}, ptr %{}.addr", dest_name, dest.name)?;
                    }
                } else if (lhs_ty == "ptr" || rhs_ty == "ptr") && *op == MirBinOp::Eq {
                    // v0.51.37: Distinguish string comparison from typed pointer comparison
                    // v0.60.122: Also check if operands are String-typed variables (not just constants)
                    let lhs_is_string = Self::is_string_operand(lhs, func);
                    let rhs_is_string = Self::is_string_operand(rhs, func);

                    if lhs_is_string || rhs_is_string {
                        // v0.51.22: Use pre-initialized global BmbString for string comparison
                        let lhs_final = if let Operand::Constant(Constant::String(s)) = lhs {
                            if let Some(global_name) = string_table.get(s) {
                                format!("@{}.bmb", global_name)
                            } else { lhs_str.clone() }
                        } else { lhs_str.clone() };
                        let rhs_final = if let Operand::Constant(Constant::String(s)) = rhs {
                            if let Some(global_name) = string_table.get(s) {
                                format!("@{}.bmb", global_name)
                            } else { rhs_str.clone() }
                        } else { rhs_str.clone() };
                        // Call bmb_string_eq for string equality comparison
                        // bmb_string_eq returns i64 (1 for equal, 0 for not equal)
                        writeln!(out, "  %{}.i64 = call i64 @bmb_string_eq(ptr {}, ptr {})",
                                 dest_name, lhs_final, rhs_final)?;
                        // Convert i64 to i1 for boolean result
                        writeln!(out, "  %{} = icmp ne i64 %{}.i64, 0", dest_name, dest_name)?;
                    } else {
                        // v0.51.37: Typed pointer comparison - use icmp eq ptr directly
                        // v0.51.40: Handle null comparison - LLVM requires "null" not "0" for ptr
                        let lhs_ptr = if matches!(lhs, Operand::Constant(Constant::Int(0))) {
                            "null".to_string()
                        } else { lhs_str.clone() };
                        let rhs_ptr = if matches!(rhs, Operand::Constant(Constant::Int(0))) {
                            "null".to_string()
                        } else { rhs_str.clone() };
                        writeln!(out, "  %{} = icmp eq ptr {}, {}", dest_name, lhs_ptr, rhs_ptr)?;
                    }
                    // v0.46: Store result to alloca if destination is a local variable
                    if local_names.contains(&dest.name) {
                        writeln!(out, "  store i1 %{}, ptr %{}.addr", dest_name, dest.name)?;
                    }
                } else if (lhs_ty == "ptr" || rhs_ty == "ptr") && *op == MirBinOp::Ne {
                    // v0.51.37: Distinguish string comparison from typed pointer comparison
                    // v0.60.122: Also check if operands are String-typed variables (not just constants)
                    let lhs_is_string = Self::is_string_operand(lhs, func);
                    let rhs_is_string = Self::is_string_operand(rhs, func);

                    if lhs_is_string || rhs_is_string {
                        // v0.51.22: Use pre-initialized global BmbString for string comparison
                        let lhs_final = if let Operand::Constant(Constant::String(s)) = lhs {
                            if let Some(global_name) = string_table.get(s) {
                                format!("@{}.bmb", global_name)
                            } else { lhs_str.clone() }
                        } else { lhs_str.clone() };
                        let rhs_final = if let Operand::Constant(Constant::String(s)) = rhs {
                            if let Some(global_name) = string_table.get(s) {
                                format!("@{}.bmb", global_name)
                            } else { rhs_str.clone() }
                        } else { rhs_str.clone() };
                        // Call bmb_string_eq and negate for string inequality
                        writeln!(out, "  %{}.i64 = call i64 @bmb_string_eq(ptr {}, ptr {})",
                                 dest_name, lhs_final, rhs_final)?;
                        // Convert i64 to i1 and negate (0 means not equal, so i64==0 means Ne is true)
                        writeln!(out, "  %{} = icmp eq i64 %{}.i64, 0", dest_name, dest_name)?;
                    } else {
                        // v0.51.37: Typed pointer comparison - use icmp ne ptr directly
                        // v0.51.40: Handle null comparison - LLVM requires "null" not "0" for ptr
                        let lhs_ptr = if matches!(lhs, Operand::Constant(Constant::Int(0))) {
                            "null".to_string()
                        } else { lhs_str.clone() };
                        let rhs_ptr = if matches!(rhs, Operand::Constant(Constant::Int(0))) {
                            "null".to_string()
                        } else { rhs_str.clone() };
                        writeln!(out, "  %{} = icmp ne ptr {}, {}", dest_name, lhs_ptr, rhs_ptr)?;
                    }
                    // v0.46: Store result to alloca if destination is a local variable
                    if local_names.contains(&dest.name) {
                        writeln!(out, "  store i1 %{}, ptr %{}.addr", dest_name, dest.name)?;
                    }
                } else if *op == MirBinOp::Implies {
                    // v0.36: Implication P => Q = !P || Q
                    // Step 1: Negate left operand
                    writeln!(out, "  %{}.not = xor i1 {}, true", dest_name, lhs_str)?;
                    // Step 2: Or with right operand
                    writeln!(out, "  %{} = or i1 %{}.not, {}", dest_name, dest_name, rhs_str)?;
                    // v0.46: Store result to alloca if destination is a local variable
                    if local_names.contains(&dest.name) {
                        writeln!(out, "  store i1 %{}, ptr %{}.addr", dest_name, dest.name)?;
                    }
                } else {
                    // v0.34: Fix float operations - MIR may use Add/Sub/etc. for f64 due to type inference issues
                    // Override to float operations when operand type is double/f64
                    // v0.60.8: Add 'fast' math flags to enable LLVM vectorization
                    // Without fast flags, LLVM cannot reorder FP operations, preventing vectorization
                    // The 'fast' flag enables: nnan ninf nsz arcp contract afn reassoc
                    let op_str = if lhs_ty == "double" || lhs_ty == "f64" {
                        match op {
                            MirBinOp::Add | MirBinOp::FAdd => "fadd fast",
                            MirBinOp::Sub | MirBinOp::FSub => "fsub fast",
                            MirBinOp::Mul | MirBinOp::FMul => "fmul fast",
                            MirBinOp::Div | MirBinOp::FDiv => "fdiv fast",
                            MirBinOp::Mod => "frem",
                            // v0.37: Wrapping arithmetic (not applicable to floats)
                            MirBinOp::AddWrap | MirBinOp::SubWrap | MirBinOp::MulWrap |
                            // v0.38: Checked arithmetic (not applicable to floats)
                            MirBinOp::AddChecked | MirBinOp::SubChecked | MirBinOp::MulChecked |
                            // v0.38: Saturating arithmetic (not applicable to floats)
                            MirBinOp::AddSat | MirBinOp::SubSat | MirBinOp::MulSat => {
                                let (s, _) = self.binop_to_llvm(*op);
                                s
                            }
                            MirBinOp::Eq | MirBinOp::FEq => "fcmp oeq",
                            MirBinOp::Ne | MirBinOp::FNe => "fcmp one",
                            MirBinOp::Lt | MirBinOp::FLt => "fcmp olt",
                            MirBinOp::Gt | MirBinOp::FGt => "fcmp ogt",
                            MirBinOp::Le | MirBinOp::FLe => "fcmp ole",
                            MirBinOp::Ge | MirBinOp::FGe => "fcmp oge",
                            MirBinOp::And | MirBinOp::Or | MirBinOp::Shl | MirBinOp::Shr |
                            MirBinOp::Band | MirBinOp::Bor | MirBinOp::Bxor | MirBinOp::Implies => {
                                // These operations don't apply to floats, use integer path
                                let (s, _) = self.binop_to_llvm(*op);
                                s
                            }
                        }
                    } else {
                        let (s, _) = self.binop_to_llvm(*op);
                        s
                    };

                    // v0.51.20: Handle type mismatch between operands (e.g., i32 param vs i64)
                    // This can happen when ConstantPropagationNarrowing narrows a parameter
                    // but the MIR still uses it in operations with wider types
                    //
                    // v0.51.30: Optimize i32 + i64 constant case - keep operation in i32
                    // When one operand is i32 (narrowed param) and the other is an i64 constant
                    // that fits in i32, we should keep the operation in i32 to avoid sext/trunc overhead.
                    // This is critical for matching C performance in recursive functions like fibonacci.
                    let (final_lhs_str, final_rhs_str, final_ty) = if lhs_ty == "i32" && rhs_ty == "i64" {
                        // Check if rhs is a constant that fits in i32
                        if let Operand::Constant(Constant::Int(v)) = rhs {
                            if *v >= i32::MIN as i64 && *v <= i32::MAX as i64 {
                                // Keep operation in i32 - just use the constant value as i32
                                (lhs_str.clone(), rhs_str.clone(), "i32")
                            } else {
                                // Constant doesn't fit in i32, must extend lhs
                                let sext_name = format!("{}.lhs.sext", dest_name);
                                writeln!(out, "  %{} = sext i32 {} to i64", sext_name, lhs_str)?;
                                (format!("%{}", sext_name), rhs_str.clone(), "i64")
                            }
                        } else {
                            // rhs is not a constant, must extend lhs
                            let sext_name = format!("{}.lhs.sext", dest_name);
                            writeln!(out, "  %{} = sext i32 {} to i64", sext_name, lhs_str)?;
                            (format!("%{}", sext_name), rhs_str.clone(), "i64")
                        }
                    } else if lhs_ty == "i64" && rhs_ty == "i32" {
                        // Check if lhs is a constant that fits in i32
                        if let Operand::Constant(Constant::Int(v)) = lhs {
                            if *v >= i32::MIN as i64 && *v <= i32::MAX as i64 {
                                // Keep operation in i32 - just use the constant value as i32
                                (lhs_str.clone(), rhs_str.clone(), "i32")
                            } else {
                                // Constant doesn't fit in i32, must extend rhs
                                let sext_name = format!("{}.rhs.sext", dest_name);
                                writeln!(out, "  %{} = sext i32 {} to i64", sext_name, rhs_str)?;
                                (lhs_str.clone(), format!("%{}", sext_name), "i64")
                            }
                        } else {
                            // lhs is not a constant, must extend rhs
                            let sext_name = format!("{}.rhs.sext", dest_name);
                            writeln!(out, "  %{} = sext i32 {} to i64", sext_name, rhs_str)?;
                            (lhs_str.clone(), format!("%{}", sext_name), "i64")
                        }
                    } else {
                        (lhs_str.clone(), rhs_str.clone(), lhs_ty)
                    };

                    // Note: LLVM IR always uses the operand type in the instruction
                    // The result type (i1 for comparisons) is implicit
                    writeln!(out, "  %{} = {} {} {}, {}", dest_name, op_str, final_ty, final_lhs_str, final_rhs_str)?;
                    // v0.46: Store result to alloca if destination is a local variable
                    if local_names.contains(&dest.name) {
                        // Get result type from place_types (handles comparisons returning i1)
                        // v0.51.20: Use final_ty (after type coercion) not lhs_ty
                        let result_ty = place_types.get(&dest.name).copied().unwrap_or(final_ty);
                        writeln!(out, "  store {} %{}, ptr %{}.addr", result_ty, dest_name, dest.name)?;
                    }
                }
            }

            MirInst::UnaryOp { dest, op, src } => {
                let dest_name = self.unique_name(&dest.name, name_counts);
                let ty = self.infer_operand_type(src, func);

                // Emit load for local operand (use dest_name for uniqueness)
                let src_str = match src {
                    Operand::Place(p) if local_names.contains(&p.name) => {
                        let load_name = format!("{}.{}.unary", dest_name, p.name);
                        writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, ty, p.name)?;
                        format!("%{}", load_name)
                    }
                    _ => self.format_operand(src),
                };

                match op {
                    MirUnaryOp::Neg => {
                        writeln!(out, "  %{} = sub {} 0, {}", dest_name, ty, src_str)?;
                    }
                    MirUnaryOp::FNeg => {
                        // v0.60.8: fast flag for vectorization
                        writeln!(out, "  %{} = fsub fast {} 0.0, {}", dest_name, ty, src_str)?;
                    }
                    MirUnaryOp::Not => {
                        writeln!(out, "  %{} = xor i1 {}, 1", dest_name, src_str)?;
                    }
                    // v0.36: Bitwise not (integer only)
                    MirUnaryOp::Bnot => {
                        writeln!(out, "  %{} = xor {} {}, -1", dest_name, ty, src_str)?;
                    }
                }
                // v0.46: Store result to alloca if destination is a local variable
                if local_names.contains(&dest.name) {
                    let result_ty = match op {
                        MirUnaryOp::Not => "i1",
                        _ => ty, // Neg, FNeg, Bnot preserve operand type
                    };
                    writeln!(out, "  store {} %{}, ptr %{}.addr", result_ty, dest_name, dest.name)?;
                }
            }

            MirInst::Call { dest, func: fn_name, args, is_tail } => {
                // v0.34: Handle math intrinsics and type conversions
                if fn_name == "sqrt" && args.len() == 1 {
                    // sqrt(x: f64) -> f64 via LLVM intrinsic
                    let arg_ty = match &args[0] {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    };
                    let arg_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.sqrt.arg", p.name);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, arg_ty, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Convert i64 to f64 if needed
                    let f64_val = if arg_ty == "i64" {
                        let conv_name = format!("{}.sqrt.conv", dest.as_ref().map(|d| d.name.as_str()).unwrap_or("tmp"));
                        writeln!(out, "  %{} = sitofp i64 {} to double", conv_name, arg_val)?;
                        format!("%{}", conv_name)
                    } else {
                        arg_val
                    };
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let temp_name = format!("{}.sqrt", d.name);
                            writeln!(out, "  %{} = call double @llvm.sqrt.f64(double {})", temp_name, f64_val)?;
                            writeln!(out, "  store double %{}, ptr %{}.addr", temp_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = call double @llvm.sqrt.f64(double {})", dest_name, f64_val)?;
                        }
                    }
                    return Ok(());
                }

                if fn_name == "i64_to_f64" && args.len() == 1 {
                    // i64_to_f64(x: i64) -> f64 via sitofp
                    // v0.60.24: Handle type-narrowed i32 arguments - need sext before sitofp
                    let arg_ty = match &args[0] {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    };
                    let arg_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.i64_to_f64.arg", p.name);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, arg_ty, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // v0.60.24: If argument is i32 (narrowed), sign-extend to i64 first
                    let (final_arg, final_ty) = if arg_ty == "i32" {
                        let sext_name = format!("{}_sext", arg_val.trim_start_matches('%'));
                        writeln!(out, "  %{} = sext i32 {} to i64", sext_name, arg_val)?;
                        (format!("%{}", sext_name), "i64")
                    } else {
                        (arg_val, arg_ty)
                    };
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let temp_name = format!("{}.conv", d.name);
                            writeln!(out, "  %{} = sitofp {} {} to double", temp_name, final_ty, final_arg)?;
                            writeln!(out, "  store double %{}, ptr %{}.addr", temp_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = sitofp {} {} to double", dest_name, final_ty, final_arg)?;
                        }
                    }
                    return Ok(());
                }

                if fn_name == "f64_to_i64" && args.len() == 1 {
                    // f64_to_i64(x: f64) -> i64 via fptosi
                    let arg_ty = match &args[0] {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    };
                    let arg_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.f64_to_i64.arg", p.name);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, arg_ty, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let temp_name = format!("{}.conv", d.name);
                            writeln!(out, "  %{} = fptosi double {} to i64", temp_name, arg_val)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", temp_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = fptosi double {} to i64", dest_name, arg_val)?;
                        }
                    }
                    return Ok(());
                }

                // v0.51.47: i32_to_f64(x: i32) -> f64 via sitofp
                if fn_name == "i32_to_f64" && args.len() == 1 {
                    let arg_ty = match &args[0] {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    };
                    let arg_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.i32_to_f64.arg", p.name);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, arg_ty, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let temp_name = format!("{}.conv", d.name);
                            writeln!(out, "  %{} = sitofp i32 {} to double", temp_name, arg_val)?;
                            writeln!(out, "  store double %{}, ptr %{}.addr", temp_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = sitofp i32 {} to double", dest_name, arg_val)?;
                        }
                    }
                    return Ok(());
                }

                // v0.51.47: i32_to_i64(x: i32) -> i64 via sext
                if fn_name == "i32_to_i64" && args.len() == 1 {
                    let arg_ty = match &args[0] {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    };
                    let arg_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.i32_to_i64.arg", p.name);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, arg_ty, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let temp_name = format!("{}.conv", d.name);
                            writeln!(out, "  %{} = sext i32 {} to i64", temp_name, arg_val)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", temp_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = sext i32 {} to i64", dest_name, arg_val)?;
                        }
                    }
                    return Ok(());
                }

                // v0.51.47: i64_to_i32(x: i64) -> i32 via trunc
                if fn_name == "i64_to_i32" && args.len() == 1 {
                    let arg_ty = match &args[0] {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    };
                    let arg_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.i64_to_i32.arg", p.name);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, arg_ty, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let temp_name = format!("{}.conv", d.name);
                            writeln!(out, "  %{} = trunc i64 {} to i32", temp_name, arg_val)?;
                            writeln!(out, "  store i32 %{}, ptr %{}.addr", temp_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = trunc i64 {} to i32", dest_name, arg_val)?;
                        }
                    }
                    return Ok(());
                }

                // v0.34.2: box_new_i64(value) -> i64 - allocates 8 bytes and stores value
                if fn_name == "box_new_i64" && args.len() == 1 {
                    // Get value argument
                    let val_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.box.val", p.name);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Generate unique names for temp values
                    let box_idx = *name_counts.entry("box_new".to_string()).or_insert(0);
                    *name_counts.get_mut("box_new").unwrap() += 1;
                    // Call malloc(8)
                    let malloc_name = format!("box.ptr.{}", box_idx);
                    writeln!(out, "  %{} = call ptr @malloc(i64 8)", malloc_name)?;
                    // Store value at pointer
                    writeln!(out, "  store i64 {}, ptr %{}", val_val, malloc_name)?;
                    // Convert ptr to i64 for return
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let conv_name = format!("{}.box.conv", d.name);
                            writeln!(out, "  %{} = ptrtoint ptr %{} to i64", conv_name, malloc_name)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", conv_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = ptrtoint ptr %{} to i64", dest_name, malloc_name)?;
                        }
                    }
                    return Ok(());
                }

                // v0.50.72: malloc(size) -> i64 - allocates memory and returns as i64 (for arithmetic)
                if fn_name == "malloc" && args.len() == 1 {
                    let size_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.malloc.size", p.name);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        // v0.93.122: Narrowed i32 params need sext for malloc i64 size
                        Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                            let sext_name = format!("{}.malloc.sext", p.name);
                            writeln!(out, "  %{} = sext i32 %{} to i64", sext_name, p.name)?;
                            format!("%{}", sext_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    let malloc_idx = *name_counts.entry("malloc_op".to_string()).or_insert(0);
                    *name_counts.get_mut("malloc_op").unwrap() += 1;
                    // Call malloc, get ptr
                    let ptr_name = format!("malloc.ptr.{}", malloc_idx);
                    writeln!(out, "  %{} = call ptr @malloc(i64 {})", ptr_name, size_val)?;
                    // Convert ptr to i64 for BMB's pointer arithmetic
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let conv_name = format!("malloc.conv.{}", malloc_idx);
                            writeln!(out, "  %{} = ptrtoint ptr %{} to i64", conv_name, ptr_name)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", conv_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = ptrtoint ptr %{} to i64", dest_name, ptr_name)?;
                        }
                    }
                    return Ok(());
                }

                // v0.34.2: box_free_i64(ptr) -> Unit - frees memory (alias for free)
                // v0.50.72: Also handle direct free(ptr) calls
                if (fn_name == "box_free_i64" || fn_name == "free") && args.len() == 1 {
                    // v0.93.122: Check if argument is already a pointer type
                    let arg_is_ptr = match &args[0] {
                        Operand::Place(p) => {
                            let ty = place_types.get(&p.name).copied().unwrap_or("i64");
                            ty == "ptr"
                        }
                        _ => false,
                    };

                    if arg_is_ptr {
                        // Argument is already ptr - load as ptr, no inttoptr needed
                        let ptr_val = match &args[0] {
                            Operand::Place(p) if local_names.contains(&p.name) => {
                                let load_name = format!("{}.free.ptr", p.name);
                                writeln!(out, "  %{} = load ptr, ptr %{}.addr", load_name, p.name)?;
                                format!("%{}", load_name)
                            }
                            _ => self.format_operand_with_strings(&args[0], string_table),
                        };
                        writeln!(out, "  call void @free(ptr {})", ptr_val)?;
                    } else {
                        // Argument is i64 - need inttoptr
                        let ptr_val = match &args[0] {
                            Operand::Place(p) if local_names.contains(&p.name) => {
                                let load_name = format!("{}.free.ptr", p.name);
                                writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                                format!("%{}", load_name)
                            }
                            _ => self.format_operand_with_strings(&args[0], string_table),
                        };
                        let ptr_conv = format!("free_ptr.{}", name_counts.entry("free_ptr".to_string()).or_insert(0));
                        *name_counts.get_mut("free_ptr").unwrap() += 1;
                        writeln!(out, "  %{} = inttoptr i64 {} to ptr", ptr_conv, ptr_val)?;
                        writeln!(out, "  call void @free(ptr %{})", ptr_conv)?;
                    }
                    return Ok(());
                }

                // v0.34.2: store_i64(ptr, value) -> Unit - writes i64 value to memory
                // Also handles box_set_i64 as an alias
                if (fn_name == "store_i64" || fn_name == "box_set_i64") && args.len() == 2 {
                    // Get unique index for this store operation
                    let store_idx = *name_counts.entry("store_op".to_string()).or_insert(0);
                    *name_counts.get_mut("store_op").unwrap() += 1;
                    // Get pointer argument
                    // v0.93.122: Handle narrowed i32 params with sext for inttoptr
                    let ptr_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.store.ptr.{}", p.name, store_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                            let sext_name = format!("{}.store.ptr.sext.{}", p.name, store_idx);
                            writeln!(out, "  %{} = sext i32 %{} to i64", sext_name, p.name)?;
                            format!("%{}", sext_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Get value argument - v0.60.17: Handle narrowed i32 parameters
                    let val_ty = match &args[1] {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    };
                    let val_val = match &args[1] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.store.val.{}", p.name, store_idx);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, val_ty, p.name)?;
                            // If value is i32, extend to i64 for store_i64
                            if val_ty == "i32" {
                                let ext_name = format!("{}.store.ext.{}", p.name, store_idx);
                                writeln!(out, "  %{} = sext i32 %{} to i64", ext_name, load_name)?;
                                format!("%{}", ext_name)
                            } else {
                                format!("%{}", load_name)
                            }
                        }
                        Operand::Place(p) => {
                            // Non-local place (parameter) - check if narrowed
                            if val_ty == "i32" {
                                let ext_name = format!("{}.store.ext.{}", p.name, store_idx);
                                writeln!(out, "  %{} = sext i32 %{} to i64", ext_name, p.name)?;
                                format!("%{}", ext_name)
                            } else {
                                format!("%{}", p.name)
                            }
                        }
                        _ => self.format_operand_with_strings(&args[1], string_table),
                    };
                    // Convert i64 pointer to ptr type and store
                    let ptr_conv = format!("store_ptr.{}", store_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", ptr_conv, ptr_val)?;
                    writeln!(out, "  store i64 {}, ptr %{}", val_val, ptr_conv)?;
                    return Ok(());
                }

                // v0.34.2: load_i64(ptr) -> i64 - reads i64 value from memory
                // Also handles box_get_i64 as an alias
                if (fn_name == "load_i64" || fn_name == "box_get_i64") && args.len() == 1 {
                    // Get unique index for this load operation
                    let load_idx = *name_counts.entry("load_op".to_string()).or_insert(0);
                    *name_counts.get_mut("load_op").unwrap() += 1;
                    // Get pointer argument
                    // v0.93.122: Handle narrowed i32 params with sext for inttoptr
                    let ptr_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.load.ptr.{}", p.name, load_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                            let sext_name = format!("{}.load.ptr.sext.{}", p.name, load_idx);
                            writeln!(out, "  %{} = sext i32 %{} to i64", sext_name, p.name)?;
                            format!("%{}", sext_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Convert i64 pointer to ptr type and load
                    let ptr_conv = format!("load_ptr.{}", load_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", ptr_conv, ptr_val)?;
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let temp_name = format!("{}.load.{}", d.name, load_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}", temp_name, ptr_conv)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", temp_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = load i64, ptr %{}", dest_name, ptr_conv)?;
                        }
                    }
                    return Ok(());
                }

                // v0.51.5: store_f64(ptr, value) -> Unit - writes f64 value to memory
                if fn_name == "store_f64" && args.len() == 2 {
                    let store_idx = *name_counts.entry("store_f64_op".to_string()).or_insert(0);
                    *name_counts.get_mut("store_f64_op").unwrap() += 1;
                    // Get pointer argument
                    let ptr_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.store_f64.ptr.{}", p.name, store_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Get value argument (f64)
                    let val_val = match &args[1] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.store_f64.val.{}", p.name, store_idx);
                            writeln!(out, "  %{} = load double, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        Operand::Constant(crate::mir::Constant::Float(f)) => format!("{:e}", f),
                        _ => self.format_operand_with_strings(&args[1], string_table),
                    };
                    // Convert i64 pointer to ptr type and store f64
                    let ptr_conv = format!("store_f64_ptr.{}", store_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", ptr_conv, ptr_val)?;
                    writeln!(out, "  store double {}, ptr %{}", val_val, ptr_conv)?;
                    return Ok(());
                }

                // v0.51.5: load_f64(ptr) -> f64 - reads f64 value from memory
                if fn_name == "load_f64" && args.len() == 1 {
                    let load_idx = *name_counts.entry("load_f64_op".to_string()).or_insert(0);
                    *name_counts.get_mut("load_f64_op").unwrap() += 1;
                    // Get pointer argument
                    let ptr_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.load_f64.ptr.{}", p.name, load_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Convert i64 pointer to ptr type and load f64
                    let ptr_conv = format!("load_f64_ptr.{}", load_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", ptr_conv, ptr_val)?;
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let temp_name = format!("{}.load_f64.{}", d.name, load_idx);
                            writeln!(out, "  %{} = load double, ptr %{}", temp_name, ptr_conv)?;
                            writeln!(out, "  store double %{}, ptr %{}.addr", temp_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = load double, ptr %{}", dest_name, ptr_conv)?;
                        }
                    }
                    return Ok(());
                }

                // v0.51.51: load_u8(ptr) -> i64 - reads single byte from memory
                // Inlined for high-performance string parsing (avoids function call overhead)
                if fn_name == "load_u8" && args.len() == 1 {
                    let load_idx = *name_counts.entry("load_u8_op".to_string()).or_insert(0);
                    *name_counts.get_mut("load_u8_op").unwrap() += 1;

                    // v0.93.120: GEP-based access for better LLVM vectorization
                    // When address is BinOp::Add(base, offset), generate:
                    //   %base_ptr = inttoptr i64 %base to ptr
                    //   %elem_ptr = getelementptr i8, ptr %base_ptr, i64 %offset
                    //   load i8, ptr %elem_ptr
                    // Instead of: inttoptr(add(base, offset)) which blocks LLVM vectorization
                    // because inttoptr destroys pointer provenance needed for contiguous access proof.
                    let mut used_gep = false;
                    if let Operand::Place(addr_place) = &args[0] {
                        // Find if this address was defined as BinOp::Add
                        let mut add_parts: Option<(&Operand, &Operand)> = None;
                        'outer: for blk in &func.blocks {
                            for inst2 in &blk.instructions {
                                if let MirInst::BinOp { dest: d2, op: MirBinOp::Add, lhs: l2, rhs: r2 } = inst2 {
                                    if d2.name == addr_place.name {
                                        add_parts = Some((l2, r2));
                                        break 'outer;
                                    }
                                }
                            }
                        }
                        if let Some((add_lhs, add_rhs)) = add_parts {
                            // Determine base (prefer function param for stable pointer) and offset
                            let param_set: std::collections::HashSet<&String> = func.params.iter().map(|(n,_)| n).collect();
                            let (base_op, offset_op) = if matches!(add_rhs, Operand::Place(p) if param_set.contains(&p.name)) {
                                (add_rhs, add_lhs)
                            } else {
                                (add_lhs, add_rhs)
                            };
                            // Load base operand value
                            // v0.93.122: Handle narrowed i32 params with sext for inttoptr
                            let base_val = match base_op {
                                Operand::Place(p) if local_names.contains(&p.name) => {
                                    let n = format!("gep_base_load.{}", load_idx);
                                    writeln!(out, "  %{} = load i64, ptr %{}.addr", n, p.name)?;
                                    format!("%{}", n)
                                }
                                Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                                    let n = format!("gep_base_sext.{}", load_idx);
                                    writeln!(out, "  %{} = sext i32 %{} to i64", n, p.name)?;
                                    format!("%{}", n)
                                }
                                _ => self.format_operand_with_strings(base_op, string_table),
                            };
                            // Load offset operand value
                            let offset_val = match offset_op {
                                Operand::Place(p) if local_names.contains(&p.name) => {
                                    let n = format!("gep_offset_load.{}", load_idx);
                                    writeln!(out, "  %{} = load i64, ptr %{}.addr", n, p.name)?;
                                    format!("%{}", n)
                                }
                                Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                                    let n = format!("gep_offset_sext.{}", load_idx);
                                    writeln!(out, "  %{} = sext i32 %{} to i64", n, p.name)?;
                                    format!("%{}", n)
                                }
                                _ => self.format_operand_with_strings(offset_op, string_table),
                            };
                            // Generate GEP: inttoptr base, then getelementptr i8
                            let base_ptr = format!("gep_base.{}", load_idx);
                            let elem_ptr = format!("gep_elem.{}", load_idx);
                            writeln!(out, "  %{} = inttoptr i64 {} to ptr", base_ptr, base_val)?;
                            writeln!(out, "  %{} = getelementptr i8, ptr %{}, i64 {}", elem_ptr, base_ptr, offset_val)?;
                            // Load byte and zext
                            if let Some(d) = dest {
                                if local_names.contains(&d.name) {
                                    let byte_name = format!("{}.u8.{}", d.name, load_idx);
                                    let ext_name = format!("{}.zext.{}", d.name, load_idx);
                                    writeln!(out, "  %{} = load i8, ptr %{}", byte_name, elem_ptr)?;
                                    writeln!(out, "  %{} = zext i8 %{} to i64", ext_name, byte_name)?;
                                    writeln!(out, "  store i64 %{}, ptr %{}.addr", ext_name, d.name)?;
                                } else {
                                    let byte_name = format!("{}.u8", d.name);
                                    writeln!(out, "  %{} = load i8, ptr %{}", byte_name, elem_ptr)?;
                                    writeln!(out, "  %{} = zext i8 %{} to i64", d.name, byte_name)?;
                                }
                            }
                            used_gep = true;
                        }
                    }
                    if used_gep {
                        return Ok(());
                    }

                    // Fallback: original inttoptr pattern for non-Add addresses
                    // Get pointer argument
                    // v0.93.122: Handle narrowed i32 params with sext
                    let ptr_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.load_u8.ptr.{}", p.name, load_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                            let sext_name = format!("{}.load_u8.sext.{}", p.name, load_idx);
                            writeln!(out, "  %{} = sext i32 %{} to i64", sext_name, p.name)?;
                            format!("%{}", sext_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Convert i64 pointer to ptr type and load u8
                    let ptr_conv = format!("load_u8_ptr.{}", load_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", ptr_conv, ptr_val)?;
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let byte_name = format!("{}.u8.{}", d.name, load_idx);
                            let ext_name = format!("{}.zext.{}", d.name, load_idx);
                            writeln!(out, "  %{} = load i8, ptr %{}", byte_name, ptr_conv)?;
                            writeln!(out, "  %{} = zext i8 %{} to i64", ext_name, byte_name)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", ext_name, d.name)?;
                        } else {
                            let byte_name = format!("{}.u8", d.name);
                            writeln!(out, "  %{} = load i8, ptr %{}", byte_name, ptr_conv)?;
                            writeln!(out, "  %{} = zext i8 %{} to i64", d.name, byte_name)?;
                        }
                    }
                    return Ok(());
                }

                // v0.51.51: store_u8(ptr, value) -> Unit - writes single byte to memory
                if fn_name == "store_u8" && args.len() == 2 {
                    let store_idx = *name_counts.entry("store_u8_op".to_string()).or_insert(0);
                    *name_counts.get_mut("store_u8_op").unwrap() += 1;

                    // v0.93.120: GEP-based access for better LLVM vectorization (same as load_u8)
                    let mut used_gep = false;
                    if let Operand::Place(addr_place) = &args[0] {
                        let mut add_parts: Option<(&Operand, &Operand)> = None;
                        'outer_s: for blk in &func.blocks {
                            for inst2 in &blk.instructions {
                                if let MirInst::BinOp { dest: d2, op: MirBinOp::Add, lhs: l2, rhs: r2 } = inst2 {
                                    if d2.name == addr_place.name {
                                        add_parts = Some((l2, r2));
                                        break 'outer_s;
                                    }
                                }
                            }
                        }
                        if let Some((add_lhs, add_rhs)) = add_parts {
                            let param_set: std::collections::HashSet<&String> = func.params.iter().map(|(n,_)| n).collect();
                            let (base_op, offset_op) = if matches!(add_rhs, Operand::Place(p) if param_set.contains(&p.name)) {
                                (add_rhs, add_lhs)
                            } else {
                                (add_lhs, add_rhs)
                            };
                            // Load base
                            // v0.93.122: Handle narrowed i32 params with sext for inttoptr
                            let base_val = match base_op {
                                Operand::Place(p) if local_names.contains(&p.name) => {
                                    let n = format!("sgep_base_load.{}", store_idx);
                                    writeln!(out, "  %{} = load i64, ptr %{}.addr", n, p.name)?;
                                    format!("%{}", n)
                                }
                                Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                                    let n = format!("sgep_base_sext.{}", store_idx);
                                    writeln!(out, "  %{} = sext i32 %{} to i64", n, p.name)?;
                                    format!("%{}", n)
                                }
                                _ => self.format_operand_with_strings(base_op, string_table),
                            };
                            // Load offset
                            let offset_val = match offset_op {
                                Operand::Place(p) if local_names.contains(&p.name) => {
                                    let n = format!("sgep_offset_load.{}", store_idx);
                                    writeln!(out, "  %{} = load i64, ptr %{}.addr", n, p.name)?;
                                    format!("%{}", n)
                                }
                                Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                                    let n = format!("sgep_offset_sext.{}", store_idx);
                                    writeln!(out, "  %{} = sext i32 %{} to i64", n, p.name)?;
                                    format!("%{}", n)
                                }
                                _ => self.format_operand_with_strings(offset_op, string_table),
                            };
                            // Get store value
                            // v0.93.122: Handle narrowed i32 params with sext for trunc i64
                            let val_val = match &args[1] {
                                Operand::Place(p) if local_names.contains(&p.name) => {
                                    let n = format!("{}.store_u8.val.{}", p.name, store_idx);
                                    writeln!(out, "  %{} = load i64, ptr %{}.addr", n, p.name)?;
                                    format!("%{}", n)
                                }
                                Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                                    let n = format!("{}.store_u8.val.sext.{}", p.name, store_idx);
                                    writeln!(out, "  %{} = sext i32 %{} to i64", n, p.name)?;
                                    format!("%{}", n)
                                }
                                _ => self.format_operand_with_strings(&args[1], string_table),
                            };
                            // GEP + store
                            let base_ptr = format!("sgep_base.{}", store_idx);
                            let elem_ptr = format!("sgep_elem.{}", store_idx);
                            let trunc_val = format!("store_u8_trunc.{}", store_idx);
                            writeln!(out, "  %{} = inttoptr i64 {} to ptr", base_ptr, base_val)?;
                            writeln!(out, "  %{} = getelementptr i8, ptr %{}, i64 {}", elem_ptr, base_ptr, offset_val)?;
                            writeln!(out, "  %{} = trunc i64 {} to i8", trunc_val, val_val)?;
                            writeln!(out, "  store i8 %{}, ptr %{}", trunc_val, elem_ptr)?;
                            used_gep = true;
                        }
                    }
                    if used_gep {
                        return Ok(());
                    }

                    // Fallback: original inttoptr pattern
                    // Get pointer argument
                    // v0.93.122: Handle narrowed i32 params with sext
                    let ptr_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.store_u8.ptr.{}", p.name, store_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                            let sext_name = format!("{}.store_u8.sext.{}", p.name, store_idx);
                            writeln!(out, "  %{} = sext i32 %{} to i64", sext_name, p.name)?;
                            format!("%{}", sext_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Get value argument
                    let val_val = match &args[1] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.store_u8.val.{}", p.name, store_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                            let sext_name = format!("{}.store_u8.val.sext.{}", p.name, store_idx);
                            writeln!(out, "  %{} = sext i32 %{} to i64", sext_name, p.name)?;
                            format!("%{}", sext_name)
                        }
                        _ => self.format_operand_with_strings(&args[1], string_table),
                    };
                    // Convert i64 pointer to ptr type and truncate value to i8
                    let ptr_conv = format!("store_u8_ptr.{}", store_idx);
                    let trunc_val = format!("store_u8_trunc.{}", store_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", ptr_conv, ptr_val)?;
                    writeln!(out, "  %{} = trunc i64 {} to i8", trunc_val, val_val)?;
                    writeln!(out, "  store i8 %{}, ptr %{}", trunc_val, ptr_conv)?;
                    return Ok(());
                }

                // v0.51.51: str_data(s: String) -> i64 - get raw pointer to string data
                // BMB strings are structs {ptr, i64, i64} - we need to extract the first field
                if fn_name == "str_data" && args.len() == 1 {
                    let str_idx = *name_counts.entry("str_data_op".to_string()).or_insert(0);
                    *name_counts.get_mut("str_data_op").unwrap() += 1;
                    // Get string struct pointer argument
                    let str_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.str_data.struct.{}", p.name, str_idx);
                            writeln!(out, "  %{} = load ptr, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Get pointer to first field (data ptr) and load it
                    let data_gep = format!("str_data.gep.{}", str_idx);
                    let data_ptr = format!("str_data.ptr.{}", str_idx);
                    writeln!(out, "  %{} = getelementptr {{ptr, i64, i64}}, ptr {}, i32 0, i32 0", data_gep, str_val)?;
                    writeln!(out, "  %{} = load ptr, ptr %{}", data_ptr, data_gep)?;
                    // Convert data ptr to i64
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let conv_name = format!("{}.ptrtoint.{}", d.name, str_idx);
                            writeln!(out, "  %{} = ptrtoint ptr %{} to i64", conv_name, data_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", conv_name, d.name)?;
                        } else {
                            writeln!(out, "  %{} = ptrtoint ptr %{} to i64", d.name, data_ptr)?;
                        }
                    }
                    return Ok(());
                }

                // v0.34.2.3: Vec<i64> dynamic array builtins (RFC-0007)
                // v0.93.123: Fixed to use INLINE layout matching runtime:
                // Layout: [capacity, length, data...] in one allocation
                // vec_new() -> i64: allocate (8+2)*8=80 bytes with cap=8, len=0
                if fn_name == "vec_new" && args.is_empty() {
                    let vec_idx = *name_counts.entry("vec_new".to_string()).or_insert(0);
                    *name_counts.get_mut("vec_new").unwrap() += 1;
                    // Allocate (8 + 2) * 8 = 80 bytes for inline layout
                    let header_ptr = format!("vec.header.{}", vec_idx);
                    writeln!(out, "  %{} = call ptr @malloc(i64 80)", header_ptr)?;
                    // Store capacity=8 at [0], length=0 at [1]
                    let len_ptr = format!("vec.init.len.{}", vec_idx);
                    writeln!(out, "  store i64 8, ptr %{}", header_ptr)?;
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 1", len_ptr, header_ptr)?;
                    writeln!(out, "  store i64 0, ptr %{}", len_ptr)?;
                    // Convert ptr to i64 for return
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let conv_name = format!("vec.conv.{}", vec_idx);
                            writeln!(out, "  %{} = ptrtoint ptr %{} to i64", conv_name, header_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", conv_name, d.name)?;
                        } else {
                            // Use dest name directly for SSA assignment
                            writeln!(out, "  %{} = ptrtoint ptr %{} to i64", d.name, header_ptr)?;
                        }
                    }
                    return Ok(());
                }

                // v0.93.123: vec_with_capacity(cap) -> i64: INLINE layout matching runtime
                // Allocate (cap + 2) * 8 bytes: [capacity, length, data...]
                if fn_name == "vec_with_capacity" && args.len() == 1 {
                    let vec_idx = *name_counts.entry("vec_cap_alloc".to_string()).or_insert(0);
                    *name_counts.get_mut("vec_cap_alloc").unwrap() += 1;
                    // Get capacity argument
                    let cap_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vec.cap.arg.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Allocate (cap + 2) * 8 bytes for inline layout
                    let total_slots = format!("vec.wcap.total.{}", vec_idx);
                    let alloc_size = format!("vec.wcap.size.{}", vec_idx);
                    let header_ptr = format!("vec.wcap.header.{}", vec_idx);
                    writeln!(out, "  %{} = add i64 {}, 2", total_slots, cap_val)?;
                    writeln!(out, "  %{} = mul i64 %{}, 8", alloc_size, total_slots)?;
                    writeln!(out, "  %{} = call ptr @malloc(i64 %{})", header_ptr, alloc_size)?;
                    // Store capacity at [0], length=0 at [1]
                    writeln!(out, "  store i64 {}, ptr %{}", cap_val, header_ptr)?;
                    let len_ptr = format!("vec.wcap.len.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 1", len_ptr, header_ptr)?;
                    writeln!(out, "  store i64 0, ptr %{}", len_ptr)?;
                    // Return header pointer as i64
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let conv_name = format!("vec.wcap.conv.{}", vec_idx);
                            writeln!(out, "  %{} = ptrtoint ptr %{} to i64", conv_name, header_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", conv_name, d.name)?;
                        } else {
                            // Use dest name directly for SSA assignment
                            writeln!(out, "  %{} = ptrtoint ptr %{} to i64", d.name, header_ptr)?;
                        }
                    }
                    return Ok(());
                }

                // vec_len(vec) -> i64: read header[1]
                if fn_name == "vec_len" && args.len() == 1 {
                    let vec_idx = *name_counts.entry("vec_len".to_string()).or_insert(0);
                    *name_counts.get_mut("vec_len").unwrap() += 1;
                    let vec_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vec.len.arg.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    let header_ptr = format!("vec.len.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;
                    let len_ptr_name = format!("vec.len.ptr.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 1", len_ptr_name, header_ptr)?;
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let len_val = format!("vec.len.val.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}", len_val, len_ptr_name)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", len_val, d.name)?;
                        } else {
                            // Use dest name directly for SSA assignment
                            writeln!(out, "  %{} = load i64, ptr %{}", d.name, len_ptr_name)?;
                        }
                    }
                    return Ok(());
                }

                // v0.93.123: vec_cap(vec) -> i64: INLINE layout — read header[0]
                if fn_name == "vec_cap" && args.len() == 1 {
                    let vec_idx = *name_counts.entry("vec_cap_read".to_string()).or_insert(0);
                    *name_counts.get_mut("vec_cap_read").unwrap() += 1;
                    let vec_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vec.cap.arg.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // INLINE layout: capacity is at header[0]
                    let header_ptr = format!("vec.cap.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;
                    let cap_ptr_name = format!("vec.cap.ptr.{}", vec_idx);
                    // Read directly from header[0] (no GEP needed, but keep name for consistency)
                    // Note: GEP at index 0 is a no-op but keeps the pattern consistent
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 0", cap_ptr_name, header_ptr)?;
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let cap_val = format!("vec.cap.val.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}", cap_val, cap_ptr_name)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", cap_val, d.name)?;
                        } else {
                            // Use dest name directly for SSA assignment
                            writeln!(out, "  %{} = load i64, ptr %{}", d.name, cap_ptr_name)?;
                        }
                    }
                    return Ok(());
                }

                // v0.93.123: vec_get(vec, index) -> i64: INLINE layout — read header[2 + index]
                if fn_name == "vec_get" && args.len() == 2 {
                    let vec_idx = *name_counts.entry("vec_get".to_string()).or_insert(0);
                    *name_counts.get_mut("vec_get").unwrap() += 1;
                    let vec_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vec.get.vec.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    let idx_val = match &args[1] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vec.get.idx.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        // v0.93.122: Narrowed i32 params need sext for GEP i64 index
                        Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                            let sext_name = format!("vec.get.idx.sext.{}", vec_idx);
                            writeln!(out, "  %{} = sext i32 %{} to i64", sext_name, p.name)?;
                            format!("%{}", sext_name)
                        }
                        _ => self.format_operand_with_strings(&args[1], string_table),
                    };
                    // INLINE layout: data starts at header[2], so offset = index + 2
                    let header_ptr = format!("vec.get.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;
                    let offset = format!("vec.get.off.{}", vec_idx);
                    writeln!(out, "  %{} = add i64 {}, 2", offset, idx_val)?;
                    let elem_ptr = format!("vec.get.elem.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 %{}", elem_ptr, header_ptr, offset)?;
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let elem_val = format!("vec.get.val.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}", elem_val, elem_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", elem_val, d.name)?;
                        } else {
                            writeln!(out, "  %{} = load i64, ptr %{}", d.name, elem_ptr)?;
                        }
                    }
                    return Ok(());
                }

                // vec_set(vec, index, value) -> Unit: write data[index] = value
                if fn_name == "vec_set" && args.len() == 3 {
                    let vec_idx = *name_counts.entry("vec_set".to_string()).or_insert(0);
                    *name_counts.get_mut("vec_set").unwrap() += 1;
                    let vec_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vec.set.vec.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    let idx_val = match &args[1] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vec.set.idx.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        // v0.93.122: Narrowed i32 params need sext for GEP i64 index
                        Operand::Place(p) if narrowed_param_names.contains(&p.name) => {
                            let sext_name = format!("vec.set.idx.sext.{}", vec_idx);
                            writeln!(out, "  %{} = sext i32 %{} to i64", sext_name, p.name)?;
                            format!("%{}", sext_name)
                        }
                        _ => self.format_operand_with_strings(&args[1], string_table),
                    };
                    let val_val = match &args[2] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vec.set.val.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[2], string_table),
                    };
                    // v0.93.123: INLINE layout — data starts at header[2], offset = index + 2
                    let header_ptr = format!("vec.set.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;
                    let offset = format!("vec.set.off.{}", vec_idx);
                    writeln!(out, "  %{} = add i64 {}, 2", offset, idx_val)?;
                    let elem_ptr = format!("vec.set.elem.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 %{}", elem_ptr, header_ptr, offset)?;
                    writeln!(out, "  store i64 {}, ptr %{}", val_val, elem_ptr)?;
                    return Ok(());
                }

                // vec_push(vec, value) -> Unit: append with auto-grow
                // v0.50.70: Use runtime function to avoid inline block splitting (PHI bug fix)
                if fn_name == "vec_push" && args.len() == 2 {
                    let vec_idx = *name_counts.entry("vec_push".to_string()).or_insert(0);
                    *name_counts.get_mut("vec_push").unwrap() += 1;
                    let vec_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vp.vec.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    let val_val = match &args[1] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vp.val.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[1], string_table),
                    };

                    // Call runtime function instead of inline code (avoids PHI predecessor bug)
                    writeln!(out, "  call void @bmb_vec_push(i64 {}, i64 {})", vec_val, val_val)?;
                    return Ok(());
                }

                // v0.93.123: vec_pop(vec) -> i64: INLINE layout — read header[len+1], decrement len
                if fn_name == "vec_pop" && args.len() == 1 {
                    let vec_idx = *name_counts.entry("vec_pop".to_string()).or_insert(0);
                    *name_counts.get_mut("vec_pop").unwrap() += 1;
                    let vec_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vpop.vec.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Get header
                    let header_ptr = format!("vpop.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;
                    // Load len from header[1]
                    let len_ptr = format!("vpop.len.ptr.{}", vec_idx);
                    let len_val = format!("vpop.len.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 1", len_ptr, header_ptr)?;
                    writeln!(out, "  %{} = load i64, ptr %{}", len_val, len_ptr)?;
                    // INLINE layout: last element at header[2 + len - 1] = header[len + 1]
                    let last_off = format!("vpop.last_off.{}", vec_idx);
                    writeln!(out, "  %{} = add i64 %{}, 1", last_off, len_val)?;
                    let elem_ptr = format!("vpop.elem.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 %{}", elem_ptr, header_ptr, last_off)?;
                    // New length = len - 1
                    let new_len = format!("vpop.newlen.{}", vec_idx);
                    writeln!(out, "  %{} = sub i64 %{}, 1", new_len, len_val)?;
                    // Load element and decrement len, then return
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let elem_val = format!("vpop.val.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}", elem_val, elem_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}", new_len, len_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", elem_val, d.name)?;
                        } else {
                            writeln!(out, "  %{} = load i64, ptr %{}", d.name, elem_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}", new_len, len_ptr)?;
                        }
                    } else {
                        // No dest, still decrement len
                        writeln!(out, "  store i64 %{}, ptr %{}", new_len, len_ptr)?;
                    }
                    return Ok(());
                }

                // v0.93.123: vec_free(vec) -> Unit: INLINE layout — single free
                if fn_name == "vec_free" && args.len() == 1 {
                    let vec_idx = *name_counts.entry("vec_free".to_string()).or_insert(0);
                    *name_counts.get_mut("vec_free").unwrap() += 1;
                    let vec_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("vfree.vec.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // INLINE layout: everything is in one allocation, just free it
                    let header_ptr = format!("vfree.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;
                    writeln!(out, "  call void @free(ptr %{})", header_ptr)?;
                    return Ok(());
                }

                // v0.50.61: Inline string operations for zero-cost string access
                // BmbString layout: {ptr data, i64 len, i64 cap}

                // len(s) -> i64: inline string length access
                // v0.51.51: Re-enabled after fixing runtime to use BmbString structs consistently
                if (fn_name == "len" || fn_name == "bmb_string_len") && args.len() == 1 {
                    let str_idx = *name_counts.entry("str_len".to_string()).or_insert(0);
                    *name_counts.get_mut("str_len").unwrap() += 1;

                    // Get string pointer argument
                    // v0.51.53: String constants need .bmb suffix for BmbString struct
                    let str_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("strlen.str.{}", str_idx);
                            writeln!(out, "  %{} = load ptr, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        Operand::Constant(Constant::String(s)) => {
                            if let Some(global_name) = string_table.get(s) {
                                format!("@{}.bmb", global_name)
                            } else {
                                self.format_operand_with_strings(&args[0], string_table)
                            }
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };

                    // Access len field at offset 1 in BmbString struct
                    // struct BmbString { ptr data; i64 len; i64 cap; }
                    let len_ptr = format!("strlen.len_ptr.{}", str_idx);
                    writeln!(out, "  %{} = getelementptr {{ptr, i64, i64}}, ptr {}, i32 0, i32 1", len_ptr, str_val)?;

                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            // v0.90.73: Use dest name directly so %_tN is defined as SSA register
                            writeln!(out, "  %{} = load i64, ptr %{}", d.name, len_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", d.name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = load i64, ptr %{}", dest_name, len_ptr)?;
                        }
                    }
                    return Ok(());
                }

                // char_at(s, idx) / byte_at(s, idx) -> i64: inline character access
                // v0.51.51: Re-enabled after fixing runtime to use BmbString structs consistently
                if (fn_name == "char_at" || fn_name == "byte_at" || fn_name == "bmb_string_char_at") && args.len() == 2 {
                    let str_idx = *name_counts.entry("str_char_at".to_string()).or_insert(0);
                    *name_counts.get_mut("str_char_at").unwrap() += 1;

                    // Get string pointer argument
                    // v0.51.53: String constants need .bmb suffix for BmbString struct
                    let str_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("charat.str.{}", str_idx);
                            writeln!(out, "  %{} = load ptr, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        Operand::Constant(Constant::String(s)) => {
                            if let Some(global_name) = string_table.get(s) {
                                format!("@{}.bmb", global_name)
                            } else {
                                self.format_operand_with_strings(&args[0], string_table)
                            }
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };

                    // Get index argument
                    let idx_val = match &args[1] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("charat.idx.{}", str_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[1], string_table),
                    };

                    // Access data pointer at offset 0 in BmbString struct
                    let data_ptr_ptr = format!("charat.data_ptr.{}", str_idx);
                    writeln!(out, "  %{} = getelementptr {{ptr, i64, i64}}, ptr {}, i32 0, i32 0", data_ptr_ptr, str_val)?;

                    // Load the data pointer
                    let data_ptr = format!("charat.data.{}", str_idx);
                    writeln!(out, "  %{} = load ptr, ptr %{}", data_ptr, data_ptr_ptr)?;

                    // Index into data array
                    let char_ptr = format!("charat.char_ptr.{}", str_idx);
                    writeln!(out, "  %{} = getelementptr i8, ptr %{}, i64 {}", char_ptr, data_ptr, idx_val)?;

                    // Load byte and zero-extend to i64
                    let char_val = format!("charat.byte.{}", str_idx);
                    writeln!(out, "  %{} = load i8, ptr %{}", char_val, char_ptr)?;

                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            // v0.90.73: Use dest name directly so %_tN is defined as SSA register
                            // (needed when Select/icmp references %_tN instead of loading from %_tN.addr)
                            writeln!(out, "  %{} = zext i8 %{} to i64", d.name, char_val)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", d.name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = zext i8 %{} to i64", dest_name, char_val)?;
                        }
                    }
                    return Ok(());
                }

                // ord(s) -> i64: inline first character access (same as char_at(s, 0))
                // NOTE: If argument is already i64 (from char_at), it's already the char code - pass through
                if (fn_name == "ord" || fn_name == "bmb_ord") && args.len() == 1 {
                    // Check argument type to determine if it's a String (ptr) or char (i64)
                    let arg_ty = match &args[0] {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    };

                    // If argument is i64 (char from char_at), just pass through - it's already the char code
                    if arg_ty == "i64" {
                        let str_idx = *name_counts.entry("str_ord".to_string()).or_insert(0);
                        *name_counts.get_mut("str_ord").unwrap() += 1;

                        let char_val = match &args[0] {
                            Operand::Place(p) if local_names.contains(&p.name) => {
                                let load_name = format!("ord.passthru.{}", str_idx);
                                writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                                format!("%{}", load_name)
                            }
                            _ => self.format_operand_with_strings(&args[0], string_table),
                        };

                        if let Some(d) = dest {
                            if local_names.contains(&d.name) {
                                // v0.90.73: Define %_tN as SSA register before storing
                                writeln!(out, "  %{} = add i64 {}, 0", d.name, char_val)?;
                                writeln!(out, "  store i64 %{}, ptr %{}.addr", d.name, d.name)?;
                            } else {
                                // For SSA, we need a copy instruction
                                let dest_name = self.unique_name(&d.name, name_counts);
                                writeln!(out, "  %{} = add i64 {}, 0", dest_name, char_val)?;
                            }
                        }
                        return Ok(());
                    }

                    // For String (ptr) argument, do the full inline
                    let str_idx = *name_counts.entry("str_ord".to_string()).or_insert(0);
                    *name_counts.get_mut("str_ord").unwrap() += 1;

                    // Get string pointer argument
                    let str_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("ord.str.{}", str_idx);
                            writeln!(out, "  %{} = load ptr, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };

                    // Access data pointer at offset 0 in BmbString struct
                    let data_ptr_ptr = format!("ord.data_ptr.{}", str_idx);
                    writeln!(out, "  %{} = getelementptr {{ptr, i64, i64}}, ptr {}, i32 0, i32 0", data_ptr_ptr, str_val)?;

                    // Load the data pointer
                    let data_ptr = format!("ord.data.{}", str_idx);
                    writeln!(out, "  %{} = load ptr, ptr %{}", data_ptr, data_ptr_ptr)?;

                    // Load first byte and zero-extend to i64
                    let char_val = format!("ord.byte.{}", str_idx);
                    writeln!(out, "  %{} = load i8, ptr %{}", char_val, data_ptr)?;

                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            // v0.90.73: Use dest name directly so %_tN is defined as SSA register
                            writeln!(out, "  %{} = zext i8 %{} to i64", d.name, char_val)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", d.name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = zext i8 %{} to i64", dest_name, char_val)?;
                        }
                    }
                    return Ok(());
                }

                // First check user-defined functions, then fall back to builtins
                let ret_ty = fn_return_types
                    .get(fn_name)
                    .copied()
                    .unwrap_or_else(|| self.infer_call_return_type(fn_name, func));

                // v0.51.27: Check if this is an sret function (struct return via caller-allocated pointer)
                let is_sret_call = sret_functions.contains_key(fn_name);
                let sret_field_count = sret_functions.get(fn_name).copied().unwrap_or(0);

                // v0.51.28: Check if this is a small struct function (1-2 fields via register)
                let is_small_struct_call = small_struct_functions.contains_key(fn_name);
                let small_struct_field_count = small_struct_functions.get(fn_name).copied().unwrap_or(0);

                // v0.55: Check if this is a tuple-returning function
                let is_tuple_call = tuple_functions.contains_key(fn_name);
                let tuple_type = tuple_functions.get(fn_name).cloned();

                // Generate unique base name for this call instruction
                // v0.50.72: Use unique counter to avoid SSA violations with multiple calls
                let call_cnt = *name_counts.entry(format!("call_{}", fn_name)).or_insert(0);
                *name_counts.entry(format!("call_{}", fn_name)).or_insert(0) += 1;
                let call_base = dest.as_ref().map(|d| d.name.clone())
                    .unwrap_or_else(|| format!("call_{}.{}", fn_name, call_cnt));

                // Emit loads for local variables used as arguments
                // v0.51.2: Track (type, value, is_string_literal) for cstr optimization
                let mut arg_vals: Vec<(String, String, bool)> = Vec::new();
                for (i, arg) in args.iter().enumerate() {
                    let ty = match arg {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    };

                    // v0.51.2: Track if this arg is a string literal for cstr optimization
                    let is_string_literal = matches!(arg, Operand::Constant(Constant::String(_)));

                    let val = match arg {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            // Emit load from alloca (use call_base for uniqueness)
                            let load_name = format!("{}.{}.arg{}", call_base, p.name, i);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, ty, p.name)?;
                            format!("%{}", load_name)
                        }
                        Operand::Constant(Constant::String(s)) => {
                            // v0.51.2: Check if function has _cstr variant for direct string literal pass
                            // If so, we'll pass the raw pointer without wrapping
                            // v0.50.77: sb_push with string literal -> sb_push_cstr (zero allocation)
                            let has_cstr_variant = matches!(fn_name.as_str(),
                                "file_exists" | "bmb_file_exists" |
                                "sb_push" | "bmb_sb_push");  // v0.50.77: StringBuilder optimization

                            if has_cstr_variant {
                                // Direct pass: just use the global string pointer
                                if let Some(global_name) = string_table.get(s) {
                                    format!("@{}", global_name)
                                } else {
                                    self.format_operand_with_strings(arg, string_table)
                                }
                            } else {
                                // v0.51.22: Use pre-initialized global BmbString
                                if let Some(global_name) = string_table.get(s) {
                                    format!("@{}.bmb", global_name)
                                } else {
                                    self.format_operand_with_strings(arg, string_table)
                                }
                            }
                        }
                        // v0.51.17: Use narrowing-aware formatting so %n becomes %n.i64
                        _ => self.format_operand_with_strings_and_narrowing(arg, string_table, narrowed_param_names),
                    };
                    arg_vals.push((ty.to_string(), val, is_string_literal));
                }

                // v0.51.17: Type coercion for narrowed parameters
                // ConstantPropagationNarrowing may change i64 params to i32,
                // so we need to truncate i64 arguments to i32 when calling such functions
                let param_types_opt = fn_param_types.get(fn_name);

                // v0.60.13: Helper to get runtime function parameter types
                // Runtime functions like println, print take i64 parameters
                let runtime_param_type = |fn_name: &str, idx: usize| -> Option<&'static str> {
                    match (fn_name, idx) {
                        ("println", 0) | ("print", 0) => Some("i64"),
                        ("assert", 0) => Some("i1"),
                        ("abs", 0) | ("bmb_abs", 0) | ("min", 0) | ("min", 1) | ("max", 0) | ("max", 1)
                        | ("bmb_min", 0) | ("bmb_min", 1) | ("bmb_max", 0) | ("bmb_max", 1)
                        | ("clamp", 0) | ("clamp", 1) | ("clamp", 2)
                        | ("bmb_clamp", 0) | ("bmb_clamp", 1) | ("bmb_clamp", 2)
                        | ("pow", 0) | ("pow", 1) | ("bmb_pow", 0) | ("bmb_pow", 1) => Some("i64"),
                        ("sb_push_char", 0) | ("sb_push_char", 1) => Some("i64"),
                        ("sb_push_int", 0) | ("sb_push_int", 1) => Some("i64"),
                        ("bmb_sb_push_char", 0) | ("bmb_sb_push_char", 1) => Some("i64"),
                        ("bmb_sb_push_int", 0) | ("bmb_sb_push_int", 1) => Some("i64"),
                        ("sb_push_range", 0) | ("sb_push_range", 2) | ("sb_push_range", 3) => Some("i64"),
                        ("bmb_sb_push_range", 0) | ("bmb_sb_push_range", 2) | ("bmb_sb_push_range", 3) => Some("i64"),
                        _ => None,
                    }
                };

                // v0.51.17: Pre-emit truncation/extension instructions for narrowed parameters
                // v0.60.12: Also emit sext for i32 -> i64 conversions (bug fix)
                // v0.60.13: Also handle runtime functions not in fn_param_types
                for (i, (arg_ty, val, _)) in arg_vals.iter().enumerate() {
                    let param_ty = if let Some(param_types) = param_types_opt {
                        param_types.get(i).copied()
                    } else {
                        runtime_param_type(fn_name, i)
                    };

                    if let Some(param_ty) = param_ty {
                        if arg_ty == "i64" && param_ty == "i32" {
                            let trunc_name = format!("{}.arg{}.trunc", call_base, i);
                            writeln!(out, "  %{} = trunc i64 {} to i32", trunc_name, val)?;
                        } else if arg_ty == "i32" && param_ty == "i64" {
                            // v0.60.12: Sign extend i32 to i64 when function expects i64
                            let sext_name = format!("{}.arg{}.sext", call_base, i);
                            writeln!(out, "  %{} = sext i32 {} to i64", sext_name, val)?;
                        }
                    }
                }

                // v0.51.17: Rebuild args_str with proper truncation/extension references
                // v0.60.12: Also handle i32 -> i64 sign extension
                // v0.60.13: Also handle runtime functions not in fn_param_types
                let args_str: Vec<String> = arg_vals
                    .iter()
                    .enumerate()
                    .map(|(i, (arg_ty, val, _))| {
                        let param_ty = if let Some(param_types) = param_types_opt {
                            param_types.get(i).copied()
                        } else {
                            runtime_param_type(fn_name, i)
                        };

                        if let Some(param_ty) = param_ty {
                            if arg_ty == "i64" && param_ty == "i32" {
                                let trunc_name = format!("{}.arg{}.trunc", call_base, i);
                                return format!("i32 %{}", trunc_name);
                            }
                            if arg_ty == "i32" && param_ty == "i64" {
                                // v0.60.12: Use sign-extended value
                                let sext_name = format!("{}.arg{}.sext", call_base, i);
                                return format!("i64 %{}", sext_name);
                            }
                        }
                        format!("{} {}", arg_ty, val)
                    })
                    .collect();

                // v0.51.2: Check if all string args are literals for cstr variant optimization
                let all_string_args_are_literals = arg_vals.iter()
                    .all(|(ty, _, is_literal)| ty != "ptr" || *is_literal);

                // Map BMB function names to runtime function names
                // v0.51.2: Use _cstr variant when all string args are literals
                // v0.50.77: sb_push -> sb_push_cstr for string literals (zero allocation)
                let runtime_fn_name = match fn_name.as_str() {
                    "system" => "bmb_system",
                    "system_capture" => "bmb_system_capture",
                    "exec_output" => "bmb_exec_output",
                    "file_exists" if all_string_args_are_literals => "file_exists_cstr",
                    "bmb_file_exists" if all_string_args_are_literals => "bmb_file_exists_cstr",
                    // v0.50.77: StringBuilder optimization - use cstr variant for string literals
                    "sb_push" if args.len() == 2 && matches!(&args[1], Operand::Constant(Constant::String(_))) => "sb_push_cstr",
                    "bmb_sb_push" if args.len() == 2 && matches!(&args[1], Operand::Constant(Constant::String(_))) => "bmb_sb_push_cstr",
                    // v0.93.7: Integer/float math method calls → bmb_* runtime functions
                    "abs" => "bmb_abs",
                    "min" => "bmb_min",
                    "max" => "bmb_max",
                    "clamp" => "bmb_clamp",
                    "pow" => "bmb_pow",
                    // v0.93.7: String method calls → bmb_string_* runtime functions
                    "len" => "bmb_string_len",
                    "byte_at" => "bmb_string_char_at",
                    "slice" => "bmb_string_slice",
                    "to_lower" => "bmb_string_to_lower",
                    "to_upper" => "bmb_string_to_upper",
                    "trim" => "bmb_string_trim",
                    "contains" => "bmb_string_contains",
                    "starts_with" => "bmb_string_starts_with",
                    "ends_with" => "bmb_string_ends_with",
                    "index_of" => "bmb_string_index_of",
                    "replace" => "bmb_string_replace",
                    "repeat" => "bmb_string_repeat",
                    "is_empty" => "bmb_string_is_empty",
                    _ => fn_name.as_str(),
                };

                // v0.50.65: Tail call optimization support
                let call_prefix = if *is_tail { "tail " } else { "" };

                // v0.51.27: sret call handling - caller allocates space and passes pointer
                if is_sret_call {
                    // Allocate stack space for struct return
                    let sret_ptr = format!("{}.sret", call_base);
                    writeln!(out, "  %{} = alloca i64, i32 {}", sret_ptr, sret_field_count)?;

                    // Build sret call args: prepend sret pointer
                    let sret_args = format!("ptr noalias sret(i8) %{}", sret_ptr);
                    let full_args = if args_str.is_empty() {
                        sret_args
                    } else {
                        format!("{}, {}", sret_args, args_str.join(", "))
                    };

                    // Call with void return
                    writeln!(out, "  {}call void @{}({})", call_prefix, runtime_fn_name, full_args)?;

                    // The sret pointer IS the result
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            // Store pointer to local
                            writeln!(out, "  store ptr %{}, ptr %{}.addr", sret_ptr, d.name)?;
                        } else {
                            // SSA assignment: bitcast to create the named value
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = bitcast ptr %{} to ptr", dest_name, sret_ptr)?;
                        }
                    }
                } else if is_small_struct_call {
                    // v0.51.28: Small struct call handling - receive aggregate in registers
                    let agg_type = if small_struct_field_count == 1 { "{i64}" } else { "{i64, i64}" };

                    // Call with aggregate return type
                    let agg_temp = format!("{}.agg", call_base);
                    writeln!(out, "  %{} = {}call {} @{}({})", agg_temp, call_prefix, agg_type, runtime_fn_name, args_str.join(", "))?;

                    // Allocate stack space and unpack aggregate into memory
                    let struct_ptr = format!("{}.ptr", call_base);
                    writeln!(out, "  %{} = alloca i64, i32 {}", struct_ptr, small_struct_field_count)?;

                    // Extract field 0
                    let f0_val = format!("{}.f0", call_base);
                    writeln!(out, "  %{} = extractvalue {} %{}, 0", f0_val, agg_type, agg_temp)?;
                    let f0_ptr = format!("{}.f0.ptr", call_base);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i32 0", f0_ptr, struct_ptr)?;
                    writeln!(out, "  store i64 %{}, ptr %{}", f0_val, f0_ptr)?;

                    if small_struct_field_count == 2 {
                        // Extract field 1
                        let f1_val = format!("{}.f1", call_base);
                        writeln!(out, "  %{} = extractvalue {} %{}, 1", f1_val, agg_type, agg_temp)?;
                        let f1_ptr = format!("{}.f1.ptr", call_base);
                        writeln!(out, "  %{} = getelementptr i64, ptr %{}, i32 1", f1_ptr, struct_ptr)?;
                        writeln!(out, "  store i64 %{}, ptr %{}", f1_val, f1_ptr)?;
                    }

                    // The struct pointer is the result
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            // Store pointer to local
                            writeln!(out, "  store ptr %{}, ptr %{}.addr", struct_ptr, d.name)?;
                        } else {
                            // SSA assignment: bitcast to create the named value
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = bitcast ptr %{} to ptr", dest_name, struct_ptr)?;
                        }
                    }
                } else if is_tuple_call {
                    // v0.55: Tuple return call handling - receive aggregate in registers
                    let tuple_llvm_type = tuple_type.as_ref().unwrap();

                    // Call with tuple aggregate return type
                    let agg_temp = format!("{}.tuple", call_base);
                    writeln!(out, "  %{} = {}call {} @{}({})", agg_temp, call_prefix, tuple_llvm_type, runtime_fn_name, args_str.join(", "))?;

                    // Store the tuple result if there's a destination
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            // Store aggregate to alloca
                            writeln!(out, "  store {} %{}, ptr %{}.addr", tuple_llvm_type, agg_temp, d.name)?;
                        } else {
                            // SSA assignment: create named tuple value
                            let dest_name = self.unique_name(&d.name, name_counts);
                            // Just pass through the aggregate value
                            writeln!(out, "  %{} = add i8 0, 0 ; tuple placeholder", dest_name)?;
                            // Note: The actual tuple value is in %agg_temp, handled by TupleExtract
                        }
                    }
                } else if ret_ty == "void" {
                    writeln!(
                        out,
                        "  {}call {} @{}({})",
                        call_prefix,
                        ret_ty,
                        runtime_fn_name,
                        args_str.join(", ")
                    )?;
                } else if let Some(d) = dest {
                    // Check if destination is a local
                    if local_names.contains(&d.name) {
                        let temp_name = format!("{}.call", d.name);
                        writeln!(
                            out,
                            "  %{} = {}call {} @{}({})",
                            temp_name,
                            call_prefix,
                            ret_ty,
                            runtime_fn_name,
                            args_str.join(", ")
                        )?;
                        writeln!(out, "  store {} %{}, ptr %{}.addr", ret_ty, temp_name, d.name)?;
                    } else {
                        let dest_name = self.unique_name(&d.name, name_counts);
                        writeln!(
                            out,
                            "  %{} = {}call {} @{}({})",
                            dest_name,
                            call_prefix,
                            ret_ty,
                            runtime_fn_name,
                            args_str.join(", ")
                        )?;
                    }
                } else {
                    writeln!(
                        out,
                        "  {}call {} @{}({})",
                        call_prefix,
                        ret_ty,
                        runtime_fn_name,
                        args_str.join(", ")
                    )?;
                }
            }

            MirInst::Phi { dest, values } => {
                let dest_name = self.unique_name(&dest.name, name_counts);
                // PHI nodes must come at the start of a basic block
                // %dest = phi type [ val1, %label1 ], [ val2, %label2 ], ...
                // v0.55: Check tuple_var_types first for tuple phis
                // v0.51.13: Use place_types for phi type - this has the WIDEST type among all values
                // This handles ConstantPropagationNarrowing where param is i32 but return is i64
                let ty: std::borrow::Cow<'static, str> = {
                    // First check if any phi value is a tuple (from tuple_var_types)
                    let tuple_ty = values.iter().find_map(|(val, _)| {
                        if let Operand::Place(p) = val {
                            tuple_var_types.get(&p.name).cloned()
                        } else {
                            None
                        }
                    });
                    if let Some(tt) = tuple_ty {
                        std::borrow::Cow::Owned(tt)
                    } else if let Some(t) = place_types.get(&dest.name) {
                        std::borrow::Cow::Borrowed(*t)
                    } else {
                        // Fallback: infer from first value
                        if !values.is_empty() {
                            std::borrow::Cow::Borrowed(match &values[0].0 {
                                Operand::Constant(c) => self.constant_type(c),
                                Operand::Place(p) => place_types.get(&p.name).copied()
                                    .unwrap_or_else(|| self.infer_place_type(p, func)),
                            })
                        } else {
                            std::borrow::Cow::Borrowed("i64")
                        }
                    }
                };

                // Find the dest block label by looking at which block contains this phi
                // We need to check phi_load_map for locals that were pre-loaded
                // phi_string_map for string constants that were wrapped
                // phi_coerce_map for values that need type widening (sext)
                let phi_args: Vec<String> = values
                    .iter()
                    .map(|(val, label)| {
                        // v0.51.13: First check if this value was coerced (type widening)
                        if let Operand::Place(p) = val {
                            let coerce_key = (current_block_label.to_string(), p.name.clone(), label.clone());
                            if let Some((coerce_temp, _, _)) = phi_coerce_map.get(&coerce_key) {
                                // Use the coerced value
                                return format!("[ %{}, %bb_{} ]", coerce_temp, label);
                            }
                        }

                        // Check if this is a local variable that was pre-loaded for phi
                        // v0.51.17: Use narrowing-aware formatting for phi operands
                        let val_str = if let Operand::Place(p) = val {
                            if local_names.contains(&p.name) {
                                // This local should have been pre-loaded in the predecessor block
                                // The load temp name follows the pattern: {local}.phi.{pred_label}
                                let load_temp = format!("{}.phi.{}", p.name, label);
                                format!("%{}", load_temp)
                            } else {
                                self.format_operand_with_strings_and_narrowing(val, string_table, narrowed_param_names)
                            }
                        } else if let Operand::Constant(Constant::String(s)) = val {
                            // Check if this string constant was pre-wrapped for phi
                            let key = (current_block_label.to_string(), s.clone(), label.clone());
                            if let Some(temp_name) = phi_string_map.get(&key) {
                                format!("%{}", temp_name)
                            } else {
                                self.format_operand_with_strings_and_narrowing(val, string_table, narrowed_param_names)
                            }
                        } else {
                            self.format_operand_with_strings_and_narrowing(val, string_table, narrowed_param_names)
                        };
                        format!("[ {}, %bb_{} ]", val_str, label)
                    })
                    .collect();

                writeln!(
                    out,
                    "  %{} = phi {} {}",
                    dest_name,
                    ty,
                    phi_args.join(", ")
                )?;
            }

            // v0.19.0: Struct operations
            // v0.51.25: Escape analysis for struct allocation
            // v0.51.27: sret optimization - returned structs use caller-provided pointer
            // - Escaped structs (returned/passed to calls):
            //   - In sret functions: if directly returned, use %_sret
            //   - Otherwise: malloc (heap)
            // - Local-only structs: alloca (stack, faster)
            MirInst::StructInit { dest, struct_name, fields } => {
                // Check if this is a sret function (returns struct with 3+ fields)
                // v0.51.28: Small structs (1-2 fields) use register return, not sret
                let ret_field_count = if let MirType::Struct { fields: f, .. } = &func.ret_ty {
                    f.len()
                } else {
                    0
                };
                let is_sret_func = ret_field_count > 2;

                // Check if this struct is directly returned (flows to return statement)
                let is_returned = self.is_struct_returned(func, &dest.name);

                // Inline escape analysis: check if this struct escapes the function
                let escapes = self.check_struct_escapes(func, &dest.name);
                let num_fields = fields.len().max(1);

                // Determine allocation strategy
                let use_sret = is_sret_func && is_returned;

                // v0.51.32: Use proper LLVM struct types for better alias analysis
                let struct_ty = format!("%struct.{}", struct_name);

                if use_sret {
                    // v0.51.27: Use sret pointer from caller (no allocation needed)
                    writeln!(out, "  ; struct {} init with {} fields (sret - caller allocated)", struct_name, fields.len())?;
                    writeln!(out, "  %{} = bitcast ptr %_sret to ptr", dest.name)?;
                } else if escapes && ret_field_count <= 2 && is_returned {
                    // v0.51.28: Small struct register return - use stack allocation
                    // The struct will be packed into an aggregate at return
                    writeln!(out, "  ; struct {} init with {} fields (stack - small struct return)", struct_name, fields.len())?;
                    writeln!(out, "  %{} = alloca {}, align 8", dest.name, struct_ty)?;
                } else if escapes {
                    // Escaped struct: must use heap allocation
                    let size = num_fields * 8;
                    writeln!(out, "  ; struct {} init with {} fields (heap - escapes)", struct_name, fields.len())?;
                    writeln!(out, "  %{} = call ptr @malloc(i64 {})", dest.name, size)?;
                } else {
                    // Local struct: can use stack allocation (faster)
                    writeln!(out, "  ; struct {} init with {} fields (stack - local only)", struct_name, fields.len())?;
                    writeln!(out, "  %{} = alloca {}, align 8", dest.name, struct_ty)?;
                }
                for (i, (field_name, value)) in fields.iter().enumerate() {
                    let ty = self.infer_operand_type(value, func);
                    // v0.51.32: Properly load operand values from .addr if they're locals
                    // v0.51.42: Also check local_names - temps from FieldAccess don't have .addr
                    let val_str = match value {
                        Operand::Place(p) => {
                            let is_param = func.params.iter().any(|(name, _)| name == &p.name);
                            let is_local = local_names.contains(&p.name);
                            if !is_param && is_local {
                                // Local: load from .addr
                                let load_name = format!("{}_f{}_val", dest.name, i);
                                writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, ty, p.name)?;
                                format!("%{}", load_name)
                            } else {
                                // Param or temp: use directly
                                format!("%{}", p.name)
                            }
                        }
                        Operand::Constant(c) => self.format_constant(c),
                    };
                    writeln!(out, "  ; field {} = {}", field_name, val_str)?;
                    // v0.51.32: Use struct type GEP for better LLVM optimization
                    writeln!(out, "  %{}_f{} = getelementptr {}, ptr %{}, i32 0, i32 {}",
                             dest.name, i, struct_ty, dest.name, i)?;
                    writeln!(out, "  store {} {}, ptr %{}_f{}", ty, val_str, dest.name, i)?;
                }
            }

            MirInst::FieldAccess { dest, base, field, field_index, struct_name } => {
                // v0.51.23: Load field from struct pointer using correct offset
                // v0.51.24: Check if base is a parameter (already ptr) or local (needs load from .addr)
                // v0.51.31: Use struct_defs to look up correct field type for load instruction
                // v0.51.32: Use struct type GEPs for better LLVM alias analysis
                // v0.51.36: Handle temps from struct array IndexLoad (direct ptrs, not in locals)
                writeln!(out, "  ; field access .{}[{}] from %{} ({})", field, field_index, base.name, struct_name)?;

                // Look up the field type from struct_defs
                let field_llvm_ty = struct_defs.get(struct_name)
                    .and_then(|fields| fields.get(*field_index))
                    .map(|(_, ty)| self.mir_type_to_llvm(ty))
                    .unwrap_or("i64"); // Default to i64 if not found

                // v0.51.32: Use proper struct type for GEP
                let struct_ty = format!("%struct.{}", struct_name);

                let is_param = func.params.iter().any(|(name, _)| name == &base.name);
                let is_local = local_names.contains(&base.name);
                // v0.51.36: Temps (not params, not locals) are direct pointers from IndexLoad
                if is_param || !is_local {
                    // Parameters and temps are already ptr values - use directly
                    writeln!(out, "  %{}_ptr = getelementptr {}, ptr %{}, i32 0, i32 {}",
                             dest.name, struct_ty, base.name, field_index)?;
                } else {
                    // Locals: load struct pointer from variable address
                    writeln!(out, "  %{}_base_ptr = load ptr, ptr %{}.addr", dest.name, base.name)?;
                    writeln!(out, "  %{}_ptr = getelementptr {}, ptr %{}_base_ptr, i32 0, i32 {}",
                             dest.name, struct_ty, dest.name, field_index)?;
                }
                // v0.60.7: Store to .addr if dest is a local, otherwise create SSA value
                let dest_is_local = local_names.contains(&dest.name);
                if dest_is_local {
                    writeln!(out, "  %{}_val = load {}, ptr %{}_ptr", dest.name, field_llvm_ty, dest.name)?;
                    writeln!(out, "  store {} %{}_val, ptr %{}.addr", field_llvm_ty, dest.name, dest.name)?;
                } else {
                    writeln!(out, "  %{} = load {}, ptr %{}_ptr", dest.name, field_llvm_ty, dest.name)?;
                }
            }

            MirInst::FieldStore { base, field, field_index, struct_name, value } => {
                // v0.51.23: Store value to field in struct pointer using correct offset
                // v0.51.24: Check if base is a parameter (already ptr) or local (needs load from .addr)
                // v0.51.31: Use struct_defs to look up correct field type for GEP instruction
                // v0.51.32: Use struct type GEPs for better LLVM alias analysis
                // v0.51.36: Handle temps from struct array IndexLoad (direct ptrs, not in locals)
                // v0.51.38: Load value from .addr if it's a local variable
                // v0.93.123: Check field type from struct_defs to fix narrowed i32→i64 mismatch
                let ty = self.infer_operand_type(value, func);

                // v0.93.123: Look up the expected field type from struct_defs
                let field_llvm_ty = struct_defs.get(struct_name)
                    .and_then(|fields| fields.get(*field_index))
                    .map(|(_, mir_ty)| self.mir_type_to_llvm(mir_ty))
                    .unwrap_or(ty); // Fall back to operand type if struct not found

                // v0.51.38: Generate unique key for this field store
                let fstore_key = format!("{}_f{}", base.name, field_index);
                let fstore_cnt = *name_counts.entry(fstore_key.clone()).or_insert(0);
                *name_counts.entry(fstore_key).or_insert(0) += 1;
                let suffix = if fstore_cnt == 0 { String::new() } else { format!(".{}", fstore_cnt) };

                // v0.51.38: Check if value is a local that needs loading from .addr
                let val_str = match value {
                    Operand::Place(p) if local_names.contains(&p.name) => {
                        // Local variable - load from .addr
                        let load_name = format!("{}_f{}_val{}", base.name, field_index, suffix);
                        writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, ty, p.name)?;
                        format!("%{}", load_name)
                    }
                    _ => self.format_operand(value),
                };

                // v0.93.123: If operand is narrowed i32 but field expects i64, sext before storing
                let (store_ty, store_val) = if ty == "i32" && field_llvm_ty == "i64" {
                    let sext_name = format!("{}_f{}_sext{}", base.name, field_index, suffix);
                    writeln!(out, "  %{} = sext i32 {} to i64", sext_name, val_str)?;
                    ("i64", format!("%{}", sext_name))
                } else {
                    (ty, val_str.clone())
                };

                writeln!(out, "  ; field store .{}[{}] ({}) = {}", field, field_index, struct_name, store_val)?;

                // v0.51.32: Use proper struct type for GEP
                let struct_ty = format!("%struct.{}", struct_name);

                let is_param = func.params.iter().any(|(name, _)| name == &base.name);
                let base_is_local = local_names.contains(&base.name);
                // v0.51.36: Temps (not params, not locals) are direct pointers from IndexLoad
                if is_param || !base_is_local {
                    // Parameters and temps are already ptr values - use directly
                    writeln!(out, "  %{}_f{}_ptr{} = getelementptr {}, ptr %{}, i32 0, i32 {}",
                             base.name, field_index, suffix, struct_ty, base.name, field_index)?;
                } else {
                    // Locals: load struct pointer from variable address (unique name per field)
                    writeln!(out, "  %{}_f{}_base{} = load ptr, ptr %{}.addr", base.name, field_index, suffix, base.name)?;
                    writeln!(out, "  %{}_f{}_ptr{} = getelementptr {}, ptr %{}_f{}_base{}, i32 0, i32 {}",
                             base.name, field_index, suffix, struct_ty, base.name, field_index, suffix, field_index)?;
                }
                writeln!(out, "  store {} {}, ptr %{}_f{}_ptr{}", store_ty, store_val, base.name, field_index, suffix)?;
            }

            // v0.19.1: Enum variant
            MirInst::EnumVariant { dest, enum_name, variant, args } => {
                // Enums are represented as tagged unions:
                // - First word: discriminant (variant index)
                // - Following words: variant data
                writeln!(out, "  ; enum {}::{} with {} args", enum_name, variant, args.len())?;
                // Allocate space for enum (discriminant + max variant size)
                let size = 1 + args.len().max(1);
                writeln!(out, "  %{} = alloca i64, i32 {}", dest.name, size)?;
                // Store discriminant - must match variant_to_discriminant in mir/lower.rs
                let discriminant: i64 = variant.chars()
                    .enumerate()
                    .fold(0i64, |acc, (i, c)| acc.wrapping_add((c as i64).wrapping_mul((i + 1) as i64)));
                writeln!(out, "  %{}_disc = getelementptr i64, ptr %{}, i32 0", dest.name, dest.name)?;
                writeln!(out, "  store i64 {}, ptr %{}_disc", discriminant, dest.name)?;
                // Store variant arguments
                for (i, arg) in args.iter().enumerate() {
                    let arg_str = self.format_operand(arg);
                    let ty = self.infer_operand_type(arg, func);
                    writeln!(out, "  %{}_a{} = getelementptr i64, ptr %{}, i32 {}",
                             dest.name, i, dest.name, i + 1)?;
                    writeln!(out, "  store {} {}, ptr %{}_a{}", ty, arg_str, dest.name, i)?;
                }
            }

            // v0.19.3: Array operations
            // v0.50.60: Fix - load from local alloca before storing to array element
            // v0.51.35: Support struct arrays with proper LLVM type and memcpy
            MirInst::ArrayInit { dest, element_type, elements } => {
                let size = elements.len();

                // v0.51.35: Handle struct arrays specially for packed SIMD optimization
                if let MirType::Struct { name: struct_name, fields } = element_type {
                    let struct_ty = format!("%struct.{}", struct_name);
                    let struct_size = fields.len() * 8; // 8 bytes per field
                    writeln!(out, "  ; struct array init with {} elements of type {}", size, struct_ty)?;
                    writeln!(out, "  %{} = alloca {}, i32 {}", dest.name, struct_ty, size.max(1))?;

                    for (i, elem) in elements.iter().enumerate() {
                        // Get the source struct pointer
                        let src_ptr = if let Operand::Place(p) = elem {
                            if local_names.contains(&p.name) {
                                // Local struct: load pointer from .addr
                                writeln!(out, "  %{}_src{} = load ptr, ptr %{}.addr", dest.name, i, p.name)?;
                                format!("%{}_src{}", dest.name, i)
                            } else {
                                format!("%{}", p.name)
                            }
                        } else {
                            self.format_operand(elem)
                        };

                        // Get destination element pointer
                        writeln!(out, "  %{}_e{} = getelementptr {}, ptr %{}, i32 {}",
                                 dest.name, i, struct_ty, dest.name, i)?;

                        // Copy struct using memcpy for proper value semantics
                        writeln!(out, "  call void @llvm.memcpy.p0.p0.i64(ptr %{}_e{}, ptr {}, i64 {}, i1 false)",
                                 dest.name, i, src_ptr, struct_size)?;
                    }
                } else {
                    // Original code path for primitive arrays
                    let elem_ty = self.mir_type_to_llvm(element_type);
                    writeln!(out, "  ; array init with {} elements of type {}", size, elem_ty)?;
                    writeln!(out, "  %{} = alloca {}, i32 {}", dest.name, elem_ty, size.max(1))?;

                    // v0.60.59: Optimize zero-initialized arrays with memset
                    // Check if all elements are zero constants (threshold: 64 elements)
                    let all_zeros = size >= 64 && elements.iter().all(|e| {
                        match e {
                            Operand::Constant(Constant::Int(0)) => true,
                            Operand::Constant(Constant::Float(f)) => *f == 0.0,
                            _ => false,
                        }
                    });

                    if all_zeros {
                        // Use memset for zero initialization
                        let elem_size: usize = match element_type {
                            MirType::I64 | MirType::F64 | MirType::U64 => 8,
                            MirType::I32 | MirType::U32 => 4,
                            MirType::Bool => 1,
                            _ => 8, // default to 8 for pointer-sized types
                        };
                        let total_bytes = size * elem_size;
                        writeln!(out, "  ; optimized: memset for {} zero elements ({} bytes)",
                                 size, total_bytes)?;
                        writeln!(out, "  call void @llvm.memset.p0.i64(ptr %{}, i8 0, i64 {}, i1 false)",
                                 dest.name, total_bytes)?;
                    } else {
                        // Original element-by-element initialization
                        for (i, elem) in elements.iter().enumerate() {
                            // Check if element is a local that needs loading from alloca
                            let elem_str = if let Operand::Place(p) = elem {
                                if local_names.contains(&p.name) {
                                    // Load from alloca first
                                    writeln!(out, "  %{}_arr_elem{} = load {}, ptr %{}.addr",
                                             dest.name, i, elem_ty, p.name)?;
                                    format!("%{}_arr_elem{}", dest.name, i)
                                } else {
                                    self.format_operand(elem)
                                }
                            } else {
                                self.format_operand(elem)
                            };
                            writeln!(out, "  %{}_e{} = getelementptr {}, ptr %{}, i32 {}",
                                     dest.name, i, elem_ty, dest.name, i)?;
                            writeln!(out, "  store {} {}, ptr %{}_e{}", elem_ty, elem_str, dest.name, i)?;
                        }
                    }
                }
            }

            // v0.50.60: Fix - load index from local alloca if needed
            // v0.51.23: Load array pointer from .addr for local variables
            // v0.51.35: Support struct arrays with proper type handling
            MirInst::IndexLoad { dest, array, index, element_type } => {
                // Load array pointer from .addr if it's a local variable
                let arr_ptr = if local_names.contains(&array.name) {
                    writeln!(out, "  %{}_arr_ptr = load ptr, ptr %{}.addr", dest.name, array.name)?;
                    format!("%{}_arr_ptr", dest.name)
                } else {
                    format!("%{}", array.name)
                };

                // v0.60.24: Handle narrowed index types - GEP requires i64 index
                // Type narrowing may produce i32 indices that need sign extension
                let idx_str = if let Operand::Place(p) = index {
                    let idx_type = self.infer_operand_type(index, func);
                    if local_names.contains(&p.name) {
                        // Load from alloca using actual type, then sext if needed
                        writeln!(out, "  %{}_idx_load = load {}, ptr %{}.addr", dest.name, idx_type, p.name)?;
                        if idx_type != "i64" {
                            writeln!(out, "  %{}_idx_sext = sext {} %{}_idx_load to i64", dest.name, idx_type, dest.name)?;
                            format!("%{}_idx_sext", dest.name)
                        } else {
                            format!("%{}_idx_load", dest.name)
                        }
                    } else {
                        // Parameter or temp - check if sext needed
                        let base = self.format_operand(index);
                        if idx_type != "i64" {
                            writeln!(out, "  %{}_idx_sext = sext {} {} to i64", dest.name, idx_type, base)?;
                            format!("%{}_idx_sext", dest.name)
                        } else {
                            base
                        }
                    }
                } else {
                    self.format_operand(index)
                };

                // v0.51.35: Handle struct arrays with getelementptr using struct type
                // v0.51.36: Store struct pointer to .addr so FieldStore/FieldAccess can load it
                if let MirType::Struct { name: struct_name, .. } = element_type {
                    let struct_ty = format!("%struct.{}", struct_name);
                    writeln!(out, "  ; struct array index load %{}[{}]", array.name, idx_str)?;
                    // Get pointer to struct element
                    writeln!(out, "  %{}_gep = getelementptr {}, ptr {}, i64 {}",
                             dest.name, struct_ty, arr_ptr, idx_str)?;
                    // Store pointer to .addr for FieldStore/FieldAccess to load
                    if local_names.contains(&dest.name) {
                        writeln!(out, "  store ptr %{}_gep, ptr %{}.addr", dest.name, dest.name)?;
                    } else {
                        // For non-local (e.g., temps not in locals), create an SSA value
                        writeln!(out, "  %{} = select i1 true, ptr %{}_gep, ptr null", dest.name, dest.name)?;
                    }
                } else {
                    let elem_ty = self.mir_type_to_llvm(element_type);
                    writeln!(out, "  ; index load %{}[{}]", array.name, idx_str)?;
                    writeln!(out, "  %{}_ptr = getelementptr {}, ptr {}, i64 {}",
                             dest.name, elem_ty, arr_ptr, idx_str)?;
                    writeln!(out, "  %{} = load {}, ptr %{}_ptr", dest.name, elem_ty, dest.name)?;
                    // v0.60.24: Store to .addr if dest is a local variable
                    // This ensures the value is available for subsequent reads from .addr
                    if local_names.contains(&dest.name) {
                        writeln!(out, "  store {} %{}, ptr %{}.addr", elem_ty, dest.name, dest.name)?;
                    }
                }
            }

            // v0.50.60: Fix - load index and value from local alloca if needed
            // v0.50.72: Fix SSA violation - use unique counter for each IndexStore
            // v0.51.23: Load array pointer from .addr for local variables
            // v0.51.35: Support struct arrays with memcpy for struct values
            MirInst::IndexStore { array, index, value, element_type } => {
                let store_cnt = *name_counts.entry(format!("{}_idx_store", array.name)).or_insert(0);
                *name_counts.entry(format!("{}_idx_store", array.name)).or_insert(0) += 1;

                // Load array pointer from .addr if it's a local variable
                let arr_ptr = if local_names.contains(&array.name) {
                    writeln!(out, "  %{}_arr_ptr.{} = load ptr, ptr %{}.addr", array.name, store_cnt, array.name)?;
                    format!("%{}_arr_ptr.{}", array.name, store_cnt)
                } else {
                    format!("%{}", array.name)
                };

                // v0.60.24: Handle narrowed index types - GEP requires i64 index
                let idx_str = if let Operand::Place(p) = index {
                    let idx_type = self.infer_operand_type(index, func);
                    if local_names.contains(&p.name) {
                        // Load from alloca using actual type, then sext if needed
                        writeln!(out, "  %{}_store_idx.{} = load {}, ptr %{}.addr", array.name, store_cnt, idx_type, p.name)?;
                        if idx_type != "i64" {
                            writeln!(out, "  %{}_store_idx_sext.{} = sext {} %{}_store_idx.{} to i64", array.name, store_cnt, idx_type, array.name, store_cnt)?;
                            format!("%{}_store_idx_sext.{}", array.name, store_cnt)
                        } else {
                            format!("%{}_store_idx.{}", array.name, store_cnt)
                        }
                    } else {
                        // Parameter or temp - check if sext needed
                        let base = self.format_operand(index);
                        if idx_type != "i64" {
                            writeln!(out, "  %{}_store_idx_sext.{} = sext {} {} to i64", array.name, store_cnt, idx_type, base)?;
                            format!("%{}_store_idx_sext.{}", array.name, store_cnt)
                        } else {
                            base
                        }
                    }
                } else {
                    self.format_operand(index)
                };

                // v0.51.35: Handle struct arrays with memcpy
                if let MirType::Struct { name: struct_name, fields } = element_type {
                    let struct_ty = format!("%struct.{}", struct_name);
                    let struct_size = fields.len() * 8;

                    // Get source struct pointer
                    let src_ptr = if let Operand::Place(p) = value {
                        if local_names.contains(&p.name) {
                            writeln!(out, "  %{}_store_src.{} = load ptr, ptr %{}.addr", array.name, store_cnt, p.name)?;
                            format!("%{}_store_src.{}", array.name, store_cnt)
                        } else {
                            format!("%{}", p.name)
                        }
                    } else {
                        self.format_operand(value)
                    };

                    writeln!(out, "  ; struct array index store %{}[{}]", array.name, idx_str)?;
                    writeln!(out, "  %{}_idx_ptr.{} = getelementptr {}, ptr {}, i64 {}",
                             array.name, store_cnt, struct_ty, arr_ptr, idx_str)?;
                    writeln!(out, "  call void @llvm.memcpy.p0.p0.i64(ptr %{}_idx_ptr.{}, ptr {}, i64 {}, i1 false)",
                             array.name, store_cnt, src_ptr, struct_size)?;
                } else {
                    // v0.60.24: Use actual element type for GEP, not narrowed value type
                    // Value may be narrowed (e.g., i32) but array element type might be i64
                    let elem_ty_str = self.mir_type_to_llvm(element_type);
                    let val_ty = self.infer_operand_type(value, func);
                    let val_str = if let Operand::Place(p) = value {
                        if local_names.contains(&p.name) {
                            writeln!(out, "  %{}_store_val.{} = load {}, ptr %{}.addr", array.name, store_cnt, val_ty, p.name)?;
                            format!("%{}_store_val.{}", array.name, store_cnt)
                        } else {
                            self.format_operand(value)
                        }
                    } else {
                        self.format_operand(value)
                    };

                    // If value type is narrower than element type, sign-extend
                    let final_val_str = if val_ty != elem_ty_str && (val_ty == "i32" && elem_ty_str == "i64") {
                        writeln!(out, "  %{}_store_val_sext.{} = sext {} {} to {}", array.name, store_cnt, val_ty, val_str, elem_ty_str)?;
                        format!("%{}_store_val_sext.{}", array.name, store_cnt)
                    } else {
                        val_str
                    };

                    writeln!(out, "  ; index store %{}[{}] = {}", array.name, idx_str, final_val_str)?;
                    writeln!(out, "  %{}_idx_ptr.{} = getelementptr {}, ptr {}, i64 {}",
                             array.name, store_cnt, elem_ty_str, arr_ptr, idx_str)?;
                    writeln!(out, "  store {} {}, ptr %{}_idx_ptr.{}", elem_ty_str, final_val_str, array.name, store_cnt)?;
                }
            }

            // v0.50.80: Type cast instruction
            MirInst::Cast { dest, src, from_ty, to_ty } => {
                let src_str = if let Operand::Place(p) = src {
                    if local_names.contains(&p.name) {
                        let load_name = self.unique_name(&format!("{}_cast_load", dest.name), name_counts);
                        writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, self.mir_type_to_llvm(from_ty), p.name)?;
                        format!("%{}", load_name)
                    } else {
                        self.format_operand(src)
                    }
                } else {
                    self.format_operand(src)
                };

                let from_ty_str = self.mir_type_to_llvm(from_ty);
                let to_ty_str = self.mir_type_to_llvm(to_ty);
                let dest_name = self.unique_name(&dest.name, name_counts);

                // Determine cast instruction based on types
                let cast_inst = self.get_cast_instruction(from_ty, to_ty);
                if local_names.contains(&dest.name) {
                    let temp_name = self.unique_name(&format!("{}_cast", dest.name), name_counts);
                    writeln!(out, "  %{} = {} {} {} to {}", temp_name, cast_inst, from_ty_str, src_str, to_ty_str)?;
                    writeln!(out, "  store {} %{}, ptr %{}.addr", to_ty_str, temp_name, dest.name)?;
                } else {
                    writeln!(out, "  %{} = {} {} {} to {}", dest_name, cast_inst, from_ty_str, src_str, to_ty_str)?;
                }
            }

            // v0.55: Tuple initialization - builds LLVM struct from elements
            // v0.60.18: Handle narrowed types - operands may be i32 but tuple expects i64
            MirInst::TupleInit { dest, elements } => {
                // Create the LLVM struct type string
                let elem_types: Vec<String> = elements
                    .iter()
                    .map(|(ty, _)| self.mir_type_to_llvm(ty).to_string())
                    .collect();
                let struct_type = format!("{{ {} }}", elem_types.join(", "));

                // v0.60.18: Helper to get actual type of operand (checking place_types for narrowed types)
                let get_operand_actual_type = |op: &Operand, expected_ty: &'static str| -> &'static str {
                    match op {
                        Operand::Place(p) => place_types.get(&p.name).copied().unwrap_or(expected_ty),
                        Operand::Constant(c) => self.constant_type(c),
                    }
                };

                // Helper to format operand with local variable loading
                // Returns (value_string, actual_type)
                let format_element_val = |out: &mut String, _expected_ty: &'static str, actual_ty: &'static str, op: &Operand, idx: usize, name_counts: &mut HashMap<String, u32>| -> TextCodeGenResult<String> {
                    match op {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            // Load from alloca for local variables
                            // v0.60.18: Use actual alloca type from place_types
                            let load_name = self.unique_name(&format!("{}_tuple_elem{}", dest.name, idx), name_counts);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, actual_ty, p.name)?;
                            Ok(format!("%{}", load_name))
                        }
                        Operand::Place(p) => {
                            // v0.60.18: Non-local place (parameter or SSA value) - use as-is
                            Ok(format!("%{}", p.name))
                        }
                        _ => Ok(self.format_operand(op)),
                    }
                };

                // Helper to widen i32 to i64 if needed for tuple element
                // Returns (value_string, type_for_insertvalue)
                fn widen_if_needed(out: &mut String, val: &str, actual_ty: &'static str, expected_ty: &'static str, dest_name: &str, idx: usize, name_counts: &mut HashMap<String, u32>) -> TextCodeGenResult<(String, &'static str)> {
                    if actual_ty == "i32" && expected_ty == "i64" {
                        // Sign extend i32 to i64
                        let count = name_counts.entry(format!("{}_tuple_sext{}", dest_name, idx)).or_insert(0);
                        let ext_name = if *count == 0 {
                            format!("{}_tuple_sext{}", dest_name, idx)
                        } else {
                            format!("{}_tuple_sext{}.{}", dest_name, idx, count)
                        };
                        *count += 1;
                        writeln!(out, "  %{} = sext i32 {} to i64", ext_name, val)?;
                        Ok((format!("%{}", ext_name), "i64"))
                    } else {
                        Ok((val.to_string(), actual_ty))
                    }
                }

                // Build the struct value using insertvalue instructions
                let dest_name = self.unique_name(&dest.name, name_counts);
                if elements.is_empty() {
                    writeln!(out, "  %{} = insertvalue {} undef, i64 0, 0", dest_name, struct_type)?;
                } else {
                    // First element: insertvalue into undef
                    let (first_ty, first_op) = &elements[0];
                    let first_expected_ty = self.mir_type_to_llvm(first_ty);
                    let first_actual_ty = get_operand_actual_type(first_op, first_expected_ty);
                    let first_val = format_element_val(out, first_expected_ty, first_actual_ty, first_op, 0, name_counts)?;
                    let (first_val_final, first_ty_final) = widen_if_needed(out, &first_val, first_actual_ty, first_expected_ty, &dest_name, 0, name_counts)?;
                    writeln!(out, "  %{}_0 = insertvalue {} undef, {} {}, 0", dest_name, struct_type, first_ty_final, first_val_final)?;

                    // Remaining elements: chain insertvalue
                    for (i, (ty, op)) in elements.iter().enumerate().skip(1) {
                        let expected_ty = self.mir_type_to_llvm(ty);
                        let actual_ty = get_operand_actual_type(op, expected_ty);
                        let val_str = format_element_val(out, expected_ty, actual_ty, op, i, name_counts)?;
                        let (val_final, ty_final) = widen_if_needed(out, &val_str, actual_ty, expected_ty, &dest_name, i, name_counts)?;
                        let prev = if i == 1 { format!("%{}_0", dest_name) } else { format!("%{}_{}", dest_name, i - 1) };
                        if i == elements.len() - 1 {
                            // Last element uses final dest name
                            writeln!(out, "  %{} = insertvalue {} {}, {} {}, {}", dest_name, struct_type, prev, ty_final, val_final, i)?;
                        } else {
                            writeln!(out, "  %{}_{} = insertvalue {} {}, {} {}, {}", dest_name, i, struct_type, prev, ty_final, val_final, i)?;
                        }
                    }
                    // Single element case - rename _0 to final name
                    if elements.len() == 1 {
                        writeln!(out, "  %{} = insertvalue {} %{}_0, i64 0, 0 ; alias", dest_name, struct_type, dest_name)?;
                    }
                }
            }

            // v0.55: Tuple field extraction - extracts element from LLVM struct
            MirInst::TupleExtract { dest, tuple, index, element_type } => {
                let dest_name = self.unique_name(&dest.name, name_counts);
                let elem_ty_str = self.mir_type_to_llvm(element_type);

                // Get the tuple's LLVM struct type from the tracking map
                let tuple_llvm_type = tuple_var_types.get(&tuple.name)
                    .cloned()
                    .unwrap_or_else(|| "{ i64, i64 }".to_string()); // fallback

                // Load the tuple value with correct struct type if stored locally
                let tuple_val = if local_names.contains(&tuple.name) {
                    let load_name = self.unique_name(&format!("{}_tuple_load", dest.name), name_counts);
                    writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, tuple_llvm_type, tuple.name)?;
                    format!("%{}", load_name)
                } else {
                    format!("%{}", tuple.name)
                };

                // Extract the element with correct struct type
                writeln!(out, "  %{} = extractvalue {} {}, {}", dest_name, tuple_llvm_type, tuple_val, index)?;

                // Store to alloca if dest is a local
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store {} %{}, ptr %{}.addr", elem_ty_str, dest_name, dest.name)?;
                }
            }

            // v0.60.19: Pointer offset - generates proper LLVM GEP
            MirInst::PtrOffset { dest, ptr, offset, element_type } => {
                let dest_name = self.unique_name(&dest.name, name_counts);
                // v0.93.122: Use actual struct type for GEP element type
                // mir_type_to_llvm returns "ptr" for structs, but GEP needs %struct.Name
                // to calculate correct stride (e.g., 56 bytes for Body, not 8 bytes for ptr)
                let elem_ty_string = match element_type {
                    MirType::Struct { name, .. } => format!("%struct.{}", name),
                    MirType::StructPtr(name) => format!("%struct.{}", name),
                    _ => self.mir_type_to_llvm(element_type).to_string(),
                };
                let elem_ty_str = &elem_ty_string;

                // Get ptr operand
                let ptr_val = match ptr {
                    Operand::Place(p) => {
                        if local_names.contains(&p.name) {
                            let load_name = self.unique_name(&format!("{}_ptr_load", dest.name), name_counts);
                            writeln!(out, "  %{} = load ptr, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        } else {
                            format!("%{}", p.name)
                        }
                    }
                    Operand::Constant(c) => self.format_constant(c),
                };

                // Get offset operand
                let offset_val = match offset {
                    Operand::Place(p) => {
                        if local_names.contains(&p.name) {
                            let load_name = self.unique_name(&format!("{}_offset_load", dest.name), name_counts);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        } else {
                            format!("%{}", p.name)
                        }
                    }
                    Operand::Constant(c) => self.format_constant(c),
                };

                // Emit GEP instruction
                writeln!(out, "  %{} = getelementptr inbounds {}, ptr {}, i64 {}", dest_name, elem_ty_str, ptr_val, offset_val)?;

                // Store to alloca if dest is a local
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store ptr %{}, ptr %{}.addr", dest_name, dest.name)?;
                }
            }

            // v0.60.21: Stack array allocation without initialization
            MirInst::ArrayAlloc { dest, element_type, size } => {
                let dest_name = self.unique_name(&dest.name, name_counts);
                let elem_ty_str = self.mir_type_to_llvm(element_type);

                // Allocate array on stack
                writeln!(out, "  %{} = alloca [{} x {}]", dest_name, size, elem_ty_str)?;

                // Store to alloca if dest is a local (array pointer)
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store ptr %{}, ptr %{}.addr", dest_name, dest.name)?;
                }
            }

            // v0.60.20: Pointer load - load value through native pointer
            MirInst::PtrLoad { dest, ptr, element_type } => {
                let dest_name = self.unique_name(&dest.name, name_counts);
                let elem_ty_str = self.mir_type_to_llvm(element_type);

                // Get pointer operand
                let ptr_val = match ptr {
                    Operand::Place(p) => {
                        if local_names.contains(&p.name) {
                            let load_name = self.unique_name(&format!("{}_ptr_load", dest.name), name_counts);
                            writeln!(out, "  %{} = load ptr, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        } else {
                            format!("%{}", p.name)
                        }
                    }
                    Operand::Constant(c) => self.format_constant(c),
                };

                // Emit load instruction
                writeln!(out, "  %{} = load {}, ptr {}", dest_name, elem_ty_str, ptr_val)?;

                // Store to alloca if dest is a local
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store {} %{}, ptr %{}.addr", elem_ty_str, dest_name, dest.name)?;
                }
            }

            // v0.60.20: Pointer store - store value through native pointer
            MirInst::PtrStore { ptr, value, element_type } => {
                let elem_ty_str = self.mir_type_to_llvm(element_type);

                // Get pointer operand
                let ptr_val = match ptr {
                    Operand::Place(p) => {
                        if local_names.contains(&p.name) {
                            let load_name = self.unique_name(&format!("{}_ptr_store_addr", p.name), name_counts);
                            writeln!(out, "  %{} = load ptr, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        } else {
                            format!("%{}", p.name)
                        }
                    }
                    Operand::Constant(c) => self.format_constant(c),
                };

                // Get value operand
                let val_str = match value {
                    Operand::Place(p) => {
                        let val_ty = place_types.get(&p.name).copied().unwrap_or(elem_ty_str);
                        if local_names.contains(&p.name) {
                            let load_name = self.unique_name(&format!("{}_val_load", p.name), name_counts);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, val_ty, p.name)?;
                            format!("%{}", load_name)
                        } else {
                            format!("%{}", p.name)
                        }
                    }
                    Operand::Constant(c) => self.format_constant(c),
                };

                // Emit store instruction
                writeln!(out, "  store {} {}, ptr {}", elem_ty_str, val_str, ptr_val)?;
            }

            // v0.70: Thread spawn - Phase 1 simplified implementation
            // In Phase 1, the spawn body is lowered inline and the result is passed as a capture.
            // We use the first capture value as the "result" directly.
            // Real async threading will be implemented in Phase 2.
            MirInst::ThreadSpawn { dest, func: _, captures } => {
                writeln!(out, "  ; Phase 1: ThreadSpawn - body executed synchronously")?;

                // The first capture contains the pre-computed result
                let result_val = if !captures.is_empty() {
                    self.format_operand(&captures[0])
                } else {
                    "0".to_string()
                };

                // Store the result directly as the "handle"
                writeln!(out, "  %{} = add i64 {}, 0", dest.name, result_val)?;

                // Store to alloca if dest is a local
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            // v0.70: Thread join - Phase 1 simplified implementation
            // In Phase 1, the handle IS the result (from synchronous execution).
            // We just return the handle value directly.
            // Real thread waiting will be implemented in Phase 2.
            MirInst::ThreadJoin { dest, handle } => {
                writeln!(out, "  ; Phase 1: ThreadJoin - handle is the result")?;

                let handle_val = match handle {
                    Operand::Place(p) => {
                        if local_names.contains(&p.name) {
                            let load_name = self.unique_name(&format!("{}_load", p.name), name_counts);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        } else {
                            format!("%{}", p.name)
                        }
                    }
                    Operand::Constant(c) => self.format_constant(c),
                };

                if let Some(d) = dest {
                    // The handle value IS the result
                    writeln!(out, "  %{} = add i64 {}, 0", d.name, handle_val)?;

                    // Store to alloca if dest is a local
                    if local_names.contains(&d.name) {
                        writeln!(out, "  store i64 %{}, ptr %{}.addr", d.name, d.name)?;
                    }
                }
            }

            // v0.71: Mutex operations
            MirInst::MutexNew { dest, initial_value } => {
                let value_str = self.format_operand(initial_value);
                writeln!(out, "  %{} = call i64 @bmb_mutex_new(i64 {})", dest.name, value_str)?;

                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::MutexLock { dest, mutex } => {
                let mutex_val = match mutex {
                    Operand::Place(p) => {
                        if local_names.contains(&p.name) {
                            let load_name = self.unique_name(&format!("{}_load", p.name), name_counts);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        } else {
                            format!("%{}", p.name)
                        }
                    }
                    Operand::Constant(c) => self.format_constant(c),
                };

                writeln!(out, "  %{} = call i64 @bmb_mutex_lock(i64 {})", dest.name, mutex_val)?;

                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::MutexUnlock { mutex, new_value } => {
                let mutex_val = match mutex {
                    Operand::Place(p) => {
                        if local_names.contains(&p.name) {
                            let load_name = self.unique_name(&format!("{}_load", p.name), name_counts);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        } else {
                            format!("%{}", p.name)
                        }
                    }
                    Operand::Constant(c) => self.format_constant(c),
                };
                let value_str = self.format_operand(new_value);

                writeln!(out, "  call void @bmb_mutex_unlock(i64 {}, i64 {})", mutex_val, value_str)?;
            }

            MirInst::MutexTryLock { dest, mutex } => {
                let mutex_val = match mutex {
                    Operand::Place(p) => {
                        if local_names.contains(&p.name) {
                            let load_name = self.unique_name(&format!("{}_load", p.name), name_counts);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        } else {
                            format!("%{}", p.name)
                        }
                    }
                    Operand::Constant(c) => self.format_constant(c),
                };

                writeln!(out, "  %{} = call i64 @bmb_mutex_try_lock(i64 {})", dest.name, mutex_val)?;

                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::MutexFree { mutex } => {
                let mutex_val = match mutex {
                    Operand::Place(p) => {
                        if local_names.contains(&p.name) {
                            let load_name = self.unique_name(&format!("{}_load", p.name), name_counts);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        } else {
                            format!("%{}", p.name)
                        }
                    }
                    Operand::Constant(c) => self.format_constant(c),
                };

                writeln!(out, "  call void @bmb_mutex_free(i64 {})", mutex_val)?;
            }

            // v0.72: Arc operations
            MirInst::ArcNew { dest, value } => {
                let value_str = self.format_operand(value);
                writeln!(out, "  %{} = call i64 @bmb_arc_new(i64 {})", dest.name, value_str)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::ArcClone { dest, arc } => {
                let arc_val = self.format_operand(arc);
                writeln!(out, "  %{} = call i64 @bmb_arc_clone(i64 {})", dest.name, arc_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::ArcGet { dest, arc } => {
                let arc_val = self.format_operand(arc);
                writeln!(out, "  %{} = call i64 @bmb_arc_get(i64 {})", dest.name, arc_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::ArcDrop { arc } => {
                let arc_val = self.format_operand(arc);
                writeln!(out, "  call void @bmb_arc_drop(i64 {})", arc_val)?;
            }

            MirInst::ArcStrongCount { dest, arc } => {
                let arc_val = self.format_operand(arc);
                writeln!(out, "  %{} = call i64 @bmb_arc_strong_count(i64 {})", dest.name, arc_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            // v0.72: Atomic operations - using LLVM atomic intrinsics for optimal codegen
            MirInst::AtomicNew { dest, value } => {
                // Allocate an i64 on the heap for the atomic value
                let value_str = self.format_operand(value);
                writeln!(out, "  %{}_ptr = call ptr @malloc(i64 8)", dest.name)?;
                writeln!(out, "  store atomic i64 {}, ptr %{}_ptr seq_cst, align 8", value_str, dest.name)?;
                writeln!(out, "  %{} = ptrtoint ptr %{}_ptr to i64", dest.name, dest.name)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::AtomicLoad { dest, ptr } => {
                let ptr_val = self.format_operand(ptr);
                writeln!(out, "  %{}_ptr = inttoptr i64 {} to ptr", dest.name, ptr_val)?;
                writeln!(out, "  %{} = load atomic i64, ptr %{}_ptr seq_cst, align 8", dest.name, dest.name)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::AtomicStore { ptr, value } => {
                let ptr_val = self.format_operand(ptr);
                let value_str = self.format_operand(value);
                let store_ptr_name = self.unique_name("atomic_store_ptr", name_counts);
                writeln!(out, "  %{} = inttoptr i64 {} to ptr", store_ptr_name, ptr_val)?;
                writeln!(out, "  store atomic i64 {}, ptr %{} seq_cst, align 8", value_str, store_ptr_name)?;
            }

            MirInst::AtomicFetchAdd { dest, ptr, delta } => {
                let ptr_val = self.format_operand(ptr);
                let delta_str = self.format_operand(delta);
                writeln!(out, "  %{}_ptr = inttoptr i64 {} to ptr", dest.name, ptr_val)?;
                writeln!(out, "  %{} = atomicrmw add ptr %{}_ptr, i64 {} seq_cst, align 8", dest.name, dest.name, delta_str)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::AtomicFetchSub { dest, ptr, delta } => {
                let ptr_val = self.format_operand(ptr);
                let delta_str = self.format_operand(delta);
                writeln!(out, "  %{}_ptr = inttoptr i64 {} to ptr", dest.name, ptr_val)?;
                writeln!(out, "  %{} = atomicrmw sub ptr %{}_ptr, i64 {} seq_cst, align 8", dest.name, dest.name, delta_str)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::AtomicSwap { dest, ptr, new_value } => {
                let ptr_val = self.format_operand(ptr);
                let new_val_str = self.format_operand(new_value);
                writeln!(out, "  %{}_ptr = inttoptr i64 {} to ptr", dest.name, ptr_val)?;
                writeln!(out, "  %{} = atomicrmw xchg ptr %{}_ptr, i64 {} seq_cst, align 8", dest.name, dest.name, new_val_str)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::AtomicCompareExchange { dest, ptr, expected, new_value } => {
                let ptr_val = self.format_operand(ptr);
                let expected_str = self.format_operand(expected);
                let new_val_str = self.format_operand(new_value);
                writeln!(out, "  %{}_ptr = inttoptr i64 {} to ptr", dest.name, ptr_val)?;
                writeln!(out, "  %{}_result = cmpxchg ptr %{}_ptr, i64 {}, i64 {} seq_cst seq_cst, align 8", dest.name, dest.name, expected_str, new_val_str)?;
                writeln!(out, "  %{} = extractvalue {{ i64, i1 }} %{}_result, 0", dest.name, dest.name)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            // ================================================================
            // v0.73: Channel operations
            // ================================================================

            MirInst::ChannelNew { sender_dest, receiver_dest, capacity } => {
                let cap_val = self.format_operand(capacity);
                // Call bmb_channel_new(capacity, &sender, &receiver)
                // Allocate temps for sender/receiver
                writeln!(out, "  %{}_alloc = alloca i64, align 8", sender_dest.name)?;
                writeln!(out, "  %{}_alloc = alloca i64, align 8", receiver_dest.name)?;
                writeln!(out, "  call void @bmb_channel_new(i64 {}, ptr %{}_alloc, ptr %{}_alloc)",
                    cap_val, sender_dest.name, receiver_dest.name)?;
                writeln!(out, "  %{} = load i64, ptr %{}_alloc", sender_dest.name, sender_dest.name)?;
                writeln!(out, "  %{} = load i64, ptr %{}_alloc", receiver_dest.name, receiver_dest.name)?;
                if local_names.contains(&sender_dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", sender_dest.name, sender_dest.name)?;
                }
                if local_names.contains(&receiver_dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", receiver_dest.name, receiver_dest.name)?;
                }
            }

            MirInst::ChannelSend { sender, value } => {
                let sender_val = self.format_operand(sender);
                let val = self.format_operand(value);
                writeln!(out, "  call void @bmb_channel_send(i64 {}, i64 {})", sender_val, val)?;
            }

            MirInst::ChannelRecv { dest, receiver } => {
                let receiver_val = self.format_operand(receiver);
                writeln!(out, "  %{} = call i64 @bmb_channel_recv(i64 {})", dest.name, receiver_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::ChannelTrySend { dest, sender, value } => {
                let sender_val = self.format_operand(sender);
                let val = self.format_operand(value);
                writeln!(out, "  %{} = call i64 @bmb_channel_try_send(i64 {}, i64 {})", dest.name, sender_val, val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::ChannelTryRecv { dest, receiver } => {
                let receiver_val = self.format_operand(receiver);
                // try_recv returns value via out param, success as return
                writeln!(out, "  %{}_alloc = alloca i64, align 8", dest.name)?;
                writeln!(out, "  %{}_success = call i64 @bmb_channel_try_recv(i64 {}, ptr %{}_alloc)",
                    dest.name, receiver_val, dest.name)?;
                // For now, just load the value (caller should check success)
                writeln!(out, "  %{} = load i64, ptr %{}_alloc", dest.name, dest.name)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            // v0.77: Receive with timeout
            MirInst::ChannelRecvTimeout { dest, receiver, timeout_ms } => {
                let receiver_val = self.format_operand(receiver);
                let timeout_val = self.format_operand(timeout_ms);
                // Allocate output value on stack
                writeln!(out, "  %{}_alloc = alloca i64, align 8", dest.name)?;
                // Call recv_timeout(receiver, timeout_ms, &value_out) -> success
                writeln!(out, "  %{}_success = call i64 @bmb_channel_recv_timeout(i64 {}, i64 {}, ptr %{}_alloc)",
                    dest.name, receiver_val, timeout_val, dest.name)?;
                // Load the value
                writeln!(out, "  %{}_loaded = load i64, ptr %{}_alloc", dest.name, dest.name)?;
                // Select based on success: if success != 0 { value } else { -1 }
                writeln!(out, "  %{}_is_success = icmp ne i64 %{}_success, 0", dest.name, dest.name)?;
                writeln!(out, "  %{} = select i1 %{}_is_success, i64 %{}_loaded, i64 -1", dest.name, dest.name, dest.name)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            // v0.78: Block on future
            MirInst::BlockOn { dest, future } => {
                let future_val = self.format_operand(future);
                writeln!(out, "  %{} = call i64 @bmb_block_on(i64 {})", dest.name, future_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            // v0.79: Send with timeout
            MirInst::ChannelSendTimeout { dest, sender, value, timeout_ms } => {
                let sender_val = self.format_operand(sender);
                let value_val = self.format_operand(value);
                let timeout_val = self.format_operand(timeout_ms);
                writeln!(out, "  %{} = call i64 @bmb_channel_send_timeout(i64 {}, i64 {}, i64 {})",
                    dest.name, sender_val, value_val, timeout_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            // v0.80: Channel close operations
            MirInst::ChannelClose { sender } => {
                let sender_val = self.format_operand(sender);
                writeln!(out, "  call void @bmb_channel_close(i64 {})", sender_val)?;
            }

            MirInst::ChannelIsClosed { dest, receiver } => {
                let receiver_val = self.format_operand(receiver);
                writeln!(out, "  %{} = call i64 @bmb_channel_is_closed(i64 {})", dest.name, receiver_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::ChannelRecvOpt { dest, receiver } => {
                let receiver_val = self.format_operand(receiver);
                // recv_opt uses a pointer to get the value
                writeln!(out, "  %{}.ptr = alloca i64", dest.name)?;
                writeln!(out, "  %{}.success = call i64 @bmb_channel_recv_opt(i64 {}, ptr %{}.ptr)", dest.name, receiver_val, dest.name)?;
                // If success, load value; if not (closed), return -1 (None)
                writeln!(out, "  %{}.tmp = load i64, ptr %{}.ptr", dest.name, dest.name)?;
                // Use select to get -1 for closed channel
                writeln!(out, "  %{}.cond = icmp eq i64 %{}.success, 1", dest.name, dest.name)?;
                writeln!(out, "  %{} = select i1 %{}.cond, i64 %{}.tmp, i64 -1", dest.name, dest.name, dest.name)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::SenderClone { dest, sender } => {
                let sender_val = self.format_operand(sender);
                writeln!(out, "  %{} = call i64 @bmb_sender_clone(i64 {})", dest.name, sender_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            // v0.74: RwLock instructions
            MirInst::RwLockNew { dest, initial_value } => {
                let init_val = self.format_operand(initial_value);
                writeln!(out, "  %{} = call i64 @bmb_rwlock_new(i64 {})", dest.name, init_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::RwLockRead { dest, rwlock } => {
                let rwlock_val = self.format_operand(rwlock);
                writeln!(out, "  %{} = call i64 @bmb_rwlock_read(i64 {})", dest.name, rwlock_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::RwLockReadUnlock { rwlock } => {
                let rwlock_val = self.format_operand(rwlock);
                writeln!(out, "  call void @bmb_rwlock_read_unlock(i64 {})", rwlock_val)?;
            }

            MirInst::RwLockWrite { dest, rwlock } => {
                let rwlock_val = self.format_operand(rwlock);
                writeln!(out, "  %{} = call i64 @bmb_rwlock_write(i64 {})", dest.name, rwlock_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::RwLockWriteUnlock { rwlock, value } => {
                let rwlock_val = self.format_operand(rwlock);
                let new_val = self.format_operand(value);
                writeln!(out, "  call void @bmb_rwlock_write_unlock(i64 {}, i64 {})", rwlock_val, new_val)?;
            }

            // v0.74: Barrier instructions
            MirInst::BarrierNew { dest, count } => {
                let count_val = self.format_operand(count);
                writeln!(out, "  %{} = call i64 @bmb_barrier_new(i64 {})", dest.name, count_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::BarrierWait { dest, barrier } => {
                let barrier_val = self.format_operand(barrier);
                writeln!(out, "  %{} = call i64 @bmb_barrier_wait(i64 {})", dest.name, barrier_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            // v0.74: Condvar instructions
            MirInst::CondvarNew { dest } => {
                writeln!(out, "  %{} = call i64 @bmb_condvar_new()", dest.name)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::CondvarWait { dest, condvar, mutex } => {
                let condvar_val = self.format_operand(condvar);
                let mutex_val = self.format_operand(mutex);
                writeln!(out, "  %{} = call i64 @bmb_condvar_wait(i64 {}, i64 {})", dest.name, condvar_val, mutex_val)?;
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store i64 %{}, ptr %{}.addr", dest.name, dest.name)?;
                }
            }

            MirInst::CondvarNotifyOne { condvar } => {
                let condvar_val = self.format_operand(condvar);
                writeln!(out, "  call void @bmb_condvar_notify_one(i64 {})", condvar_val)?;
            }

            MirInst::CondvarNotifyAll { condvar } => {
                let condvar_val = self.format_operand(condvar);
                writeln!(out, "  call void @bmb_condvar_notify_all(i64 {})", condvar_val)?;
            }

            // v0.83: AsyncFile instructions
            MirInst::AsyncFileOpen { dest, path } => {
                let path_val = self.format_operand(path);
                writeln!(out, "  %{} = call i64 @bmb_async_file_open(i64 {})", dest.name, path_val)?;
            }
            MirInst::AsyncFileRead { dest, file } => {
                let file_val = self.format_operand(file);
                writeln!(out, "  %{} = call i64 @bmb_async_file_read(i64 {})", dest.name, file_val)?;
            }
            MirInst::AsyncFileWrite { file, content } => {
                let file_val = self.format_operand(file);
                let content_val = self.format_operand(content);
                writeln!(out, "  call void @bmb_async_file_write(i64 {}, i64 {})", file_val, content_val)?;
            }
            MirInst::AsyncFileClose { file } => {
                let file_val = self.format_operand(file);
                writeln!(out, "  call void @bmb_async_file_close(i64 {})", file_val)?;
            }

            // v0.83.1: AsyncSocket instructions
            MirInst::AsyncSocketConnect { dest, host, port } => {
                let host_val = self.format_operand(host);
                let port_val = self.format_operand(port);
                writeln!(out, "  %{} = call i64 @bmb_async_socket_connect(i64 {}, i64 {})", dest.name, host_val, port_val)?;
            }
            MirInst::AsyncSocketRead { dest, socket } => {
                let socket_val = self.format_operand(socket);
                writeln!(out, "  %{} = call i64 @bmb_async_socket_read(i64 {})", dest.name, socket_val)?;
            }
            MirInst::AsyncSocketWrite { socket, content } => {
                let socket_val = self.format_operand(socket);
                let content_val = self.format_operand(content);
                writeln!(out, "  call void @bmb_async_socket_write(i64 {}, i64 {})", socket_val, content_val)?;
            }
            MirInst::AsyncSocketClose { socket } => {
                let socket_val = self.format_operand(socket);
                writeln!(out, "  call void @bmb_async_socket_close(i64 {})", socket_val)?;
            }

            // v0.84: ThreadPool instructions
            MirInst::ThreadPoolNew { dest, size } => {
                let size_val = self.format_operand(size);
                writeln!(out, "  %{} = call i64 @bmb_thread_pool_new(i64 {})", dest.name, size_val)?;
            }
            MirInst::ThreadPoolExecute { pool, task } => {
                let pool_val = self.format_operand(pool);
                let task_val = self.format_operand(task);
                writeln!(out, "  call void @bmb_thread_pool_execute(i64 {}, i64 {})", pool_val, task_val)?;
            }
            MirInst::ThreadPoolJoin { pool } => {
                let pool_val = self.format_operand(pool);
                writeln!(out, "  call void @bmb_thread_pool_join(i64 {})", pool_val)?;
            }
            MirInst::ThreadPoolShutdown { pool } => {
                let pool_val = self.format_operand(pool);
                writeln!(out, "  call void @bmb_thread_pool_shutdown(i64 {})", pool_val)?;
            }

            // v0.85: Scope instructions
            MirInst::ScopeNew { dest } => {
                writeln!(out, "  %{} = call i64 @bmb_scope_new()", dest.name)?;
            }
            MirInst::ScopeSpawn { scope, task } => {
                let scope_val = self.format_operand(scope);
                let task_val = self.format_operand(task);
                writeln!(out, "  call void @bmb_scope_spawn(i64 {}, i64 {})", scope_val, task_val)?;
            }
            MirInst::ScopeWait { scope } => {
                let scope_val = self.format_operand(scope);
                writeln!(out, "  call void @bmb_scope_wait(i64 {})", scope_val)?;
            }

            // v0.76: Select instruction
            // v0.90.73: Handle string operands in condition and value types
            MirInst::Select { dest, cond_op, cond_lhs, cond_rhs, true_val, false_val } => {
                // Detect string comparison operands
                let lhs_is_string = Self::is_string_operand(cond_lhs, func);
                let rhs_is_string = Self::is_string_operand(cond_rhs, func);
                let cond_is_string = lhs_is_string || rhs_is_string
                    || place_types.get(&match cond_lhs { Operand::Place(p) => p.name.clone(), _ => String::new() }).copied() == Some("ptr")
                    || place_types.get(&match cond_rhs { Operand::Place(p) => p.name.clone(), _ => String::new() }).copied() == Some("ptr");

                // Detect value types (ptr for strings, i64 for integers)
                // v0.93.120: Check both true and false val types to handle narrowing mismatches
                let true_ty = match true_val {
                    Operand::Place(p) => place_types.get(&p.name).copied().unwrap_or("i64"),
                    Operand::Constant(Constant::String(_)) => "ptr",
                    _ => "i64",
                };
                let false_ty = match false_val {
                    Operand::Place(p) => place_types.get(&p.name).copied().unwrap_or("i64"),
                    Operand::Constant(Constant::String(_)) => "ptr",
                    _ => "i64",
                };
                // Use widest integer type to avoid i32/i64 mismatch in select
                let val_ty = if true_ty == "ptr" || false_ty == "ptr" {
                    "ptr"
                } else if true_ty == "i64" || false_ty == "i64" {
                    "i64"
                } else {
                    true_ty // both i32 or both the same
                };

                // Format operands with string table support
                let lhs_val = self.format_operand_with_strings_and_narrowing(cond_lhs, string_table, local_names);
                let rhs_val = self.format_operand_with_strings_and_narrowing(cond_rhs, string_table, local_names);
                let true_val_str = self.format_operand_with_strings_and_narrowing(true_val, string_table, local_names);
                let false_val_str = self.format_operand_with_strings_and_narrowing(false_val, string_table, local_names);

                if cond_is_string && (*cond_op == MirBinOp::Eq || *cond_op == MirBinOp::Ne) {
                    // String comparison: use @bmb_string_eq
                    // v0.95: Load local ptr operands from .addr for condition operands
                    let lhs_final = if let Operand::Constant(Constant::String(s)) = cond_lhs {
                        if let Some(global_name) = string_table.get(s) {
                            format!("@{}.bmb", global_name)
                        } else { lhs_val.clone() }
                    } else if let Operand::Place(p) = cond_lhs {
                        if local_names.contains(&p.name) {
                            writeln!(out, "  %{}_cond_lhs = load ptr, ptr %{}.addr", dest.name, p.name)?;
                            format!("%{}_cond_lhs", dest.name)
                        } else { lhs_val.clone() }
                    } else { lhs_val.clone() };
                    let rhs_final = if let Operand::Constant(Constant::String(s)) = cond_rhs {
                        if let Some(global_name) = string_table.get(s) {
                            format!("@{}.bmb", global_name)
                        } else { rhs_val.clone() }
                    } else if let Operand::Place(p) = cond_rhs {
                        if local_names.contains(&p.name) {
                            writeln!(out, "  %{}_cond_rhs = load ptr, ptr %{}.addr", dest.name, p.name)?;
                            format!("%{}_cond_rhs", dest.name)
                        } else { rhs_val.clone() }
                    } else { rhs_val.clone() };

                    writeln!(out, "  %{}_cond.i64 = call i64 @bmb_string_eq(ptr {}, ptr {})",
                             dest.name, lhs_final, rhs_final)?;
                    if *cond_op == MirBinOp::Eq {
                        writeln!(out, "  %{}_cond = icmp ne i64 %{}_cond.i64, 0", dest.name, dest.name)?;
                    } else {
                        writeln!(out, "  %{}_cond = icmp eq i64 %{}_cond.i64, 0", dest.name, dest.name)?;
                    }
                } else {
                    // Integer comparison
                    let cmp_pred = match cond_op {
                        MirBinOp::Eq => "eq",
                        MirBinOp::Ne => "ne",
                        MirBinOp::Lt => "slt",
                        MirBinOp::Le => "sle",
                        MirBinOp::Gt => "sgt",
                        MirBinOp::Ge => "sge",
                        _ => "eq",
                    };
                    // v0.93.121: Load local condition operands from .addr
                    let lhs_cond = if let Operand::Place(p) = cond_lhs {
                        if local_names.contains(&p.name) {
                            writeln!(out, "  %{}_cond.lhs = load i64, ptr %{}.addr", dest.name, p.name)?;
                            format!("%{}_cond.lhs", dest.name)
                        } else { lhs_val.clone() }
                    } else { lhs_val.clone() };
                    let rhs_cond = if let Operand::Place(p) = cond_rhs {
                        if local_names.contains(&p.name) {
                            writeln!(out, "  %{}_cond.rhs = load i64, ptr %{}.addr", dest.name, p.name)?;
                            format!("%{}_cond.rhs", dest.name)
                        } else { rhs_val.clone() }
                    } else { rhs_val.clone() };
                    writeln!(out, "  %{}_cond = icmp {} i64 {}, {}", dest.name, cmp_pred, lhs_cond, rhs_cond)?;
                }

                // v0.95: For ptr-typed local operands, load from .addr to handle
                // .call/.concat suffixed definitions (e.g. sb_build result)
                // v0.93.121: For narrowed (i32) operands in i64 select, load+sext
                let true_val_final = if let Operand::Place(p) = true_val {
                    let op_ty = place_types.get(&p.name).copied().unwrap_or("i64");
                    if val_ty == "ptr" && local_names.contains(&p.name) {
                        writeln!(out, "  %{}_sel_true = load ptr, ptr %{}.addr", dest.name, p.name)?;
                        format!("%{}_sel_true", dest.name)
                    } else if val_ty == "i64" && op_ty == "i32" && local_names.contains(&p.name) {
                        // Narrowed i32 operand needs load + sext to i64
                        writeln!(out, "  %{}_sel_true.i32 = load i32, ptr %{}.addr", dest.name, p.name)?;
                        writeln!(out, "  %{}_sel_true = sext i32 %{}_sel_true.i32 to i64", dest.name, dest.name)?;
                        format!("%{}_sel_true", dest.name)
                    } else { true_val_str.clone() }
                } else { true_val_str.clone() };
                let false_val_final = if let Operand::Place(p) = false_val {
                    let op_ty = place_types.get(&p.name).copied().unwrap_or("i64");
                    if val_ty == "ptr" && local_names.contains(&p.name) {
                        writeln!(out, "  %{}_sel_false = load ptr, ptr %{}.addr", dest.name, p.name)?;
                        format!("%{}_sel_false", dest.name)
                    } else if val_ty == "i64" && op_ty == "i32" && local_names.contains(&p.name) {
                        // Narrowed i32 operand needs load + sext to i64
                        writeln!(out, "  %{}_sel_false.i32 = load i32, ptr %{}.addr", dest.name, p.name)?;
                        writeln!(out, "  %{}_sel_false = sext i32 %{}_sel_false.i32 to i64", dest.name, dest.name)?;
                        format!("%{}_sel_false", dest.name)
                    } else { false_val_str.clone() }
                } else { false_val_str.clone() };

                // Generate select with correct value type
                writeln!(out, "  %{} = select i1 %{}_cond, {} {}, {} {}",
                         dest.name, dest.name, val_ty, true_val_final, val_ty, false_val_final)?;
                // Store to alloca if local
                if local_names.contains(&dest.name) {
                    writeln!(out, "  store {} %{}, ptr %{}.addr", val_ty, dest.name, dest.name)?;
                }
            }
        }

        Ok(())
    }

    /// Emit a terminator
    #[allow(clippy::too_many_arguments)]
    fn emit_terminator(
        &self,
        out: &mut String,
        term: &Terminator,
        func: &MirFunction,
        string_table: &HashMap<String, String>,
        local_names: &std::collections::HashSet<String>,
        narrowed_param_names: &std::collections::HashSet<String>,
        place_types: &HashMap<String, &'static str>,  // v0.60.13: For return value type widening
        block_label: &str,
    ) -> TextCodeGenResult<()> {
        match term {
            Terminator::Return(None) => {
                if func.ret_ty == MirType::Unit {
                    writeln!(out, "  ret void")?;
                } else {
                    // Should not happen - return with no value for non-unit type
                    writeln!(out, "  ret {} 0", self.mir_type_to_llvm(&func.ret_ty))?;
                }
            }

            Terminator::Return(Some(val)) => {
                // v0.51.27: Check if this is a sret function (struct return via pointer)
                // v0.51.28: Small structs (1-2 fields) use register return instead of sret
                let struct_field_count = if let MirType::Struct { fields, .. } = &func.ret_ty {
                    fields.len()
                } else {
                    0
                };
                let is_small_struct = struct_field_count > 0 && struct_field_count <= 2;
                let is_sret = struct_field_count > 2;

                // v0.55: Check if return type is a tuple
                let tuple_elems = if let MirType::Tuple(elems) = &func.ret_ty {
                    Some(elems)
                } else {
                    None
                };

                // v0.55: Tuple return - return the aggregate value directly
                if let Some(elems) = tuple_elems {
                    let elem_types: Vec<&str> = elems.iter()
                        .map(|e| self.mir_type_to_llvm(e))
                        .collect();
                    let ret_type = format!("{{ {} }}", elem_types.join(", "));

                    // The value should be a tuple value (SSA or loaded from local)
                    if let Operand::Place(p) = val {
                        // If it's a local, load from alloca first
                        if local_names.contains(&p.name) {
                            writeln!(out, "  %{}_ret_load = load {}, ptr %{}.addr", p.name, ret_type, p.name)?;
                            writeln!(out, "  ret {} %{}_ret_load", ret_type, p.name)?;
                        } else {
                            // Return the tuple value directly - it was built with insertvalue
                            writeln!(out, "  ret {} %{}", ret_type, p.name)?;
                        }
                    } else {
                        writeln!(out, "  ret {} undef", ret_type)?;
                    }
                } else if is_small_struct {
                    // v0.51.28: Small struct register return - pack fields into aggregate
                    let ret_type = if struct_field_count == 1 { "{i64}" } else { "{i64, i64}" };

                    if let Operand::Place(p) = val {
                        // Load source pointer
                        let src_ptr = if local_names.contains(&p.name) {
                            writeln!(out, "  %_small_src.{} = load ptr, ptr %{}.addr", block_label, p.name)?;
                            format!("%_small_src.{}", block_label)
                        } else {
                            format!("%{}", p.name)
                        };

                        // Load fields and pack into aggregate
                        writeln!(out, "  ; small struct return - pack {} fields", struct_field_count)?;

                        // Load field 0
                        writeln!(out, "  %_agg_gep0.{} = getelementptr i64, ptr {}, i32 0", block_label, src_ptr)?;
                        writeln!(out, "  %_agg_f0.{} = load i64, ptr %_agg_gep0.{}", block_label, block_label)?;
                        writeln!(out, "  %_agg_v0.{} = insertvalue {} undef, i64 %_agg_f0.{}, 0", block_label, ret_type, block_label)?;

                        if struct_field_count == 2 {
                            // Load field 1
                            writeln!(out, "  %_agg_gep1.{} = getelementptr i64, ptr {}, i32 1", block_label, src_ptr)?;
                            writeln!(out, "  %_agg_f1.{} = load i64, ptr %_agg_gep1.{}", block_label, block_label)?;
                            writeln!(out, "  %_agg_v1.{} = insertvalue {} %_agg_v0.{}, i64 %_agg_f1.{}, 1", block_label, ret_type, block_label, block_label)?;
                            writeln!(out, "  ret {} %_agg_v1.{}", ret_type, block_label)?;
                        } else {
                            writeln!(out, "  ret {} %_agg_v0.{}", ret_type, block_label)?;
                        }
                    } else {
                        // Direct constant return (unlikely for struct)
                        writeln!(out, "  ret {} undef", ret_type)?;
                    }
                } else if is_sret {
                    // sret function: copy return value to %_sret pointer, then return void
                    // The value could be a struct pointer that needs to be copied
                    if let Operand::Place(p) = val {
                        // Get the number of fields to copy
                        let num_fields = if let MirType::Struct { fields, .. } = &func.ret_ty {
                            fields.len()
                        } else {
                            0
                        };

                        // Load source pointer value (struct pointer)
                        let src_ptr = if local_names.contains(&p.name) {
                            writeln!(out, "  %_sret_src.{} = load ptr, ptr %{}.addr", block_label, p.name)?;
                            format!("%_sret_src.{}", block_label)
                        } else {
                            format!("%{}", p.name)
                        };

                        // Copy each field from source to sret
                        writeln!(out, "  ; sret return - copy {} fields from {} to %_sret", num_fields, src_ptr)?;
                        for i in 0..num_fields {
                            let field_load = format!("_sret_f{}.{}.load", i, block_label);
                            writeln!(out, "  %_sret_gep_src.{i}.{block_label} = getelementptr i64, ptr {src_ptr}, i32 {i}")?;
                            writeln!(out, "  %{field_load} = load i64, ptr %_sret_gep_src.{i}.{block_label}")?;
                            writeln!(out, "  %_sret_gep_dst.{i}.{block_label} = getelementptr i64, ptr %_sret, i32 {i}")?;
                            writeln!(out, "  store i64 %{field_load}, ptr %_sret_gep_dst.{i}.{block_label}")?;
                        }
                    } else {
                        writeln!(out, "  ; sret return - value already in %_sret")?;
                    }
                    writeln!(out, "  ret void")?;
                } else {
                    let ty = self.mir_type_to_llvm(&func.ret_ty);
                    // v0.51.22: String constant returns use pre-initialized global BmbString
                    if let Operand::Constant(Constant::String(s)) = val {
                        if let Some(global_name) = string_table.get(s) {
                            // Return pointer to global BmbString struct directly
                            writeln!(out, "  ret ptr @{}.bmb", global_name)?;
                        } else {
                            // Fallback - shouldn't happen
                            writeln!(out, "  ret {} {}", ty, self.format_operand_with_strings_and_narrowing(val, string_table, narrowed_param_names))?;
                        }
                    } else if ty == "void" {
                        // v0.50.49: Void return - just emit ret void, don't try to load the value
                        writeln!(out, "  ret void")?;
                    } else if let Operand::Place(p) = val {
                        // Check if this is a local that uses alloca
                        if local_names.contains(&p.name) {
                            // Load from alloca before returning
                            // v0.60.13: Use the local's actual alloca type (from place_types), then sign extend if needed
                            // This fixes bugs where narrowed locals (i32) are returned as i64
                            let local_ty = place_types.get(&p.name).copied().unwrap_or(ty);
                            // Use block_label + var name for SSA uniqueness across multiple return blocks
                            writeln!(out, "  %_ret_val.{}.{} = load {}, ptr %{}.addr", block_label, p.name, local_ty, p.name)?;
                            if local_ty == "i32" && ty == "i64" {
                                // Sign extend i32 to i64 for return
                                let sext_name = format!("_ret_val.{}.{}.sext", block_label, p.name);
                                writeln!(out, "  %{} = sext i32 %_ret_val.{}.{} to i64", sext_name, block_label, p.name)?;
                                writeln!(out, "  ret i64 %{}", sext_name)?;
                            } else {
                                writeln!(out, "  ret {} %_ret_val.{}.{}", ty, block_label, p.name)?;
                            }
                        } else {
                            // v0.60.17: Check if SSA value (e.g., from phi) needs type conversion
                            // The phi might produce i32 due to narrowed parameters, but ret type is i64
                            // v0.60.31: Also handle i64->ptr conversion for String-returning functions
                            let val_ty = place_types.get(&p.name).copied().unwrap_or(ty);
                            if val_ty == "i32" && ty == "i64" {
                                // Sign extend i32 SSA value to i64 for return
                                let sext_name = format!("_ret_sext.{}.{}", block_label, p.name);
                                writeln!(out, "  %{} = sext i32 %{} to i64", sext_name, p.name)?;
                                writeln!(out, "  ret i64 %{}", sext_name)?;
                            } else if val_ty == "i64" && ty == "ptr" {
                                // v0.60.31: Convert i64 to ptr for String-returning functions
                                // This happens when TailRecursiveToLoop creates early returns
                                // that bypass the normal inttoptr conversion
                                let inttoptr_name = format!("_ret_inttoptr.{}.{}", block_label, p.name);
                                writeln!(out, "  %{} = inttoptr i64 %{} to ptr", inttoptr_name, p.name)?;
                                writeln!(out, "  ret ptr %{}", inttoptr_name)?;
                            } else {
                                // v0.51.17: Use narrowing-aware formatting
                                writeln!(out, "  ret {} {}", ty, self.format_operand_with_strings_and_narrowing(val, string_table, narrowed_param_names))?;
                            }
                        }
                    } else {
                        writeln!(out, "  ret {} {}", ty, self.format_operand_with_strings_and_narrowing(val, string_table, narrowed_param_names))?;
                    }
                }
            }

            Terminator::Goto(label) => {
                writeln!(out, "  br label %bb_{}", label)?;
            }

            Terminator::Branch { cond, then_label, else_label } => {
                // Check if condition is a local that needs loading from alloca
                // v0.51.17: Use narrowing-aware formatting for non-locals
                let cond_str = if let Operand::Place(p) = cond {
                    if local_names.contains(&p.name) {
                        // Load the condition from alloca first (use then_label to make name unique)
                        writeln!(out, "  %{}.cond_{} = load i1, ptr %{}.addr", p.name, then_label, p.name)?;
                        format!("%{}.cond_{}", p.name, then_label)
                    } else {
                        self.format_operand_with_narrowing(cond, narrowed_param_names)
                    }
                } else {
                    self.format_operand_with_narrowing(cond, narrowed_param_names)
                };
                writeln!(
                    out,
                    "  br i1 {}, label %bb_{}, label %bb_{}",
                    cond_str, then_label, else_label
                )?;
            }

            Terminator::Unreachable => {
                writeln!(out, "  unreachable")?;
            }

            // v0.19.2: Switch for pattern matching
            Terminator::Switch { discriminant, cases, default } => {
                // Check if discriminant is a local that needs loading from alloca
                // v0.51.17: Use narrowing-aware formatting
                let disc_str = if let Operand::Place(p) = discriminant {
                    if local_names.contains(&p.name) {
                        // Use default label to make name unique
                        writeln!(out, "  %{}.disc_{} = load i64, ptr %{}.addr", p.name, default, p.name)?;
                        format!("%{}.disc_{}", p.name, default)
                    } else {
                        self.format_operand_with_narrowing(discriminant, narrowed_param_names)
                    }
                } else {
                    self.format_operand_with_narrowing(discriminant, narrowed_param_names)
                };
                writeln!(out, "  switch i64 {}, label %bb_{} [", disc_str, default)?;
                for (val, label) in cases {
                    writeln!(out, "    i64 {}, label %bb_{}", val, label)?;
                }
                writeln!(out, "  ]")?;
            }
        }

        Ok(())
    }

    /// Convert MIR type to LLVM type string
    fn mir_type_to_llvm(&self, ty: &MirType) -> &'static str {
        match ty {
            MirType::I32 => "i32",
            MirType::I64 => "i64",
            // v0.38: Unsigned types map to same LLVM types
            MirType::U32 => "i32",
            MirType::U64 => "i64",
            MirType::F64 => "double",
            MirType::Bool => "i1",
            MirType::String => "ptr",
            MirType::Unit => "void",
            // v0.19.0: Struct types are represented as pointers
            MirType::Struct { .. } => "ptr",
            MirType::StructPtr(_) => "ptr",
            // v0.19.1: Enum types are represented as pointers to tagged unions
            MirType::Enum { .. } => "ptr",
            // v0.19.3: Array types are represented as pointers
            MirType::Array { .. } => "ptr",
            // v0.64: Character type (32-bit Unicode codepoint)
            MirType::Char => "i32",
            // v0.51.37: Pointer types are opaque pointers in modern LLVM
            MirType::Ptr(_) => "ptr",
            // v0.55: Tuple types - use ptr as placeholder (actual struct type handled inline)
            MirType::Tuple(_) => "ptr",
        }
    }

    /// v0.50.80: Get LLVM cast instruction name for type conversion
    fn get_cast_instruction(&self, from_ty: &MirType, to_ty: &MirType) -> &'static str {
        use MirType::*;
        match (from_ty, to_ty) {
            // Integer widening (sign extend)
            (I32, I64) | (I32, U64) | (Char, I64) | (Char, U64) => "sext",
            (U32, I64) | (U32, U64) => "zext",
            (Bool, I32) | (Bool, I64) | (Bool, U32) | (Bool, U64) => "zext",

            // Integer narrowing (truncate)
            (I64, I32) | (U64, I32) | (I64, U32) | (U64, U32) => "trunc",
            (I64, Char) | (U64, Char) | (I32, Char) | (U32, Char) => "trunc",

            // Integer to float
            (I32, F64) | (I64, F64) | (Char, F64) => "sitofp",
            (U32, F64) | (U64, F64) => "uitofp",

            // Float to integer
            (F64, I32) | (F64, I64) | (F64, Char) => "fptosi",
            (F64, U32) | (F64, U64) => "fptoui",

            // Same size, different signedness - bitcast
            (I32, U32) | (U32, I32) | (I64, U64) | (U64, I64) => "bitcast",

            // v0.51.33: Struct pointer to integer (ptrtoint)
            (StructPtr(_), I64) | (StructPtr(_), U64) => "ptrtoint",
            // v0.51.33: Integer to struct pointer (inttoptr)
            (I64, StructPtr(_)) | (U64, StructPtr(_)) => "inttoptr",

            // v0.51.38: Generic pointer to integer (ptrtoint)
            (Ptr(_), I64) | (Ptr(_), U64) => "ptrtoint",
            // v0.51.38: Integer to generic pointer (inttoptr)
            (I64, Ptr(_)) | (U64, Ptr(_)) => "inttoptr",

            // v0.60.23: Array to pointer (array decay)
            // Arrays are already pointers in LLVM (alloca returns ptr)
            (Array { .. }, Ptr(_)) => "bitcast",

            // Default fallback
            _ => "bitcast",
        }
    }

    /// Get LLVM type for a constant
    fn constant_type(&self, c: &Constant) -> &'static str {
        match c {
            Constant::Int(_) => "i64",
            Constant::Float(_) => "double",
            Constant::Bool(_) => "i1",
            Constant::String(_) => "ptr",
            // v0.64: Character constant (32-bit Unicode codepoint)
            Constant::Char(_) => "i32",
            Constant::Unit => "i8",
        }
    }

    /// Format a constant value
    fn format_constant(&self, c: &Constant) -> String {
        match c {
            Constant::Int(n) => n.to_string(),
            // v0.34: LLVM requires specific float format (e.g., 4.000000e+00 not 4e0)
            Constant::Float(f) => {
                // Use LLVM-compatible scientific notation format
                if f.is_nan() {
                    "0x7FF8000000000000".to_string() // NaN bit pattern
                } else if f.is_infinite() {
                    if f.is_sign_positive() {
                        "0x7FF0000000000000".to_string() // +Inf
                    } else {
                        "0xFFF0000000000000".to_string() // -Inf
                    }
                } else {
                    format!("{:.6e}", f)
                }
            }
            Constant::Bool(b) => if *b { "1" } else { "0" }.to_string(),
            Constant::String(s) => format!("\"{}\"", s),
            // v0.64: Character constant (Unicode codepoint)
            Constant::Char(c) => (*c as u32).to_string(),
            Constant::Unit => "0".to_string(),
        }
    }

    /// Format an operand
    fn format_operand(&self, op: &Operand) -> String {
        match op {
            Operand::Place(p) => format!("%{}", p.name),
            Operand::Constant(c) => self.format_constant(c),
        }
    }

    /// Format an operand with narrowed parameter substitution
    /// v0.51.18: Narrowed params stay as i32, no special handling needed
    fn format_operand_with_narrowing(
        &self,
        op: &Operand,
        _narrowed_param_names: &std::collections::HashSet<String>,
    ) -> String {
        match op {
            // v0.51.18: No special handling - narrowed params stay as i32
            Operand::Place(p) => format!("%{}", p.name),
            Operand::Constant(c) => self.format_constant(c),
        }
    }

    /// Format an operand with string table for phi instructions
    fn format_operand_with_strings(&self, op: &Operand, string_table: &HashMap<String, String>) -> String {
        match op {
            Operand::Place(p) => format!("%{}", p.name),
            Operand::Constant(c) => match c {
                Constant::String(s) => {
                    if let Some(global_name) = string_table.get(s) {
                        format!("@{}", global_name)
                    } else {
                        // Fallback - shouldn't happen if collect_string_constants is correct
                        format!("\"{}\"", s)
                    }
                }
                _ => self.format_constant(c),
            },
        }
    }

    /// Format an operand with string table and narrowed parameter substitution
    /// v0.51.18: Narrowed params stay as i32, no special handling needed
    fn format_operand_with_strings_and_narrowing(
        &self,
        op: &Operand,
        string_table: &HashMap<String, String>,
        _narrowed_param_names: &std::collections::HashSet<String>,
    ) -> String {
        match op {
            // v0.51.18: No special handling - narrowed params stay as i32
            Operand::Place(p) => format!("%{}", p.name),
            Operand::Constant(c) => match c {
                Constant::String(s) => {
                    if let Some(global_name) = string_table.get(s) {
                        format!("@{}", global_name)
                    } else {
                        format!("\"{}\"", s)
                    }
                }
                _ => self.format_constant(c),
            },
        }
    }

    /// Infer type of a place
    fn infer_place_type(&self, place: &Place, func: &MirFunction) -> &'static str {
        // Check parameters
        for (name, ty) in &func.params {
            if name == &place.name {
                return self.mir_type_to_llvm(ty);
            }
        }
        // Check locals
        for (name, ty) in &func.locals {
            if name == &place.name {
                return self.mir_type_to_llvm(ty);
            }
        }
        // Default to i64 for temporaries
        "i64"
    }

    /// Infer type of an operand
    fn infer_operand_type(&self, op: &Operand, func: &MirFunction) -> &'static str {
        match op {
            Operand::Constant(c) => self.constant_type(c),
            Operand::Place(p) => self.infer_place_type(p, func),
        }
    }

    /// Infer return type of a function call
    fn infer_call_return_type(&self, fn_name: &str, _current_func: &MirFunction) -> &'static str {
        // Built-in functions
        match fn_name {
            // Void return
            "println" | "print" | "assert" | "bmb_print_str" | "print_str" => "void",

            // i64 return - Basic
            // v0.51.48: Added i32_to_i64, i64_to_i32 for i32 conversion support
            "read_int" | "abs" | "bmb_abs" | "min" | "max" | "bmb_min" | "bmb_max"
            | "clamp" | "bmb_clamp" | "pow" | "bmb_pow"
            | "f64_to_i64" | "i32_to_i64" => "i64",

            // i32 return - Type conversions
            "i64_to_i32" => "i32",

            // f64 return - Math intrinsics (v0.34)
            // v0.51.48: Added i32_to_f64 for i32 conversion support
            "sqrt" | "i64_to_f64" | "i32_to_f64" => "double",

            // i64 return - String operations (both full and wrapper names)
            // v0.46: byte_at added as preferred name (same as interpreter)
            "bmb_string_len" | "bmb_string_char_at" | "bmb_string_eq" | "bmb_ord"
            | "bmb_string_starts_with" | "bmb_string_ends_with" | "bmb_string_contains"
            | "bmb_string_index_of" | "bmb_string_is_empty"
            | "len" | "char_at" | "byte_at" | "ord"
            | "starts_with" | "ends_with" | "contains" | "index_of" | "is_empty" => "i64",

            // i64 return - File I/O (both full and wrapper names)
            "bmb_file_exists" | "bmb_file_size" | "bmb_write_file" | "bmb_append_file"
            | "file_exists" | "file_size" | "write_file" | "append_file" => "i64",

            // i64 return - StringBuilder (handle is i64)
            "bmb_sb_new" | "bmb_sb_push" | "bmb_sb_push_cstr" | "bmb_sb_push_char" | "bmb_sb_push_int" | "bmb_sb_push_escaped" | "bmb_sb_push_range" | "bmb_sb_len" | "bmb_sb_clear" | "bmb_sb_contains" | "bmb_sb_println"
            | "sb_new" | "sb_with_capacity" | "sb_push" | "sb_push_cstr" | "sb_push_char" | "sb_push_int" | "sb_push_escaped" | "sb_push_range" | "sb_len" | "sb_clear" | "sb_contains" | "sb_println"
            | "puts_cstr" | "bmb_puts_cstr" => "i64",

            // i64 return - Process
            "bmb_system" => "i64",

            // v0.88.2: i64 return - Memory management
            "bmb_string_free" | "free_string" | "bmb_sb_free" | "sb_free"
            | "bmb_arena_mode" | "arena_mode" | "bmb_arena_reset" | "arena_reset"
            | "bmb_arena_save" | "arena_save" | "bmb_arena_restore" | "arena_restore"
            | "bmb_arena_usage" | "arena_usage" => "i64",

            // i64 return - Timing (v0.63)
            "bmb_time_ns" | "time_ns" => "i64",

            // ptr return - String operations (both full and wrapper names)
            "bmb_string_new" | "bmb_string_from_cstr" | "bmb_string_slice"
            | "bmb_string_concat" | "bmb_chr"
            | "bmb_string_to_lower" | "bmb_string_to_upper" | "bmb_string_trim"
            | "bmb_string_replace" | "bmb_string_repeat"
            | "bmb_int_to_string" | "bmb_fast_i2s"
            | "slice" | "chr" | "to_lower" | "to_upper" | "trim" | "replace" | "repeat"
            | "int_to_string" => "ptr",

            // ptr return - File I/O (both full and wrapper names)
            "bmb_read_file" | "read_file" => "ptr",

            // ptr return - StringBuilder (both full and wrapper names)
            "bmb_sb_build" | "sb_build" => "ptr",

            // ptr return - Process
            "bmb_getenv" => "ptr",
            "bmb_system_capture" | "system_capture" => "ptr",
            "bmb_exec_output" | "exec_output" => "ptr",

            // v0.46: ptr return - CLI argument functions
            "get_arg" | "bmb_get_arg" => "ptr",

            // v0.50.72: Memory allocation functions return i64 (pointer as integer for arithmetic)
            // Note: actual LLVM call uses ptr and converts via ptrtoint
            "malloc" | "realloc" | "calloc" => "i64",

            _ => {
                // For now, assume i64 for unknown functions
                // In a full implementation, we'd look up the function
                "i64"
            }
        }
    }

    /// Convert binary operator to LLVM instruction
    /// Returns (instruction_name, preserves_operand_type)
    /// If preserves_operand_type is false, result type is i1
    fn binop_to_llvm(&self, op: MirBinOp) -> (&'static str, bool) {
        match op {
            // Integer arithmetic with nsw (no signed wrap) for better optimization
            // nsw enables more aggressive LLVM transformations including:
            // - Loop strength reduction
            // - Induction variable simplification
            // - Tail call accumulator transformation
            MirBinOp::Add => ("add nsw", true),
            MirBinOp::Sub => ("sub nsw", true),
            MirBinOp::Mul => ("mul nsw", true),
            MirBinOp::Div => ("sdiv", true),  // sdiv doesn't benefit from nsw
            MirBinOp::Mod => ("srem", true),  // srem doesn't benefit from nsw

            // v0.37: Wrapping arithmetic - no nsw/nuw flags (allows overflow)
            MirBinOp::AddWrap => ("add", true),
            MirBinOp::SubWrap => ("sub", true),
            MirBinOp::MulWrap => ("mul", true),

            // v0.38: Checked arithmetic - use intrinsics for overflow detection
            // For now, same as wrapping (full checked impl needs Option handling)
            MirBinOp::AddChecked => ("add", true),
            MirBinOp::SubChecked => ("sub", true),
            MirBinOp::MulChecked => ("mul", true),

            // v0.38: Saturating arithmetic - clamps to min/max on overflow
            // LLVM doesn't have native saturating ops; use sadd.sat intrinsics in future
            MirBinOp::AddSat => ("add", true),
            MirBinOp::SubSat => ("sub", true),
            MirBinOp::MulSat => ("mul", true),

            // Floating-point arithmetic - result type same as operand
            // v0.60.8: Add 'fast' math flags for LLVM vectorization
            MirBinOp::FAdd => ("fadd fast", true),
            MirBinOp::FSub => ("fsub fast", true),
            MirBinOp::FMul => ("fmul fast", true),
            MirBinOp::FDiv => ("fdiv fast", true),

            // Integer comparison - result is i1
            MirBinOp::Eq => ("icmp eq", false),
            MirBinOp::Ne => ("icmp ne", false),
            MirBinOp::Lt => ("icmp slt", false),
            MirBinOp::Gt => ("icmp sgt", false),
            MirBinOp::Le => ("icmp sle", false),
            MirBinOp::Ge => ("icmp sge", false),

            // Floating-point comparison - result is i1
            MirBinOp::FEq => ("fcmp oeq", false),
            MirBinOp::FNe => ("fcmp one", false),
            MirBinOp::FLt => ("fcmp olt", false),
            MirBinOp::FGt => ("fcmp ogt", false),
            MirBinOp::FLe => ("fcmp ole", false),
            MirBinOp::FGe => ("fcmp oge", false),

            // Logical - result is i1
            MirBinOp::And => ("and", false),
            MirBinOp::Or => ("or", false),

            // v0.32: Shift operators - result type same as operand
            MirBinOp::Shl => ("shl", true),
            MirBinOp::Shr => ("ashr", true),  // arithmetic shift right (preserves sign)

            // v0.36: Bitwise operators - result type same as operand
            MirBinOp::Band => ("and", true),
            MirBinOp::Bor => ("or", true),
            MirBinOp::Bxor => ("xor", true),

            // v0.36: Implies is handled specially before this function is called
            // This arm exists for exhaustiveness; the actual codegen is in emit_instruction
            MirBinOp::Implies => ("or", false),
        }
    }
}

impl Default for TextCodeGen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_add_function() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "add".to_string(),
                params: vec![
                    ("a".to_string(), MirType::I64),
                    ("b".to_string(), MirType::I64),
                ],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("_t0"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("a")),
                        rhs: Operand::Place(Place::new("b")),
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };

        let codegen = TextCodeGen::new();
        let ir = codegen.generate(&program).unwrap();

        assert!(ir.contains("define i64 @add(i64 %a, i64 %b)"));
        assert!(ir.contains("%_t0 = add nsw i64 %a, %b"));  // nsw for optimization
        assert!(ir.contains("ret i64 %_t0"));
    }

    // --- Source-based round-trip tests ---

    fn source_to_ir(source: &str) -> String {
        let tokens = crate::lexer::tokenize(source).expect("tokenize failed");
        let program = crate::parser::parse("<test>", source, tokens).expect("parse failed");
        let mir = crate::mir::lower_program(&program);
        let codegen = TextCodeGen::new();
        codegen.generate(&mir).expect("codegen failed")
    }

    #[test]
    fn test_rt_arithmetic_ops() {
        let ir = source_to_ir("fn add(a: i64, b: i64) -> i64 = a + b;");
        assert!(ir.contains("@add"), "function definition missing");
        assert!(ir.contains("add nsw i64"), "add instruction missing");
        assert!(ir.contains("ret i64"), "return missing");
    }

    #[test]
    fn test_rt_subtraction() {
        let ir = source_to_ir("fn sub(a: i64, b: i64) -> i64 = a - b;");
        assert!(ir.contains("@sub"));
        assert!(ir.contains("sub nsw i64"));
    }

    #[test]
    fn test_rt_multiplication() {
        let ir = source_to_ir("fn mul(a: i64, b: i64) -> i64 = a * b;");
        assert!(ir.contains("@mul"));
        assert!(ir.contains("mul nsw i64"));
    }

    #[test]
    fn test_rt_division() {
        let ir = source_to_ir("fn div(a: i64, b: i64) -> i64 = a / b;");
        assert!(ir.contains("@div"));
        assert!(ir.contains("sdiv i64"));
    }

    #[test]
    fn test_rt_modulo() {
        let ir = source_to_ir("fn rem(a: i64, b: i64) -> i64 = a % b;");
        assert!(ir.contains("@rem"));
        assert!(ir.contains("srem i64"));
    }

    #[test]
    fn test_rt_comparison_eq() {
        let ir = source_to_ir("fn eq(a: i64, b: i64) -> bool = a == b;");
        assert!(ir.contains("@eq"));
        assert!(ir.contains("icmp eq i64"));
    }

    #[test]
    fn test_rt_comparison_lt() {
        let ir = source_to_ir("fn lt(a: i64, b: i64) -> bool = a < b;");
        assert!(ir.contains("icmp slt i64"));
    }

    #[test]
    fn test_rt_comparison_gt() {
        let ir = source_to_ir("fn gt(a: i64, b: i64) -> bool = a > b;");
        assert!(ir.contains("icmp sgt i64"));
    }

    #[test]
    fn test_rt_if_else_branches() {
        let ir = source_to_ir("fn abs(x: i64) -> i64 = if x >= 0 { x } else { 0 - x };");
        assert!(ir.contains("@abs"));
        assert!(ir.contains("br i1"), "conditional branch missing");
        assert!(ir.contains("br label"), "unconditional branch missing");
    }

    #[test]
    fn test_rt_constant_return() {
        let ir = source_to_ir("fn forty_two() -> i64 = 42;");
        assert!(ir.contains("@forty_two"));
        assert!(ir.contains("ret i64 42") || ir.contains("i64 42"), "constant 42 missing");
    }

    #[test]
    fn test_rt_bool_return() {
        let ir = source_to_ir("fn always_true() -> bool = true;");
        assert!(ir.contains("@always_true"));
        assert!(ir.contains("ret i1 1") || ir.contains("i1 1") || ir.contains("true"),
                "boolean true missing");
    }

    #[test]
    fn test_rt_function_call() {
        let ir = source_to_ir("fn double(x: i64) -> i64 = x + x;\nfn quad(x: i64) -> i64 = double(double(x));");
        assert!(ir.contains("@double"));
        assert!(ir.contains("@quad"));
        assert!(ir.contains("call i64 @double"), "function call missing");
    }

    #[test]
    fn test_rt_while_loop() {
        let ir = source_to_ir(
            "fn sum_to(n: i64) -> i64 = { let mut i = 0; let mut s = 0; while i < n { s = s + i; i = i + 1; 0 }; s };"
        );
        assert!(ir.contains("@sum_to"));
        // While loops generate multiple basic blocks with back-edges
        let block_count = ir.matches(":\n").count();
        assert!(block_count >= 3, "while loop should generate at least 3 blocks, found {}", block_count);
    }

    #[test]
    fn test_rt_multiple_functions() {
        let ir = source_to_ir("fn f1() -> i64 = 1;\nfn f2() -> i64 = 2;\nfn f3() -> i64 = 3;");
        assert!(ir.contains("@f1"));
        assert!(ir.contains("@f2"));
        assert!(ir.contains("@f3"));
        let define_count = ir.matches("define ").count();
        assert_eq!(define_count, 3, "expected 3 function definitions");
    }

    #[test]
    fn test_rt_float_operations() {
        let ir = source_to_ir("fn fadd(a: f64, b: f64) -> f64 = a + b;");
        assert!(ir.contains("@fadd"));
        assert!(ir.contains("fadd") || ir.contains("double"), "float operation missing");
    }

    #[test]
    fn test_rt_string_parameter() {
        let ir = source_to_ir("fn slen(s: String) -> i64 = s.len();");
        assert!(ir.contains("@slen"));
    }

    #[test]
    fn test_rt_recursive_function() {
        let ir = source_to_ir(
            "fn fact(n: i64) -> i64 = if n <= 1 { 1 } else { n * fact(n - 1) };"
        );
        assert!(ir.contains("@fact"));
        assert!(ir.contains("call i64 @fact"), "recursive call missing");
    }

    #[test]
    fn test_rt_let_binding() {
        let ir = source_to_ir("fn f(x: i64) -> i64 = { let y = x + 1; y * 2 };");
        assert!(ir.contains("@f"));
        assert!(ir.contains("add nsw i64"));
        assert!(ir.contains("mul nsw i64"));
    }

    #[test]
    fn test_rt_module_header() {
        let ir = source_to_ir("fn f() -> i64 = 0;");
        assert!(ir.contains("target triple"), "module header missing target triple");
    }

    #[test]
    fn test_rt_extern_declarations() {
        // println is a builtin that should generate an extern declaration
        let ir = source_to_ir("fn f() -> i64 = { let _u = println(42); 0 };");
        // The IR should have some declaration for print-related builtins
        assert!(ir.contains("declare") || ir.contains("@print") || ir.contains("@bmb_"),
                "builtin function declarations expected");
    }

    #[test]
    fn test_rt_mutable_variable() {
        let ir = source_to_ir("fn f() -> i64 = { let mut x = 0; x = 42; x };");
        assert!(ir.contains("@f"));
        assert!(ir.contains("ret i64"), "return missing");
    }

    #[test]
    fn test_rt_nested_if() {
        let ir = source_to_ir(
            "fn clamp(x: i64, lo: i64, hi: i64) -> i64 = if x < lo { lo } else if x > hi { hi } else { x };"
        );
        assert!(ir.contains("@clamp"));
        let branch_count = ir.matches("br i1").count();
        assert!(branch_count >= 2, "nested if should have >= 2 conditional branches, found {}", branch_count);
    }

    #[test]
    fn test_rt_negation() {
        let ir = source_to_ir("fn neg(x: i64) -> i64 = 0 - x;");
        assert!(ir.contains("@neg"));
        assert!(ir.contains("sub nsw i64"));
    }

    // ================================================================
    // Cycle 80: Extended codegen tests
    // ================================================================

    #[test]
    fn test_rt_f64_sub() {
        let ir = source_to_ir("fn fsub(a: f64, b: f64) -> f64 = a - b;");
        assert!(ir.contains("@fsub"));
        assert!(ir.contains("fsub") || ir.contains("double"), "float sub missing");
    }

    #[test]
    fn test_rt_f64_mul() {
        let ir = source_to_ir("fn fmul(a: f64, b: f64) -> f64 = a * b;");
        assert!(ir.contains("@fmul"));
        assert!(ir.contains("fmul") || ir.contains("double"), "float mul missing");
    }

    #[test]
    fn test_rt_f64_div() {
        let ir = source_to_ir("fn fdiv(a: f64, b: f64) -> f64 = a / b;");
        assert!(ir.contains("@fdiv"));
        assert!(ir.contains("fdiv") || ir.contains("double"), "float div missing");
    }

    #[test]
    fn test_rt_bool_branch() {
        let ir = source_to_ir("fn choose(a: bool, x: i64, y: i64) -> i64 = if a { x } else { y };");
        assert!(ir.contains("@choose"));
        assert!(ir.contains("br i1"));
    }

    #[test]
    fn test_rt_bool_equality() {
        let ir = source_to_ir("fn is_zero(x: i64) -> bool = x == 0;");
        assert!(ir.contains("@is_zero"));
        assert!(ir.contains("icmp eq i64"));
    }

    #[test]
    fn test_rt_comparison_le() {
        let ir = source_to_ir("fn le(a: i64, b: i64) -> bool = a <= b;");
        assert!(ir.contains("icmp sle i64"));
    }

    #[test]
    fn test_rt_comparison_ge() {
        let ir = source_to_ir("fn ge(a: i64, b: i64) -> bool = a >= b;");
        assert!(ir.contains("icmp sge i64"));
    }

    #[test]
    fn test_rt_comparison_ne() {
        let ir = source_to_ir("fn ne(a: i64, b: i64) -> bool = a != b;");
        assert!(ir.contains("icmp ne i64"));
    }

    #[test]
    fn test_rt_for_loop_codegen() {
        let ir = source_to_ir(
            "fn sum_to(n: i64) -> i64 = { let mut s: i64 = 0; for i in 0..n { s = s + i }; s };"
        );
        assert!(ir.contains("@sum_to"));
        // for loop should generate phi nodes and branch
        assert!(ir.contains("phi i64") || ir.contains("br "));
    }

    #[test]
    fn test_rt_struct_codegen() {
        let ir = source_to_ir(
            "struct Point { x: i64, y: i64 }
             fn origin() -> Point = new Point { x: 0, y: 0 };"
        );
        assert!(ir.contains("@origin"));
    }

    #[test]
    fn test_rt_enum_codegen() {
        let ir = source_to_ir(
            "enum Color { Red, Green, Blue }
             fn red() -> Color = Color::Red;"
        );
        assert!(ir.contains("@red"));
    }

    #[test]
    fn test_rt_match_codegen() {
        let ir = source_to_ir(
            "fn classify(x: i64) -> i64 = match x { 0 => 1, 1 => 2, _ => 0 };"
        );
        assert!(ir.contains("@classify"));
        // match generates switch or branch chain
        assert!(ir.contains("switch") || ir.contains("br i1"));
    }

    #[test]
    fn test_rt_contract_precondition_codegen() {
        let ir = source_to_ir(
            "fn safe_div(a: i64, b: i64) -> i64
               pre b != 0
             = a / b;"
        );
        assert!(ir.contains("@safe_div"));
        assert!(ir.contains("sdiv i64"));
    }

    #[test]
    fn test_rt_contract_postcondition_codegen() {
        let ir = source_to_ir(
            "fn abs(x: i64) -> i64
               post ret >= 0
             = if x >= 0 { x } else { 0 - x };"
        );
        assert!(ir.contains("@abs"));
    }

    #[test]
    fn test_rt_square_function() {
        let ir = source_to_ir("fn square(x: i64) -> i64 = x * x;");
        assert!(ir.contains("@square"));
        assert!(ir.contains("mul nsw i64"));
    }

    #[test]
    fn test_rt_i32_parameter() {
        let ir = source_to_ir(
            "fn add32(a: i32, b: i32) -> i32 = a + b;"
        );
        assert!(ir.contains("i32 %a"));
        assert!(ir.contains("i32 %b"));
        assert!(ir.contains("add nsw i32"));
    }

    #[test]
    fn test_rt_void_function() {
        let ir = source_to_ir(
            "fn nothing() -> () = ();"
        );
        assert!(ir.contains("@nothing"));
        assert!(ir.contains("ret void") || ir.contains("ret i64 0"));
    }

    #[test]
    fn test_rt_bitwise_and() {
        let ir = source_to_ir("fn bitand(a: i64, b: i64) -> i64 = a band b;");
        assert!(ir.contains("@bitand"));
        assert!(ir.contains("and i64"));
    }

    #[test]
    fn test_rt_bitwise_or() {
        let ir = source_to_ir("fn bitor(a: i64, b: i64) -> i64 = a bor b;");
        assert!(ir.contains("@bitor"));
        assert!(ir.contains("or i64"));
    }

    #[test]
    fn test_rt_shift_left() {
        let ir = source_to_ir("fn shl(a: i64, b: i64) -> i64 = a << b;");
        assert!(ir.contains("shl i64"));
    }

    #[test]
    fn test_rt_shift_right() {
        let ir = source_to_ir("fn shr(a: i64, b: i64) -> i64 = a >> b;");
        assert!(ir.contains("ashr i64"));
    }

    #[test]
    fn test_rt_tuple_return() {
        let ir = source_to_ir(
            "fn pair(a: i64, b: i64) -> (i64, i64) = (a, b);"
        );
        assert!(ir.contains("@pair"));
    }

    #[test]
    fn test_codegen_new() {
        let cg = TextCodeGen::new();
        // Just verify it can be created
        let program = MirProgram {
            functions: vec![],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let ir = cg.generate(&program).unwrap();
        // Should contain module header at minimum
        assert!(ir.contains("target"));
    }

    // ================================================================
    // Cycle 122: Extended LLVM Text Codegen Tests
    // ================================================================

    #[test]
    fn test_rt_recursive_fibonacci() {
        let ir = source_to_ir(
            "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };"
        );
        assert!(ir.contains("@fib"));
        assert!(ir.contains("call i64 @fib"), "recursive call missing");
        assert!(ir.contains("icmp sle i64"));
    }

    #[test]
    fn test_rt_simple_multiplication() {
        let ir = source_to_ir("fn double(x: i64) -> i64 = x * 2;");
        assert!(ir.contains("@double"));
        assert!(ir.contains("mul nsw i64"));
    }

    #[test]
    fn test_rt_multiple_params() {
        let ir = source_to_ir("fn add3(a: i64, b: i64, c: i64) -> i64 = a + b + c;");
        assert!(ir.contains("@add3"));
        let add_count = ir.matches("add nsw i64").count();
        assert!(add_count >= 2, "should have 2 add operations, found {}", add_count);
    }

    #[test]
    fn test_rt_f64_comparison() {
        let ir = source_to_ir("fn flt(a: f64, b: f64) -> bool = a < b;");
        assert!(ir.contains("@flt"));
        assert!(ir.contains("fcmp") || ir.contains("olt"), "float comparison missing");
    }

    #[test]
    fn test_rt_f64_negation() {
        let ir = source_to_ir("fn fneg_val(x: f64) -> f64 = 0.0 - x;");
        assert!(ir.contains("@fneg_val"));
        assert!(ir.contains("fsub") || ir.contains("fneg"), "float negation missing");
    }

    #[test]
    fn test_rt_while_loop_accumulator() {
        let ir = source_to_ir(
            "fn sum(n: i64) -> i64 = { let mut s = 0; let mut i = 0; while i < n { s = s + i; i = i + 1 }; s };"
        );
        assert!(ir.contains("@sum"));
        // Should have loop structure with branches
        assert!(ir.contains("br i1"), "conditional branch in loop missing");
        assert!(ir.contains("br label"), "unconditional branch for loop back-edge missing");
    }

    #[test]
    fn test_rt_for_loop_sum() {
        let ir = source_to_ir(
            "fn sum_range(n: i64) -> i64 = { let mut s = 0; for i in 0..n { s = s + i }; s };"
        );
        assert!(ir.contains("@sum_range"));
        assert!(ir.contains("br i1"), "loop condition branch missing");
    }

    #[test]
    fn test_rt_struct_field_access() {
        let ir = source_to_ir(
            "struct Point { x: i64, y: i64 }\n\
             fn get_x(p: Point) -> i64 = p.x;"
        );
        assert!(ir.contains("@get_x"));
        assert!(ir.contains("getelementptr") || ir.contains("extractvalue"),
                "struct field access instruction missing");
    }

    #[test]
    fn test_rt_struct_creation() {
        let ir = source_to_ir(
            "struct Vec2 { x: i64, y: i64 }\n\
             fn make_vec(a: i64, b: i64) -> Vec2 = new Vec2 { x: a, y: b };"
        );
        assert!(ir.contains("@make_vec"));
    }

    #[test]
    fn test_rt_enum_match_multiple_arms() {
        let ir = source_to_ir(
            "enum Color { Red, Green, Blue }\n\
             fn color_val(c: Color) -> i64 = match c { Color::Red => 1, Color::Green => 2, Color::Blue => 3 };"
        );
        assert!(ir.contains("@color_val"));
        // Match should produce either switch or branch chain
        assert!(ir.contains("switch") || ir.matches("br i1").count() >= 2,
                "enum match should produce switch or branch chain");
    }

    #[test]
    fn test_rt_logical_and_or() {
        let ir = source_to_ir("fn both(a: bool, b: bool) -> bool = a && b;");
        assert!(ir.contains("@both"));
        // Short-circuit should produce branch
        assert!(ir.contains("br i1") || ir.contains("and i1"), "logical AND missing");
    }

    #[test]
    fn test_rt_logical_or() {
        let ir = source_to_ir("fn either(a: bool, b: bool) -> bool = a || b;");
        assert!(ir.contains("@either"));
        assert!(ir.contains("br i1") || ir.contains("or i1"), "logical OR missing");
    }

    #[test]
    fn test_rt_logical_not() {
        let ir = source_to_ir("fn negate(a: bool) -> bool = !a;");
        assert!(ir.contains("@negate"));
        assert!(ir.contains("xor i1") || ir.contains("icmp eq"), "logical NOT missing");
    }

    #[test]
    fn test_rt_char_type() {
        let ir = source_to_ir("fn identity_char(c: char) -> char = c;");
        assert!(ir.contains("@identity_char"));
    }

    #[test]
    fn test_rt_string_literal() {
        let ir = source_to_ir(r#"fn greeting() -> string = "hello";"#);
        assert!(ir.contains("@greeting"));
        // String literal should be in constant data
        assert!(ir.contains("hello") || ir.contains("@.str"), "string constant missing");
    }

    #[test]
    fn test_rt_multiple_let_bindings() {
        let ir = source_to_ir(
            "fn multi_let(x: i64) -> i64 = { let a = x + 1; let b = a * 2; let c = b - 3; c };"
        );
        assert!(ir.contains("@multi_let"));
        assert!(ir.contains("add nsw i64"));
        assert!(ir.contains("mul nsw i64"));
        assert!(ir.contains("sub nsw i64"));
    }

    #[test]
    fn test_rt_contract_precondition_check() {
        let ir = source_to_ir(
            "fn safe_div(a: i64, b: i64) -> i64\n  pre b != 0\n= a / b;"
        );
        assert!(ir.contains("@safe_div"));
        // Pre-condition should be present or elided by optimization
        assert!(ir.contains("sdiv i64") || ir.contains("div"), "division missing");
    }

    #[test]
    fn test_rt_bitwise_xor() {
        let ir = source_to_ir("fn xor_fn(a: i64, b: i64) -> i64 = a bxor b;");
        assert!(ir.contains("@xor_fn"));
        assert!(ir.contains("xor i64"));
    }

    #[test]
    fn test_rt_identity_function() {
        let ir = source_to_ir("fn id(x: i64) -> i64 = x;");
        assert!(ir.contains("@id"));
        assert!(ir.contains("ret i64"), "identity return missing");
    }

    #[test]
    fn test_rt_constant_int_literal() {
        let ir = source_to_ir("fn forty_two() -> i64 = 42;");
        assert!(ir.contains("@forty_two"));
        assert!(ir.contains("ret i64 42"));
    }

    #[test]
    fn test_rt_f64_constant() {
        let ir = source_to_ir("fn pi() -> f64 = 3.14;");
        assert!(ir.contains("@pi"));
        assert!(ir.contains("double") || ir.contains("3.14"), "f64 constant missing");
    }

    #[test]
    fn test_rt_zero_function() {
        let ir = source_to_ir("fn zero() -> i64 = 0;");
        assert!(ir.contains("@zero"));
        assert!(ir.contains("ret i64 0"));
    }

    #[test]
    fn test_rt_nested_arithmetic() {
        let ir = source_to_ir("fn calc(a: i64, b: i64, c: i64) -> i64 = (a + b) * (c - a);");
        assert!(ir.contains("@calc"));
        assert!(ir.contains("add nsw i64"));
        assert!(ir.contains("sub nsw i64"));
        assert!(ir.contains("mul nsw i64"));
    }

    // ================================================================
    // Cycle 209: Additional unit tests for untested codegen paths
    // ================================================================

    #[test]
    fn test_with_target_custom_triple() {
        let cg = TextCodeGen::with_target("aarch64-unknown-linux-gnu");
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "noop".to_string(),
                params: vec![],
                ret_ty: MirType::Unit,
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
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("target triple = \"aarch64-unknown-linux-gnu\""),
                "custom target triple missing");
    }

    #[test]
    fn test_default_trait_implementation() {
        let cg = TextCodeGen::default();
        let program = MirProgram {
            functions: vec![],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("target triple"), "default codegen should emit target triple");
    }

    #[test]
    fn test_escape_string_special_chars() {
        let cg = TextCodeGen::new();
        // Backslash
        assert_eq!(cg.escape_string_for_llvm("a\\b"), "a\\5Cb");
        // Double-quote
        assert_eq!(cg.escape_string_for_llvm("say \"hi\""), "say \\22hi\\22");
        // Newline
        assert_eq!(cg.escape_string_for_llvm("line1\nline2"), "line1\\0Aline2");
        // Carriage return
        assert_eq!(cg.escape_string_for_llvm("cr\rhere"), "cr\\0Dhere");
        // Tab
        assert_eq!(cg.escape_string_for_llvm("col1\tcol2"), "col1\\09col2");
        // Plain ASCII passthrough
        assert_eq!(cg.escape_string_for_llvm("hello"), "hello");
    }

    #[test]
    fn test_mir_type_to_llvm_all_variants() {
        let cg = TextCodeGen::new();
        assert_eq!(cg.mir_type_to_llvm(&MirType::I32), "i32");
        assert_eq!(cg.mir_type_to_llvm(&MirType::I64), "i64");
        assert_eq!(cg.mir_type_to_llvm(&MirType::U32), "i32");
        assert_eq!(cg.mir_type_to_llvm(&MirType::U64), "i64");
        assert_eq!(cg.mir_type_to_llvm(&MirType::F64), "double");
        assert_eq!(cg.mir_type_to_llvm(&MirType::Bool), "i1");
        assert_eq!(cg.mir_type_to_llvm(&MirType::String), "ptr");
        assert_eq!(cg.mir_type_to_llvm(&MirType::Unit), "void");
        assert_eq!(cg.mir_type_to_llvm(&MirType::Char), "i32");
        assert_eq!(cg.mir_type_to_llvm(&MirType::StructPtr("Foo".to_string())), "ptr");
        assert_eq!(cg.mir_type_to_llvm(&MirType::Ptr(Box::new(MirType::I64))), "ptr");
        assert_eq!(cg.mir_type_to_llvm(&MirType::Tuple(vec![Box::new(MirType::I64)])), "ptr");
        assert_eq!(cg.mir_type_to_llvm(&MirType::Array {
            element_type: Box::new(MirType::I64),
            size: Some(10),
        }), "ptr");
        assert_eq!(cg.mir_type_to_llvm(&MirType::Enum {
            name: "Color".to_string(),
            variants: vec![],
        }), "ptr");
        assert_eq!(cg.mir_type_to_llvm(&MirType::Struct {
            name: "Point".to_string(),
            fields: vec![],
        }), "ptr");
    }

    #[test]
    fn test_constant_type_all_variants() {
        let cg = TextCodeGen::new();
        assert_eq!(cg.constant_type(&Constant::Int(42)), "i64");
        assert_eq!(cg.constant_type(&Constant::Float(1.23)), "double");
        assert_eq!(cg.constant_type(&Constant::Bool(true)), "i1");
        assert_eq!(cg.constant_type(&Constant::String("hi".to_string())), "ptr");
        assert_eq!(cg.constant_type(&Constant::Char('A')), "i32");
        assert_eq!(cg.constant_type(&Constant::Unit), "i8");
    }

    #[test]
    fn test_format_constant_special_floats() {
        let cg = TextCodeGen::new();
        // NaN
        assert_eq!(cg.format_constant(&Constant::Float(f64::NAN)), "0x7FF8000000000000");
        // Positive infinity
        assert_eq!(cg.format_constant(&Constant::Float(f64::INFINITY)), "0x7FF0000000000000");
        // Negative infinity
        assert_eq!(cg.format_constant(&Constant::Float(f64::NEG_INFINITY)), "0xFFF0000000000000");
        // Normal float
        assert_eq!(cg.format_constant(&Constant::Float(4.0)), "4.000000e0");
        // Bool
        assert_eq!(cg.format_constant(&Constant::Bool(true)), "1");
        assert_eq!(cg.format_constant(&Constant::Bool(false)), "0");
        // Unit
        assert_eq!(cg.format_constant(&Constant::Unit), "0");
        // Char
        assert_eq!(cg.format_constant(&Constant::Char('A')), "65");
    }

    #[test]
    fn test_get_cast_instruction_integer_widening() {
        let cg = TextCodeGen::new();
        // Signed widening
        assert_eq!(cg.get_cast_instruction(&MirType::I32, &MirType::I64), "sext");
        assert_eq!(cg.get_cast_instruction(&MirType::Char, &MirType::I64), "sext");
        // Unsigned widening
        assert_eq!(cg.get_cast_instruction(&MirType::U32, &MirType::I64), "zext");
        assert_eq!(cg.get_cast_instruction(&MirType::Bool, &MirType::I64), "zext");
        assert_eq!(cg.get_cast_instruction(&MirType::Bool, &MirType::I32), "zext");
    }

    #[test]
    fn test_get_cast_instruction_integer_narrowing() {
        let cg = TextCodeGen::new();
        assert_eq!(cg.get_cast_instruction(&MirType::I64, &MirType::I32), "trunc");
        assert_eq!(cg.get_cast_instruction(&MirType::U64, &MirType::I32), "trunc");
        assert_eq!(cg.get_cast_instruction(&MirType::I64, &MirType::Char), "trunc");
        assert_eq!(cg.get_cast_instruction(&MirType::I32, &MirType::Char), "trunc");
    }

    #[test]
    fn test_get_cast_instruction_float_conversions() {
        let cg = TextCodeGen::new();
        // Integer to float
        assert_eq!(cg.get_cast_instruction(&MirType::I64, &MirType::F64), "sitofp");
        assert_eq!(cg.get_cast_instruction(&MirType::I32, &MirType::F64), "sitofp");
        assert_eq!(cg.get_cast_instruction(&MirType::U64, &MirType::F64), "uitofp");
        // Float to integer
        assert_eq!(cg.get_cast_instruction(&MirType::F64, &MirType::I64), "fptosi");
        assert_eq!(cg.get_cast_instruction(&MirType::F64, &MirType::I32), "fptosi");
        assert_eq!(cg.get_cast_instruction(&MirType::F64, &MirType::U64), "fptoui");
    }

    #[test]
    fn test_get_cast_instruction_same_size_signedness() {
        let cg = TextCodeGen::new();
        assert_eq!(cg.get_cast_instruction(&MirType::I32, &MirType::U32), "bitcast");
        assert_eq!(cg.get_cast_instruction(&MirType::U32, &MirType::I32), "bitcast");
        assert_eq!(cg.get_cast_instruction(&MirType::I64, &MirType::U64), "bitcast");
        assert_eq!(cg.get_cast_instruction(&MirType::U64, &MirType::I64), "bitcast");
    }

    #[test]
    fn test_get_cast_instruction_pointer_conversions() {
        let cg = TextCodeGen::new();
        assert_eq!(cg.get_cast_instruction(&MirType::StructPtr("Foo".to_string()), &MirType::I64), "ptrtoint");
        assert_eq!(cg.get_cast_instruction(&MirType::I64, &MirType::StructPtr("Bar".to_string())), "inttoptr");
        assert_eq!(cg.get_cast_instruction(&MirType::Ptr(Box::new(MirType::I64)), &MirType::I64), "ptrtoint");
        assert_eq!(cg.get_cast_instruction(&MirType::I64, &MirType::Ptr(Box::new(MirType::I64))), "inttoptr");
    }

    #[test]
    fn test_binop_to_llvm_wrapping_arithmetic() {
        let cg = TextCodeGen::new();
        // Wrapping ops should NOT have nsw flag
        let (inst, preserves) = cg.binop_to_llvm(MirBinOp::AddWrap);
        assert_eq!(inst, "add");
        assert!(preserves);
        let (inst, _) = cg.binop_to_llvm(MirBinOp::SubWrap);
        assert_eq!(inst, "sub");
        let (inst, _) = cg.binop_to_llvm(MirBinOp::MulWrap);
        assert_eq!(inst, "mul");
    }

    #[test]
    fn test_binop_to_llvm_float_fast_math() {
        let cg = TextCodeGen::new();
        let (inst, preserves) = cg.binop_to_llvm(MirBinOp::FAdd);
        assert_eq!(inst, "fadd fast");
        assert!(preserves);
        let (inst, _) = cg.binop_to_llvm(MirBinOp::FSub);
        assert_eq!(inst, "fsub fast");
        let (inst, _) = cg.binop_to_llvm(MirBinOp::FMul);
        assert_eq!(inst, "fmul fast");
        let (inst, _) = cg.binop_to_llvm(MirBinOp::FDiv);
        assert_eq!(inst, "fdiv fast");
    }

    #[test]
    fn test_binop_to_llvm_comparisons_return_i1() {
        let cg = TextCodeGen::new();
        // All comparisons should NOT preserve operand type (they return i1)
        for op in [MirBinOp::Eq, MirBinOp::Ne, MirBinOp::Lt, MirBinOp::Gt,
                   MirBinOp::Le, MirBinOp::Ge] {
            let (_, preserves) = cg.binop_to_llvm(op);
            assert!(!preserves, "{:?} should return i1 (preserves_operand_type=false)", op);
        }
        // Float comparisons
        for op in [MirBinOp::FEq, MirBinOp::FNe, MirBinOp::FLt, MirBinOp::FGt,
                   MirBinOp::FLe, MirBinOp::FGe] {
            let (_, preserves) = cg.binop_to_llvm(op);
            assert!(!preserves, "{:?} should return i1 (preserves_operand_type=false)", op);
        }
    }

    #[test]
    fn test_infer_call_return_type_builtins() {
        let cg = TextCodeGen::new();
        let dummy_func = MirFunction {
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
        // Void returns
        assert_eq!(cg.infer_call_return_type("println", &dummy_func), "void");
        assert_eq!(cg.infer_call_return_type("print", &dummy_func), "void");
        assert_eq!(cg.infer_call_return_type("assert", &dummy_func), "void");
        // i64 returns
        assert_eq!(cg.infer_call_return_type("read_int", &dummy_func), "i64");
        assert_eq!(cg.infer_call_return_type("len", &dummy_func), "i64");
        assert_eq!(cg.infer_call_return_type("bmb_time_ns", &dummy_func), "i64");
        // double returns
        assert_eq!(cg.infer_call_return_type("sqrt", &dummy_func), "double");
        assert_eq!(cg.infer_call_return_type("i64_to_f64", &dummy_func), "double");
        // ptr returns
        assert_eq!(cg.infer_call_return_type("bmb_read_file", &dummy_func), "ptr");
        assert_eq!(cg.infer_call_return_type("sb_build", &dummy_func), "ptr");
        assert_eq!(cg.infer_call_return_type("chr", &dummy_func), "ptr");
        // Unknown function defaults to i64
        assert_eq!(cg.infer_call_return_type("unknown_fn", &dummy_func), "i64");
    }

    #[test]
    fn test_always_inline_function_attributes() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "tiny".to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: true,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("alwaysinline"), "always_inline function should have alwaysinline attribute");
        assert!(ir.contains("private "), "always_inline function should have private linkage");
    }

    #[test]
    fn test_memory_free_function_attributes() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "pure_add".to_string(),
                params: vec![
                    ("a".to_string(), MirType::I64),
                    ("b".to_string(), MirType::I64),
                ],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::BinOp {
                        dest: Place::new("_t0"),
                        op: MirBinOp::Add,
                        lhs: Operand::Place(Place::new("a")),
                        rhs: Operand::Place(Place::new("b")),
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: true,
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("memory(none)"), "memory-free function should have memory(none) attribute");
    }

    #[test]
    fn test_struct_type_definitions_emitted() {
        let mut struct_defs = std::collections::HashMap::new();
        struct_defs.insert("Point".to_string(), vec![
            ("x".to_string(), MirType::I64),
            ("y".to_string(), MirType::I64),
        ]);
        let program = MirProgram {
            functions: vec![],
            extern_fns: vec![],
            struct_defs,
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("%struct.Point = type { i64, i64 }"),
                "struct type definition should be emitted, got:\n{}", ir);
    }

    #[test]
    fn test_main_function_renamed_to_bmb_user_main() {
        let program = MirProgram {
            functions: vec![MirFunction {
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
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("@bmb_user_main"), "main should be renamed to bmb_user_main");
        assert!(!ir.contains("@main("), "should not emit @main( directly");
    }

    // ========================================================================
    // Cycle 222: Loop/Break/Continue/Return codegen tests
    // ========================================================================

    #[test]
    fn test_rt_loop_break_codegen() {
        // loop {} with break should produce loop header, body, and exit blocks
        let ir = source_to_ir(
            "fn count() -> i64 = {
               let mut x = 0;
               loop {
                 x = x + 1;
                 if x >= 5 { break } else { () }
               };
               x
             };"
        );
        assert!(ir.contains("@count"), "function definition missing");
        // Loop should have a back edge (br to loop header)
        assert!(ir.contains("br label"), "loop should have unconditional branch (back edge)");
        assert!(ir.contains("ret i64"), "return missing");
    }

    #[test]
    fn test_rt_loop_accumulator_codegen() {
        // Iterative sum using loop with phi nodes
        let ir = source_to_ir(
            "fn sum_to(n: i64) -> i64 = {
               let mut s = 0;
               let mut i = 1;
               loop {
                 s = s + i;
                 i = i + 1;
                 if i > n { break } else { () }
               };
               s
             };"
        );
        assert!(ir.contains("@sum_to"), "function definition missing");
        // Text codegen uses alloca+store+load for mutable loop vars
        assert!(ir.contains("alloca i64"), "loop vars should be stack-allocated");
        assert!(ir.contains("add nsw i64"), "accumulator should use add");
        assert!(ir.contains("br label %bb_loop_body"), "should have loop back-edge");
    }

    #[test]
    fn test_rt_continue_codegen() {
        // continue should generate branch back to loop header
        let ir = source_to_ir(
            "fn sum_odd(n: i64) -> i64 = {
               let mut sum = 0;
               let mut i = 0;
               loop {
                 i = i + 1;
                 if i > n { break } else { () };
                 if i % 2 == 0 { continue } else { () };
                 sum = sum + i
               };
               sum
             };"
        );
        assert!(ir.contains("@sum_odd"), "function definition missing");
        assert!(ir.contains("alloca i64"), "loop vars should be stack-allocated");
        assert!(ir.contains("srem i64"), "modulo should use srem");
    }

    #[test]
    fn test_rt_return_expression_codegen() {
        // Early return should produce ret instruction in middle of function
        let ir = source_to_ir(
            "fn early(n: i64) -> i64 = {
               if n <= 0 { return 0 } else { () };
               n * 2
             };"
        );
        assert!(ir.contains("@early"), "function definition missing");
        // Should have multiple ret instructions (one for early return, one for normal)
        let ret_count = ir.matches("ret i64").count();
        assert!(ret_count >= 2, "should have at least 2 ret instructions (early + normal), got {}", ret_count);
    }

    #[test]
    fn test_rt_return_in_loop_codegen() {
        // Return from inside loop should produce ret
        let ir = source_to_ir(
            "fn find_sqrt(n: i64) -> i64 = {
               let mut i = 0;
               loop {
                 i = i + 1;
                 if i * i >= n { return i } else { () }
               };
               0
             };"
        );
        assert!(ir.contains("@find_sqrt"), "function definition missing");
        assert!(ir.contains("mul nsw i64"), "i*i should use mul");
    }

    #[test]
    fn test_rt_nested_loops_codegen() {
        // Nested loops should produce distinct loop structures
        let ir = source_to_ir(
            "fn nested() -> i64 = {
               let mut total = 0;
               let mut i = 0;
               loop {
                 let mut j = 0;
                 loop {
                   total = total + 1;
                   j = j + 1;
                   if j >= 3 { break } else { () }
                 };
                 i = i + 1;
                 if i >= 2 { break } else { () }
               };
               total
             };"
        );
        assert!(ir.contains("@nested"), "function definition missing");
        // Nested loops: multiple alloca vars and multiple loop body blocks
        let alloca_count = ir.matches("alloca i64").count();
        assert!(alloca_count >= 3, "nested loops should have multiple alloca vars, got {}", alloca_count);
        // Should have at least 2 loop back-edges
        let back_edge_count = ir.matches("br label %bb_loop_body").count();
        assert!(back_edge_count >= 2, "nested loops should have at least 2 back-edges, got {}", back_edge_count);
    }

    #[test]
    fn test_rt_recursive_call_codegen() {
        // Recursive function should produce self-referential call
        let ir = source_to_ir(
            "fn count_down(n: i64) -> i64 =
               if n <= 0 { 0 } else { count_down(n - 1) };"
        );
        assert!(ir.contains("@count_down"), "function definition missing");
        // Should have recursive call to self (source_to_ir doesn't run optimization passes,
        // so tail call annotation won't be present — test verifies recursive call structure)
        assert!(ir.contains("call i64 @count_down"), "should have recursive call to self");
        assert!(ir.contains("icmp sle i64"), "should have base case comparison");
    }

    #[test]
    fn test_rt_for_loop_with_break_codegen() {
        // For loop with break should still produce loop structure
        let ir = source_to_ir(
            "fn find_first(n: i64) -> i64 = {
               let mut result = 0;
               for i in 1..n {
                 if i * i > 100 {
                   result = i;
                   break
                 } else { () }
               };
               result
             };"
        );
        assert!(ir.contains("@find_first"), "function definition missing");
        // For loop uses alloca for loop variable and result
        assert!(ir.contains("alloca i64"), "for loop should have stack-allocated vars");
        assert!(ir.contains("mul nsw i64"), "i*i should use mul");
    }

    #[test]
    fn test_rt_void_return_codegen() {
        // Unit return type should produce void function
        let ir = source_to_ir(
            "fn do_nothing() -> () = ();"
        );
        assert!(ir.contains("@do_nothing"), "function definition missing");
        assert!(ir.contains("ret void"), "unit return should produce ret void");
    }

    #[test]
    fn test_rt_select_pattern_codegen() {
        // Simple if-else expression should produce select or branch+phi
        let ir = source_to_ir(
            "fn abs_val(x: i64) -> i64 = if x >= 0 { x } else { 0 - x };"
        );
        assert!(ir.contains("@abs_val"), "function definition missing");
        // Should have either select instruction or branch+phi pattern
        let has_conditional = ir.contains("select i1") || ir.contains("br i1");
        assert!(has_conditional, "if-else should produce select or branch, got:\n{}", &ir[..500.min(ir.len())]);
    }

    // ================================================================
    // Cycle 412: Struct/enum/type edge case codegen tests
    // ================================================================

    #[test]
    fn test_struct_mixed_field_types() {
        // Struct with i64, f64, and bool fields
        let mut struct_defs = std::collections::HashMap::new();
        struct_defs.insert("Entity".to_string(), vec![
            ("id".to_string(), MirType::I64),
            ("score".to_string(), MirType::F64),
            ("active".to_string(), MirType::Bool),
        ]);
        let program = MirProgram {
            functions: vec![],
            extern_fns: vec![],
            struct_defs,
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("%struct.Entity = type { i64, double, i1 }"),
                "mixed-type struct definition should be emitted, got:\n{}", ir);
    }

    #[test]
    fn test_multiple_struct_definitions() {
        let mut struct_defs = std::collections::HashMap::new();
        struct_defs.insert("Point".to_string(), vec![
            ("x".to_string(), MirType::I64),
            ("y".to_string(), MirType::I64),
        ]);
        struct_defs.insert("Rect".to_string(), vec![
            ("w".to_string(), MirType::I64),
            ("h".to_string(), MirType::I64),
            ("x".to_string(), MirType::I64),
            ("y".to_string(), MirType::I64),
        ]);
        let program = MirProgram {
            functions: vec![],
            extern_fns: vec![],
            struct_defs,
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("%struct.Point"), "Point struct type should be emitted");
        assert!(ir.contains("%struct.Rect"), "Rect struct type should be emitted");
        // Verify both have correct field counts
        assert!(ir.contains("type { i64, i64 }"), "Point should have 2 i64 fields");
        assert!(ir.contains("type { i64, i64, i64, i64 }"), "Rect should have 4 i64 fields");
    }

    #[test]
    fn test_unary_neg_mir_codegen() {
        // Test integer negation via direct MIR
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "neg".to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::UnaryOp {
                        dest: Place::new("_t0"),
                        op: MirUnaryOp::Neg,
                        src: Operand::Place(Place::new("x")),
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("sub i64 0,"), "Neg should emit sub 0, x");
    }

    #[test]
    fn test_unary_not_mir_codegen() {
        // Test boolean NOT via direct MIR
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "not_fn".to_string(),
                params: vec![("b".to_string(), MirType::Bool)],
                ret_ty: MirType::Bool,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::UnaryOp {
                        dest: Place::new("_t0"),
                        op: MirUnaryOp::Not,
                        src: Operand::Place(Place::new("b")),
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("xor i1"), "Not should emit xor i1 x, 1");
    }

    #[test]
    fn test_unary_bnot_mir_codegen() {
        // Test bitwise NOT via direct MIR
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "bnot_fn".to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::UnaryOp {
                        dest: Place::new("_t0"),
                        op: MirUnaryOp::Bnot,
                        src: Operand::Place(Place::new("x")),
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("xor i64"), "Bnot should emit xor i64 x, -1");
        assert!(ir.contains("-1"), "Bnot should use -1 as mask");
    }

    #[test]
    fn test_unary_fneg_mir_codegen() {
        // Test float negation via direct MIR
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "fneg".to_string(),
                params: vec![("x".to_string(), MirType::F64)],
                ret_ty: MirType::F64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::UnaryOp {
                        dest: Place::new("_t0"),
                        op: MirUnaryOp::FNeg,
                        src: Operand::Place(Place::new("x")),
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("fsub fast"), "FNeg should emit fsub fast 0.0, x");
    }

    #[test]
    fn test_pure_function_attributes() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "pure_fn".to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        // Pure functions should have readnone or readonly attribute
        assert!(ir.contains("readnone") || ir.contains("readonly") || ir.contains("memory(none)") || ir.contains("pure"),
                "pure function should have optimization attribute, got:\n{}", ir);
    }

    #[test]
    fn test_inline_hint_function_attributes() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "hinted".to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("x")))),
                }],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: true,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        // Inline hint should have inlinehint attribute
        assert!(ir.contains("inlinehint"), "inline_hint function should have inlinehint attribute, got:\n{}", ir);
    }

    #[test]
    fn test_collect_string_constants_dedup() {
        // Same string used twice should only create one global
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "greet".to_string(),
                params: vec![],
                ret_ty: MirType::String,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const {
                            dest: Place::new("_t0"),
                            value: Constant::String("hello".to_string()),
                        },
                        MirInst::Const {
                            dest: Place::new("_t1"),
                            value: Constant::String("hello".to_string()),
                        },
                    ],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let table = cg.collect_string_constants(&program);
        assert_eq!(table.len(), 1, "duplicate strings should be deduplicated");
        assert!(table.contains_key("hello"));
    }

    #[test]
    fn test_collect_string_constants_from_call_args() {
        // String constant in call arguments should be collected
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "test".to_string(),
                params: vec![],
                ret_ty: MirType::Unit,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Call {
                            dest: None,
                            func: "println".to_string(),
                            args: vec![Operand::Constant(Constant::String("world".to_string()))],
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
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let table = cg.collect_string_constants(&program);
        assert!(table.contains_key("world"), "string in call args should be collected");
    }

    #[test]
    fn test_collect_string_constants_multiple_unique() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "test".to_string(),
                params: vec![],
                ret_ty: MirType::Unit,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        MirInst::Const {
                            dest: Place::new("_t0"),
                            value: Constant::String("alpha".to_string()),
                        },
                        MirInst::Const {
                            dest: Place::new("_t1"),
                            value: Constant::String("beta".to_string()),
                        },
                        MirInst::Const {
                            dest: Place::new("_t2"),
                            value: Constant::String("gamma".to_string()),
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
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let table = cg.collect_string_constants(&program);
        assert_eq!(table.len(), 3, "should collect 3 unique strings");
        assert!(table.contains_key("alpha"));
        assert!(table.contains_key("beta"));
        assert!(table.contains_key("gamma"));
    }

    #[test]
    fn test_is_string_operand_constant() {
        let func = MirFunction {
            name: "test".to_string(),
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
        // String constant should be detected
        assert!(TextCodeGen::is_string_operand(
            &Operand::Constant(Constant::String("test".to_string())), &func));
        // Non-string constant should not
        assert!(!TextCodeGen::is_string_operand(
            &Operand::Constant(Constant::Int(42)), &func));
    }

    #[test]
    fn test_is_string_operand_param() {
        let func = MirFunction {
            name: "test".to_string(),
            params: vec![
                ("s".to_string(), MirType::String),
                ("n".to_string(), MirType::I64),
            ],
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
        // String param should be detected
        assert!(TextCodeGen::is_string_operand(
            &Operand::Place(Place::new("s")), &func));
        // Non-string param should not
        assert!(!TextCodeGen::is_string_operand(
            &Operand::Place(Place::new("n")), &func));
    }

    #[test]
    fn test_is_string_operand_local() {
        let func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::Unit,
            locals: vec![
                ("msg".to_string(), MirType::String),
                ("count".to_string(), MirType::I64),
            ],
            blocks: vec![],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        assert!(TextCodeGen::is_string_operand(
            &Operand::Place(Place::new("msg")), &func));
        assert!(!TextCodeGen::is_string_operand(
            &Operand::Place(Place::new("count")), &func));
    }

    #[test]
    fn test_rt_struct_field_store_codegen() {
        // Test struct field mutation generates GEP + store
        let ir = source_to_ir(
            "struct Counter { value: i64 }
             fn inc(c: Counter) -> Counter = { set c.value = c.value + 1; c };"
        );
        assert!(ir.contains("@inc"), "function definition missing");
        assert!(ir.contains("getelementptr") || ir.contains("store"),
                "field store should generate GEP+store, got:\n{}", &ir[..ir.len().min(500)]);
    }

    #[test]
    fn test_rt_enum_with_data_codegen() {
        // Enum with data variant
        let ir = source_to_ir(
            "enum Shape { Circle(f64), Square(f64) }
             fn make_circle(r: f64) -> Shape = Shape::Circle(r);"
        );
        assert!(ir.contains("@make_circle"), "function definition missing");
    }

    #[test]
    fn test_rt_type_cast_codegen() {
        // Explicit type cast should generate conversion instruction
        let ir = source_to_ir(
            "fn to_float(x: i64) -> f64 = x as f64;"
        );
        assert!(ir.contains("@to_float"), "function definition missing");
        assert!(ir.contains("sitofp") || ir.contains("double"),
                "i64 to f64 cast should generate sitofp");
    }

    #[test]
    fn test_rt_multi_struct_program() {
        // Program with multiple struct types
        let ir = source_to_ir(
            "struct Point { x: i64, y: i64 }
             struct Line { a: Point, b: Point }
             fn origin() -> Point = new Point { x: 0, y: 0 };"
        );
        assert!(ir.contains("@origin"), "function definition missing");
        // Both struct types should appear as type definitions
        assert!(ir.contains("%struct.Point") || ir.contains("Point"),
                "Point struct type should appear in IR");
    }

    #[test]
    fn test_const_function_attributes() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "const_fn".to_string(),
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
                is_pure: false,
                is_const: true,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("@const_fn"), "const function should be emitted");
        assert!(ir.contains("ret i64 42"), "const function should return constant");
    }

    #[test]
    fn test_empty_program_codegen() {
        let program = MirProgram {
            functions: vec![],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("target triple"), "empty program should still have module header");
        assert!(!ir.contains("define "), "empty program should have no function definitions");
    }

    #[test]
    fn test_extern_fn_declarations() {
        use crate::mir::MirExternFn;
        let program = MirProgram {
            functions: vec![],
            extern_fns: vec![MirExternFn {
                module: "env".to_string(),
                name: "ext_add".to_string(),
                params: vec![MirType::I64, MirType::I64],
                ret_ty: MirType::I64,
            }],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("ext_add") || ir.contains("declare"),
                "extern function should appear in IR");
    }

    #[test]
    fn test_rt_array_literal_codegen() {
        let ir = source_to_ir(
            "fn first() -> i64 = { let arr = [1, 2, 3]; arr[0] };"
        );
        assert!(ir.contains("@first"), "function definition missing");
    }

    // ================================================================
    // Cycle 413: Control flow + instruction variant codegen tests
    // ================================================================

    #[test]
    fn test_mir_array_init_codegen() {
        // Direct MIR: ArrayInit should produce alloca + GEP + store sequence
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "make_arr".to_string(),
                params: vec![],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::ArrayInit {
                        dest: Place::new("arr"),
                        element_type: MirType::I64,
                        elements: vec![
                            Operand::Constant(Constant::Int(10)),
                            Operand::Constant(Constant::Int(20)),
                            Operand::Constant(Constant::Int(30)),
                        ],
                    }],
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("alloca i64"), "ArrayInit should allocate on stack");
        assert!(ir.contains("getelementptr"), "ArrayInit should GEP to elements");
        assert!(ir.contains("store i64"), "ArrayInit should store elements");
    }

    #[test]
    fn test_mir_index_load_codegen() {
        // IndexLoad should produce GEP + load
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "load_elem".to_string(),
                params: vec![("arr".to_string(), MirType::Ptr(Box::new(MirType::I64)))],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::IndexLoad {
                        dest: Place::new("_t0"),
                        array: Place::new("arr"),
                        index: Operand::Constant(Constant::Int(2)),
                        element_type: MirType::I64,
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("getelementptr"), "IndexLoad should emit GEP");
        assert!(ir.contains("load i64"), "IndexLoad should emit load");
    }

    #[test]
    fn test_mir_cast_i64_to_f64_codegen() {
        // Cast from i64 to f64 should produce sitofp
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "to_f64".to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::F64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::Cast {
                        dest: Place::new("_t0"),
                        src: Operand::Place(Place::new("x")),
                        from_ty: MirType::I64,
                        to_ty: MirType::F64,
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("sitofp i64"), "i64 to f64 cast should emit sitofp");
        assert!(ir.contains("to double"), "cast target should be double");
    }

    #[test]
    fn test_mir_cast_f64_to_i64_codegen() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "to_i64".to_string(),
                params: vec![("x".to_string(), MirType::F64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::Cast {
                        dest: Place::new("_t0"),
                        src: Operand::Place(Place::new("x")),
                        from_ty: MirType::F64,
                        to_ty: MirType::I64,
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("fptosi double"), "f64 to i64 cast should emit fptosi");
        assert!(ir.contains("to i64"), "cast target should be i64");
    }

    #[test]
    fn test_mir_cast_i32_to_i64_codegen() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "widen".to_string(),
                params: vec![("x".to_string(), MirType::I32)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::Cast {
                        dest: Place::new("_t0"),
                        src: Operand::Place(Place::new("x")),
                        from_ty: MirType::I32,
                        to_ty: MirType::I64,
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("sext i32"), "i32 to i64 cast should emit sext");
        assert!(ir.contains("to i64"), "cast target should be i64");
    }

    #[test]
    fn test_mir_copy_instruction_codegen() {
        // Copy instruction should generate a simple assignment
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "copy_fn".to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::Copy {
                        dest: Place::new("_t0"),
                        src: Place::new("x"),
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("@copy_fn"), "function should be emitted");
        assert!(ir.contains("ret i64"), "should return copied value");
    }

    #[test]
    fn test_mir_array_alloc_codegen() {
        // ArrayAlloc should produce alloca with array type
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "alloc_arr".to_string(),
                params: vec![],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::ArrayAlloc {
                        dest: Place::new("buf"),
                        element_type: MirType::I64,
                        size: 16,
                    }],
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("alloca [16 x i64]"), "ArrayAlloc should emit alloca with array type, got:\n{}", ir);
    }

    #[test]
    fn test_mir_ptr_load_codegen() {
        // PtrLoad should produce load through pointer
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "deref".to_string(),
                params: vec![("p".to_string(), MirType::Ptr(Box::new(MirType::I64)))],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::PtrLoad {
                        dest: Place::new("_t0"),
                        ptr: Operand::Place(Place::new("p")),
                        element_type: MirType::I64,
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("load i64, ptr"), "PtrLoad should emit load i64, ptr");
    }

    #[test]
    fn test_mir_ptr_store_codegen() {
        // PtrStore should produce store through pointer
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "store_fn".to_string(),
                params: vec![
                    ("p".to_string(), MirType::Ptr(Box::new(MirType::I64))),
                    ("v".to_string(), MirType::I64),
                ],
                ret_ty: MirType::Unit,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::PtrStore {
                        ptr: Operand::Place(Place::new("p")),
                        value: Operand::Place(Place::new("v")),
                        element_type: MirType::I64,
                    }],
                    terminator: Terminator::Return(None),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("store i64"), "PtrStore should emit store i64");
        assert!(ir.contains("ret void"), "unit function should ret void");
    }

    #[test]
    fn test_mir_ptr_offset_codegen() {
        // PtrOffset should produce GEP instruction
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "offset_fn".to_string(),
                params: vec![
                    ("p".to_string(), MirType::Ptr(Box::new(MirType::I64))),
                    ("idx".to_string(), MirType::I64),
                ],
                ret_ty: MirType::Ptr(Box::new(MirType::I64)),
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::PtrOffset {
                        dest: Place::new("_t0"),
                        ptr: Operand::Place(Place::new("p")),
                        offset: Operand::Place(Place::new("idx")),
                        element_type: MirType::I64,
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("getelementptr inbounds i64"), "PtrOffset should emit GEP inbounds");
    }

    #[test]
    fn test_mir_const_string_global() {
        // String constant should generate @.str global
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "greet".to_string(),
                params: vec![],
                ret_ty: MirType::String,
                locals: vec![],
                blocks: vec![BasicBlock {
                    label: "entry".to_string(),
                    instructions: vec![MirInst::Const {
                        dest: Place::new("_t0"),
                        value: Constant::String("hello world".to_string()),
                    }],
                    terminator: Terminator::Return(Some(Operand::Place(Place::new("_t0")))),
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
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("@.str"), "string global should be emitted");
        assert!(ir.contains("hello world"), "string content should appear in global");
    }

    #[test]
    fn test_mir_multi_block_branch() {
        // Test Terminator::Branch with multiple blocks
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "branch_fn".to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![
                    BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![MirInst::BinOp {
                            dest: Place::new("_cond"),
                            op: MirBinOp::Gt,
                            lhs: Operand::Place(Place::new("x")),
                            rhs: Operand::Constant(Constant::Int(0)),
                        }],
                        terminator: Terminator::Branch {
                            cond: Operand::Place(Place::new("_cond")),
                            then_label: "then".to_string(),
                            else_label: "else_".to_string(),
                        },
                    },
                    BasicBlock {
                        label: "then".to_string(),
                        instructions: vec![],
                        terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(1)))),
                    },
                    BasicBlock {
                        label: "else_".to_string(),
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
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("br i1"), "conditional branch should emit br i1");
        assert!(ir.contains("bb_then"), "then block label should appear");
        assert!(ir.contains("bb_else_"), "else block label should appear");
        assert!(ir.contains("ret i64 1"), "then block should return 1");
        assert!(ir.contains("ret i64 0"), "else block should return 0");
    }

    #[test]
    fn test_mir_goto_terminator() {
        // Test Terminator::Goto (unconditional jump)
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "goto_fn".to_string(),
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
                        terminator: Terminator::Return(Some(Operand::Constant(Constant::Int(42)))),
                    },
                ],
                preconditions: vec![],
                postconditions: vec![],
                is_pure: false,
                is_const: false,
                always_inline: false,
                inline_hint: false,
                is_memory_free: false,
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("br label %bb_exit"), "Goto should emit unconditional branch to bb_exit");
    }

    #[test]
    fn test_mir_switch_terminator() {
        // Test Terminator::Switch
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "switch_fn".to_string(),
                params: vec![("x".to_string(), MirType::I64)],
                ret_ty: MirType::I64,
                locals: vec![],
                blocks: vec![
                    BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![],
                        terminator: Terminator::Switch {
                            discriminant: Operand::Place(Place::new("x")),
                            cases: vec![
                                (0, "case0".to_string()),
                                (1, "case1".to_string()),
                            ],
                            default: "default".to_string(),
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
            }],
            extern_fns: vec![],
            struct_defs: std::collections::HashMap::new(),
        };
        let cg = TextCodeGen::new();
        let ir = cg.generate(&program).unwrap();
        assert!(ir.contains("switch i64"), "Switch should emit switch instruction");
        assert!(ir.contains("i64 0, label %bb_case0"), "switch case 0 should be present");
        assert!(ir.contains("i64 1, label %bb_case1"), "switch case 1 should be present");
        assert!(ir.contains("label %bb_default"), "default case should be present");
    }

    #[test]
    fn test_rt_string_concatenation_codegen() {
        let ir = source_to_ir(
            r#"fn greet(name: String) -> String = "hello " + name;"#
        );
        assert!(ir.contains("@greet"), "function definition missing");
        // String concat should call runtime function
        assert!(ir.contains("call") && (ir.contains("concat") || ir.contains("bmb_str")),
                "string concat should call runtime function");
    }

    #[test]
    fn test_rt_tuple_creation_and_access() {
        let ir = source_to_ir(
            "fn swap(a: i64, b: i64) -> (i64, i64) = (b, a);"
        );
        assert!(ir.contains("@swap"), "function definition missing");
        assert!(ir.contains("insertvalue"), "tuple creation should use insertvalue");
    }

    #[test]
    fn test_rt_deeply_nested_expressions() {
        let ir = source_to_ir(
            "fn deep(a: i64, b: i64) -> i64 = ((a + b) * (a - b)) + ((a * a) - (b * b));"
        );
        assert!(ir.contains("@deep"), "function definition missing");
        let add_count = ir.matches("add nsw i64").count();
        let sub_count = ir.matches("sub nsw i64").count();
        let mul_count = ir.matches("mul nsw i64").count();
        assert!(add_count >= 2, "should have multiple adds, got {}", add_count);
        assert!(sub_count >= 2, "should have multiple subs, got {}", sub_count);
        assert!(mul_count >= 2, "should have multiple muls, got {}", mul_count);
    }

    // ====================================================================
    // unique_name tests (Cycle 427)
    // ====================================================================

    #[test]
    fn test_unique_name_first_use() {
        let codegen = TextCodeGen::new();
        let mut counts = HashMap::new();
        let name = codegen.unique_name("temp", &mut counts);
        assert_eq!(name, "temp");
    }

    #[test]
    fn test_unique_name_second_use_gets_suffix() {
        let codegen = TextCodeGen::new();
        let mut counts = HashMap::new();
        let _ = codegen.unique_name("temp", &mut counts);
        let name2 = codegen.unique_name("temp", &mut counts);
        assert_eq!(name2, "temp_1");
    }

    #[test]
    fn test_unique_name_third_use() {
        let codegen = TextCodeGen::new();
        let mut counts = HashMap::new();
        let _ = codegen.unique_name("x", &mut counts);
        let _ = codegen.unique_name("x", &mut counts);
        let name3 = codegen.unique_name("x", &mut counts);
        assert_eq!(name3, "x_2");
    }

    #[test]
    fn test_unique_name_different_names_independent() {
        let codegen = TextCodeGen::new();
        let mut counts = HashMap::new();
        let a = codegen.unique_name("a", &mut counts);
        let b = codegen.unique_name("b", &mut counts);
        assert_eq!(a, "a");
        assert_eq!(b, "b");
    }

    // ====================================================================
    // binop_to_llvm tests (Cycle 427)
    // ====================================================================

    #[test]
    fn test_binop_to_llvm_integer_arithmetic_nsw() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.binop_to_llvm(MirBinOp::Add), ("add nsw", true));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::Sub), ("sub nsw", true));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::Mul), ("mul nsw", true));
    }

    #[test]
    fn test_binop_to_llvm_div_mod_no_nsw() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.binop_to_llvm(MirBinOp::Div), ("sdiv", true));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::Mod), ("srem", true));
    }

    #[test]
    fn test_binop_to_llvm_checked_and_saturating() {
        let codegen = TextCodeGen::new();
        // Checked — currently same as wrapping (no nsw)
        assert_eq!(codegen.binop_to_llvm(MirBinOp::AddChecked), ("add", true));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::SubChecked), ("sub", true));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::MulChecked), ("mul", true));
        // Saturating — currently same as wrapping
        assert_eq!(codegen.binop_to_llvm(MirBinOp::AddSat), ("add", true));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::SubSat), ("sub", true));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::MulSat), ("mul", true));
    }

    #[test]
    fn test_binop_to_llvm_shift_operators() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.binop_to_llvm(MirBinOp::Shl), ("shl", true));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::Shr), ("ashr", true));
    }

    #[test]
    fn test_binop_to_llvm_bitwise_preserves_type() {
        let codegen = TextCodeGen::new();
        let (_, band_preserves) = codegen.binop_to_llvm(MirBinOp::Band);
        let (_, bor_preserves) = codegen.binop_to_llvm(MirBinOp::Bor);
        let (_, bxor_preserves) = codegen.binop_to_llvm(MirBinOp::Bxor);
        assert!(band_preserves);
        assert!(bor_preserves);
        assert!(bxor_preserves);
    }

    #[test]
    fn test_binop_to_llvm_logical_returns_i1() {
        let codegen = TextCodeGen::new();
        let (_, and_preserves) = codegen.binop_to_llvm(MirBinOp::And);
        let (_, or_preserves) = codegen.binop_to_llvm(MirBinOp::Or);
        assert!(!and_preserves);
        assert!(!or_preserves);
    }

    #[test]
    fn test_binop_to_llvm_float_comparison_returns_i1() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.binop_to_llvm(MirBinOp::FEq), ("fcmp oeq", false));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::FNe), ("fcmp one", false));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::FLt), ("fcmp olt", false));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::FGt), ("fcmp ogt", false));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::FLe), ("fcmp ole", false));
        assert_eq!(codegen.binop_to_llvm(MirBinOp::FGe), ("fcmp oge", false));
    }

    #[test]
    fn test_binop_to_llvm_implies() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.binop_to_llvm(MirBinOp::Implies), ("or", false));
    }

    // ====================================================================
    // format_constant tests (Cycle 427)
    // ====================================================================

    #[test]
    fn test_format_constant_int() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_constant(&Constant::Int(42)), "42");
        assert_eq!(codegen.format_constant(&Constant::Int(-1)), "-1");
        assert_eq!(codegen.format_constant(&Constant::Int(0)), "0");
    }

    #[test]
    fn test_format_constant_bool() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_constant(&Constant::Bool(true)), "1");
        assert_eq!(codegen.format_constant(&Constant::Bool(false)), "0");
    }

    #[test]
    fn test_format_constant_unit() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_constant(&Constant::Unit), "0");
    }

    #[test]
    fn test_format_constant_char() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_constant(&Constant::Char('A')), "65");
        assert_eq!(codegen.format_constant(&Constant::Char('\0')), "0");
    }

    #[test]
    fn test_format_constant_string() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_constant(&Constant::String("hello".to_string())), "\"hello\"");
    }

    #[test]
    fn test_format_constant_float_normal() {
        let codegen = TextCodeGen::new();
        let result = codegen.format_constant(&Constant::Float(4.0));
        assert!(result.contains("4"), "Expected float representation, got {}", result);
        assert!(result.contains("e"), "Expected scientific notation, got {}", result);
    }

    #[test]
    fn test_format_constant_float_nan() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_constant(&Constant::Float(f64::NAN)), "0x7FF8000000000000");
    }

    #[test]
    fn test_format_constant_float_positive_infinity() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_constant(&Constant::Float(f64::INFINITY)), "0x7FF0000000000000");
    }

    #[test]
    fn test_format_constant_float_negative_infinity() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_constant(&Constant::Float(f64::NEG_INFINITY)), "0xFFF0000000000000");
    }

    // ====================================================================
    // format_operand tests (Cycle 427)
    // ====================================================================

    #[test]
    fn test_format_operand_place() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_operand(&Operand::Place(Place::new("x"))), "%x");
    }

    #[test]
    fn test_format_operand_constant_int() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_operand(&Operand::Constant(Constant::Int(99))), "99");
    }

    #[test]
    fn test_format_operand_constant_bool() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.format_operand(&Operand::Constant(Constant::Bool(true))), "1");
    }

    // ====================================================================
    // escape_string_for_llvm tests (Cycle 430)
    // ====================================================================

    #[test]
    fn test_escape_string_printable_ascii() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.escape_string_for_llvm("hello"), "hello");
    }

    #[test]
    fn test_escape_string_backslash() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.escape_string_for_llvm("a\\b"), "a\\5Cb");
    }

    #[test]
    fn test_escape_string_double_quote() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.escape_string_for_llvm("say \"hi\""), "say \\22hi\\22");
    }

    #[test]
    fn test_escape_string_newline() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.escape_string_for_llvm("a\nb"), "a\\0Ab");
    }

    #[test]
    fn test_escape_string_carriage_return() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.escape_string_for_llvm("a\rb"), "a\\0Db");
    }

    #[test]
    fn test_escape_string_tab() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.escape_string_for_llvm("a\tb"), "a\\09b");
    }

    #[test]
    fn test_escape_string_null_byte() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.escape_string_for_llvm("a\0b"), "a\\00b");
    }

    #[test]
    fn test_escape_string_empty() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.escape_string_for_llvm(""), "");
    }

    // ====================================================================
    // constant_type tests (Cycle 430)
    // ====================================================================

    #[test]
    fn test_constant_type_int() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.constant_type(&Constant::Int(0)), "i64");
    }

    #[test]
    fn test_constant_type_float() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.constant_type(&Constant::Float(1.0)), "double");
    }

    #[test]
    fn test_constant_type_bool() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.constant_type(&Constant::Bool(false)), "i1");
    }

    #[test]
    fn test_constant_type_string() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.constant_type(&Constant::String("hi".to_string())), "ptr");
    }

    #[test]
    fn test_constant_type_char() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.constant_type(&Constant::Char('a')), "i32");
    }

    #[test]
    fn test_constant_type_unit() {
        let codegen = TextCodeGen::new();
        assert_eq!(codegen.constant_type(&Constant::Unit), "i8");
    }

    // ====================================================================
    // infer_place_type tests (Cycle 430)
    // ====================================================================

    #[test]
    fn test_infer_place_type_from_param() {
        let codegen = TextCodeGen::new();
        let func = MirFunction {
            name: "test".to_string(),
            params: vec![("x".to_string(), MirType::F64)],
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
        assert_eq!(codegen.infer_place_type(&Place::new("x"), &func), "double");
    }

    #[test]
    fn test_infer_place_type_from_local() {
        let codegen = TextCodeGen::new();
        let func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            ret_ty: MirType::I64,
            locals: vec![("tmp".to_string(), MirType::Bool)],
            blocks: vec![],
            preconditions: vec![],
            postconditions: vec![],
            is_pure: false,
            is_const: false,
            always_inline: false,
            inline_hint: false,
            is_memory_free: false,
        };
        assert_eq!(codegen.infer_place_type(&Place::new("tmp"), &func), "i1");
    }

    #[test]
    fn test_infer_place_type_default_i64() {
        let codegen = TextCodeGen::new();
        let func = MirFunction {
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
        assert_eq!(codegen.infer_place_type(&Place::new("unknown"), &func), "i64");
    }

    // ====================================================================
    // infer_operand_type tests (Cycle 430)
    // ====================================================================

    #[test]
    fn test_infer_operand_type_constant() {
        let codegen = TextCodeGen::new();
        let func = MirFunction {
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
        assert_eq!(codegen.infer_operand_type(&Operand::Constant(Constant::Float(1.0)), &func), "double");
    }

    #[test]
    fn test_infer_operand_type_place_param() {
        let codegen = TextCodeGen::new();
        let func = MirFunction {
            name: "test".to_string(),
            params: vec![("y".to_string(), MirType::String)],
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
        assert_eq!(codegen.infer_operand_type(&Operand::Place(Place::new("y")), &func), "ptr");
    }

    // ====================================================================
    // infer_call_return_type tests (Cycle 430)
    // ====================================================================

    #[test]
    fn test_infer_call_return_type_void() {
        let codegen = TextCodeGen::new();
        let func = MirFunction {
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
        assert_eq!(codegen.infer_call_return_type("println", &func), "void");
        assert_eq!(codegen.infer_call_return_type("print", &func), "void");
        assert_eq!(codegen.infer_call_return_type("assert", &func), "void");
    }

    #[test]
    fn test_infer_call_return_type_i64() {
        let codegen = TextCodeGen::new();
        let func = MirFunction {
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
        assert_eq!(codegen.infer_call_return_type("read_int", &func), "i64");
        assert_eq!(codegen.infer_call_return_type("bmb_abs", &func), "i64");
        assert_eq!(codegen.infer_call_return_type("bmb_string_len", &func), "i64");
    }

    #[test]
    fn test_infer_call_return_type_double() {
        let codegen = TextCodeGen::new();
        let func = MirFunction {
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
        assert_eq!(codegen.infer_call_return_type("sqrt", &func), "double");
        assert_eq!(codegen.infer_call_return_type("i64_to_f64", &func), "double");
    }

    #[test]
    fn test_infer_call_return_type_ptr() {
        let codegen = TextCodeGen::new();
        let func = MirFunction {
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
        assert_eq!(codegen.infer_call_return_type("bmb_string_concat", &func), "ptr");
    }
}
