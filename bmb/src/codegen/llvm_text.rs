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

        // Emit string globals
        self.emit_string_globals(&mut output, &string_table)?;

        // Runtime declarations
        self.emit_runtime_declarations(&mut output)?;

        // Generate functions with string table and function type map
        for func in &program.functions {
            self.emit_function_with_strings(&mut output, func, &string_table, &fn_return_types)?;
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
                    if let MirInst::Const { value: Constant::String(s), .. } = inst {
                        if !table.contains_key(s) {
                            table.insert(s.clone(), format!(".str.{}", counter));
                            counter += 1;
                        }
                    }
                    // Check for string constants in call arguments
                    if let MirInst::Call { args, .. } = inst {
                        for arg in args {
                            if let Operand::Constant(Constant::String(s)) = arg {
                                if !table.contains_key(s) {
                                    table.insert(s.clone(), format!(".str.{}", counter));
                                    counter += 1;
                                }
                            }
                        }
                    }
                    // Check for string constants in phi values
                    if let MirInst::Phi { values, .. } = inst {
                        for (val, _label) in values {
                            if let Operand::Constant(Constant::String(s)) = val {
                                if !table.contains_key(s) {
                                    table.insert(s.clone(), format!(".str.{}", counter));
                                    counter += 1;
                                }
                            }
                        }
                    }
                    // Check for string constants in BinOp operands
                    if let MirInst::BinOp { lhs, rhs, .. } = inst {
                        for operand in [lhs, rhs] {
                            if let Operand::Constant(Constant::String(s)) = operand {
                                if !table.contains_key(s) {
                                    table.insert(s.clone(), format!(".str.{}", counter));
                                    counter += 1;
                                }
                            }
                        }
                    }
                }
                // Check for string constants in Return terminator
                if let Terminator::Return(Some(Operand::Constant(Constant::String(s)))) = &block.terminator {
                    if !table.contains_key(s) {
                        table.insert(s.clone(), format!(".str.{}", counter));
                        counter += 1;
                    }
                }
            }
        }

        table
    }

    /// Emit string global constants
    fn emit_string_globals(&self, out: &mut String, table: &HashMap<String, String>) -> TextCodeGenResult<()> {
        if table.is_empty() {
            return Ok(());
        }

        writeln!(out, "; String constants")?;
        for (content, name) in table {
            // Escape the string for LLVM IR
            let escaped = self.escape_string_for_llvm(content);
            let len = content.len() + 1; // +1 for null terminator
            writeln!(out, "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"",
                     name, len, escaped)?;
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
        writeln!(out, "; Runtime declarations - String operations")?;
        writeln!(out, "declare ptr @bmb_string_new(ptr, i64)")?;
        writeln!(out, "declare ptr @bmb_string_from_cstr(ptr)")?;
        writeln!(out, "declare i64 @bmb_string_len(ptr)")?;
        writeln!(out, "declare i64 @bmb_string_char_at(ptr, i64)")?;
        writeln!(out, "declare ptr @bmb_string_slice(ptr, i64, i64)")?;
        writeln!(out, "declare ptr @bmb_string_concat(ptr, ptr)")?;
        writeln!(out, "declare i64 @bmb_string_eq(ptr, ptr)")?;
        writeln!(out, "declare ptr @bmb_chr(i64)")?;
        writeln!(out, "declare i64 @bmb_ord(ptr)")?;
        writeln!(out, "declare void @bmb_print_str(ptr)")?;
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
        writeln!(out, "; Runtime declarations - Method name wrappers")?;
        writeln!(out, "declare i64 @len(ptr)")?;
        writeln!(out, "declare i64 @char_at(ptr, i64)")?;
        writeln!(out, "declare ptr @slice(ptr, i64, i64)")?;
        writeln!(out, "declare ptr @chr(i64)")?;
        writeln!(out, "declare i64 @ord(ptr)")?;
        writeln!(out, "declare void @print_str(ptr)")?;
        writeln!(out)?;

        // File I/O wrappers
        writeln!(out, "declare i64 @file_exists(ptr)")?;
        writeln!(out, "declare i64 @file_size(ptr)")?;
        writeln!(out, "declare ptr @read_file(ptr)")?;
        writeln!(out, "declare i64 @write_file(ptr, ptr)")?;
        writeln!(out, "declare i64 @append_file(ptr, ptr)")?;
        writeln!(out)?;

        // StringBuilder wrappers
        writeln!(out, "declare i64 @sb_new()")?;
        writeln!(out, "declare i64 @sb_push(i64, ptr)")?;
        writeln!(out, "declare i64 @sb_len(i64)")?;
        writeln!(out, "declare ptr @sb_build(i64)")?;
        writeln!(out, "declare i64 @sb_clear(i64)")?;
        writeln!(out)?;

        // v0.34: Math intrinsics for Phase 34.4 Benchmark Gate
        writeln!(out, "; Runtime declarations - Math intrinsics")?;
        writeln!(out, "declare double @llvm.sqrt.f64(double)")?;
        writeln!(out)?;

        // v0.34.2: Memory allocation for Phase 34.2 Dynamic Collections
        writeln!(out, "; Runtime declarations - Memory allocation")?;
        writeln!(out, "declare ptr @malloc(i64)")?;
        writeln!(out, "declare ptr @realloc(ptr, i64)")?;
        writeln!(out, "declare void @free(ptr)")?;
        writeln!(out, "declare ptr @calloc(i64, i64)")?;
        writeln!(out)?;

        Ok(())
    }

    /// Emit a function definition (legacy - without string table)
    #[allow(dead_code)]
    fn emit_function(&self, out: &mut String, func: &MirFunction) -> TextCodeGenResult<()> {
        let empty_str_table = HashMap::new();
        let empty_fn_types = HashMap::new();
        self.emit_function_with_strings(out, func, &empty_str_table, &empty_fn_types)
    }

    /// Build a map of place names to their types by pre-scanning instructions
    fn build_place_type_map(
        &self,
        func: &MirFunction,
        fn_return_types: &HashMap<String, &'static str>,
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
                    MirInst::BinOp { dest, op, lhs, .. } => {
                        // Determine result type based on operator
                        let lhs_ty = match lhs {
                            Operand::Constant(c) => self.constant_type(c),
                            Operand::Place(p) => place_types.get(&p.name).copied().unwrap_or("i64"),
                        };

                        let result_ty = match op {
                            // Comparison operators return i1
                            MirBinOp::Eq | MirBinOp::Ne | MirBinOp::Lt | MirBinOp::Le
                            | MirBinOp::Gt | MirBinOp::Ge => "i1",
                            // String concat returns ptr
                            MirBinOp::Add if lhs_ty == "ptr" => "ptr",
                            // Logical ops preserve operand type
                            MirBinOp::And | MirBinOp::Or => lhs_ty,
                            // Arithmetic ops preserve operand type
                            _ => lhs_ty,
                        };
                        place_types.insert(dest.name.clone(), result_ty);
                    }
                    MirInst::Phi { dest, values } => {
                        // Use type from first value
                        if let Some((val, _)) = values.first() {
                            let ty = match val {
                                Operand::Constant(c) => self.constant_type(c),
                                Operand::Place(p) => place_types.get(&p.name).copied().unwrap_or("i64"),
                            };
                            place_types.insert(dest.name.clone(), ty);
                        }
                    }
                    MirInst::Copy { dest, src } => {
                        // Copy inherits type from source
                        let ty = place_types.get(&src.name).copied().unwrap_or("i64");
                        place_types.insert(dest.name.clone(), ty);
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
    ) -> TextCodeGenResult<()> {
        // Pre-scan to build place type map
        let place_types = self.build_place_type_map(func, fn_return_types);

        // Track defined names to handle SSA violations from MIR
        let mut name_counts: HashMap<String, u32> = HashMap::new();

        // Function signature
        let ret_type = self.mir_type_to_llvm(&func.ret_ty);
        let params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| format!("{} %{}", self.mir_type_to_llvm(ty), name))
            .collect();

        // Mark parameters as defined
        for (name, _) in &func.params {
            name_counts.insert(name.clone(), 1);
        }

        // Function attributes for optimization:
        // - nounwind: BMB doesn't have exceptions, enables better codegen
        // - For main: no special attributes (ABI compatibility)
        // - Attributes go AFTER the parameter list in LLVM IR syntax
        let attrs = if func.name == "main" { "" } else { " nounwind" };

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

        // Build map of (phi_dest_block, local_name, pred_block) -> load_temp_name
        // This is needed because phi nodes must reference SSA values, not memory locations
        // So we emit loads before terminators in predecessor blocks
        let mut phi_load_map: std::collections::HashMap<(String, String, String), String> =
            std::collections::HashMap::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                if let MirInst::Phi { dest: _, values } = inst {
                    for (val, pred_label) in values {
                        if let Operand::Place(p) = val {
                            // Check if this place is a local variable
                            if func.locals.iter().any(|(n, _)| n == &p.name) {
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
                            if !phi_string_map.contains_key(&key) {
                                let temp_name = format!("_str_phi_{}", string_phi_counter);
                                string_phi_counter += 1;
                                phi_string_map.insert(key, temp_name);
                            }
                        }
                    }
                }
            }
        }

        // Collect local variable names for alloca-based handling
        // Using alloca avoids SSA dominance issues when locals are assigned in branches
        // Exclude: void-typed locals (can't allocate)
        let local_names: std::collections::HashSet<String> = func.locals.iter()
            .filter(|(_, ty)| self.mir_type_to_llvm(ty) != "void")
            .map(|(name, _)| name.clone())
            .collect();

        // Emit entry block with allocas for local variables (excluding phi-referenced ones)
        // Use "alloca_entry" to avoid conflicts with user variables named "entry"
        if !local_names.is_empty() {
            writeln!(out, "alloca_entry:")?;
            for (name, ty) in &func.locals {
                if local_names.contains(name) {
                    let llvm_ty = self.mir_type_to_llvm(ty);
                    // Skip void types - they can't be allocated
                    if llvm_ty != "void" {
                        writeln!(out, "  %{}.addr = alloca {}", name, llvm_ty)?;
                    }
                }
            }
            // Jump to the actual first block
            if let Some(first_block) = func.blocks.first() {
                writeln!(out, "  br label %bb_{}", first_block.label)?;
            }
        }

        // Emit basic blocks with place type information
        for block in &func.blocks {
            self.emit_block_with_strings(out, block, func, string_table, fn_return_types, &place_types, &mut name_counts, &local_names, &phi_load_map, &phi_string_map)?;
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
        let empty_place_types = HashMap::new();
        let mut empty_name_counts = HashMap::new();
        let empty_local_names = std::collections::HashSet::new();
        let empty_phi_map = std::collections::HashMap::new();
        let empty_phi_string_map = std::collections::HashMap::new();
        self.emit_block_with_strings(out, block, func, &empty_str_table, &empty_fn_types, &empty_place_types, &mut empty_name_counts, &empty_local_names, &empty_phi_map, &empty_phi_string_map)
    }

    /// Emit a basic block with string table support
    fn emit_block_with_strings(
        &self,
        out: &mut String,
        block: &BasicBlock,
        func: &MirFunction,
        string_table: &HashMap<String, String>,
        fn_return_types: &HashMap<String, &'static str>,
        place_types: &HashMap<String, &'static str>,
        name_counts: &mut HashMap<String, u32>,
        local_names: &std::collections::HashSet<String>,
        phi_load_map: &std::collections::HashMap<(String, String, String), String>,
        phi_string_map: &std::collections::HashMap<(String, String, String), String>,
    ) -> TextCodeGenResult<()> {
        // Use bb_ prefix to avoid collision with variable names
        writeln!(out, "bb_{}:", block.label)?;

        // Emit instructions (pass phi_load_map for phi node handling)
        for inst in &block.instructions {
            self.emit_instruction_with_strings(out, inst, func, string_table, fn_return_types, place_types, name_counts, local_names, phi_load_map, phi_string_map, &block.label)?;
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

        // Emit bmb_string_from_cstr calls for string constants in phi nodes
        // This must happen BEFORE the terminator
        for ((_dest_block, string_val, pred_block), temp_name) in phi_string_map {
            if pred_block == &block.label {
                // Look up the global string constant name
                if let Some(global_name) = string_table.get(string_val) {
                    writeln!(out, "  %{} = call ptr @bmb_string_from_cstr(ptr @{})", temp_name, global_name)?;
                }
            }
        }

        // Emit terminator
        self.emit_terminator(out, &block.terminator, func, string_table, local_names)?;

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
    fn emit_instruction_with_strings(
        &self,
        out: &mut String,
        inst: &MirInst,
        func: &MirFunction,
        string_table: &HashMap<String, String>,
        fn_return_types: &HashMap<String, &'static str>,
        place_types: &HashMap<String, &'static str>,
        name_counts: &mut HashMap<String, u32>,
        local_names: &std::collections::HashSet<String>,
        _phi_load_map: &std::collections::HashMap<(String, String, String), String>,
        phi_string_map: &std::collections::HashMap<(String, String, String), String>,
        current_block_label: &str,
    ) -> TextCodeGenResult<()> {
        match inst {
            MirInst::Const { dest, value } => {
                let ty = self.constant_type(value);
                // Check if destination is a local (uses alloca)
                if local_names.contains(&dest.name) {
                    // For locals, emit a temp then store
                    let temp_name = format!("{}.tmp", dest.name);
                    match value {
                        Constant::Int(n) => {
                            writeln!(out, "  %{} = add {} 0, {}", temp_name, ty, n)?;
                        }
                        Constant::Bool(b) => {
                            let v = if *b { 1 } else { 0 };
                            writeln!(out, "  %{} = add {} 0, {}", temp_name, ty, v)?;
                        }
                        Constant::Float(f) => {
                            writeln!(out, "  %{} = fadd {} 0.0, {}", temp_name, ty, f)?;
                        }
                        Constant::Unit => {
                            writeln!(out, "  %{} = add i8 0, 0", temp_name)?;
                        }
                        Constant::String(s) => {
                            if let Some(global_name) = string_table.get(s) {
                                writeln!(out, "  %{} = call ptr @bmb_string_from_cstr(ptr @{})",
                                         temp_name, global_name)?;
                            } else {
                                writeln!(out, "  ; string constant not in table: {}", s)?;
                                writeln!(out, "  %{} = add ptr null, null", temp_name)?;
                            }
                        }
                    }
                    writeln!(out, "  store {} %{}, ptr %{}.addr", ty, temp_name, dest.name)?;
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
                            writeln!(out, "  %{} = fadd {} 0.0, {}", dest_name, ty, f)?;
                        }
                        Constant::Unit => {
                            // Unit type - just assign 0
                            writeln!(out, "  %{} = add i8 0, 0", dest_name)?;
                        }
                        Constant::String(s) => {
                            // Phase 32.3: String constants are loaded via bmb_string_from_cstr
                            if let Some(global_name) = string_table.get(s) {
                                writeln!(out, "  %{} = call ptr @bmb_string_from_cstr(ptr @{})",
                                         dest_name, global_name)?;
                            } else {
                                // Fallback if string not in table (shouldn't happen)
                                writeln!(out, "  ; string constant not in table: {}", s)?;
                            }
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
                let src_val = if local_names.contains(&src.name) {
                    let load_name = format!("{}.load", src.name);
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
                let lhs_str = match lhs {
                    Operand::Place(p) if local_names.contains(&p.name) => {
                        let load_name = format!("{}.{}.lhs", dest_name, p.name);
                        writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, lhs_ty, p.name)?;
                        format!("%{}", load_name)
                    }
                    _ => self.format_operand_with_strings(lhs, string_table),
                };
                let rhs_str = match rhs {
                    Operand::Place(p) if local_names.contains(&p.name) => {
                        let load_name = format!("{}.{}.rhs", dest_name, p.name, );
                        writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, rhs_ty, p.name)?;
                        format!("%{}", load_name)
                    }
                    _ => self.format_operand_with_strings(rhs, string_table),
                };

                // String concatenation: either operand is ptr with Add op
                if (lhs_ty == "ptr" || rhs_ty == "ptr") && *op == MirBinOp::Add {
                    // Wrap string constant operands with bmb_string_from_cstr
                    let lhs_final = if let Operand::Constant(Constant::String(s)) = lhs {
                        if let Some(global_name) = string_table.get(s) {
                            let wrapper_name = format!("{}.lhs.str", dest_name);
                            writeln!(out, "  %{} = call ptr @bmb_string_from_cstr(ptr @{})", wrapper_name, global_name)?;
                            format!("%{}", wrapper_name)
                        } else { lhs_str.clone() }
                    } else { lhs_str.clone() };
                    let rhs_final = if let Operand::Constant(Constant::String(s)) = rhs {
                        if let Some(global_name) = string_table.get(s) {
                            let wrapper_name = format!("{}.rhs.str", dest_name);
                            writeln!(out, "  %{} = call ptr @bmb_string_from_cstr(ptr @{})", wrapper_name, global_name)?;
                            format!("%{}", wrapper_name)
                        } else { rhs_str.clone() }
                    } else { rhs_str.clone() };
                    // Call bmb_string_concat for string concatenation
                    writeln!(out, "  %{} = call ptr @bmb_string_concat(ptr {}, ptr {})",
                             dest_name, lhs_final, rhs_final)?;
                } else if (lhs_ty == "ptr" || rhs_ty == "ptr") && *op == MirBinOp::Eq {
                    // Wrap string constant operands with bmb_string_from_cstr
                    let lhs_final = if let Operand::Constant(Constant::String(s)) = lhs {
                        if let Some(global_name) = string_table.get(s) {
                            let wrapper_name = format!("{}.lhs.str", dest_name);
                            writeln!(out, "  %{} = call ptr @bmb_string_from_cstr(ptr @{})", wrapper_name, global_name)?;
                            format!("%{}", wrapper_name)
                        } else { lhs_str.clone() }
                    } else { lhs_str.clone() };
                    let rhs_final = if let Operand::Constant(Constant::String(s)) = rhs {
                        if let Some(global_name) = string_table.get(s) {
                            let wrapper_name = format!("{}.rhs.str", dest_name);
                            writeln!(out, "  %{} = call ptr @bmb_string_from_cstr(ptr @{})", wrapper_name, global_name)?;
                            format!("%{}", wrapper_name)
                        } else { rhs_str.clone() }
                    } else { rhs_str.clone() };
                    // Call bmb_string_eq for string equality comparison
                    // bmb_string_eq returns i64 (1 for equal, 0 for not equal)
                    writeln!(out, "  %{}.i64 = call i64 @bmb_string_eq(ptr {}, ptr {})",
                             dest_name, lhs_final, rhs_final)?;
                    // Convert i64 to i1 for boolean result
                    writeln!(out, "  %{} = icmp ne i64 %{}.i64, 0", dest_name, dest_name)?;
                } else if (lhs_ty == "ptr" || rhs_ty == "ptr") && *op == MirBinOp::Ne {
                    // Wrap string constant operands with bmb_string_from_cstr
                    let lhs_final = if let Operand::Constant(Constant::String(s)) = lhs {
                        if let Some(global_name) = string_table.get(s) {
                            let wrapper_name = format!("{}.lhs.str", dest_name);
                            writeln!(out, "  %{} = call ptr @bmb_string_from_cstr(ptr @{})", wrapper_name, global_name)?;
                            format!("%{}", wrapper_name)
                        } else { lhs_str.clone() }
                    } else { lhs_str.clone() };
                    let rhs_final = if let Operand::Constant(Constant::String(s)) = rhs {
                        if let Some(global_name) = string_table.get(s) {
                            let wrapper_name = format!("{}.rhs.str", dest_name);
                            writeln!(out, "  %{} = call ptr @bmb_string_from_cstr(ptr @{})", wrapper_name, global_name)?;
                            format!("%{}", wrapper_name)
                        } else { rhs_str.clone() }
                    } else { rhs_str.clone() };
                    // Call bmb_string_eq and negate for string inequality
                    writeln!(out, "  %{}.i64 = call i64 @bmb_string_eq(ptr {}, ptr {})",
                             dest_name, lhs_final, rhs_final)?;
                    // Convert i64 to i1 and negate (0 means not equal, so i64==0 means Ne is true)
                    writeln!(out, "  %{} = icmp eq i64 %{}.i64, 0", dest_name, dest_name)?;
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
                            MirBinOp::Eq | MirBinOp::FEq => "fcmp oeq",
                            MirBinOp::Ne | MirBinOp::FNe => "fcmp one",
                            MirBinOp::Lt | MirBinOp::FLt => "fcmp olt",
                            MirBinOp::Gt | MirBinOp::FGt => "fcmp ogt",
                            MirBinOp::Le | MirBinOp::FLe => "fcmp ole",
                            MirBinOp::Ge | MirBinOp::FGe => "fcmp oge",
                            MirBinOp::And | MirBinOp::Or => {
                                let (s, _) = self.binop_to_llvm(*op);
                                s
                            }
                        }
                    } else {
                        let (s, _) = self.binop_to_llvm(*op);
                        s
                    };
                    // Note: LLVM IR always uses the operand type in the instruction
                    // The result type (i1 for comparisons) is implicit
                    writeln!(out, "  %{} = {} {} {}, {}", dest_name, op_str, lhs_ty, lhs_str, rhs_str)?;
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
                }
            }

            MirInst::Call { dest, func: fn_name, args } => {
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

                // v0.34.2: box_free_i64(ptr) -> Unit - frees memory (alias for free)
                if fn_name == "box_free_i64" && args.len() == 1 {
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
                // This is complex - generates inline growth logic
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

                    // Get header pointer
                    let header_ptr = format!("vp.header.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 {} to ptr", header_ptr, vec_val)?;

                    // Load current ptr, len, cap
                    let ptr_i64 = format!("vp.ptr.{}", vec_idx);
                    let len_ptr = format!("vp.len.ptr.{}", vec_idx);
                    let len_val = format!("vp.len.{}", vec_idx);
                    let cap_ptr = format!("vp.cap.ptr.{}", vec_idx);
                    let cap_val = format!("vp.cap.{}", vec_idx);

                    writeln!(out, "  %{} = load i64, ptr %{}", ptr_i64, header_ptr)?;
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 1", len_ptr, header_ptr)?;
                    writeln!(out, "  %{} = load i64, ptr %{}", len_val, len_ptr)?;
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 2", cap_ptr, header_ptr)?;
                    writeln!(out, "  %{} = load i64, ptr %{}", cap_val, cap_ptr)?;

                    // Check if need to grow: len >= cap
                    let need_grow = format!("vp.need_grow.{}", vec_idx);
                    writeln!(out, "  %{} = icmp sge i64 %{}, %{}", need_grow, len_val, cap_val)?;

                    // Branch: if need grow, allocate new; else store directly
                    let grow_label = format!("vp.grow.{}", vec_idx);
                    let store_label = format!("vp.store.{}", vec_idx);
                    let cont_label = format!("vp.cont.{}", vec_idx);
                    writeln!(out, "  br i1 %{}, label %{}, label %{}", need_grow, grow_label, store_label)?;

                    // Grow block
                    writeln!(out, "{}:", grow_label)?;
                    // new_cap = cap == 0 ? 4 : cap * 2
                    let is_zero = format!("vp.is_zero.{}", vec_idx);
                    writeln!(out, "  %{} = icmp eq i64 %{}, 0", is_zero, cap_val)?;
                    let doubled = format!("vp.doubled.{}", vec_idx);
                    writeln!(out, "  %{} = mul i64 %{}, 2", doubled, cap_val)?;
                    let new_cap = format!("vp.new_cap.{}", vec_idx);
                    writeln!(out, "  %{} = select i1 %{}, i64 4, i64 %{}", new_cap, is_zero, doubled)?;
                    // new_size = new_cap * 8
                    let new_size = format!("vp.new_size.{}", vec_idx);
                    writeln!(out, "  %{} = mul i64 %{}, 8", new_size, new_cap)?;
                    // Check if ptr == 0 for first alloc vs realloc
                    let ptr_is_null = format!("vp.ptr_null.{}", vec_idx);
                    writeln!(out, "  %{} = icmp eq i64 %{}, 0", ptr_is_null, ptr_i64)?;
                    let alloc_label = format!("vp.alloc.{}", vec_idx);
                    let realloc_label = format!("vp.realloc.{}", vec_idx);
                    let merge_label = format!("vp.merge.{}", vec_idx);
                    writeln!(out, "  br i1 %{}, label %{}, label %{}", ptr_is_null, alloc_label, realloc_label)?;

                    // First allocation
                    writeln!(out, "{}:", alloc_label)?;
                    let new_data_a = format!("vp.new_data.a.{}", vec_idx);
                    writeln!(out, "  %{} = call ptr @malloc(i64 %{})", new_data_a, new_size)?;
                    writeln!(out, "  br label %{}", merge_label)?;

                    // Realloc
                    writeln!(out, "{}:", realloc_label)?;
                    let old_ptr = format!("vp.old_ptr.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 %{} to ptr", old_ptr, ptr_i64)?;
                    let new_data_r = format!("vp.new_data.r.{}", vec_idx);
                    writeln!(out, "  %{} = call ptr @realloc(ptr %{}, i64 %{})", new_data_r, old_ptr, new_size)?;
                    writeln!(out, "  br label %{}", merge_label)?;

                    // Merge block with phi
                    writeln!(out, "{}:", merge_label)?;
                    let new_data = format!("vp.new_data.{}", vec_idx);
                    writeln!(out, "  %{} = phi ptr [ %{}, %{} ], [ %{}, %{} ]", new_data, new_data_a, alloc_label, new_data_r, realloc_label)?;
                    // Update header: ptr and cap
                    let new_ptr_i64 = format!("vp.new_ptr.{}", vec_idx);
                    writeln!(out, "  %{} = ptrtoint ptr %{} to i64", new_ptr_i64, new_data)?;
                    writeln!(out, "  store i64 %{}, ptr %{}", new_ptr_i64, header_ptr)?;
                    writeln!(out, "  store i64 %{}, ptr %{}", new_cap, cap_ptr)?;
                    writeln!(out, "  br label %{}", store_label)?;

                    // Store block
                    writeln!(out, "{}:", store_label)?;
                    // Re-load ptr since it may have changed
                    let cur_ptr = format!("vp.cur_ptr.{}", vec_idx);
                    writeln!(out, "  %{} = load i64, ptr %{}", cur_ptr, header_ptr)?;
                    let data_ptr = format!("vp.data_ptr.{}", vec_idx);
                    writeln!(out, "  %{} = inttoptr i64 %{} to ptr", data_ptr, cur_ptr)?;
                    // Re-load len
                    let cur_len = format!("vp.cur_len.{}", vec_idx);
                    writeln!(out, "  %{} = load i64, ptr %{}", cur_len, len_ptr)?;
                    // Store value at data[len]
                    let slot = format!("vp.slot.{}", vec_idx);
                    writeln!(out, "  %{} = getelementptr i64, ptr %{}, i64 %{}", slot, data_ptr, cur_len)?;
                    writeln!(out, "  store i64 {}, ptr %{}", val_val, slot)?;
                    // Increment len
                    let new_len = format!("vp.new_len.{}", vec_idx);
                    writeln!(out, "  %{} = add i64 %{}, 1", new_len, cur_len)?;
                    writeln!(out, "  store i64 %{}, ptr %{}", new_len, len_ptr)?;
                    writeln!(out, "  br label %{}", cont_label)?;

                    // Continue
                    writeln!(out, "{}:", cont_label)?;
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

                // First check user-defined functions, then fall back to builtins
                let ret_ty = fn_return_types
                    .get(fn_name)
                    .copied()
                    .unwrap_or_else(|| self.infer_call_return_type(fn_name, func));

                // Generate unique base name for this call instruction
                let call_base = dest.as_ref().map(|d| d.name.clone()).unwrap_or_else(|| format!("call_{}", fn_name));

                // Emit loads for local variables used as arguments
                let mut arg_vals: Vec<(String, String)> = Vec::new(); // (type, value)
                for (i, arg) in args.iter().enumerate() {
                    let ty = match arg {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    };

                    let val = match arg {
                        Operand::Place(p) if local_names.contains(&p.name) => {
                            // Emit load from alloca (use call_base for uniqueness)
                            let load_name = format!("{}.{}.arg{}", call_base, p.name, i);
                            writeln!(out, "  %{} = load {}, ptr %{}.addr", load_name, ty, p.name)?;
                            format!("%{}", load_name)
                        }
                        Operand::Constant(Constant::String(s)) => {
                            // String constants need to be wrapped with bmb_string_from_cstr
                            if let Some(global_name) = string_table.get(s) {
                                let wrapper_name = format!("{}.strarg{}", call_base, i);
                                writeln!(out, "  %{} = call ptr @bmb_string_from_cstr(ptr @{})", wrapper_name, global_name)?;
                                format!("%{}", wrapper_name)
                            } else {
                                self.format_operand_with_strings(arg, string_table)
                            }
                        }
                        _ => self.format_operand_with_strings(arg, string_table),
                    };
                    arg_vals.push((ty.to_string(), val));
                }

                let args_str: Vec<String> = arg_vals
                    .iter()
                    .map(|(ty, val)| format!("{} {}", ty, val))
                    .collect();

                if ret_ty == "void" {
                    writeln!(
                        out,
                        "  call {} @{}({})",
                        ret_ty,
                        fn_name,
                        args_str.join(", ")
                    )?;
                } else if let Some(d) = dest {
                    // Check if destination is a local
                    if local_names.contains(&d.name) {
                        let temp_name = format!("{}.call", d.name);
                        writeln!(
                            out,
                            "  %{} = call {} @{}({})",
                            temp_name,
                            ret_ty,
                            fn_name,
                            args_str.join(", ")
                        )?;
                        writeln!(out, "  store {} %{}, ptr %{}.addr", ret_ty, temp_name, d.name)?;
                    } else {
                        let dest_name = self.unique_name(&d.name, name_counts);
                        writeln!(
                            out,
                            "  %{} = call {} @{}({})",
                            dest_name,
                            ret_ty,
                            fn_name,
                            args_str.join(", ")
                        )?;
                    }
                } else {
                    writeln!(
                        out,
                        "  call {} @{}({})",
                        ret_ty,
                        fn_name,
                        args_str.join(", ")
                    )?;
                }
            }

            MirInst::Phi { dest, values } => {
                let dest_name = self.unique_name(&dest.name, name_counts);
                // PHI nodes must come at the start of a basic block
                // %dest = phi type [ val1, %label1 ], [ val2, %label2 ], ...
                let ty = if !values.is_empty() {
                    // Use place_types for accurate type inference
                    match &values[0].0 {
                        Operand::Constant(c) => self.constant_type(c),
                        Operand::Place(p) => place_types.get(&p.name).copied()
                            .unwrap_or_else(|| self.infer_place_type(p, func)),
                    }
                } else {
                    "i64" // Default fallback
                };

                // Find the dest block label by looking at which block contains this phi
                // We need to check phi_load_map for locals that were pre-loaded
                // and phi_string_map for string constants that were wrapped
                let phi_args: Vec<String> = values
                    .iter()
                    .map(|(val, label)| {
                        // Check if this is a local variable that was pre-loaded for phi
                        let val_str = if let Operand::Place(p) = val {
                            if local_names.contains(&p.name) {
                                // This local should have been pre-loaded in the predecessor block
                                // The load temp name follows the pattern: {local}.phi.{pred_label}
                                let load_temp = format!("{}.phi.{}", p.name, label);
                                format!("%{}", load_temp)
                            } else {
                                self.format_operand_with_strings(val, string_table)
                            }
                        } else if let Operand::Constant(Constant::String(s)) = val {
                            // Check if this string constant was pre-wrapped for phi
                            let key = (current_block_label.to_string(), s.clone(), label.clone());
                            if let Some(temp_name) = phi_string_map.get(&key) {
                                format!("%{}", temp_name)
                            } else {
                                self.format_operand_with_strings(val, string_table)
                            }
                        } else {
                            self.format_operand_with_strings(val, string_table)
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
            MirInst::StructInit { dest, struct_name, fields } => {
                // In LLVM, we allocate space for the struct and store each field
                // For now, treat struct as a pointer (i64) and use insertvalue
                writeln!(out, "  ; struct {} init with {} fields", struct_name, fields.len())?;
                // Create zeroinitializer and insertvalue for each field
                writeln!(out, "  %{} = alloca i64, i32 {}", dest.name, fields.len().max(1))?;
                for (i, (field_name, value)) in fields.iter().enumerate() {
                    let val_str = self.format_operand(value);
                    writeln!(out, "  ; field {} = {}", field_name, val_str)?;
                    let ty = self.infer_operand_type(value, func);
                    writeln!(out, "  %{}_f{} = getelementptr i64, ptr %{}, i32 {}",
                             dest.name, i, dest.name, i)?;
                    writeln!(out, "  store {} {}, ptr %{}_f{}", ty, val_str, dest.name, i)?;
                }
            }

            MirInst::FieldAccess { dest, base, field } => {
                // Load field from struct pointer
                writeln!(out, "  ; field access .{} from %{}", field, base.name)?;
                // For now, just load from base (simplified - needs field offset calculation)
                writeln!(out, "  %{} = load i64, ptr %{}", dest.name, base.name)?;
            }

            MirInst::FieldStore { base, field, value } => {
                // Store value to field in struct pointer
                let val_str = self.format_operand(value);
                let ty = self.infer_operand_type(value, func);
                writeln!(out, "  ; field store .{} = {}", field, val_str)?;
                writeln!(out, "  store {} {}, ptr %{}", ty, val_str, base.name)?;
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
            MirInst::ArrayInit { dest, element_type, elements } => {
                let elem_ty = self.mir_type_to_llvm(element_type);
                let size = elements.len();
                writeln!(out, "  ; array init with {} elements of type {}", size, elem_ty)?;
                writeln!(out, "  %{} = alloca {}, i32 {}", dest.name, elem_ty, size.max(1))?;
                for (i, elem) in elements.iter().enumerate() {
                    let elem_str = self.format_operand(elem);
                    writeln!(out, "  %{}_e{} = getelementptr {}, ptr %{}, i32 {}",
                             dest.name, i, elem_ty, dest.name, i)?;
                    writeln!(out, "  store {} {}, ptr %{}_e{}", elem_ty, elem_str, dest.name, i)?;
                }
            }

            MirInst::IndexLoad { dest, array, index } => {
                let idx_str = self.format_operand(index);
                writeln!(out, "  ; index load %{}[{}]", array.name, idx_str)?;
                writeln!(out, "  %{}_ptr = getelementptr i64, ptr %{}, i64 {}",
                         dest.name, array.name, idx_str)?;
                writeln!(out, "  %{} = load i64, ptr %{}_ptr", dest.name, dest.name)?;
            }

            MirInst::IndexStore { array, index, value } => {
                let idx_str = self.format_operand(index);
                let val_str = self.format_operand(value);
                let ty = self.infer_operand_type(value, func);
                writeln!(out, "  ; index store %{}[{}] = {}", array.name, idx_str, val_str)?;
                writeln!(out, "  %{}_idx_ptr = getelementptr {}, ptr %{}, i64 {}",
                         array.name, ty, array.name, idx_str)?;
                writeln!(out, "  store {} {}, ptr %{}_idx_ptr", ty, val_str, array.name)?;
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
                let ty = self.mir_type_to_llvm(&func.ret_ty);
                // Special handling for string constant returns
                if let Operand::Constant(Constant::String(s)) = val {
                    if let Some(global_name) = string_table.get(s) {
                        writeln!(out, "  %_ret_str = call ptr @bmb_string_from_cstr(ptr @{})", global_name)?;
                        writeln!(out, "  ret ptr %_ret_str")?;
                    } else {
                        // Fallback - shouldn't happen
                        writeln!(out, "  ret {} {}", ty, self.format_operand_with_strings(val, string_table))?;
                    }
                } else if let Operand::Place(p) = val {
                    // Check if this is a local that uses alloca
                    if local_names.contains(&p.name) {
                        // Load from alloca before returning
                        writeln!(out, "  %_ret_val = load {}, ptr %{}.addr", ty, p.name)?;
                        writeln!(out, "  ret {} %_ret_val", ty)?;
                    } else {
                        writeln!(out, "  ret {} {}", ty, self.format_operand_with_strings(val, string_table))?;
                    }
                } else {
                    writeln!(out, "  ret {} {}", ty, self.format_operand_with_strings(val, string_table))?;
                }
            }

            Terminator::Goto(label) => {
                writeln!(out, "  br label %bb_{}", label)?;
            }

            Terminator::Branch { cond, then_label, else_label } => {
                // Check if condition is a local that needs loading from alloca
                let cond_str = if let Operand::Place(p) = cond {
                    if local_names.contains(&p.name) {
                        // Load the condition from alloca first (use then_label to make name unique)
                        writeln!(out, "  %{}.cond_{} = load i1, ptr %{}.addr", p.name, then_label, p.name)?;
                        format!("%{}.cond_{}", p.name, then_label)
                    } else {
                        self.format_operand(cond)
                    }
                } else {
                    self.format_operand(cond)
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
                let disc_str = if let Operand::Place(p) = discriminant {
                    if local_names.contains(&p.name) {
                        // Use default label to make name unique
                        writeln!(out, "  %{}.disc_{} = load i64, ptr %{}.addr", p.name, default, p.name)?;
                        format!("%{}.disc_{}", p.name, default)
                    } else {
                        self.format_operand(discriminant)
                    }
                } else {
                    self.format_operand(discriminant)
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
        }
    }

    /// Get LLVM type for a constant
    fn constant_type(&self, c: &Constant) -> &'static str {
        match c {
            Constant::Int(_) => "i64",
            Constant::Float(_) => "double",
            Constant::Bool(_) => "i1",
            Constant::String(_) => "ptr",
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

    /// Format an operand with potential load for local variables
    /// Returns (needs_load_prefix, formatted_value)
    /// If needs_load_prefix is Some, caller should emit a load instruction first
    fn format_operand_with_locals(
        &self,
        op: &Operand,
        string_table: &HashMap<String, String>,
        local_names: &std::collections::HashSet<String>,
        load_counter: &mut u32,
    ) -> (Option<(String, String, &'static str)>, String) {
        match op {
            Operand::Place(p) => {
                if local_names.contains(&p.name) {
                    // Need to emit load from alloca
                    *load_counter += 1;
                    let load_name = format!("{}.ld{}", p.name, *load_counter);
                    // Return info for caller to emit load
                    (Some((load_name.clone(), p.name.clone(), "ptr")), format!("%{}", load_name))
                } else {
                    (None, format!("%{}", p.name))
                }
            }
            Operand::Constant(c) => match c {
                Constant::String(s) => {
                    if let Some(global_name) = string_table.get(s) {
                        (None, format!("@{}", global_name))
                    } else {
                        (None, format!("\"{}\"", s))
                    }
                }
                _ => (None, self.format_constant(c)),
            },
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
            "bmb_string_len" | "bmb_string_char_at" | "bmb_string_eq" | "bmb_ord"
            | "len" | "char_at" | "ord" => "i64",

            // i64 return - File I/O (both full and wrapper names)
            "bmb_file_exists" | "bmb_file_size" | "bmb_write_file" | "bmb_append_file"
            | "file_exists" | "file_size" | "write_file" | "append_file" => "i64",

            // i64 return - StringBuilder (handle is i64)
            "bmb_sb_new" | "bmb_sb_push" | "bmb_sb_len" | "bmb_sb_clear"
            | "sb_new" | "sb_push" | "sb_len" | "sb_clear" => "i64",

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
            }],
            extern_fns: vec![],
        };

        let codegen = TextCodeGen::new();
        let ir = codegen.generate(&program).unwrap();

        assert!(ir.contains("define i64 @add(i64 %a, i64 %b)"));
        assert!(ir.contains("%_t0 = add nsw i64 %a, %b"));  // nsw for optimization
        assert!(ir.contains("ret i64 %_t0"));
    }
}
