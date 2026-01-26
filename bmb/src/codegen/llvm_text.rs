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

    /// Generate complete LLVM IR module as text
    pub fn generate(&self, program: &MirProgram) -> TextCodeGenResult<String> {
        let mut output = String::new();

        // Module header
        writeln!(output, "; ModuleID = bmb_program")?;
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
                if let MirType::Struct { fields, .. } = &f.ret_ty {
                    if fields.len() > 2 {  // Only 3+ fields use sret
                        return Some((f.name.clone(), fields.len()));
                    }
                }
                None
            })
            .collect();

        // v0.51.28: Small struct functions (1-2 fields) use register return
        let small_struct_functions: HashMap<String, usize> = program
            .functions
            .iter()
            .filter_map(|f| {
                if let MirType::Struct { fields, .. } = &f.ret_ty {
                    if !fields.is_empty() && fields.len() <= 2 {
                        return Some((f.name.clone(), fields.len()));
                    }
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
            self.emit_function_with_strings(&mut output, func, &string_table, &fn_return_types, &fn_param_types, &sret_functions, &small_struct_functions, &program.struct_defs)?;
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
        writeln!(out, "declare i64 @read_int()")?;
        writeln!(out, "declare void @assert(i1)")?;
        writeln!(out, "declare i64 @bmb_abs(i64)")?;  // bmb_ prefix to avoid stdlib conflict
        writeln!(out, "declare i64 @min(i64, i64)")?;
        writeln!(out, "declare i64 @max(i64, i64)")?;
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
        writeln!(out, "declare i64 @bmb_string_eq(ptr, ptr) memory(argmem: read) nounwind willreturn")?;
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
        writeln!(out, "declare i64 @bmb_append_file(ptr, ptr)")?;
        writeln!(out)?;

        // Phase 32.3: StringBuilder runtime functions
        writeln!(out, "; Runtime declarations - StringBuilder")?;
        writeln!(out, "declare i64 @bmb_sb_new()")?;
        writeln!(out, "declare i64 @bmb_sb_push(i64, ptr)")?;
        writeln!(out, "declare i64 @bmb_sb_push_char(i64, i64)")?;
        writeln!(out, "declare i64 @bmb_sb_push_int(i64, i64)")?;  // v0.50.73
        writeln!(out, "declare i64 @bmb_sb_push_escaped(i64, ptr)")?;  // v0.50.74
        writeln!(out, "declare i64 @bmb_sb_len(i64)")?;
        writeln!(out, "declare ptr @bmb_sb_build(i64)")?;
        writeln!(out, "declare i64 @bmb_sb_clear(i64)")?;
        writeln!(out)?;

        // Phase 32.3: Process execution runtime functions
        writeln!(out, "; Runtime declarations - Process execution")?;
        writeln!(out, "declare i64 @bmb_system(ptr)")?;
        writeln!(out, "declare ptr @bmb_getenv(ptr)")?;
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
        writeln!(out, "declare i64 @sb_push(i64, ptr) nounwind")?;
        writeln!(out, "declare i64 @sb_push_cstr(i64, ptr) nounwind")?;  // v0.50.77: zero allocation for string literals
        writeln!(out, "declare i64 @sb_push_char(i64, i64) nounwind")?;
        writeln!(out, "declare i64 @sb_push_int(i64, i64) nounwind")?;  // v0.50.73
        writeln!(out, "declare i64 @sb_push_escaped(i64, ptr) nounwind")?;  // v0.50.74
        writeln!(out, "declare i64 @sb_len(i64) nounwind willreturn")?;
        writeln!(out, "declare ptr @sb_build(i64) nounwind")?;
        writeln!(out, "declare i64 @sb_clear(i64) nounwind")?;
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
        writeln!(out)?;

        // v0.34.2: Memory allocation for Phase 34.2 Dynamic Collections
        // v0.51.26: Added noalias and nounwind for better optimization
        writeln!(out, "; Runtime declarations - Memory allocation")?;
        writeln!(out, "declare noalias ptr @malloc(i64) nounwind allocsize(0)")?;
        writeln!(out, "declare noalias ptr @realloc(ptr, i64) nounwind allocsize(1)")?;
        writeln!(out, "declare void @free(ptr nocapture) nounwind")?;
        writeln!(out, "declare noalias ptr @calloc(i64, i64) nounwind allocsize(0,1)")?;
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
        writeln!(out, "declare void @hashmap_free(i64) nounwind")?;
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
        let empty_struct_defs = HashMap::new();
        self.emit_function_with_strings(out, func, &empty_str_table, &empty_fn_types, &empty_fn_param_types, &empty_sret_functions, &empty_small_struct_functions, &empty_struct_defs)
    }

    /// v0.51.25: Check if a specific struct variable escapes the function
    /// A struct escapes if it's returned, passed to a call, or copied to something that escapes
    fn check_struct_escapes(&self, func: &MirFunction, struct_name: &str) -> bool {
        use crate::mir::{Terminator, Operand};

        for block in &func.blocks {
            // Check if returned
            if let Terminator::Return(Some(Operand::Place(p))) = &block.terminator {
                if p.name == struct_name {
                    return true;
                }
            }

            for inst in &block.instructions {
                match inst {
                    // Passed to a call
                    MirInst::Call { args, .. } => {
                        for arg in args {
                            if let Operand::Place(p) = arg {
                                if p.name == struct_name {
                                    return true;
                                }
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
                            if let Operand::Place(p) = val {
                                if p.name == struct_name {
                                    return true;
                                }
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
            if let Terminator::Return(Some(Operand::Place(p))) = &block.terminator {
                if p.name == struct_name {
                    return true;
                }
            }

            // Check if flows through phi to return
            for inst in &block.instructions {
                if let MirInst::Phi { dest, values } = inst {
                    // If phi dest is returned and this struct is one of phi inputs
                    if self.is_struct_returned_inner(func, &dest.name, visited) {
                        for (val, _) in values {
                            if let Operand::Place(p) = val {
                                if p.name == struct_name {
                                    return true;
                                }
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
            if let Terminator::Return(Some(Operand::Place(p))) = &block.terminator {
                if struct_vars.contains(&p.name) {
                    escaped.insert(p.name.clone());
                }
            }

            // Check instructions for call arguments and assignments
            for inst in &block.instructions {
                match inst {
                    // Calls: any struct passed as argument escapes
                    MirInst::Call { args, .. } => {
                        for arg in args {
                            if let Operand::Place(p) = arg {
                                if struct_vars.contains(&p.name) {
                                    escaped.insert(p.name.clone());
                                }
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
                            if let Operand::Place(p) = val {
                                if struct_vars.contains(&p.name) {
                                    // If this phi is returned, the struct escapes
                                    // Mark it conservatively
                                    escaped.insert(p.name.clone());
                                }
                            }
                        }
                        // If dest is escaped, mark all incoming struct values
                        if escaped.contains(&dest.name) {
                            for (val, _) in values {
                                if let Operand::Place(p) = val {
                                    if struct_vars.contains(&p.name) {
                                        escaped.insert(p.name.clone());
                                    }
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
                        place_types.insert(dest.name.clone(), self.constant_type(value));
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
                                (_, "i64") | ("i64", _) => "i64",
                                (_, "double") | ("double", _) => "double",
                                (_, "ptr") | ("ptr", _) => "ptr",
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
                            // Compare integer widths: i64 > i32 > i1
                            widest_ty = match (widest_ty, ty) {
                                (_, "i64") | ("i64", _) => "i64",
                                (_, "double") | ("double", _) => "double",
                                (_, "ptr") | ("ptr", _) => "ptr",
                                ("i32", "i32") => "i32",
                                ("i32", "i1") | ("i1", "i32") => "i32",
                                ("i1", "i1") => "i1",
                                _ => ty, // Default to the new type
                            };
                        }
                        place_types.insert(dest.name.clone(), widest_ty);
                    }
                    MirInst::Copy { dest, src } => {
                        // Copy inherits type from source
                        let ty = place_types.get(&src.name).copied().unwrap_or("i64");
                        place_types.insert(dest.name.clone(), ty);
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
                    _ => {}
                }
            }
        }

        place_types
    }

    /// Emit a function definition with string table support
    fn emit_function_with_strings(
        &self,
        out: &mut String,
        func: &MirFunction,
        string_table: &HashMap<String, String>,
        fn_return_types: &HashMap<String, &'static str>,
        fn_param_types: &HashMap<String, Vec<&'static str>>,
        sret_functions: &HashMap<String, usize>,
        small_struct_functions: &HashMap<String, usize>,
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

        // v0.51.27: Detect struct return functions for sret optimization
        // v0.51.28: Small structs (1-2 fields) use register return, larger structs use sret
        let struct_field_count = if let MirType::Struct { fields, .. } = &func.ret_ty {
            fields.len()
        } else {
            0
        };
        let is_small_struct = struct_field_count > 0 && struct_field_count <= 2;
        let is_sret = struct_field_count > 2;  // Only use sret for 3+ field structs

        // Function signature - small structs use aggregate return type
        let ret_type = if is_sret {
            "void"
        } else if is_small_struct {
            if struct_field_count == 1 { "{i64}" } else { "{i64, i64}" }
        } else {
            self.mir_type_to_llvm(&func.ret_ty)
        };
        let mut params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| format!("{} %{}", self.mir_type_to_llvm(ty), name))
            .collect();

        // v0.51.27: Add sret parameter for struct return functions
        if is_sret {
            params.insert(0, format!("ptr noalias sret(i8) %_sret"));
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
        let attrs = if func.name == "main" {
            String::new()
        } else if func.always_inline && func.is_memory_free {
            " alwaysinline nounwind willreturn mustprogress memory(none)".to_string()
        } else if func.always_inline {
            " alwaysinline nounwind willreturn mustprogress".to_string()
        } else if func.is_memory_free {
            " nounwind willreturn mustprogress memory(none)".to_string()
        } else {
            " nounwind willreturn mustprogress".to_string()
        };

        // v0.31.23: Rename BMB main to bmb_user_main so C runtime can provide real main()
        // This enables argv support through bmb_init_argv called from real main()
        let emitted_name = if func.name == "main" { "bmb_user_main" } else { &func.name };

        writeln!(
            out,
            "define {} @{}({}){} {{",
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
                        let needs_coerce = match (val_ty, phi_ty) {
                            ("i32", "i64") => true,
                            ("i1", "i64") | ("i1", "i32") => true,
                            _ => false,
                        };

                        if needs_coerce {
                            if let Operand::Place(p) = val {
                                let key = (block.label.clone(), p.name.clone(), pred_label.clone());
                                let temp_name = format!("_phi_sext_{}", coerce_counter);
                                coerce_counter += 1;
                                phi_coerce_map.insert(key, (temp_name, val_ty, phi_ty));
                            }
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
                    // v0.51.30: Use place_types for alloca to handle narrowed types correctly
                    // This ensures that when a BinaryOp produces i32 (due to narrowing optimization),
                    // the alloca is also i32, avoiding mismatched store/load sizes.
                    let llvm_ty = place_types.get(name).copied().unwrap_or_else(|| self.mir_type_to_llvm(ty));
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
            self.emit_block_with_strings(out, block, func, string_table, fn_return_types, fn_param_types, sret_functions, small_struct_functions, &place_types, &mut name_counts, &local_names, &narrowed_param_names, &phi_load_map, &phi_string_map, &phi_coerce_map, struct_defs)?;
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
        let empty_struct_defs = HashMap::new();
        self.emit_block_with_strings(out, block, func, &empty_str_table, &empty_fn_types, &empty_fn_param_types, &empty_sret_functions, &empty_small_struct_functions, &empty_place_types, &mut empty_name_counts, &empty_local_names, &empty_narrowed, &empty_phi_map, &empty_phi_string_map, &empty_phi_coerce_map, &empty_struct_defs)
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
            self.emit_instruction_with_strings(out, inst, func, string_table, fn_return_types, fn_param_types, sret_functions, small_struct_functions, place_types, name_counts, local_names, narrowed_param_names, phi_load_map, phi_string_map, phi_coerce_map, &block.label, struct_defs)?;
        }

        // Emit loads for locals that will be used in phi nodes of successor blocks
        // This must happen BEFORE the terminator
        for ((_dest_block, local_name, pred_block), load_temp) in phi_load_map {
            if pred_block == &block.label {
                // Use place_types if available (more accurate), fall back to func.locals
                let llvm_ty = if let Some(ty) = place_types.get(local_name) {
                    *ty
                } else if let Some((_, ty)) = func.locals.iter().find(|(n, _)| n == local_name) {
                    self.mir_type_to_llvm(ty)
                } else {
                    "ptr" // Default to ptr for unknown types
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
        self.emit_terminator(out, &block.terminator, func, string_table, local_names, narrowed_param_names, &block.label)?;

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
                let ty = self.constant_type(value);
                // Check if destination is a local (uses alloca)
                if local_names.contains(&dest.name) {
                    // v0.51.33: Store constants directly to allocas without intermediate SSA values
                    // This eliminates unnecessary `add 0, const` instructions
                    match value {
                        Constant::Int(n) => {
                            writeln!(out, "  store {} {}, ptr %{}.addr", ty, n, dest.name)?;
                        }
                        Constant::Bool(b) => {
                            let v = if *b { 1 } else { 0 };
                            writeln!(out, "  store {} {}, ptr %{}.addr", ty, v, dest.name)?;
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
                            writeln!(out, "  store {} {}, ptr %{}.addr", ty, f_str, dest.name)?;
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
                            writeln!(out, "  store {} {}, ptr %{}.addr", ty, *c as u32, dest.name)?;
                        }
                    }
                } else {
                    let dest_name = self.unique_name(&dest.name, name_counts);
                    // Use add with 0 for integer constants (LLVM IR idiom)
                    match value {
                        Constant::Int(n) => {
                            writeln!(out, "  %{} = add {} 0, {}", dest_name, ty, n)?;
                        }
                        Constant::Bool(b) => {
                            let v = if *b { 1 } else { 0 };
                            writeln!(out, "  %{} = add {} 0, {}", dest_name, ty, v)?;
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
                            writeln!(out, "  %{} = fadd {} 0.0, {}", dest_name, ty, f_str)?;
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
                            writeln!(out, "  %{} = add {} 0, {}", dest_name, ty, *c as u32)?;
                        }
                    }
                }
            }

            MirInst::Copy { dest, src } => {
                // Use place_types for accurate type inference
                let ty = place_types.get(&src.name).copied()
                    .unwrap_or_else(|| self.infer_place_type(src, func));

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
                    writeln!(out, "  store {} {}, ptr %{}.addr", ty, src_val, dest.name)?;
                } else {
                    let dest_name = self.unique_name(&dest.name, name_counts);
                    if ty == "ptr" {
                        // For pointers, use select with always-true condition
                        writeln!(out, "  %{} = select i1 true, ptr {}, ptr null", dest_name, src_val)?;
                    } else if ty == "f64" {
                        // For floats, use fadd
                        writeln!(out, "  %{} = fadd {} {}, 0.0", dest_name, ty, src_val)?;
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
                    // String comparison: at least one operand is a string constant
                    let lhs_is_string = matches!(lhs, Operand::Constant(Constant::String(_)));
                    let rhs_is_string = matches!(rhs, Operand::Constant(Constant::String(_)));

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
                        writeln!(out, "  %{} = icmp eq ptr {}, {}", dest_name, lhs_str, rhs_str)?;
                    }
                    // v0.46: Store result to alloca if destination is a local variable
                    if local_names.contains(&dest.name) {
                        writeln!(out, "  store i1 %{}, ptr %{}.addr", dest_name, dest.name)?;
                    }
                } else if (lhs_ty == "ptr" || rhs_ty == "ptr") && *op == MirBinOp::Ne {
                    // v0.51.37: Distinguish string comparison from typed pointer comparison
                    let lhs_is_string = matches!(lhs, Operand::Constant(Constant::String(_)));
                    let rhs_is_string = matches!(rhs, Operand::Constant(Constant::String(_)));

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
                        writeln!(out, "  %{} = icmp ne ptr {}, {}", dest_name, lhs_str, rhs_str)?;
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
                    let op_str = if lhs_ty == "double" || lhs_ty == "f64" {
                        match op {
                            MirBinOp::Add | MirBinOp::FAdd => "fadd",
                            MirBinOp::Sub | MirBinOp::FSub => "fsub",
                            MirBinOp::Mul | MirBinOp::FMul => "fmul",
                            MirBinOp::Div | MirBinOp::FDiv => "fdiv",
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
                        writeln!(out, "  %{} = fsub {} 0.0, {}", dest_name, ty, src_str)?;
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
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let temp_name = format!("{}.conv", d.name);
                            writeln!(out, "  %{} = sitofp i64 {} to double", temp_name, arg_val)?;
                            writeln!(out, "  store double %{}, ptr %{}.addr", temp_name, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = sitofp i64 {} to double", dest_name, arg_val)?;
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
                    // Get pointer argument
                    let ptr_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.free.ptr", p.name);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Convert i64 to ptr and call free
                    let ptr_conv = format!("free_ptr.{}", name_counts.entry("free_ptr".to_string()).or_insert(0));
                    *name_counts.get_mut("free_ptr").unwrap() += 1;
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", ptr_conv, ptr_val)?;
                    writeln!(out, "  call void @free(ptr %{})", ptr_conv)?;
                    return Ok(());
                }

                // v0.34.2: store_i64(ptr, value) -> Unit - writes i64 value to memory
                // Also handles box_set_i64 as an alias
                if (fn_name == "store_i64" || fn_name == "box_set_i64") && args.len() == 2 {
                    // Get unique index for this store operation
                    let store_idx = *name_counts.entry("store_op".to_string()).or_insert(0);
                    *name_counts.get_mut("store_op").unwrap() += 1;
                    // Get pointer argument
                    let ptr_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.store.ptr.{}", p.name, store_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };
                    // Get value argument
                    let val_val = match &args[1] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.store.val.{}", p.name, store_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
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
                    let ptr_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("{}.load.ptr.{}", p.name, load_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
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

                // v0.34.2.3: Vec<i64> dynamic array builtins (RFC-0007)
                // vec_new() -> i64: allocate header (24 bytes) with zeroed ptr/len/cap
                if fn_name == "vec_new" && args.is_empty() {
                    let vec_idx = *name_counts.entry("vec_new".to_string()).or_insert(0);
                    *name_counts.get_mut("vec_new").unwrap() += 1;
                    // Call malloc(24) for header
                    let header_ptr = format!("vec.header.{}", vec_idx);
                    writeln!(out, "  %{} = call ptr @malloc(i64 24)", header_ptr)?;
                    // Zero out header (ptr=0, len=0, cap=0)
                    let zero_ptr = format!("vec.zero.ptr.{}", vec_idx);
                    let len_ptr = format!("vec.zero.len.{}", vec_idx);
                    let cap_ptr = format!("vec.zero.cap.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 0", zero_ptr, header_ptr)?;
                    writeln!(out, "  store i64 0, ptr %{}", zero_ptr)?;
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 1", len_ptr, header_ptr)?;
                    writeln!(out, "  store i64 0, ptr %{}", len_ptr)?;
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 2", cap_ptr, header_ptr)?;
                    writeln!(out, "  store i64 0, ptr %{}", cap_ptr)?;
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

                // vec_with_capacity(cap) -> i64: allocate header + data array
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
                    // Allocate header
                    let header_ptr = format!("vec.wcap.header.{}", vec_idx);
                    writeln!(out, "  %{} = call ptr @malloc(i64 24)", header_ptr)?;
                    // Allocate data: cap * 8 bytes
                    let data_size = format!("vec.wcap.size.{}", vec_idx);
                    let data_ptr = format!("vec.wcap.data.{}", vec_idx);
                    writeln!(out, "  %{} = mul i64 {}, 8", data_size, cap_val)?;
                    writeln!(out, "  %{} = call ptr @malloc(i64 %{})", data_ptr, data_size)?;
                    // Store data ptr at header[0]
                    let data_as_i64 = format!("vec.wcap.ptr.{}", vec_idx);
                    writeln!(out, "  %{} = ptrtoint ptr %{} to i64", data_as_i64, data_ptr)?;
                    let h0 = format!("vec.wcap.h0.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 0", h0, header_ptr)?;
                    writeln!(out, "  store i64 %{}, ptr %{}", data_as_i64, h0)?;
                    // Store len=0 at header[1]
                    let h1 = format!("vec.wcap.h1.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 1", h1, header_ptr)?;
                    writeln!(out, "  store i64 0, ptr %{}", h1)?;
                    // Store cap at header[2]
                    let h2 = format!("vec.wcap.h2.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 2", h2, header_ptr)?;
                    writeln!(out, "  store i64 {}, ptr %{}", cap_val, h2)?;
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

                // vec_cap(vec) -> i64: read header[2]
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
                    let header_ptr = format!("vec.cap.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;
                    let cap_ptr_name = format!("vec.cap.ptr.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 2", cap_ptr_name, header_ptr)?;
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

                // vec_get(vec, index) -> i64: read data[index]
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
                        _ => self.format_operand_with_strings(&args[1], string_table),
                    };
                    // Get header and data pointer
                    let header_ptr = format!("vec.get.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;
                    let data_i64 = format!("vec.get.data.i64.{}", vec_idx);
                    writeln!(out, "  %{} = load i64, ptr %{}", data_i64, header_ptr)?;
                    let data_ptr = format!("vec.get.data.ptr.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 %{} to ptr", data_ptr, data_i64)?;
                    // Get element at index
                    let elem_ptr = format!("vec.get.elem.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 {}", elem_ptr, data_ptr, idx_val)?;
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let elem_val = format!("vec.get.val.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}", elem_val, elem_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", elem_val, d.name)?;
                        } else {
                            // Use dest name directly for SSA assignment
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
                    // Get header and data pointer
                    let header_ptr = format!("vec.set.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;
                    let data_i64 = format!("vec.set.data.i64.{}", vec_idx);
                    writeln!(out, "  %{} = load i64, ptr %{}", data_i64, header_ptr)?;
                    let data_ptr = format!("vec.set.data.ptr.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 %{} to ptr", data_ptr, data_i64)?;
                    // Store value at index
                    let elem_ptr = format!("vec.set.elem.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 {}", elem_ptr, data_ptr, idx_val)?;
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

                // vec_pop(vec) -> i64: remove and return last element
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
                    // Load ptr and len
                    let ptr_i64 = format!("vpop.ptr.{}", vec_idx);
                    let len_ptr = format!("vpop.len.ptr.{}", vec_idx);
                    let len_val = format!("vpop.len.{}", vec_idx);
                    writeln!(out, "  %{} = load i64, ptr %{}", ptr_i64, header_ptr)?;
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 1", len_ptr, header_ptr)?;
                    writeln!(out, "  %{} = load i64, ptr %{}", len_val, len_ptr)?;
                    // Get last element: data[len-1]
                    let data_ptr = format!("vpop.data.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 %{} to ptr", data_ptr, ptr_i64)?;
                    let last_idx = format!("vpop.last_idx.{}", vec_idx);
                    writeln!(out, "  %{} = sub i64 %{}, 1", last_idx, len_val)?;
                    let elem_ptr = format!("vpop.elem.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 %{}", elem_ptr, data_ptr, last_idx)?;
                    // Load element and decrement len, then return
                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let elem_val = format!("vpop.val.{}", vec_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}", elem_val, elem_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}", last_idx, len_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", elem_val, d.name)?;
                        } else {
                            // Use dest name directly for SSA assignment
                            writeln!(out, "  %{} = load i64, ptr %{}", d.name, elem_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}", last_idx, len_ptr)?;
                        }
                    } else {
                        // No dest, still decrement len
                        writeln!(out, "  store i64 %{}, ptr %{}", last_idx, len_ptr)?;
                    }
                    return Ok(());
                }

                // vec_free(vec) -> Unit: free data array and header
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
                    // Get header and data ptr
                    let header_ptr = format!("vfree.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;
                    let ptr_i64 = format!("vfree.ptr.{}", vec_idx);
                    writeln!(out, "  %{} = load i64, ptr %{}", ptr_i64, header_ptr)?;
                    // Check if data ptr is non-null
                    let ptr_nonnull = format!("vfree.nonnull.{}", vec_idx);
                    writeln!(out, "  %{} = icmp ne i64 %{}, 0", ptr_nonnull, ptr_i64)?;
                    let free_data_label = format!("vfree.data.{}", vec_idx);
                    let free_header_label = format!("vfree.header.lbl.{}", vec_idx);
                    writeln!(out, "  br i1 %{}, label %{}, label %{}", ptr_nonnull, free_data_label, free_header_label)?;
                    // Free data array
                    writeln!(out, "{}:", free_data_label)?;
                    let data_ptr = format!("vfree.data.ptr.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 %{} to ptr", data_ptr, ptr_i64)?;
                    writeln!(out, "  call void @free(ptr %{})", data_ptr)?;
                    writeln!(out, "  br label %{}", free_header_label)?;
                    // Free header
                    writeln!(out, "{}:", free_header_label)?;
                    writeln!(out, "  call void @free(ptr %{})", header_ptr)?;
                    return Ok(());
                }

                // v0.50.61: Inline string operations for zero-cost string access
                // BmbString layout: {ptr data, i64 len, i64 cap}

                // len(s) -> i64: inline string length access
                if (fn_name == "len" || fn_name == "bmb_string_len") && args.len() == 1 {
                    let str_idx = *name_counts.entry("str_len".to_string()).or_insert(0);
                    *name_counts.get_mut("str_len").unwrap() += 1;

                    // Get string pointer argument
                    let str_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("strlen.str.{}", str_idx);
                            writeln!(out, "  %{} = load ptr, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
                        }
                        _ => self.format_operand_with_strings(&args[0], string_table),
                    };

                    // Access len field at offset 1 in BmbString struct
                    // struct BmbString { ptr data; i64 len; i64 cap; }
                    let len_ptr = format!("strlen.len_ptr.{}", str_idx);
                    writeln!(out, "  %{} = getelementptr {{ptr, i64, i64}}, ptr {}, i32 0, i32 1", len_ptr, str_val)?;

                    if let Some(d) = dest {
                        if local_names.contains(&d.name) {
                            let len_val = format!("strlen.len.{}", str_idx);
                            writeln!(out, "  %{} = load i64, ptr %{}", len_val, len_ptr)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", len_val, d.name)?;
                        } else {
                            let dest_name = self.unique_name(&d.name, name_counts);
                            writeln!(out, "  %{} = load i64, ptr %{}", dest_name, len_ptr)?;
                        }
                    }
                    return Ok(());
                }

                // char_at(s, idx) / byte_at(s, idx) -> i64: inline character access
                if (fn_name == "char_at" || fn_name == "byte_at" || fn_name == "bmb_string_char_at") && args.len() == 2 {
                    let str_idx = *name_counts.entry("str_char_at".to_string()).or_insert(0);
                    *name_counts.get_mut("str_char_at").unwrap() += 1;

                    // Get string pointer argument
                    let str_val = match &args[0] {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            let load_name = format!("charat.str.{}", str_idx);
                            writeln!(out, "  %{} = load ptr, ptr %{}.addr", load_name, p.name)?;
                            format!("%{}", load_name)
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
                            let ext_val = format!("charat.ext.{}", str_idx);
                            writeln!(out, "  %{} = zext i8 %{} to i64", ext_val, char_val)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", ext_val, d.name)?;
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
                                writeln!(out, "  store i64 {}, ptr %{}.addr", char_val, d.name)?;
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
                            let ext_val = format!("ord.ext.{}", str_idx);
                            writeln!(out, "  %{} = zext i8 %{} to i64", ext_val, char_val)?;
                            writeln!(out, "  store i64 %{}, ptr %{}.addr", ext_val, d.name)?;
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

                // v0.51.17: Pre-emit truncation instructions for narrowed parameters
                if let Some(param_types) = param_types_opt {
                    for (i, (arg_ty, val, _)) in arg_vals.iter().enumerate() {
                        if let Some(&param_ty) = param_types.get(i) {
                            if arg_ty == "i64" && param_ty == "i32" {
                                let trunc_name = format!("{}.arg{}.trunc", call_base, i);
                                writeln!(out, "  %{} = trunc i64 {} to i32", trunc_name, val)?;
                            }
                        }
                    }
                }

                // v0.51.17: Rebuild args_str with proper truncation references
                let args_str: Vec<String> = arg_vals
                    .iter()
                    .enumerate()
                    .map(|(i, (arg_ty, val, _))| {
                        if let Some(param_types) = param_types_opt {
                            if let Some(&param_ty) = param_types.get(i) {
                                if arg_ty == "i64" && param_ty == "i32" {
                                    let trunc_name = format!("{}.arg{}.trunc", call_base, i);
                                    return format!("i32 %{}", trunc_name);
                                }
                                if arg_ty == "i32" && param_ty == "i64" {
                                    // Sign extend would be needed, but this case is rarer
                                    return format!("{} {}", param_ty, val);
                                }
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
                    "file_exists" if all_string_args_are_literals => "file_exists_cstr",
                    "bmb_file_exists" if all_string_args_are_literals => "bmb_file_exists_cstr",
                    // v0.50.77: StringBuilder optimization - use cstr variant for string literals
                    "sb_push" if args.len() == 2 && matches!(&args[1], Operand::Constant(Constant::String(_))) => "sb_push_cstr",
                    "bmb_sb_push" if args.len() == 2 && matches!(&args[1], Operand::Constant(Constant::String(_))) => "bmb_sb_push_cstr",
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
                // v0.51.13: Use place_types for phi type - this has the WIDEST type among all values
                // This handles ConstantPropagationNarrowing where param is i32 but return is i64
                let ty = place_types.get(&dest.name).copied().unwrap_or_else(|| {
                    // Fallback: infer from first value
                    if !values.is_empty() {
                        match &values[0].0 {
                            Operand::Constant(c) => self.constant_type(c),
                            Operand::Place(p) => place_types.get(&p.name).copied()
                                .unwrap_or_else(|| self.infer_place_type(p, func)),
                        }
                    } else {
                        "i64"
                    }
                });

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
                    let val_str = match value {
                        Operand::Place(p) => {
                            let is_param = func.params.iter().any(|(name, _)| name == &p.name);
                            if !is_param {
                                // Local: load from .addr
                                let load_name = format!("{}_f{}_val", dest.name, i);
                                writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, ty, p.name)?;
                                format!("%{}", load_name)
                            } else {
                                // Param: use directly
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
                writeln!(out, "  %{} = load {}, ptr %{}_ptr", dest.name, field_llvm_ty, dest.name)?;
            }

            MirInst::FieldStore { base, field, field_index, struct_name, value } => {
                // v0.51.23: Store value to field in struct pointer using correct offset
                // v0.51.24: Check if base is a parameter (already ptr) or local (needs load from .addr)
                // v0.51.31: Use struct_defs to look up correct field type for GEP instruction
                // v0.51.32: Use struct type GEPs for better LLVM alias analysis
                // v0.51.36: Handle temps from struct array IndexLoad (direct ptrs, not in locals)
                // v0.51.38: Load value from .addr if it's a local variable
                let ty = self.infer_operand_type(value, func);

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
                writeln!(out, "  ; field store .{}[{}] ({}) = {}", field, field_index, struct_name, val_str)?;

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
                writeln!(out, "  store {} {}, ptr %{}_f{}_ptr{}", ty, val_str, base.name, field_index, suffix)?;
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
                // Store discriminant (simplified: hash of variant name)
                let discriminant: i64 = variant.bytes().fold(0i64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as i64));
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

                let idx_str = if let Operand::Place(p) = index {
                    if local_names.contains(&p.name) {
                        writeln!(out, "  %{}_idx_load = load i64, ptr %{}.addr", dest.name, p.name)?;
                        format!("%{}_idx_load", dest.name)
                    } else {
                        self.format_operand(index)
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

                let idx_str = if let Operand::Place(p) = index {
                    if local_names.contains(&p.name) {
                        writeln!(out, "  %{}_store_idx.{} = load i64, ptr %{}.addr", array.name, store_cnt, p.name)?;
                        format!("%{}_store_idx.{}", array.name, store_cnt)
                    } else {
                        self.format_operand(index)
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
                    let ty = self.infer_operand_type(value, func);
                    let val_str = if let Operand::Place(p) = value {
                        if local_names.contains(&p.name) {
                            writeln!(out, "  %{}_store_val.{} = load {}, ptr %{}.addr", array.name, store_cnt, ty, p.name)?;
                            format!("%{}_store_val.{}", array.name, store_cnt)
                        } else {
                            self.format_operand(value)
                        }
                    } else {
                        self.format_operand(value)
                    };
                    writeln!(out, "  ; index store %{}[{}] = {}", array.name, idx_str, val_str)?;
                    writeln!(out, "  %{}_idx_ptr.{} = getelementptr {}, ptr {}, i64 {}",
                             array.name, store_cnt, ty, arr_ptr, idx_str)?;
                    writeln!(out, "  store {} {}, ptr %{}_idx_ptr.{}", ty, val_str, array.name, store_cnt)?;
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
        }

        Ok(())
    }

    /// Emit a terminator
    fn emit_terminator(
        &self,
        out: &mut String,
        term: &Terminator,
        func: &MirFunction,
        string_table: &HashMap<String, String>,
        local_names: &std::collections::HashSet<String>,
        narrowed_param_names: &std::collections::HashSet<String>,
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

                if is_small_struct {
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
                            writeln!(out, "  %{} = getelementptr i64, ptr {}, i32 {}", format!("_sret_gep_src.{}.{}", i, block_label), src_ptr, i)?;
                            writeln!(out, "  %{} = load i64, ptr %{}", field_load, format!("_sret_gep_src.{}.{}", i, block_label))?;
                            writeln!(out, "  %{} = getelementptr i64, ptr %_sret, i32 {}", format!("_sret_gep_dst.{}.{}", i, block_label), i)?;
                            writeln!(out, "  store i64 %{}, ptr %{}", field_load, format!("_sret_gep_dst.{}.{}", i, block_label))?;
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
                            // Use block_label + var name for SSA uniqueness across multiple return blocks
                            writeln!(out, "  %_ret_val.{}.{} = load {}, ptr %{}.addr", block_label, p.name, ty, p.name)?;
                            writeln!(out, "  ret {} %_ret_val.{}.{}", ty, block_label, p.name)?;
                        } else {
                            // v0.51.17: Use narrowing-aware formatting
                            writeln!(out, "  ret {} {}", ty, self.format_operand_with_strings_and_narrowing(val, string_table, narrowed_param_names))?;
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
            "read_int" | "abs" | "bmb_abs" | "min" | "max" | "f64_to_i64" => "i64",

            // f64 return - Math intrinsics (v0.34)
            "sqrt" | "i64_to_f64" => "double",

            // i64 return - String operations (both full and wrapper names)
            // v0.46: byte_at added as preferred name (same as interpreter)
            "bmb_string_len" | "bmb_string_char_at" | "bmb_string_eq" | "bmb_ord"
            | "len" | "char_at" | "byte_at" | "ord" => "i64",

            // i64 return - File I/O (both full and wrapper names)
            "bmb_file_exists" | "bmb_file_size" | "bmb_write_file" | "bmb_append_file"
            | "file_exists" | "file_size" | "write_file" | "append_file" => "i64",

            // i64 return - StringBuilder (handle is i64)
            "bmb_sb_new" | "bmb_sb_push" | "bmb_sb_push_cstr" | "bmb_sb_push_char" | "bmb_sb_push_int" | "bmb_sb_push_escaped" | "bmb_sb_len" | "bmb_sb_clear"
            | "sb_new" | "sb_push" | "sb_push_cstr" | "sb_push_char" | "sb_push_int" | "sb_push_escaped" | "sb_len" | "sb_clear" => "i64",

            // i64 return - Process
            "bmb_system" => "i64",

            // ptr return - String operations (both full and wrapper names)
            "bmb_string_new" | "bmb_string_from_cstr" | "bmb_string_slice"
            | "bmb_string_concat" | "bmb_chr"
            | "slice" | "chr" => "ptr",

            // ptr return - File I/O (both full and wrapper names)
            "bmb_read_file" | "read_file" => "ptr",

            // ptr return - StringBuilder (both full and wrapper names)
            "bmb_sb_build" | "sb_build" => "ptr",

            // ptr return - Process
            "bmb_getenv" => "ptr",

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
            MirBinOp::FAdd => ("fadd", true),
            MirBinOp::FSub => ("fsub", true),
            MirBinOp::FMul => ("fmul", true),
            MirBinOp::FDiv => ("fdiv", true),

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
}
