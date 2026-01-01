//! LLVM Code Generation using inkwell
//!
//! This module generates LLVM IR from MIR and compiles to object files.

use std::collections::HashMap;
use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::OptimizationLevel;
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};
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
        let module = context.create_module("bmb_program");
        let builder = context.create_builder();

        let mut ctx = LlvmContext::new(&context, &module, &builder);

        // Declare built-in functions
        ctx.declare_builtins();

        // Generate code for all functions
        for func in &program.functions {
            ctx.gen_function(func)?;
        }

        // Write to object file
        self.write_object_file(&module, output)
    }

    /// Generate LLVM IR as string
    pub fn generate_ir(&self, program: &MirProgram) -> CodeGenResult<String> {
        let context = Context::create();
        let module = context.create_module("bmb_program");
        let builder = context.create_builder();

        let mut ctx = LlvmContext::new(&context, &module, &builder);

        // Declare built-in functions
        ctx.declare_builtins();

        // Generate code for all functions
        for func in &program.functions {
            ctx.gen_function(func)?;
        }

        Ok(module.print_to_string().to_string())
    }

    /// Write module to object file
    fn write_object_file(&self, module: &Module, output: &Path) -> CodeGenResult<()> {
        // Initialize all targets
        Target::initialize_all(&InitializationConfig::default());

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                self.opt_level.into(),
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or(CodeGenError::TargetMachineError)?;

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
    module: &'ctx Module<'ctx>,
    builder: &'ctx Builder<'ctx>,

    /// Function lookup table
    functions: HashMap<String, FunctionValue<'ctx>>,

    /// Variable lookup table (local to current function)
    variables: HashMap<String, PointerValue<'ctx>>,

    /// Block lookup table (local to current function)
    blocks: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>>,
}

impl<'ctx> LlvmContext<'ctx> {
    fn new(
        context: &'ctx Context,
        module: &'ctx Module<'ctx>,
        builder: &'ctx Builder<'ctx>,
    ) -> Self {
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
    }

    /// Convert MIR type to LLVM type
    fn mir_type_to_llvm(&self, ty: &MirType) -> BasicTypeEnum<'ctx> {
        match ty {
            MirType::I32 => self.context.i32_type().into(),
            MirType::I64 => self.context.i64_type().into(),
            MirType::F64 => self.context.f64_type().into(),
            MirType::Bool => self.context.bool_type().into(),
            MirType::Unit => self.context.i8_type().into(), // Unit represented as i8
        }
    }

    /// Generate code for a function
    fn gen_function(&mut self, func: &MirFunction) -> CodeGenResult<()> {
        // Clear per-function state
        self.variables.clear();
        self.blocks.clear();

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

        // Create function
        let function = self.module.add_function(&func.name, fn_type, None);
        self.functions.insert(func.name.clone(), function);

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
            self.variables.insert(name.clone(), alloca);
        }

        // Allocate locals
        for (name, ty) in &func.locals {
            let llvm_ty = self.mir_type_to_llvm(ty);
            let alloca = self.builder.build_alloca(llvm_ty, name)
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            self.variables.insert(name.clone(), alloca);
        }

        // Generate code for each block
        for block in &func.blocks {
            self.gen_basic_block(block, function)?;
        }

        Ok(())
    }

    /// Generate code for a basic block
    fn gen_basic_block(
        &mut self,
        block: &BasicBlock,
        _function: FunctionValue<'ctx>,
    ) -> CodeGenResult<()> {
        let bb = self.blocks.get(&block.label).unwrap();
        self.builder.position_at_end(*bb);

        // Generate instructions
        for inst in &block.instructions {
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
                    if let Some(ret_val) = call_result.try_as_basic_value().left() {
                        self.store_to_place(dest_place, ret_val)?;
                    }
                }
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
        }

        Ok(())
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
            Constant::Unit => self.context.i8_type().const_int(0, false).into(),
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
        let ptr = self
            .variables
            .get(&place.name)
            .ok_or_else(|| CodeGenError::UnknownVariable(place.name.clone()))?;

        // Get the pointee type from the alloca
        let pointee_type = ptr.get_type().get_element_type();

        self.builder
            .build_load(pointee_type.try_into().unwrap(), *ptr, &place.name)
            .map_err(|e| CodeGenError::LlvmError(e.to_string()))
    }

    /// Store a value to a place
    fn store_to_place(&mut self, place: &Place, value: BasicValueEnum<'ctx>) -> CodeGenResult<()> {
        // Get or create the variable
        let ptr = if let Some(ptr) = self.variables.get(&place.name) {
            *ptr
        } else {
            // Create a new alloca for temporaries
            let alloca = self.builder.build_alloca(value.get_type(), &place.name)
                .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
            self.variables.insert(place.name.clone(), alloca);
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
            // Integer arithmetic
            MirBinOp::Add => {
                let result = self.builder
                    .build_int_add(lhs.into_int_value(), rhs.into_int_value(), "add")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Sub => {
                let result = self.builder
                    .build_int_sub(lhs.into_int_value(), rhs.into_int_value(), "sub")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Mul => {
                let result = self.builder
                    .build_int_mul(lhs.into_int_value(), rhs.into_int_value(), "mul")
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

            // Integer comparison
            MirBinOp::Eq => {
                let result = self.builder
                    .build_int_compare(IntPredicate::EQ, lhs.into_int_value(), rhs.into_int_value(), "eq")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
            }
            MirBinOp::Ne => {
                let result = self.builder
                    .build_int_compare(IntPredicate::NE, lhs.into_int_value(), rhs.into_int_value(), "ne")
                    .map_err(|e| CodeGenError::LlvmError(e.to_string()))?;
                Ok(result.into())
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
        }
    }
}
