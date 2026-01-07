//! Text-based LLVM IR Generation
//!
//! This module generates LLVM IR as text (.ll files) that can be compiled
//! with clang or llc. It doesn't require the LLVM C API, making it more
//! portable and easier to debug.
//!
//! The generated IR is compatible with the bootstrap compiler output.

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

        // Runtime declarations
        self.emit_runtime_declarations(&mut output)?;

        // Generate functions
        for func in &program.functions {
            self.emit_function(&mut output, func)?;
        }

        Ok(output)
    }

    /// Emit runtime function declarations
    fn emit_runtime_declarations(&self, out: &mut String) -> TextCodeGenResult<()> {
        writeln!(out, "; Runtime declarations")?;
        writeln!(out, "declare void @println(i64)")?;
        writeln!(out, "declare void @print(i64)")?;
        writeln!(out, "declare i64 @read_int()")?;
        writeln!(out, "declare void @assert(i1)")?;
        writeln!(out, "declare i64 @bmb_abs(i64)")?;  // bmb_ prefix to avoid stdlib conflict
        writeln!(out, "declare i64 @min(i64, i64)")?;
        writeln!(out, "declare i64 @max(i64, i64)")?;
        writeln!(out)?;
        Ok(())
    }

    /// Emit a function definition
    fn emit_function(&self, out: &mut String, func: &MirFunction) -> TextCodeGenResult<()> {
        // Function signature
        let ret_type = self.mir_type_to_llvm(&func.ret_ty);
        let params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| format!("{} %{}", self.mir_type_to_llvm(ty), name))
            .collect();

        // Function attributes for optimization:
        // - nounwind: BMB doesn't have exceptions, enables better codegen
        // - For main: no special attributes (ABI compatibility)
        // - Attributes go AFTER the parameter list in LLVM IR syntax
        let attrs = if func.name == "main" { "" } else { " nounwind" };

        writeln!(
            out,
            "define {} @{}({}){} {{",
            ret_type,
            func.name,
            params.join(", "),
            attrs
        )?;

        // Emit basic blocks
        for block in &func.blocks {
            self.emit_block(out, block, func)?;
        }

        writeln!(out, "}}")?;
        writeln!(out)?;

        Ok(())
    }

    /// Emit a basic block
    fn emit_block(
        &self,
        out: &mut String,
        block: &BasicBlock,
        func: &MirFunction,
    ) -> TextCodeGenResult<()> {
        writeln!(out, "{}:", block.label)?;

        // Emit instructions
        for inst in &block.instructions {
            self.emit_instruction(out, inst, func)?;
        }

        // Emit terminator
        self.emit_terminator(out, &block.terminator, func)?;

        Ok(())
    }

    /// Emit an instruction
    fn emit_instruction(
        &self,
        out: &mut String,
        inst: &MirInst,
        func: &MirFunction,
    ) -> TextCodeGenResult<()> {
        match inst {
            MirInst::Const { dest, value } => {
                let ty = self.constant_type(value);
                // Use add with 0 for integer constants (LLVM IR idiom)
                match value {
                    Constant::Int(n) => {
                        writeln!(out, "  %{} = add {} 0, {}", dest.name, ty, n)?;
                    }
                    Constant::Bool(b) => {
                        let v = if *b { 1 } else { 0 };
                        writeln!(out, "  %{} = add {} 0, {}", dest.name, ty, v)?;
                    }
                    Constant::Float(f) => {
                        writeln!(out, "  %{} = fadd {} 0.0, {}", dest.name, ty, f)?;
                    }
                    Constant::Unit => {
                        // Unit type - just assign 0
                        writeln!(out, "  %{} = add i8 0, 0", dest.name)?;
                    }
                    Constant::String(_) => {
                        // String constants require global declarations - simplified for now
                        writeln!(out, "  ; string constant not supported in bootstrap")?;
                    }
                }
            }

            MirInst::Copy { dest, src } => {
                let ty = self.infer_place_type(src, func);
                writeln!(out, "  %{} = add {} %{}, 0", dest.name, ty, src.name)?;
            }

            MirInst::BinOp { dest, op, lhs, rhs } => {
                let ty = self.infer_operand_type(lhs, func);
                let lhs_str = self.format_operand(lhs);
                let rhs_str = self.format_operand(rhs);

                let (op_str, _preserves_type) = self.binop_to_llvm(*op);
                // Note: LLVM IR always uses the operand type in the instruction
                // The result type (i1 for comparisons) is implicit
                writeln!(out, "  %{} = {} {} {}, {}", dest.name, op_str, ty, lhs_str, rhs_str)?;
            }

            MirInst::UnaryOp { dest, op, src } => {
                let ty = self.infer_operand_type(src, func);
                let src_str = self.format_operand(src);

                match op {
                    MirUnaryOp::Neg => {
                        writeln!(out, "  %{} = sub {} 0, {}", dest.name, ty, src_str)?;
                    }
                    MirUnaryOp::FNeg => {
                        writeln!(out, "  %{} = fsub {} 0.0, {}", dest.name, ty, src_str)?;
                    }
                    MirUnaryOp::Not => {
                        writeln!(out, "  %{} = xor i1 {}, 1", dest.name, src_str)?;
                    }
                }
            }

            MirInst::Call { dest, func: fn_name, args } => {
                let ret_ty = self.infer_call_return_type(fn_name, func);
                let args_str: Vec<String> = args
                    .iter()
                    .map(|arg| {
                        let ty = self.infer_operand_type(arg, func);
                        format!("{} {}", ty, self.format_operand(arg))
                    })
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
                    writeln!(
                        out,
                        "  %{} = call {} @{}({})",
                        d.name,
                        ret_ty,
                        fn_name,
                        args_str.join(", ")
                    )?;
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
                // PHI nodes must come at the start of a basic block
                // %dest = phi type [ val1, %label1 ], [ val2, %label2 ], ...
                let ty = if !values.is_empty() {
                    self.infer_operand_type(&values[0].0, func)
                } else {
                    "i64" // Default fallback
                };

                let phi_args: Vec<String> = values
                    .iter()
                    .map(|(val, label)| {
                        format!("[ {}, %{} ]", self.format_operand(val), label)
                    })
                    .collect();

                writeln!(
                    out,
                    "  %{} = phi {} {}",
                    dest.name,
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
                writeln!(out, "  ret {} {}", ty, self.format_operand(val))?;
            }

            Terminator::Goto(label) => {
                writeln!(out, "  br label %{}", label)?;
            }

            Terminator::Branch { cond, then_label, else_label } => {
                let cond_str = self.format_operand(cond);
                writeln!(
                    out,
                    "  br i1 {}, label %{}, label %{}",
                    cond_str, then_label, else_label
                )?;
            }

            Terminator::Unreachable => {
                writeln!(out, "  unreachable")?;
            }

            // v0.19.2: Switch for pattern matching
            Terminator::Switch { discriminant, cases, default } => {
                let disc_str = self.format_operand(discriminant);
                writeln!(out, "  switch i64 {}, label %{} [", disc_str, default)?;
                for (val, label) in cases {
                    writeln!(out, "    i64 {}, label %{}", val, label)?;
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
            Constant::Float(f) => format!("{:e}", f),
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
            "println" | "print" | "assert" => "void",
            "read_int" | "abs" | "min" | "max" => "i64",
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
