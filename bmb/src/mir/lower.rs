//! AST to MIR lowering
//!
//! This module converts the high-level AST into MIR by:
//! - Flattening nested expressions into sequences of instructions
//! - Making control flow explicit through basic blocks
//! - Converting operators based on operand types

use crate::ast::{Attribute, BinOp, Expr, FnDef, Item, LiteralPattern, MatchArm, Pattern, Program, Spanned, Type, UnOp};

use super::{
    CmpOp, Constant, ContractFact, LoweringContext, MirBinOp, MirExternFn, MirFunction, MirInst,
    MirProgram, MirType, MirUnaryOp, Operand, Place, Terminator,
};

/// v0.60.254: Type definitions for struct and enum name resolution
/// v0.60.261: Added type_params for generic struct monomorphization
struct TypeDefs {
    structs: std::collections::HashMap<String, Vec<(String, Type)>>,
    enums: std::collections::HashMap<String, Vec<(String, Vec<Type>)>>,
    /// v0.60.261: Type parameters for generic structs (struct name -> param names)
    struct_type_params: std::collections::HashMap<String, Vec<String>>,
}

/// Lower an entire program to MIR
pub fn lower_program(program: &Program) -> MirProgram {
    // v0.51.24: Collect struct type definitions FIRST (full type info, not just field names)
    // This is needed to properly convert Type::Named to MirType::Struct
    // v0.60.261: Also collect type parameters for generic struct monomorphization
    let mut struct_type_defs: std::collections::HashMap<String, Vec<(String, Type)>> = std::collections::HashMap::new();
    let mut struct_type_params: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for item in &program.items {
        if let Item::StructDef(struct_def) = item {
            let fields: Vec<(String, Type)> = struct_def.fields
                .iter()
                .map(|f| (f.name.node.clone(), f.ty.node.clone()))
                .collect();
            struct_type_defs.insert(struct_def.name.node.clone(), fields);
            // v0.60.261: Store type parameter names for generic structs
            if !struct_def.type_params.is_empty() {
                let param_names: Vec<String> = struct_def.type_params
                    .iter()
                    .map(|tp| tp.name.clone())
                    .collect();
                struct_type_params.insert(struct_def.name.node.clone(), param_names);
            }
        }
    }

    // v0.60.254: Collect enum type definitions for proper Type::Named resolution
    let mut enum_type_defs: std::collections::HashMap<String, Vec<(String, Vec<Type>)>> = std::collections::HashMap::new();
    for item in &program.items {
        if let Item::EnumDef(enum_def) = item {
            let variants: Vec<(String, Vec<Type>)> = enum_def.variants
                .iter()
                .map(|v| (v.name.node.clone(), v.fields.iter().map(|f| f.node.clone()).collect()))
                .collect();
            enum_type_defs.insert(enum_def.name.node.clone(), variants);
        }
    }

    let type_defs = TypeDefs {
        structs: struct_type_defs.clone(),
        enums: enum_type_defs,
        struct_type_params,
    };

    // v0.35.4: First pass - collect all function return types
    // v0.51.24: Use struct_type_defs to properly resolve named types
    // v0.60.254: Now also resolves enum types via type_defs
    let mut func_return_types = std::collections::HashMap::new();
    for item in &program.items {
        if let Item::FnDef(fn_def) = item {
            let ret_ty = ast_type_to_mir_with_type_defs(&fn_def.ret_ty.node, &type_defs);
            func_return_types.insert(fn_def.name.node.clone(), ret_ty);
        }
    }

    // v0.51.23: Collect struct definitions for field index lookup (field names only)
    let mut struct_defs = std::collections::HashMap::new();
    for item in &program.items {
        if let Item::StructDef(struct_def) = item {
            let field_names: Vec<String> = struct_def.fields
                .iter()
                .map(|f| f.name.node.clone())
                .collect();
            struct_defs.insert(struct_def.name.node.clone(), field_names);
        }
    }

    let functions = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::FnDef(fn_def) => Some(lower_function(fn_def, &func_return_types, &struct_defs, &type_defs)),
            // Type definitions, use statements, extern fns, traits, impl blocks, and type aliases don't produce MIR functions
            Item::StructDef(_) | Item::EnumDef(_) | Item::Use(_) | Item::ExternFn(_) |
            Item::TraitDef(_) | Item::ImplBlock(_) | Item::TypeAlias(_) => None,
        })
        .collect();

    // Collect extern function declarations (v0.13.0)
    let extern_fns = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::ExternFn(e) => Some(lower_extern_fn(e)),
            _ => None,
        })
        .collect();

    // v0.51.31: Convert struct_type_defs to MirType format for codegen
    // v0.60.254: Now uses type_defs for proper enum resolution
    let mir_struct_defs: std::collections::HashMap<String, Vec<(String, MirType)>> = type_defs.structs
        .iter()
        .map(|(name, fields)| {
            let mir_fields: Vec<(String, MirType)> = fields
                .iter()
                .map(|(field_name, field_ty)| {
                    (field_name.clone(), ast_type_to_mir_with_type_defs(field_ty, &type_defs))
                })
                .collect();
            (name.clone(), mir_fields)
        })
        .collect();

    MirProgram {
        functions,
        extern_fns,
        struct_defs: mir_struct_defs,
    }
}

/// Lower an extern function declaration to MIR (v0.13.0)
fn lower_extern_fn(extern_fn: &crate::ast::ExternFn) -> MirExternFn {
    // Extract module name from @link attribute or use default
    let module = extern_fn
        .link_name
        .clone()
        .unwrap_or_else(|| extract_module_from_attrs(&extern_fn.attributes));

    let params = extern_fn
        .params
        .iter()
        .map(|p| ast_type_to_mir(&p.ty.node))
        .collect();

    let ret_ty = ast_type_to_mir(&extern_fn.ret_ty.node);

    MirExternFn {
        module,
        name: extern_fn.name.node.clone(),
        params,
        ret_ty,
    }
}

/// Extract module name from attributes (v0.13.0)
/// Checks for @wasi, @libc, etc. to determine module name
fn extract_module_from_attrs(attrs: &[Attribute]) -> String {
    for attr in attrs {
        match attr.name() {
            "wasi" => return "wasi_snapshot_preview1".to_string(),
            "libc" => return "env".to_string(),
            _ => {}
        }
    }
    // Default module name
    "env".to_string()
}

/// Lower a function definition to MIR
fn lower_function(
    fn_def: &FnDef,
    func_return_types: &std::collections::HashMap<String, MirType>,
    struct_defs: &std::collections::HashMap<String, Vec<String>>,
    type_defs: &TypeDefs,
) -> MirFunction {
    let mut ctx = LoweringContext::new();

    // v0.35.4: Add user-defined function return types to context
    for (name, ty) in func_return_types {
        ctx.func_return_types.insert(name.clone(), ty.clone());
    }

    // v0.51.23: Add struct definitions for field index lookup
    for (name, fields) in struct_defs {
        ctx.struct_defs.insert(name.clone(), fields.clone());
    }

    // v0.51.31: Add struct type definitions for field type lookup
    // v0.60.254: Now uses type_defs for proper enum resolution
    for (name, fields) in &type_defs.structs {
        let mir_fields: Vec<(String, MirType)> = fields
            .iter()
            .map(|(field_name, field_ty)| {
                (field_name.clone(), ast_type_to_mir_with_type_defs(field_ty, type_defs))
            })
            .collect();
        ctx.struct_type_defs.insert(name.clone(), mir_fields);
    }

    // Register parameters
    // v0.51.24: Use ast_type_to_mir_with_type_defs to properly resolve named struct/enum types
    // v0.51.36: Also track array element types for struct arrays
    // v0.60.261: Also track generic struct types (e.g., Pair<i64, i64>)
    let params: Vec<(String, MirType)> = fn_def
        .params
        .iter()
        .map(|p| {
            let ty = ast_type_to_mir_with_type_defs(&p.ty.node, type_defs);
            ctx.params.insert(p.name.node.clone(), ty.clone());
            // v0.51.23: Track struct type for parameters
            // v0.51.37: Also handle pointer types (*Node)
            // v0.60.261: Also handle generic struct types
            if let Type::Named(struct_name) = &p.ty.node {
                if ctx.struct_defs.contains_key(struct_name) {
                    ctx.var_struct_types.insert(p.name.node.clone(), struct_name.clone());
                }
            } else if let Type::Generic { name: struct_name, .. } = &p.ty.node {
                // v0.60.261: For generic structs, use base struct name for field index lookup
                // The generic struct has the same fields as the base definition
                if ctx.struct_defs.contains_key(struct_name) {
                    ctx.var_struct_types.insert(p.name.node.clone(), struct_name.clone());
                }
            } else if let Type::Ptr(inner) = &p.ty.node {
                if let Type::Named(struct_name) = inner.as_ref() {
                    if ctx.struct_defs.contains_key(struct_name) {
                        ctx.var_struct_types.insert(p.name.node.clone(), struct_name.clone());
                    }
                }
            }
            // v0.51.36: Track array element types for struct array parameters
            if let Type::Array(elem_ty, _) = &p.ty.node {
                let elem_mir_ty = ast_type_to_mir_with_type_defs(elem_ty.as_ref(), type_defs);
                ctx.array_element_types.insert(p.name.node.clone(), elem_mir_ty);
            }
            // v0.60.20: Track pointer element types for ptr[i] indexing
            if let Type::Ptr(elem_ty) = &p.ty.node {
                let elem_mir_ty = ast_type_to_mir_with_type_defs(elem_ty.as_ref(), type_defs);
                ctx.array_element_types.insert(p.name.node.clone(), elem_mir_ty);
            }
            (p.name.node.clone(), ty)
        })
        .collect();

    let ret_ty = ast_type_to_mir_with_type_defs(&fn_def.ret_ty.node, type_defs);

    // Lower the function body
    let result = lower_expr(&fn_def.body, &mut ctx);

    // Finish with a return
    ctx.finish_block(Terminator::Return(Some(result)));

    // Collect locals (including temp_types for tuple variables)
    // v0.60.3: temp_types contains tuple type info that wasn't in locals
    // This is needed for PHI type inference in LLVM codegen
    let mut locals: Vec<(String, MirType)> = ctx.locals.clone().into_iter().collect();
    for (name, ty) in ctx.temp_types.iter() {
        if !ctx.locals.contains_key(name) {
            locals.push((name.clone(), ty.clone()));
        }
    }

    // v0.38: Extract contract facts for optimization
    let preconditions = extract_contract_facts(fn_def.pre.as_ref());
    let postconditions = extract_contract_facts(fn_def.post.as_ref());

    // v0.38.3: Extract @pure and @const attributes
    let is_pure = has_attribute(&fn_def.attributes, "pure");
    let is_const = has_attribute(&fn_def.attributes, "const");
    // v0.59: Support explicit @alwaysinline attribute to bypass size heuristics
    let explicit_always_inline = has_attribute(&fn_def.attributes, "alwaysinline");
    // v0.59: Support @inline attribute as an alias for @alwaysinline
    let explicit_inline = has_attribute(&fn_def.attributes, "inline");

    MirFunction {
        name: fn_def.name.node.clone(),
        params,
        ret_ty,
        locals,
        blocks: ctx.blocks,
        preconditions,
        postconditions,
        is_pure,
        is_const,
        always_inline: explicit_always_inline || explicit_inline, // v0.59: Can be set by source attribute OR AggressiveInlining pass
        inline_hint: false, // v0.51.52: Set by AggressiveInlining pass for medium-sized functions
        is_memory_free: is_pure, // v0.69: @pure implies memory(none) for LLVM optimization
    }
}

/// v0.38.3: Check if a function has a specific attribute
fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.name() == name)
}

/// v0.38: Extract contract facts from a pre/post condition expression
/// Converts AST expressions like `x >= 0 && y < len` into ContractFact list
fn extract_contract_facts(expr: Option<&Spanned<Expr>>) -> Vec<ContractFact> {
    let mut facts = Vec::new();
    if let Some(e) = expr {
        extract_facts_from_expr(&e.node, &mut facts);
    }
    facts
}

/// Recursively extract facts from an expression
fn extract_facts_from_expr(expr: &Expr, facts: &mut Vec<ContractFact>) {
    match expr {
        // Handle && (conjunction of facts)
        Expr::Binary { op, left, right } if *op == BinOp::And => {
            extract_facts_from_expr(&left.node, facts);
            extract_facts_from_expr(&right.node, facts);
        }
        // Handle comparison operators: x >= 0, x < len, etc.
        Expr::Binary { op, left, right } => {
            if let Some(cmp_op) = binop_to_cmp_op(op) {
                // Pattern: var op constant
                if let (Expr::Var(var), Expr::IntLit(val)) = (&left.node, &right.node) {
                    facts.push(ContractFact::VarCmp {
                        var: var.clone(),
                        op: cmp_op,
                        value: *val,
                    });
                }
                // Pattern: constant op var (flip the comparison)
                else if let (Expr::IntLit(val), Expr::Var(var)) = (&left.node, &right.node) {
                    facts.push(ContractFact::VarCmp {
                        var: var.clone(),
                        op: flip_cmp_op(cmp_op),
                        value: *val,
                    });
                }
                // Pattern: var op var
                else if let (Expr::Var(lhs_var), Expr::Var(rhs_var)) = (&left.node, &right.node) {
                    facts.push(ContractFact::VarVarCmp {
                        lhs: lhs_var.clone(),
                        op: cmp_op,
                        rhs: rhs_var.clone(),
                    });
                }
            }
        }
        _ => {}
    }
}

/// Convert BinOp to CmpOp
fn binop_to_cmp_op(op: &BinOp) -> Option<CmpOp> {
    match op {
        BinOp::Lt => Some(CmpOp::Lt),
        BinOp::Le => Some(CmpOp::Le),
        BinOp::Gt => Some(CmpOp::Gt),
        BinOp::Ge => Some(CmpOp::Ge),
        BinOp::Eq => Some(CmpOp::Eq),
        BinOp::Ne => Some(CmpOp::Ne),
        _ => None,
    }
}

/// Flip comparison for constant op var → var flipped_op constant
fn flip_cmp_op(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::Lt => CmpOp::Gt,
        CmpOp::Le => CmpOp::Ge,
        CmpOp::Gt => CmpOp::Lt,
        CmpOp::Ge => CmpOp::Le,
        CmpOp::Eq => CmpOp::Eq,
        CmpOp::Ne => CmpOp::Ne,
    }
}

/// v0.51.41: Compute the size of a type in bytes
fn compute_type_size(ty: &Type, ctx: &LoweringContext) -> i64 {
    match ty {
        Type::I32 | Type::U32 => 4,
        Type::I64 | Type::U64 | Type::F64 => 8,
        Type::Bool => 1,
        Type::Char => 4, // Unicode codepoint
        Type::Unit => 0,
        Type::Ptr(_) => 8, // 64-bit pointers
        Type::Ref(_) | Type::RefMut(_) => 8, // References are pointers
        Type::Named(name) => {
            // Look up struct size from struct_type_defs (has full type info)
            if let Some(fields) = ctx.struct_type_defs.get(name) {
                fields.iter().map(|(_, field_ty)| compute_mir_type_size(field_ty)).sum()
            } else {
                8 // Default to pointer size for unknown types
            }
        }
        Type::Array(elem_ty, size) => {
            compute_type_size(elem_ty, ctx) * (*size as i64)
        }
        Type::Tuple(elems) => {
            elems.iter().map(|e| compute_type_size(e, ctx)).sum()
        }
        // Generic types, function types, etc. - default to pointer size
        _ => 8,
    }
}

/// v0.51.41: Compute size of MIR type in bytes
fn compute_mir_type_size(ty: &MirType) -> i64 {
    match ty {
        MirType::I32 | MirType::U32 => 4,
        MirType::I64 | MirType::U64 | MirType::F64 => 8,
        MirType::Bool => 1,
        MirType::Char => 4,
        MirType::Unit => 0,
        MirType::Ptr(_) | MirType::StructPtr(_) => 8,
        MirType::Array { element_type, size } => {
            let elem_size = compute_mir_type_size(element_type);
            elem_size * (size.unwrap_or(1) as i64)
        }
        MirType::Struct { fields, .. } => {
            fields.iter().map(|(_, ty)| compute_mir_type_size(ty)).sum()
        }
        // Default for other types (String, Enum, etc.)
        _ => 8,
    }
}

/// Lower an expression, returning the operand holding its result
fn lower_expr(expr: &Spanned<Expr>, ctx: &mut LoweringContext) -> Operand {
    match &expr.node {
        Expr::IntLit(n) => Operand::Constant(Constant::Int(*n)),

        Expr::FloatLit(f) => Operand::Constant(Constant::Float(*f)),

        Expr::BoolLit(b) => Operand::Constant(Constant::Bool(*b)),

        Expr::StringLit(s) => Operand::Constant(Constant::String(s.clone())),

        // v0.64: Character literal
        Expr::CharLit(c) => Operand::Constant(Constant::Char(*c)),

        Expr::Unit => Operand::Constant(Constant::Unit),

        // v0.51.40: Null pointer literal - lowered as integer 0
        Expr::Null => Operand::Constant(Constant::Int(0)),

        // v0.51.41: Sizeof - compute size based on type
        Expr::Sizeof { ty } => {
            let size = compute_type_size(&ty.node, ctx);
            Operand::Constant(Constant::Int(size))
        }

        // v0.70: Spawn expression - creates a new thread
        // Phase 2: Detect simple function call patterns for real async threading.
        // For `spawn { func(args) }`, we pass the function name and arguments
        // so codegen can generate a wrapper and spawn a real thread.
        Expr::Spawn { body } => {
            // Helper to extract Call from expression (handles Block wrapper)
            fn extract_call(expr: &Spanned<Expr>) -> Option<(&String, &Vec<Spanned<Expr>>)> {
                match &expr.node {
                    // Direct call: spawn func(args)
                    Expr::Call { func, args } => Some((func, args)),
                    // Block with single expression: spawn { func(args) }
                    Expr::Block(stmts) if stmts.len() == 1 => {
                        if let Expr::Call { func, args } = &stmts[0].node {
                            Some((func, args))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }

            // Check if body is a simple function call pattern
            let (target_func, captures) = if let Some((func, args)) = extract_call(body) {
                // Lower arguments to get their values (evaluated before spawning)
                let arg_operands: Vec<Operand> = args
                    .iter()
                    .map(|arg| lower_expr(arg, ctx))
                    .collect();
                (func.clone(), arg_operands)
            } else {
                // Not a simple call pattern, fall back to Phase 1 (synchronous)
                let body_operand = lower_expr(body, ctx);
                (format!("__spawn_inline_{}", ctx.spawn_counter), vec![body_operand])
            };

            ctx.spawn_counter += 1;

            // Emit ThreadSpawn instruction
            let dest = ctx.fresh_temp();
            ctx.push_inst(MirInst::ThreadSpawn {
                dest: dest.clone(),
                func: target_func,
                captures,
            });

            // The result is a thread handle
            ctx.locals.insert(dest.name.clone(), MirType::I64);
            Operand::Place(dest)
        }

        // v0.71: Mutex creation expression
        Expr::MutexNew { value } => {
            // Lower the initial value expression
            let init_operand = lower_expr(value, ctx);

            // Create destination for the mutex handle
            let dest = ctx.fresh_temp();

            // Emit MutexNew instruction
            ctx.push_inst(MirInst::MutexNew {
                dest: dest.clone(),
                initial_value: init_operand,
            });

            // The mutex is represented as an i64 handle
            ctx.locals.insert(dest.name.clone(), MirType::I64);

            Operand::Place(dest)
        }

        // v0.73: Channel creation expression
        Expr::ChannelNew { capacity, .. } => {
            // Lower the capacity expression
            let cap_operand = lower_expr(capacity, ctx);

            // Create destinations for sender and receiver
            let sender_dest = ctx.fresh_temp();
            let receiver_dest = ctx.fresh_temp();

            // Emit ChannelNew instruction
            ctx.push_inst(MirInst::ChannelNew {
                sender_dest: sender_dest.clone(),
                receiver_dest: receiver_dest.clone(),
                capacity: cap_operand,
            });

            // Register the types
            ctx.locals.insert(sender_dest.name.clone(), MirType::I64);
            ctx.locals.insert(receiver_dest.name.clone(), MirType::I64);

            // Return a tuple of (sender, receiver)
            let tuple_dest = ctx.fresh_temp();
            ctx.push_inst(MirInst::TupleInit {
                dest: tuple_dest.clone(),
                elements: vec![
                    (MirType::I64, Operand::Place(sender_dest)),
                    (MirType::I64, Operand::Place(receiver_dest)),
                ],
            });
            ctx.locals.insert(tuple_dest.name.clone(), MirType::Tuple(vec![
                Box::new(MirType::I64),
                Box::new(MirType::I64),
            ]));

            Operand::Place(tuple_dest)
        }

        Expr::Var(name) => Operand::Place(Place::new(name.clone())),

        Expr::Binary { left, op, right } => {
            let lhs = lower_expr(left, ctx);
            let rhs = lower_expr(right, ctx);
            let dest = ctx.fresh_temp();

            // Determine the MIR operator based on operand types
            let lhs_ty = ctx.operand_type(&lhs);
            let rhs_ty = ctx.operand_type(&rhs);

            // v0.60.19: Check for pointer arithmetic (ptr + i or ptr - i)
            let is_ptr_add = (*op == BinOp::Add || *op == BinOp::Sub)
                && lhs_ty.is_pointer_type()
                && (rhs_ty == MirType::I64 || rhs_ty == MirType::I32);
            let is_add_ptr = *op == BinOp::Add
                && (lhs_ty == MirType::I64 || lhs_ty == MirType::I32)
                && rhs_ty.is_pointer_type();

            if is_ptr_add {
                // ptr + i or ptr - i: generate PtrOffset
                let element_type = lhs_ty.pointer_element_type().unwrap_or(MirType::I64);
                let result_ty = lhs_ty.clone();
                ctx.locals.insert(dest.name.clone(), result_ty);

                // v0.60.20: Register struct type for PtrOffset destination so FieldStore/FieldAccess work
                if let MirType::StructPtr(struct_name) = &element_type {
                    ctx.var_struct_types.insert(dest.name.clone(), struct_name.clone());
                }

                // For subtraction, negate the offset
                let offset = if *op == BinOp::Sub {
                    let neg_dest = ctx.fresh_temp();
                    ctx.locals.insert(neg_dest.name.clone(), rhs_ty.clone());
                    ctx.push_inst(MirInst::UnaryOp {
                        dest: neg_dest.clone(),
                        op: MirUnaryOp::Neg,
                        src: rhs.clone(),
                    });
                    Operand::Place(neg_dest)
                } else {
                    rhs
                };

                ctx.push_inst(MirInst::PtrOffset {
                    dest: dest.clone(),
                    ptr: lhs,
                    offset,
                    element_type,
                });
            } else if is_add_ptr {
                // i + ptr: swap operands and generate PtrOffset
                let element_type = rhs_ty.pointer_element_type().unwrap_or(MirType::I64);
                let result_ty = rhs_ty.clone();
                ctx.locals.insert(dest.name.clone(), result_ty);

                // v0.60.20: Register struct type for PtrOffset destination so FieldStore/FieldAccess work
                if let MirType::StructPtr(struct_name) = &element_type {
                    ctx.var_struct_types.insert(dest.name.clone(), struct_name.clone());
                }

                ctx.push_inst(MirInst::PtrOffset {
                    dest: dest.clone(),
                    ptr: rhs,
                    offset: lhs,
                    element_type,
                });
            } else {
                // Standard binary operation
                let mir_op = ast_binop_to_mir(*op, &lhs_ty);

                // v0.35.4: Store result type for temporary variable
                let result_ty = mir_op.result_type(&lhs_ty);
                ctx.locals.insert(dest.name.clone(), result_ty);

                ctx.push_inst(MirInst::BinOp {
                    dest: dest.clone(),
                    op: mir_op,
                    lhs,
                    rhs,
                });
            }

            Operand::Place(dest)
        }

        Expr::Unary { op, expr: inner } => {
            let src = lower_expr(inner, ctx);
            let dest = ctx.fresh_temp();

            let src_ty = ctx.operand_type(&src);
            let mir_op = ast_unop_to_mir(*op, &src_ty);

            // v0.35.4: Store result type for temporary variable
            let result_ty = mir_op.result_type(&src_ty);
            ctx.locals.insert(dest.name.clone(), result_ty);

            ctx.push_inst(MirInst::UnaryOp {
                dest: dest.clone(),
                op: mir_op,
                src,
            });

            Operand::Place(dest)
        }

        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            // Evaluate condition
            let cond_op = lower_expr(cond, ctx);

            // Create labels for branches
            let then_label = ctx.fresh_label("then");
            let else_label = ctx.fresh_label("else");
            let merge_label = ctx.fresh_label("merge");

            // Result place for the if expression (will be assigned via PHI)
            let result = ctx.fresh_temp();

            // Branch based on condition
            ctx.finish_block(Terminator::Branch {
                cond: cond_op,
                then_label: then_label.clone(),
                else_label: else_label.clone(),
            });

            // Then block - generate result but don't copy
            ctx.start_block(then_label.clone());
            let then_result = lower_expr(then_branch, ctx);
            // Remember the actual label where the value was generated
            // (in case lowering created additional blocks)
            let then_exit_label = ctx.current_block_label().to_string();
            ctx.finish_block(Terminator::Goto(merge_label.clone()));

            // Else block - generate result but don't copy
            ctx.start_block(else_label.clone());
            let else_result = lower_expr(else_branch, ctx);
            let else_exit_label = ctx.current_block_label().to_string();
            ctx.finish_block(Terminator::Goto(merge_label.clone()));

            // Merge block with PHI node
            ctx.start_block(merge_label);

            // v0.46: Register PHI result type to ensure proper type inference
            // Use the then_result's type since both branches should have the same type
            let phi_result_ty = ctx.operand_type(&then_result);
            ctx.locals.insert(result.name.clone(), phi_result_ty);

            ctx.push_inst(MirInst::Phi {
                dest: result.clone(),
                values: vec![
                    (then_result, then_exit_label),
                    (else_result, else_exit_label),
                ],
            });

            Operand::Place(result)
        }

        Expr::Let {
            name,
            mutable: _,
            ty,
            value,
            body,
        } => {
            // Lower the value
            let value_op = lower_expr(value, ctx);

            // Determine type
            let mir_ty = if let Some(ty_span) = ty {
                ast_type_to_mir(&ty_span.node)
            } else {
                ctx.operand_type(&value_op)
            };

            // Register local
            ctx.locals.insert(name.clone(), mir_ty);

            // Assign to the variable
            let var_place = Place::new(name.clone());
            match value_op {
                Operand::Constant(c) => {
                    ctx.push_inst(MirInst::Const {
                        dest: var_place,
                        value: c,
                    });
                }
                Operand::Place(src) => {
                    // v0.51.23: Propagate struct type info for field access tracking
                    if let Some(struct_name) = ctx.var_struct_types.get(&src.name).cloned() {
                        ctx.var_struct_types.insert(name.clone(), struct_name);
                    }
                    // v0.51.35: Propagate array element types for struct array handling
                    if let Some(elem_ty) = ctx.array_element_types.get(&src.name).cloned() {
                        ctx.array_element_types.insert(name.clone(), elem_ty);
                    }
                    ctx.push_inst(MirInst::Copy {
                        dest: var_place,
                        src,
                    });
                }
            }

            // Lower the body
            lower_expr(body, ctx)
        }

        // v0.60.21: Uninitialized let binding for stack arrays
        Expr::LetUninit { name, mutable: _, ty, body } => {
            let mir_ty = ast_type_to_mir(&ty.node);

            // Only array types should reach here (type checker enforces this)
            if let MirType::Array { element_type, size: Some(n) } = &mir_ty {
                let var_place = Place::new(name.clone());
                ctx.locals.insert(name.clone(), mir_ty.clone());
                ctx.array_element_types.insert(name.clone(), *element_type.clone());

                ctx.push_inst(MirInst::ArrayAlloc {
                    dest: var_place,
                    element_type: *element_type.clone(),
                    size: *n,
                });
            } else {
                // Should not reach here due to type checker, but handle gracefully
                ctx.locals.insert(name.clone(), mir_ty);
            }

            // Lower the body
            lower_expr(body, ctx)
        }

        Expr::Assign { name, value } => {
            // Lower the value
            let value_op = lower_expr(value, ctx);

            // Assign to the variable (must already exist)
            let var_place = Place::new(name.clone());
            match value_op {
                Operand::Constant(c) => {
                    ctx.push_inst(MirInst::Const {
                        dest: var_place,
                        value: c,
                    });
                }
                Operand::Place(src) => {
                    ctx.push_inst(MirInst::Copy {
                        dest: var_place,
                        src,
                    });
                }
            }

            // Assignment expression returns the assigned value
            Operand::Place(Place::new(name.clone()))
        }

        // v0.37: Invariant is for SMT verification, MIR lowering ignores it
        // v0.60.16: Push loop context for break/continue support
        Expr::While { cond, invariant: _, body } => {
            // Create labels for loop structure
            let cond_label = ctx.fresh_label("while_cond");
            let body_label = ctx.fresh_label("while_body");
            let exit_label = ctx.fresh_label("while_exit");

            // Jump to condition check
            ctx.finish_block(Terminator::Goto(cond_label.clone()));

            // Condition block
            ctx.start_block(cond_label.clone());
            let cond_op = lower_expr(cond, ctx);
            ctx.finish_block(Terminator::Branch {
                cond: cond_op,
                then_label: body_label.clone(),
                else_label: exit_label.clone(),
            });

            // v0.60.16: Push loop context for break/continue
            ctx.loop_context_stack.push((cond_label.clone(), exit_label.clone()));

            // Body block
            ctx.start_block(body_label);
            let _ = lower_expr(body, ctx);
            ctx.finish_block(Terminator::Goto(cond_label));

            // v0.60.16: Pop loop context
            ctx.loop_context_stack.pop();

            // Exit block
            ctx.start_block(exit_label);

            // While loop returns unit
            Operand::Constant(Constant::Unit)
        }

        Expr::Call { func, args } => {
            // Lower arguments
            let arg_ops: Vec<Operand> = args.iter().map(|arg| lower_expr(arg, ctx)).collect();

            // Check if this is a void function (runtime functions that return void)
            let is_void_func = matches!(func.as_str(), "println" | "print" | "assert");

            if is_void_func {
                ctx.push_inst(MirInst::Call {
                    dest: None,
                    func: func.clone(),
                    args: arg_ops,
                    is_tail: false, // v0.50.65: void functions are not tail calls
                });
                Operand::Constant(Constant::Unit)
            } else {
                let dest = ctx.fresh_temp();

                // v0.35.4: Store return type for Call result
                // v0.46: Also handle runtime functions with known return types
                let ret_ty = if let Some(ty) = ctx.func_return_types.get(func) {
                    ty.clone()
                } else {
                    // Runtime functions with known return types
                    match func.as_str() {
                        // String-returning runtime functions
                        // v0.46: get_arg returns string (pointer to BmbString)
                        // v0.46: sb_build returns string (pointer to BmbString)
                        // v0.50.53: chr returns string (single-char BmbString)
                        "int_to_string" | "read_file" | "slice" | "digit_char" | "get_arg" | "sb_build" | "chr" => MirType::String,
                        // i64-returning runtime functions
                        // v0.46: arg_count returns i64
                        // v0.50.71: file_exists returns i64 (0 or 1), not bool - runtime uses int64_t
                        "byte_at" | "len" | "strlen" | "cstr_byte_at" | "arg_count" | "file_exists" => MirType::I64,
                        // Bool-returning runtime functions (actual C bool or i1)
                        "cstr_eq" => MirType::Bool,
                        // Default to i64 for unknown functions
                        _ => MirType::I64,
                    }
                };
                // v0.51.24: If return type is a struct, register in var_struct_types for field_index lookup
                // v0.51.37: Also handle pointer return types (*Node)
                if let MirType::Struct { name: struct_name, .. } = &ret_ty {
                    ctx.var_struct_types.insert(dest.name.clone(), struct_name.clone());
                } else if let MirType::Ptr(inner) = &ret_ty {
                    if let MirType::StructPtr(struct_name) = inner.as_ref() {
                        ctx.var_struct_types.insert(dest.name.clone(), struct_name.clone());
                    }
                }
                ctx.locals.insert(dest.name.clone(), ret_ty);

                ctx.push_inst(MirInst::Call {
                    dest: Some(dest.clone()),
                    func: func.clone(),
                    args: arg_ops,
                    is_tail: false, // v0.50.65: will be marked true by tail_call_optimization pass
                });
                Operand::Place(dest)
            }
        }

        Expr::Block(exprs) => {
            if exprs.is_empty() {
                return Operand::Constant(Constant::Unit);
            }

            // Lower all expressions, return the last one
            let mut result = Operand::Constant(Constant::Unit);
            for expr in exprs {
                result = lower_expr(expr, ctx);
            }
            result
        }

        Expr::Ret => {
            // 'ret' in postconditions refers to the return value
            // In MIR lowering, we don't handle contracts - just return unit
            Operand::Constant(Constant::Unit)
        }

        // v0.19.0: Struct initialization
        // v0.51.35: Register struct type in temp_types for proper array element type inference
        Expr::StructInit { name, fields } => {
            // Lower each field value
            let mir_fields: Vec<(String, Operand)> = fields
                .iter()
                .map(|(field_name, field_value)| {
                    let value_op = lower_expr(field_value, ctx);
                    (field_name.node.clone(), value_op)
                })
                .collect();

            // Create destination for the struct
            let dest = ctx.fresh_temp();

            // v0.51.23: Track struct type for the destination
            ctx.var_struct_types.insert(dest.name.clone(), name.clone());

            // v0.51.35: Register the struct type in temp_types for operand_type()
            let struct_mir_type = if let Some(field_types) = ctx.struct_type_defs.get(name) {
                MirType::Struct {
                    name: name.clone(),
                    fields: field_types.iter()
                        .map(|(n, t)| (n.clone(), Box::new(t.clone())))
                        .collect(),
                }
            } else {
                // Fallback: create type from field names only
                MirType::Struct {
                    name: name.clone(),
                    fields: mir_fields.iter()
                        .map(|(n, _)| (n.clone(), Box::new(MirType::I64)))
                        .collect(),
                }
            };
            ctx.temp_types.insert(dest.name.clone(), struct_mir_type);

            ctx.push_inst(MirInst::StructInit {
                dest: dest.clone(),
                struct_name: name.clone(),
                fields: mir_fields,
            });

            Operand::Place(dest)
        }

        // v0.19.0: Field access
        Expr::FieldAccess { expr, field } => {
            // Lower the base expression
            let base_op = lower_expr(expr, ctx);

            // Convert operand to place if needed
            let base_place = operand_to_place(base_op, ctx);

            // v0.51.23: Compute field index using struct type info
            // v0.51.31: Also capture struct_name for field type lookup in codegen
            let struct_name = ctx.place_struct_type(&base_place).unwrap_or_default();
            let field_index = if struct_name.is_empty() {
                0
            } else {
                ctx.field_index(&struct_name, &field.node)
            };

            // v0.51.31: Get the field type for type inference
            let field_ty = ctx.field_type(&struct_name, &field.node);

            // Create destination for the field value
            let dest = ctx.fresh_temp();

            // v0.51.31: Register temp type for operand_type() to find
            // (Don't add to locals - temps don't need allocas)
            ctx.temp_types.insert(dest.name.clone(), field_ty.clone());

            // v0.51.39: If field type is a pointer to struct, register the inner struct name
            // so subsequent field accesses on this value work correctly
            match &field_ty {
                MirType::StructPtr(inner_name) => {
                    ctx.var_struct_types.insert(dest.name.clone(), inner_name.clone());
                }
                MirType::Ptr(inner) => {
                    if let MirType::StructPtr(inner_name) = inner.as_ref() {
                        ctx.var_struct_types.insert(dest.name.clone(), inner_name.clone());
                    }
                }
                _ => {}
            }

            ctx.push_inst(MirInst::FieldAccess {
                dest: dest.clone(),
                base: base_place,
                field: field.node.clone(),
                field_index,
                struct_name,
            });

            Operand::Place(dest)
        }

        // v0.55: Tuple field access (compile-time constant index)
        Expr::TupleField { expr, index } => {
            // Lower the tuple expression
            let tuple_op = lower_expr(expr, ctx);

            // Convert operand to place if needed
            let tuple_place = operand_to_place(tuple_op.clone(), ctx);

            // Get the element type from the tuple's type
            let element_type = match ctx.operand_type(&tuple_op) {
                MirType::Tuple(elems) if *index < elems.len() => (*elems[*index]).clone(),
                _ => MirType::I64, // Fallback for unknown types
            };

            // Create destination for the element value
            let dest = ctx.fresh_temp();

            // v0.55: Use TupleExtract for proper struct-based tuple access
            ctx.push_inst(MirInst::TupleExtract {
                dest: dest.clone(),
                tuple: tuple_place,
                index: *index,
                element_type,
            });

            Operand::Place(dest)
        }

        // v0.19.1: Enum variant construction
        Expr::EnumVariant { enum_name, variant, args } => {
            // Lower each argument
            let mir_args: Vec<Operand> = args
                .iter()
                .map(|arg| lower_expr(arg, ctx))
                .collect();

            // Create destination for the enum value
            let dest = ctx.fresh_temp();

            ctx.push_inst(MirInst::EnumVariant {
                dest: dest.clone(),
                enum_name: enum_name.clone(),
                variant: variant.clone(),
                args: mir_args,
            });

            Operand::Place(dest)
        }

        // v0.5 Phase 3: Range expression (returns pair of start/end as start for now)
        Expr::Range { start, .. } => {
            // For MIR, we just return the start value
            // Full range support would require a Range data structure
            lower_expr(start, ctx)
        }

        // v0.5 Phase 3: For loop (lowered to while loop pattern)
        Expr::For { var, iter, body } => {
            // Lower the iterator (expecting Range expression)
            // Extract start and end from range
            let (start_op, end_op) = match &iter.node {
                Expr::Range { start, end, .. } => {
                    (lower_expr(start, ctx), lower_expr(end, ctx))
                }
                _ => {
                    // Non-range iterator - just evaluate and return unit
                    let _ = lower_expr(iter, ctx);
                    return Operand::Constant(Constant::Unit);
                }
            };

            // Register loop variable
            let mir_ty = ctx.operand_type(&start_op);
            ctx.locals.insert(var.clone(), mir_ty);

            // Initialize loop variable with start value
            let var_place = Place::new(var.clone());
            match start_op {
                Operand::Constant(c) => {
                    ctx.push_inst(MirInst::Const {
                        dest: var_place.clone(),
                        value: c,
                    });
                }
                Operand::Place(src) => {
                    ctx.push_inst(MirInst::Copy {
                        dest: var_place.clone(),
                        src,
                    });
                }
            }

            // Store end value in a temp for comparison
            let end_place = operand_to_place(end_op, ctx);

            // Create labels for loop structure
            let cond_label = ctx.fresh_label("for_cond");
            let body_label = ctx.fresh_label("for_body");
            let exit_label = ctx.fresh_label("for_exit");

            // Jump to condition check
            ctx.finish_block(Terminator::Goto(cond_label.clone()));

            // Condition block: i < end
            ctx.start_block(cond_label.clone());
            let cond_temp = ctx.fresh_temp();
            ctx.push_inst(MirInst::BinOp {
                dest: cond_temp.clone(),
                op: MirBinOp::Lt,
                lhs: Operand::Place(var_place.clone()),
                rhs: Operand::Place(end_place),
            });
            ctx.finish_block(Terminator::Branch {
                cond: Operand::Place(cond_temp),
                then_label: body_label.clone(),
                else_label: exit_label.clone(),
            });

            // Body block
            ctx.start_block(body_label);
            let _ = lower_expr(body, ctx);

            // Increment loop variable: i = i + 1
            let inc_temp = ctx.fresh_temp();
            ctx.push_inst(MirInst::BinOp {
                dest: inc_temp.clone(),
                op: MirBinOp::Add,
                lhs: Operand::Place(var_place.clone()),
                rhs: Operand::Constant(Constant::Int(1)),
            });
            ctx.push_inst(MirInst::Copy {
                dest: var_place,
                src: inc_temp,
            });
            ctx.finish_block(Terminator::Goto(cond_label));

            // Exit block
            ctx.start_block(exit_label);

            // For loop returns unit
            Operand::Constant(Constant::Unit)
        }

        Expr::Match { expr, arms } => {
            // v0.19.2: Improved pattern matching with Switch terminator
            if arms.is_empty() {
                return Operand::Constant(Constant::Unit);
            }

            // Evaluate the match expression
            let match_val = lower_expr(expr, ctx);
            let match_place = match &match_val {
                Operand::Place(p) => p.clone(),
                Operand::Constant(c) => {
                    // Store constant in a temp
                    let temp = ctx.fresh_temp();
                    ctx.push_inst(MirInst::Const {
                        dest: temp.clone(),
                        value: c.clone(),
                    });
                    temp
                }
            };

            // Create labels for each arm and merge point
            let arm_labels: Vec<String> = arms.iter()
                .enumerate()
                .map(|(i, _)| ctx.fresh_label(&format!("match_arm_{}", i)))
                .collect();
            let merge_label = ctx.fresh_label("match_merge");
            let default_label = ctx.fresh_label("match_default");

            // Analyze patterns to generate switch cases
            // v0.60.262: Also get wildcard arm index for proper default handling
            let (cases, wildcard_arm_index) = compile_match_patterns(arms, &arm_labels, &default_label);

            // v0.60.262: Use wildcard arm label as switch default if present
            let actual_default_label = match wildcard_arm_index {
                Some(idx) => arm_labels[idx].clone(),
                None => default_label.clone(),
            };

            // Close current block with switch terminator
            ctx.finish_block(Terminator::Switch {
                discriminant: Operand::Place(match_place.clone()),
                cases,
                default: actual_default_label,
            });

            // Result place for PHI node
            let result_place = ctx.fresh_temp();
            let mut phi_values: Vec<(Operand, String)> = Vec::new();

            // Generate code for each arm
            for (i, arm) in arms.iter().enumerate() {
                ctx.start_block(arm_labels[i].clone());

                // Bind pattern variables if needed
                bind_pattern_variables(&arm.pattern.node, &match_place, ctx);

                // Evaluate arm body
                let arm_result = lower_expr(&arm.body, ctx);

                // Store result for PHI
                let arm_end_label = ctx.current_block_label().to_string();
                phi_values.push((arm_result, arm_end_label));

                // Jump to merge block
                ctx.finish_block(Terminator::Goto(merge_label.clone()));
            }

            // v0.60.262: Only generate default block if no wildcard arm
            // (Wildcard arm handles all unmatched cases)
            if wildcard_arm_index.is_none() {
                ctx.start_block(default_label);
                ctx.finish_block(Terminator::Unreachable);
            }

            // Generate merge block with PHI
            ctx.start_block(merge_label);

            // v0.46: Register PHI result type to ensure proper type inference
            // Use the first arm's result type since all arms should have the same type
            if let Some((first_result, _)) = phi_values.first() {
                let phi_result_ty = ctx.operand_type(first_result);
                ctx.locals.insert(result_place.name.clone(), phi_result_ty);
            }

            ctx.push_inst(MirInst::Phi {
                dest: result_place.clone(),
                values: phi_values,
            });

            Operand::Place(result_place)
        }

        // v0.5 Phase 5: References (simplified - just evaluate inner)
        Expr::Ref(inner) | Expr::RefMut(inner) => {
            lower_expr(inner, ctx)
        }

        // v0.60.20: Dereference - for native pointers, generate PtrLoad
        Expr::Deref(inner) => {
            let ptr_op = lower_expr(inner, ctx);
            let ptr_ty = ctx.operand_type(&ptr_op);

            // Check if this is a native pointer type
            if let MirType::Ptr(element_type) = ptr_ty {
                // Generate PtrLoad to load value through pointer
                let dest = ctx.fresh_temp();
                // Track the result type for downstream type inference
                ctx.temp_types.insert(dest.name.clone(), *element_type.clone());
                ctx.push_inst(MirInst::PtrLoad {
                    dest: dest.clone(),
                    ptr: ptr_op,
                    element_type: *element_type,
                });
                Operand::Place(dest)
            } else {
                // For references, just pass through (semantically the same)
                ptr_op
            }
        }

        // v0.19.3: Array support
        // v0.51.35: Track element types for struct array support
        Expr::ArrayLit(elems) => {
            // Lower each element
            let mir_elements: Vec<Operand> = elems
                .iter()
                .map(|e| lower_expr(e, ctx))
                .collect();

            // Infer element type from first element (or default to i64)
            let element_type = if !mir_elements.is_empty() {
                ctx.operand_type(&mir_elements[0])
            } else {
                MirType::I64
            };

            let dest = ctx.fresh_temp();
            // v0.51.35: Track array element type for IndexLoad/IndexStore
            ctx.array_element_types.insert(dest.name.clone(), element_type.clone());
            ctx.push_inst(MirInst::ArrayInit {
                dest: dest.clone(),
                element_type,
                elements: mir_elements,
            });
            Operand::Place(dest)
        }

        // v0.60.22: Array repeat [val; N] - creates array of N repeated values
        Expr::ArrayRepeat { value, count } => {
            // Lower the value expression once
            let value_op = lower_expr(value, ctx);
            let element_type = ctx.operand_type(&value_op);

            // Create N copies of the operand for ArrayInit
            let mir_elements: Vec<Operand> = (0..*count)
                .map(|_| value_op.clone())
                .collect();

            let dest = ctx.fresh_temp();
            ctx.array_element_types.insert(dest.name.clone(), element_type.clone());
            ctx.push_inst(MirInst::ArrayInit {
                dest: dest.clone(),
                element_type,
                elements: mir_elements,
            });
            Operand::Place(dest)
        }

        // v0.55: Tuple expressions - native heterogeneous tuple support
        Expr::Tuple(elems) => {
            // Lower each element and collect its type
            let typed_elements: Vec<(MirType, Operand)> = elems
                .iter()
                .map(|e| {
                    let op = lower_expr(e, ctx);
                    let ty = ctx.operand_type(&op);
                    (ty, op)
                })
                .collect();

            // Create tuple type for tracking
            let tuple_type = MirType::Tuple(
                typed_elements.iter().map(|(ty, _)| Box::new(ty.clone())).collect()
            );

            let dest = ctx.fresh_temp();
            // Track tuple type for TupleExtract
            ctx.temp_types.insert(dest.name.clone(), tuple_type);
            ctx.push_inst(MirInst::TupleInit {
                dest: dest.clone(),
                elements: typed_elements,
            });
            Operand::Place(dest)
        }

        Expr::Index { expr, index } => {
            // v0.19.3: Array indexing
            // v0.51.35: Track element type for struct array support
            let arr = lower_expr(expr, ctx);
            let arr_place = match &arr {
                Operand::Place(p) => p.clone(),
                Operand::Constant(_) => {
                    // Store constant in a temp
                    let temp = ctx.fresh_temp();
                    ctx.push_inst(MirInst::Copy {
                        dest: temp.clone(),
                        src: Place::new("_const_arr"), // This is simplified
                    });
                    temp
                }
            };

            // v0.51.35: Get element type from tracked array types or default to I64
            let element_type = ctx.array_element_types.get(&arr_place.name)
                .cloned()
                .unwrap_or(MirType::I64);

            let idx = lower_expr(index, ctx);
            let dest = ctx.fresh_temp();

            // v0.51.36: Register struct type for IndexLoad dest so FieldStore/FieldAccess work
            if let MirType::Struct { name: struct_name, .. } = &element_type {
                ctx.var_struct_types.insert(dest.name.clone(), struct_name.clone());
            }

            // v0.60.20: Register element type for dest so operand_type returns correct type
            // This is needed for float operations to use FMul/FAdd instead of Mul/Add
            ctx.locals.insert(dest.name.clone(), element_type.clone());

            ctx.push_inst(MirInst::IndexLoad {
                dest: dest.clone(),
                array: arr_place,
                index: idx,
                element_type,
            });
            Operand::Place(dest)
        }

        // v0.51: Index assignment: arr[i] = value
        // v0.51.35: Track element type for struct array support
        Expr::IndexAssign { array, index, value } => {
            let arr = lower_expr(array, ctx);
            let arr_place = match &arr {
                Operand::Place(p) => p.clone(),
                Operand::Constant(_) => {
                    // Store constant in a temp (unlikely for mutable arrays)
                    let temp = ctx.fresh_temp();
                    ctx.push_inst(MirInst::Copy {
                        dest: temp.clone(),
                        src: Place::new("_const_arr"),
                    });
                    temp
                }
            };

            // v0.51.35: Get element type from tracked array types or default to I64
            let element_type = ctx.array_element_types.get(&arr_place.name)
                .cloned()
                .unwrap_or(MirType::I64);

            let idx = lower_expr(index, ctx);
            let val = lower_expr(value, ctx);

            ctx.push_inst(MirInst::IndexStore {
                array: arr_place,
                index: idx,
                value: val,
                element_type,
            });

            // Return unit
            Operand::Constant(Constant::Unit)
        }

        // v0.51.23: Field assignment: obj.field = value
        // v0.51.31: Added struct_name for field type lookup in codegen
        Expr::FieldAssign { object, field, value } => {
            // Lower the object expression
            let obj_op = lower_expr(object, ctx);
            let base_place = operand_to_place(obj_op, ctx);

            // Compute field index and struct name using struct type info
            let struct_name = ctx.place_struct_type(&base_place).unwrap_or_default();
            let field_index = if struct_name.is_empty() {
                0
            } else {
                ctx.field_index(&struct_name, &field.node)
            };

            // Lower the value expression
            let val = lower_expr(value, ctx);

            ctx.push_inst(MirInst::FieldStore {
                base: base_place,
                field: field.node.clone(),
                field_index,
                struct_name,
                value: val,
            });

            // Return unit
            Operand::Constant(Constant::Unit)
        }

        // v0.60.21: Dereference assignment: set *ptr = value
        // Stores value through a native pointer using PtrStore instruction
        Expr::DerefAssign { ptr, value } => {
            let ptr_op = lower_expr(ptr, ctx);
            let val_op = lower_expr(value, ctx);

            // Get element type from the pointer type
            let element_type = match ctx.operand_type(&ptr_op) {
                MirType::Ptr(inner) => *inner,
                _ => MirType::I64, // Fallback
            };

            ctx.push_inst(MirInst::PtrStore {
                ptr: ptr_op,
                value: val_op,
                element_type,
            });

            // Return unit
            Operand::Constant(Constant::Unit)
        }

        // v0.19.4: Method calls - static dispatch
        // Methods are lowered as function calls with receiver as first argument
        // The method name is prefixed with the receiver type for name mangling
        Expr::MethodCall { receiver, method, args } => {
            // Lower the receiver expression
            let recv_op = lower_expr(receiver, ctx);

            // v0.70: Special handling for Thread.join() method
            // Thread.join() is lowered to MirInst::ThreadJoin, not a regular call
            if method == "join" && args.is_empty() {
                let dest = ctx.fresh_temp();
                ctx.locals.insert(dest.name.clone(), MirType::I64);
                ctx.push_inst(MirInst::ThreadJoin {
                    dest: Some(dest.clone()),
                    handle: recv_op,
                });
                return Operand::Place(dest);
            }

            // v0.71: Special handling for Mutex methods
            // lock() - acquires lock and returns current value
            if method == "lock" && args.is_empty() {
                let dest = ctx.fresh_temp();
                ctx.locals.insert(dest.name.clone(), MirType::I64);
                ctx.push_inst(MirInst::MutexLock {
                    dest: dest.clone(),
                    mutex: recv_op,
                });
                return Operand::Place(dest);
            }

            // unlock(value) - stores value and releases lock
            if method == "unlock" && args.len() == 1 {
                let value_op = lower_expr(&args[0], ctx);
                ctx.push_inst(MirInst::MutexUnlock {
                    mutex: recv_op,
                    new_value: value_op,
                });
                return Operand::Constant(Constant::Unit);
            }

            // try_lock() - attempts to acquire lock, returns 1 if success, 0 if failed
            if method == "try_lock" && args.is_empty() {
                let dest = ctx.fresh_temp();
                ctx.locals.insert(dest.name.clone(), MirType::I64);
                ctx.push_inst(MirInst::MutexTryLock {
                    dest: dest.clone(),
                    mutex: recv_op,
                });
                return Operand::Place(dest);
            }

            // free() - deallocates the mutex
            if method == "free" && args.is_empty() {
                ctx.push_inst(MirInst::MutexFree { mutex: recv_op });
                return Operand::Constant(Constant::Unit);
            }

            // v0.73: Sender<T> methods
            // send(value) - blocking send
            if method == "send" && args.len() == 1 {
                let value_op = lower_expr(&args[0], ctx);
                ctx.push_inst(MirInst::ChannelSend {
                    sender: recv_op,
                    value: value_op,
                });
                return Operand::Constant(Constant::Unit);
            }

            // try_send(value) - non-blocking send, returns 1 if success, 0 if failed
            if method == "try_send" && args.len() == 1 {
                let value_op = lower_expr(&args[0], ctx);
                let dest = ctx.fresh_temp();
                ctx.locals.insert(dest.name.clone(), MirType::I64);
                ctx.push_inst(MirInst::ChannelTrySend {
                    dest: dest.clone(),
                    sender: recv_op,
                    value: value_op,
                });
                return Operand::Place(dest);
            }

            // clone() - clone the sender for MPSC
            if method == "clone" && args.is_empty() {
                let dest = ctx.fresh_temp();
                ctx.locals.insert(dest.name.clone(), MirType::I64);
                ctx.push_inst(MirInst::SenderClone {
                    dest: dest.clone(),
                    sender: recv_op,
                });
                return Operand::Place(dest);
            }

            // v0.73: Receiver<T> methods
            // recv() - blocking receive
            if method == "recv" && args.is_empty() {
                let dest = ctx.fresh_temp();
                ctx.locals.insert(dest.name.clone(), MirType::I64);
                ctx.push_inst(MirInst::ChannelRecv {
                    dest: dest.clone(),
                    receiver: recv_op,
                });
                return Operand::Place(dest);
            }

            // try_recv() - non-blocking receive, returns value or -1 if empty
            if method == "try_recv" && args.is_empty() {
                let dest = ctx.fresh_temp();
                ctx.locals.insert(dest.name.clone(), MirType::I64);
                ctx.push_inst(MirInst::ChannelTryRecv {
                    dest: dest.clone(),
                    receiver: recv_op,
                });
                return Operand::Place(dest);
            }

            // Build the argument list: receiver first, then the rest
            let mut call_args = vec![recv_op];
            for arg in args {
                call_args.push(lower_expr(arg, ctx));
            }

            // Generate the function call with the method name
            // In a full implementation, the method name would be mangled with the type
            let dest = ctx.fresh_temp();

            // v0.46: Register method call return types
            // String methods have known return types
            let ret_type = match method.as_str() {
                "len" | "byte_at" => MirType::I64,
                "slice" => MirType::String,
                // Default to checking user-defined function return types
                _ => ctx.func_return_types.get(method).cloned().unwrap_or(MirType::I64),
            };
            ctx.locals.insert(dest.name.clone(), ret_type);

            ctx.push_inst(MirInst::Call {
                dest: Some(dest.clone()),
                func: method.clone(),
                args: call_args,
                is_tail: false, // v0.50.65: method calls handled same as function calls
            });
            Operand::Place(dest)
        }

        // v0.2: State references (handled during contract verification, not MIR)
        Expr::StateRef { expr, .. } => {
            // During MIR lowering, we just evaluate the expression
            // The .pre/.post semantics are handled by the SMT translator
            lower_expr(expr, ctx)
        }

        // v0.2: Refinement self-reference (translated to __it__ variable)
        Expr::It => Operand::Place(Place::new("__it__")),

        // v0.20.0: Closure expressions
        // TODO: Implement closure desugaring to struct with captured variables
        // For now, just lower the body expression
        Expr::Closure { body, .. } => lower_expr(body, ctx),

        // v0.31: Todo expression - panic at runtime
        Expr::Todo { .. } => {
            // In MIR, todo becomes a call to panic intrinsic
            // For now, return unit as this should never be reached
            Operand::Constant(crate::mir::Constant::Unit)
        }

        // v0.36: Additional control flow
        // Loop - lower body, infinite loop handled at codegen
        Expr::Loop { body } => {
            lower_expr(body, ctx)
        }

        // v0.60.16: Break - jump to loop exit
        Expr::Break { value } => {
            // If break has a value, lower it (but while loops ignore it)
            if let Some(v) = value {
                let _ = lower_expr(v, ctx);
            }

            // Get the exit label from the loop context stack
            if let Some((_, exit_label)) = ctx.loop_context_stack.last() {
                // Finish current block with jump to exit
                ctx.finish_block(Terminator::Goto(exit_label.clone()));
                // Start a new unreachable block for code after break
                let unreachable_label = ctx.fresh_label("after_break");
                ctx.start_block(unreachable_label);
            }
            // Return unit (break is a divergent expression)
            Operand::Constant(crate::mir::Constant::Unit)
        }

        // v0.60.16: Continue - jump to loop condition
        Expr::Continue => {
            // Get the continue label from the loop context stack
            if let Some((cond_label, _)) = ctx.loop_context_stack.last() {
                // Finish current block with jump to condition check
                ctx.finish_block(Terminator::Goto(cond_label.clone()));
                // Start a new unreachable block for code after continue
                let unreachable_label = ctx.fresh_label("after_continue");
                ctx.start_block(unreachable_label);
            }
            // Return unit (continue is a divergent expression)
            Operand::Constant(crate::mir::Constant::Unit)
        }

        // Return - placeholder, full implementation requires control flow
        Expr::Return { value } => {
            match value {
                Some(v) => lower_expr(v, ctx),
                None => Operand::Constant(crate::mir::Constant::Unit),
            }
        }

        // v0.37: Quantifiers - these are for SMT verification only
        // At MIR level, they should not appear (should be stripped earlier)
        // For now, we return unit as placeholder
        Expr::Forall { .. } | Expr::Exists { .. } => {
            Operand::Constant(crate::mir::Constant::Unit)
        }

        // v0.50.80: Type cast - emit explicit Cast instruction
        // Required for correct phi node types in if-else with mixed types
        // v0.51.32: Extended to support struct pointer casts
        Expr::Cast { expr, ty } => {
            let src_op = lower_expr(expr, ctx);
            // v0.51.33: Check if source is a struct pointer for correct type tracking
            let from_ty = if let Operand::Place(p) = &src_op {
                if let Some(struct_name) = ctx.var_struct_types.get(&p.name) {
                    MirType::StructPtr(struct_name.clone())  // Struct pointers need special handling
                } else {
                    ctx.operand_type(&src_op)
                }
            } else {
                ctx.operand_type(&src_op)
            };
            let to_ty = ast_type_to_mir(&ty.node);

            // v0.51.32: If casting to a struct type, we need to track the type
            // even if the underlying representation is the same (i64 pointer)
            // v0.51.34: Avoid generating Copy instructions for struct casts
            // Just register the existing place as having the struct type
            // v0.51.37: Also handle pointer types (*Node)
            let struct_name_opt = if let Type::Named(struct_name) = &ty.node {
                if ctx.struct_defs.contains_key(struct_name) {
                    Some(struct_name.clone())
                } else { None }
            } else if let Type::Ptr(inner) = &ty.node {
                if let Type::Named(struct_name) = inner.as_ref() {
                    if ctx.struct_defs.contains_key(struct_name) {
                        Some(struct_name.clone())
                    } else { None }
                } else { None }
            } else { None };

            if let Some(struct_name) = struct_name_opt {
                // v0.51.37: For pointer types, we need actual Cast instruction
                // to generate proper inttoptr/null in LLVM IR
                let is_ptr_cast = matches!(&ty.node, Type::Ptr(_));

                if is_ptr_cast {
                    // Pointer casts need actual Cast instruction for correct LLVM IR
                    let dest = ctx.fresh_temp();
                    ctx.push_inst(MirInst::Cast {
                        dest: dest.clone(),
                        src: src_op,
                        from_ty: from_ty.clone(),
                        to_ty: to_ty.clone(),
                    });
                    ctx.var_struct_types.insert(dest.name.clone(), struct_name.clone());
                    // v0.60.19: Also register the destination type for pointer type lookup
                    ctx.locals.insert(dest.name.clone(), to_ty.clone());
                    // v0.60.20: Track element type for pointer indexing (ptr[i])
                    // This allows IndexLoad/IndexStore to know the element type
                    if let MirType::Ptr(inner) = &to_ty {
                        ctx.array_element_types.insert(dest.name.clone(), *inner.clone());
                    } else {
                        // For *StructName, track the struct type
                        ctx.array_element_types.insert(dest.name.clone(), MirType::StructPtr(struct_name));
                    }
                    return Operand::Place(dest);
                }

                match &src_op {
                    Operand::Constant(c) => {
                        // Constants need a temp to hold the value
                        let dest = ctx.fresh_temp();
                        ctx.push_inst(MirInst::Const {
                            dest: dest.clone(),
                            value: c.clone(),
                        });
                        ctx.var_struct_types.insert(dest.name.clone(), struct_name);
                        return Operand::Place(dest);
                    }
                    Operand::Place(src) => {
                        // v0.51.34: Reuse existing place, just register struct type
                        // This eliminates unnecessary Copy instructions that become
                        // `add i64 %val, 0` in LLVM IR
                        ctx.var_struct_types.insert(src.name.clone(), struct_name);
                        return src_op;
                    }
                }
            }

            // v0.60.23: Array to pointer cast - [T; N] -> *T
            // Arrays decay to pointers to their first element
            if let MirType::Array { element_type, .. } = &from_ty {
                if let MirType::Ptr(target_elem) = &to_ty {
                    // Verify element types match
                    if element_type.as_ref() == target_elem.as_ref() {
                        // For arrays allocated on stack (alloca), the array variable
                        // is already a pointer to the first element
                        // Just register the new type and return
                        let dest = ctx.fresh_temp();
                        ctx.push_inst(MirInst::Cast {
                            dest: dest.clone(),
                            src: src_op.clone(),
                            from_ty: from_ty.clone(),
                            to_ty: to_ty.clone(),
                        });
                        ctx.locals.insert(dest.name.clone(), to_ty.clone());
                        ctx.array_element_types.insert(dest.name.clone(), *element_type.clone());
                        return Operand::Place(dest);
                    }
                }
            }

            // v0.51.33: Check if source is a struct pointer
            // Struct pointer to i64 requires ptrtoint in LLVM IR
            let src_is_struct_ptr = matches!(&from_ty, MirType::StructPtr(_));

            // If types are the same and source is NOT a struct pointer, no cast needed
            if from_ty == to_ty && !src_is_struct_ptr {
                return src_op;
            }

            // Generate cast instruction for numeric type conversions
            let dest = ctx.fresh_temp();
            ctx.push_inst(MirInst::Cast {
                dest: dest.clone(),
                src: src_op.clone(),
                from_ty,
                to_ty: to_ty.clone(),
            });

            // v0.60.19: Register the destination type for pointer casts
            // This ensures operand_type() returns the correct pointer type
            ctx.locals.insert(dest.name.clone(), to_ty.clone());

            // v0.60.20: Track element type for primitive pointer casts like *f64
            if let MirType::Ptr(inner) = &to_ty {
                ctx.array_element_types.insert(dest.name.clone(), *inner.clone());
            }

            Operand::Place(dest)
        }
    }
}

/// Convert an operand to a place, emitting a Const instruction if needed
fn operand_to_place(op: Operand, ctx: &mut LoweringContext) -> Place {
    match op {
        Operand::Place(p) => p,
        Operand::Constant(c) => {
            let temp = ctx.fresh_temp();
            ctx.push_inst(MirInst::Const {
                dest: temp.clone(),
                value: c,
            });
            temp
        }
    }
}

/// Convert AST type to MIR type
fn ast_type_to_mir(ty: &Type) -> MirType {
    match ty {
        Type::I32 => MirType::I32,
        Type::I64 => MirType::I64,
        // v0.38: Unsigned types
        Type::U32 => MirType::U32,
        Type::U64 => MirType::U64,
        Type::F64 => MirType::F64,
        Type::Bool => MirType::Bool,
        Type::String => MirType::String,
        // v0.64: Character type
        Type::Char => MirType::Char,
        Type::Unit => MirType::Unit,
        Type::Range(elem) => ast_type_to_mir(elem), // Range represented by its element type
        Type::Named(_) => MirType::I64, // Named types default to pointer-sized int for now
        // v0.13.1: Type variables are unresolved, treat as opaque (pointer-sized)
        Type::TypeVar(_) => MirType::I64,
        // v0.13.1: Generic types are treated as their container (pointer-sized for now)
        Type::Generic { .. } => MirType::I64,
        // v0.19.0: Struct types now fully supported
        Type::Struct { name, fields } => MirType::Struct {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(fname, fty)| (fname.clone(), Box::new(ast_type_to_mir(fty))))
                .collect(),
        },
        // v0.19.1: Enum types now fully supported
        Type::Enum { name, variants } => MirType::Enum {
            name: name.clone(),
            variants: variants
                .iter()
                .map(|(vname, vtypes)| {
                    (vname.clone(), vtypes.iter().map(|t| Box::new(ast_type_to_mir(t))).collect())
                })
                .collect(),
        },
        // v0.5 Phase 5: References to arrays become MirType::Array (pointers in LLVM)
        // v0.50.50: Fix array parameter codegen - array refs must be ptr, not i64
        Type::Ref(inner) | Type::RefMut(inner) => {
            if let Type::Array(elem, size) = inner.as_ref() {
                MirType::Array {
                    element_type: Box::new(ast_type_to_mir(elem)),
                    size: Some(*size),
                }
            } else {
                // Non-array references stay as i64 (pointer-sized)
                MirType::I64
            }
        }
        // v0.5 Phase 6: Arrays are represented as MirType::Array
        // v0.50.50: Fix - use MirType::Array instead of I64 for proper LLVM ptr codegen
        Type::Array(elem, size) => MirType::Array {
            element_type: Box::new(ast_type_to_mir(elem)),
            size: Some(*size),
        },
        // v0.2: Refined types use base type
        Type::Refined { base, .. } => ast_type_to_mir(base),
        // v0.20.0: Fn types are function pointers (pointer-sized)
        Type::Fn { .. } => MirType::I64,
        // v0.31: Never type - unreachable code, use Unit
        Type::Never => MirType::Unit,
        // v0.37: Nullable type - convert inner type (for MIR, nullable is just a tagged union)
        Type::Nullable(inner) => ast_type_to_mir(inner),
        // v0.55: Tuple type - native heterogeneous tuple support for struct-based LLVM codegen
        Type::Tuple(elems) => MirType::Tuple(
            elems.iter().map(|e| Box::new(ast_type_to_mir(e))).collect()
        ),
        // v0.51.37: Pointer type - preserve the pointee type for proper codegen
        // For struct pointers, use StructPtr to avoid issues with unknown struct definitions
        Type::Ptr(inner) => {
            match inner.as_ref() {
                Type::Named(name) => MirType::Ptr(Box::new(MirType::StructPtr(name.clone()))),
                _ => MirType::Ptr(Box::new(ast_type_to_mir(inner)))
            }
        }
        // v0.70: Thread type - represented as i64 handle
        Type::Thread(_) => MirType::I64,
        // v0.71: Mutex type - represented as i64 handle
        Type::Mutex(_) => MirType::I64,
        // v0.72: Arc and Atomic types - represented as i64 handle
        Type::Arc(_) => MirType::I64,
        Type::Atomic(_) => MirType::I64,
        // v0.73: Sender and Receiver types - represented as i64 handle
        Type::Sender(_) => MirType::I64,
        Type::Receiver(_) => MirType::I64,
    }
}

/// v0.60.261: Substitute type variables in a type with concrete types
/// Used for generic struct/enum monomorphization
fn substitute_type_vars(ty: &Type, substitutions: &std::collections::HashMap<&str, &Type>) -> Type {
    match ty {
        Type::TypeVar(name) => {
            if let Some(&concrete) = substitutions.get(name.as_str()) {
                concrete.clone()
            } else {
                ty.clone()
            }
        }
        Type::Generic { name, type_args } => {
            // Recursively substitute in type arguments
            Type::Generic {
                name: name.clone(),
                type_args: type_args.iter()
                    .map(|arg| Box::new(substitute_type_vars(arg, substitutions)))
                    .collect(),
            }
        }
        Type::Ptr(inner) => Type::Ptr(Box::new(substitute_type_vars(inner, substitutions))),
        Type::Ref(inner) => Type::Ref(Box::new(substitute_type_vars(inner, substitutions))),
        Type::RefMut(inner) => Type::RefMut(Box::new(substitute_type_vars(inner, substitutions))),
        Type::Array(elem, size) => Type::Array(Box::new(substitute_type_vars(elem, substitutions)), *size),
        Type::Nullable(inner) => Type::Nullable(Box::new(substitute_type_vars(inner, substitutions))),
        Type::Tuple(elems) => Type::Tuple(
            elems.iter().map(|e| Box::new(substitute_type_vars(e, substitutions))).collect()
        ),
        // Primitive types and others don't need substitution
        _ => ty.clone(),
    }
}

/// v0.60.261: Convert a type to a suffix string for monomorphized names
/// e.g., i64 -> "i64", Pair<i64, i64> -> "Pair_i64_i64"
fn type_to_suffix(ty: &Type) -> String {
    match ty {
        Type::I32 => "i32".to_string(),
        Type::I64 => "i64".to_string(),
        Type::U32 => "u32".to_string(),
        Type::U64 => "u64".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::String => "str".to_string(),
        Type::Char => "char".to_string(),
        Type::Unit => "unit".to_string(),
        Type::Named(name) => name.clone(),
        Type::TypeVar(name) => name.clone(),
        Type::Generic { name, type_args } => {
            format!("{}_{}", name,
                type_args.iter()
                    .map(|t| type_to_suffix(t))
                    .collect::<Vec<_>>()
                    .join("_"))
        }
        Type::Ptr(inner) => format!("ptr_{}", type_to_suffix(inner)),
        Type::Ref(inner) | Type::RefMut(inner) => format!("ref_{}", type_to_suffix(inner)),
        Type::Array(elem, size) => format!("arr{}_{}", size, type_to_suffix(elem)),
        Type::Nullable(inner) => format!("opt_{}", type_to_suffix(inner)),
        Type::Tuple(elems) => format!("tup_{}",
            elems.iter().map(|e| type_to_suffix(e)).collect::<Vec<_>>().join("_")),
        _ => "unknown".to_string(),
    }
}

/// v0.60.254: Convert AST type to MIR type with struct AND enum definition lookup
/// This properly converts Type::Named to MirType::Struct or MirType::Enum when the name is known
fn ast_type_to_mir_with_type_defs(
    ty: &Type,
    type_defs: &TypeDefs,
) -> MirType {
    match ty {
        Type::I32 => MirType::I32,
        Type::I64 => MirType::I64,
        Type::U32 => MirType::U32,
        Type::U64 => MirType::U64,
        Type::F64 => MirType::F64,
        Type::Bool => MirType::Bool,
        Type::String => MirType::String,
        Type::Char => MirType::Char,
        Type::Unit => MirType::Unit,
        Type::Range(elem) => ast_type_to_mir_with_type_defs(elem, type_defs),
        // v0.51.24: Named types that are structs get converted to MirType::Struct
        // v0.60.254: Also check for enum types
        Type::Named(name) => {
            if let Some(fields) = type_defs.structs.get(name) {
                MirType::Struct {
                    name: name.clone(),
                    fields: fields
                        .iter()
                        .map(|(fname, fty)| (fname.clone(), Box::new(ast_type_to_mir_with_type_defs(fty, type_defs))))
                        .collect(),
                }
            } else if let Some(variants) = type_defs.enums.get(name) {
                // v0.60.254: Named enum types get converted to MirType::Enum
                MirType::Enum {
                    name: name.clone(),
                    variants: variants
                        .iter()
                        .map(|(vname, vtypes)| {
                            (vname.clone(), vtypes.iter().map(|t| Box::new(ast_type_to_mir_with_type_defs(t, type_defs))).collect())
                        })
                        .collect(),
                }
            } else {
                // Not a known struct or enum, fall back to i64
                MirType::I64
            }
        }
        Type::TypeVar(_) => MirType::I64,
        // v0.60.261: Handle generic type instantiation (e.g., Pair<i64, i64>)
        // Look up the base struct definition and substitute type arguments
        Type::Generic { name, type_args } => {
            if let Some(fields) = type_defs.structs.get(name) {
                // Get type parameter names for this generic struct
                let param_names = type_defs.struct_type_params.get(name)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);

                // Create a mapping from type parameter name to concrete type
                let substitutions: std::collections::HashMap<&str, &Type> = param_names.iter()
                    .zip(type_args.iter())
                    .map(|(param, arg)| (param.as_str(), arg.as_ref()))
                    .collect();

                // Substitute type arguments in field types
                let substituted_fields: Vec<(String, Box<MirType>)> = fields.iter()
                    .map(|(fname, fty)| {
                        let concrete_ty = substitute_type_vars(fty, &substitutions);
                        (fname.clone(), Box::new(ast_type_to_mir_with_type_defs(&concrete_ty, type_defs)))
                    })
                    .collect();

                // Generate a monomorphized name (e.g., "Pair_i64_i64")
                let mono_name = format!("{}_{}", name,
                    type_args.iter()
                        .map(|t| type_to_suffix(t))
                        .collect::<Vec<_>>()
                        .join("_"));

                MirType::Struct {
                    name: mono_name,
                    fields: substituted_fields,
                }
            } else if let Some(variants) = type_defs.enums.get(name) {
                // v0.60.261: Also handle generic enums (e.g., Option<i64>)
                let param_names = type_defs.struct_type_params.get(name)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);

                let substitutions: std::collections::HashMap<&str, &Type> = param_names.iter()
                    .zip(type_args.iter())
                    .map(|(param, arg)| (param.as_str(), arg.as_ref()))
                    .collect();

                let substituted_variants: Vec<(String, Vec<Box<MirType>>)> = variants.iter()
                    .map(|(vname, vtypes)| {
                        let concrete_types: Vec<Box<MirType>> = vtypes.iter()
                            .map(|vt| {
                                let concrete_ty = substitute_type_vars(vt, &substitutions);
                                Box::new(ast_type_to_mir_with_type_defs(&concrete_ty, type_defs))
                            })
                            .collect();
                        (vname.clone(), concrete_types)
                    })
                    .collect();

                let mono_name = format!("{}_{}", name,
                    type_args.iter()
                        .map(|t| type_to_suffix(t))
                        .collect::<Vec<_>>()
                        .join("_"));

                MirType::Enum {
                    name: mono_name,
                    variants: substituted_variants,
                }
            } else {
                // Unknown generic type, fall back to i64
                MirType::I64
            }
        }
        Type::Struct { name, fields } => MirType::Struct {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(fname, fty)| (fname.clone(), Box::new(ast_type_to_mir_with_type_defs(fty, type_defs))))
                .collect(),
        },
        Type::Enum { name, variants } => MirType::Enum {
            name: name.clone(),
            variants: variants
                .iter()
                .map(|(vname, vtypes)| {
                    (vname.clone(), vtypes.iter().map(|t| Box::new(ast_type_to_mir_with_type_defs(t, type_defs))).collect())
                })
                .collect(),
        },
        Type::Ref(inner) | Type::RefMut(inner) => {
            if let Type::Array(elem, size) = inner.as_ref() {
                MirType::Array {
                    element_type: Box::new(ast_type_to_mir_with_type_defs(elem, type_defs)),
                    size: Some(*size),
                }
            } else {
                MirType::I64
            }
        }
        Type::Array(elem, size) => MirType::Array {
            element_type: Box::new(ast_type_to_mir_with_type_defs(elem, type_defs)),
            size: Some(*size),
        },
        Type::Refined { base, .. } => ast_type_to_mir_with_type_defs(base, type_defs),
        Type::Fn { .. } => MirType::I64,
        Type::Never => MirType::Unit,
        Type::Nullable(inner) => ast_type_to_mir_with_type_defs(inner, type_defs),
        // v0.55: Tuple type - native heterogeneous tuple support
        Type::Tuple(elems) => MirType::Tuple(
            elems.iter().map(|e| Box::new(ast_type_to_mir_with_type_defs(e, type_defs))).collect()
        ),
        // v0.51.37: Pointer type - preserve the pointee type for proper codegen
        // For struct pointers, use StructPtr to avoid infinite recursion on self-referential types
        Type::Ptr(inner) => {
            match inner.as_ref() {
                Type::Named(name) if type_defs.structs.contains_key(name) => {
                    MirType::Ptr(Box::new(MirType::StructPtr(name.clone())))
                }
                _ => MirType::Ptr(Box::new(ast_type_to_mir_with_type_defs(inner, type_defs)))
            }
        }
        // v0.70: Thread type - represented as i64 handle
        Type::Thread(_) => MirType::I64,
        // v0.71: Mutex type - represented as i64 handle
        Type::Mutex(_) => MirType::I64,
        // v0.72: Arc and Atomic types - represented as i64 handle
        Type::Arc(_) => MirType::I64,
        Type::Atomic(_) => MirType::I64,
        // v0.73: Sender and Receiver types - represented as i64 handle
        Type::Sender(_) => MirType::I64,
        Type::Receiver(_) => MirType::I64,
    }
}

/// v0.51.24: Convert AST type to MIR type with struct definition lookup
/// This properly converts Type::Named to MirType::Struct when the name is a known struct
fn ast_type_to_mir_with_structs(
    ty: &Type,
    struct_type_defs: &std::collections::HashMap<String, Vec<(String, Type)>>,
) -> MirType {
    match ty {
        Type::I32 => MirType::I32,
        Type::I64 => MirType::I64,
        Type::U32 => MirType::U32,
        Type::U64 => MirType::U64,
        Type::F64 => MirType::F64,
        Type::Bool => MirType::Bool,
        Type::String => MirType::String,
        Type::Char => MirType::Char,
        Type::Unit => MirType::Unit,
        Type::Range(elem) => ast_type_to_mir_with_structs(elem, struct_type_defs),
        // v0.51.24: Named types that are structs get converted to MirType::Struct
        Type::Named(name) => {
            if let Some(fields) = struct_type_defs.get(name) {
                MirType::Struct {
                    name: name.clone(),
                    fields: fields
                        .iter()
                        .map(|(fname, fty)| (fname.clone(), Box::new(ast_type_to_mir_with_structs(fty, struct_type_defs))))
                        .collect(),
                }
            } else {
                // Not a known struct, fall back to i64
                MirType::I64
            }
        }
        Type::TypeVar(_) => MirType::I64,
        Type::Generic { .. } => MirType::I64,
        Type::Struct { name, fields } => MirType::Struct {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(fname, fty)| (fname.clone(), Box::new(ast_type_to_mir_with_structs(fty, struct_type_defs))))
                .collect(),
        },
        Type::Enum { name, variants } => MirType::Enum {
            name: name.clone(),
            variants: variants
                .iter()
                .map(|(vname, vtypes)| {
                    (vname.clone(), vtypes.iter().map(|t| Box::new(ast_type_to_mir_with_structs(t, struct_type_defs))).collect())
                })
                .collect(),
        },
        Type::Ref(inner) | Type::RefMut(inner) => {
            if let Type::Array(elem, size) = inner.as_ref() {
                MirType::Array {
                    element_type: Box::new(ast_type_to_mir_with_structs(elem, struct_type_defs)),
                    size: Some(*size),
                }
            } else {
                MirType::I64
            }
        }
        Type::Array(elem, size) => MirType::Array {
            element_type: Box::new(ast_type_to_mir_with_structs(elem, struct_type_defs)),
            size: Some(*size),
        },
        Type::Refined { base, .. } => ast_type_to_mir_with_structs(base, struct_type_defs),
        Type::Fn { .. } => MirType::I64,
        Type::Never => MirType::Unit,
        Type::Nullable(inner) => ast_type_to_mir_with_structs(inner, struct_type_defs),
        // v0.55: Tuple type - native heterogeneous tuple support
        Type::Tuple(elems) => MirType::Tuple(
            elems.iter().map(|e| Box::new(ast_type_to_mir_with_structs(e, struct_type_defs))).collect()
        ),
        // v0.51.37: Pointer type - preserve the pointee type for proper codegen
        // For struct pointers, use StructPtr to avoid infinite recursion on self-referential types
        Type::Ptr(inner) => {
            match inner.as_ref() {
                Type::Named(name) if struct_type_defs.contains_key(name) => {
                    MirType::Ptr(Box::new(MirType::StructPtr(name.clone())))
                }
                _ => MirType::Ptr(Box::new(ast_type_to_mir_with_structs(inner, struct_type_defs)))
            }
        }
        // v0.70: Thread type - represented as i64 handle
        Type::Thread(_) => MirType::I64,
        // v0.71: Mutex type - represented as i64 handle
        Type::Mutex(_) => MirType::I64,
        // v0.72: Arc and Atomic types - represented as i64 handle
        Type::Arc(_) => MirType::I64,
        Type::Atomic(_) => MirType::I64,
        // v0.73: Sender and Receiver types - represented as i64 handle
        Type::Sender(_) => MirType::I64,
        Type::Receiver(_) => MirType::I64,
    }
}

/// Convert AST binary operator to MIR operator
fn ast_binop_to_mir(op: BinOp, ty: &MirType) -> MirBinOp {
    match (op, ty.is_float()) {
        (BinOp::Add, false) => MirBinOp::Add,
        (BinOp::Add, true) => MirBinOp::FAdd,
        (BinOp::Sub, false) => MirBinOp::Sub,
        (BinOp::Sub, true) => MirBinOp::FSub,
        (BinOp::Mul, false) => MirBinOp::Mul,
        (BinOp::Mul, true) => MirBinOp::FMul,
        (BinOp::Div, false) => MirBinOp::Div,
        (BinOp::Div, true) => MirBinOp::FDiv,
        (BinOp::Mod, _) => MirBinOp::Mod,
        // v0.37: Wrapping arithmetic (integer only)
        (BinOp::AddWrap, _) => MirBinOp::AddWrap,
        (BinOp::SubWrap, _) => MirBinOp::SubWrap,
        (BinOp::MulWrap, _) => MirBinOp::MulWrap,
        // v0.38: Checked arithmetic (integer only)
        (BinOp::AddChecked, _) => MirBinOp::AddChecked,
        (BinOp::SubChecked, _) => MirBinOp::SubChecked,
        (BinOp::MulChecked, _) => MirBinOp::MulChecked,
        // v0.38: Saturating arithmetic (integer only)
        (BinOp::AddSat, _) => MirBinOp::AddSat,
        (BinOp::SubSat, _) => MirBinOp::SubSat,
        (BinOp::MulSat, _) => MirBinOp::MulSat,
        (BinOp::Eq, false) => MirBinOp::Eq,
        (BinOp::Eq, true) => MirBinOp::FEq,
        (BinOp::Ne, false) => MirBinOp::Ne,
        (BinOp::Ne, true) => MirBinOp::FNe,
        (BinOp::Lt, false) => MirBinOp::Lt,
        (BinOp::Lt, true) => MirBinOp::FLt,
        (BinOp::Gt, false) => MirBinOp::Gt,
        (BinOp::Gt, true) => MirBinOp::FGt,
        (BinOp::Le, false) => MirBinOp::Le,
        (BinOp::Le, true) => MirBinOp::FLe,
        (BinOp::Ge, false) => MirBinOp::Ge,
        (BinOp::Ge, true) => MirBinOp::FGe,
        (BinOp::And, _) => MirBinOp::And,
        (BinOp::Or, _) => MirBinOp::Or,
        // v0.32: Shift operators (integer only)
        (BinOp::Shl, _) => MirBinOp::Shl,
        (BinOp::Shr, _) => MirBinOp::Shr,
        // v0.36: Bitwise operators (integer only)
        (BinOp::Band, _) => MirBinOp::Band,
        (BinOp::Bor, _) => MirBinOp::Bor,
        (BinOp::Bxor, _) => MirBinOp::Bxor,
        // v0.36: Logical implication
        (BinOp::Implies, _) => MirBinOp::Implies,
    }
}

/// Convert AST unary operator to MIR operator
fn ast_unop_to_mir(op: UnOp, ty: &MirType) -> MirUnaryOp {
    match (op, ty.is_float()) {
        (UnOp::Neg, false) => MirUnaryOp::Neg,
        (UnOp::Neg, true) => MirUnaryOp::FNeg,
        (UnOp::Not, _) => MirUnaryOp::Not,
        // v0.36: Bitwise not (integer only)
        (UnOp::Bnot, _) => MirUnaryOp::Bnot,
    }
}

// v0.19.2: Pattern matching helper functions

/// Compile match patterns to switch cases
/// Returns (cases, wildcard_arm_index):
///   - cases: list of (discriminant_value, target_label) pairs
///   - wildcard_arm_index: Some(index) if there's a wildcard/var arm, None otherwise
fn compile_match_patterns(
    arms: &[MatchArm],
    arm_labels: &[String],
    _default_label: &str,
) -> (Vec<(i64, String)>, Option<usize>) {
    let mut cases = Vec::new();
    let mut wildcard_arm_index: Option<usize> = None;

    for (i, arm) in arms.iter().enumerate() {
        match &arm.pattern.node {
            Pattern::Literal(lit) => {
                let value = match lit {
                    LiteralPattern::Int(n) => *n,
                    LiteralPattern::Bool(b) => if *b { 1 } else { 0 },
                    LiteralPattern::Float(f) => *f as i64, // Lossy but necessary for switch
                    LiteralPattern::String(_) => i as i64, // Use index as placeholder
                };
                cases.push((value, arm_labels[i].clone()));
            }
            Pattern::EnumVariant { variant, .. } => {
                // For enum variants, use a deterministic discriminant based on variant name
                // In a real compiler, this would come from the type info
                let disc = variant_to_discriminant(variant);
                cases.push((disc, arm_labels[i].clone()));
            }
            Pattern::Wildcard | Pattern::Var(_) => {
                // v0.60.262: Wildcard/var patterns catch all - record the arm index
                // This arm's label will be used as the switch default target
                wildcard_arm_index = Some(i);
            }
            Pattern::Struct { .. } => {
                // Struct patterns need field matching - for now, use index
                cases.push((i as i64, arm_labels[i].clone()));
            }
            // v0.39: Range pattern
            Pattern::Range { .. } => {
                // Range patterns need runtime checks - for now, use index
                cases.push((i as i64, arm_labels[i].clone()));
            }
            // v0.40: Or-pattern
            Pattern::Or(_) => {
                // Or-patterns need to try each alternative - for now, use index
                cases.push((i as i64, arm_labels[i].clone()));
            }
            // v0.41: Binding pattern - treated like the inner pattern for switching
            Pattern::Binding { pattern, .. } => {
                // Delegate to inner pattern's logic - for now, use index
                match &pattern.node {
                    Pattern::Wildcard | Pattern::Var(_) => {
                        // v0.60.262: Record wildcard arm index for binding patterns too
                        wildcard_arm_index = Some(i);
                    }
                    _ => {
                        cases.push((i as i64, arm_labels[i].clone()));
                    }
                }
            }
            // v0.42: Tuple pattern - use index for now
            Pattern::Tuple(_) => {
                cases.push((i as i64, arm_labels[i].clone()));
            }
            // v0.44: Array pattern - use index for now
            Pattern::Array(_) => {
                cases.push((i as i64, arm_labels[i].clone()));
            }
            // v0.45: Array rest pattern - use index for now
            Pattern::ArrayRest { .. } => {
                cases.push((i as i64, arm_labels[i].clone()));
            }
        }
    }

    (cases, wildcard_arm_index)
}

/// Convert variant name to discriminant value
fn variant_to_discriminant(variant: &str) -> i64 {
    // Simple hash-based discriminant for now
    // In a full implementation, this would use the enum definition order
    let mut hash: i64 = 0;
    for (i, c) in variant.chars().enumerate() {
        hash = hash.wrapping_add((c as i64).wrapping_mul((i + 1) as i64));
    }
    hash
}

/// Bind pattern variables to values extracted from the match expression
fn bind_pattern_variables(pattern: &Pattern, match_place: &Place, ctx: &mut LoweringContext) {
    match pattern {
        Pattern::Var(name) => {
            // Create a copy instruction to bind the variable
            let var_place = Place::new(name.clone());
            ctx.push_inst(MirInst::Copy {
                dest: var_place.clone(),
                src: match_place.clone(),
            });
            // Register the variable type (infer from match place or default to i64)
            if let Some(ty) = ctx.locals.get(&match_place.name).cloned() {
                ctx.locals.insert(name.clone(), ty);
            } else if let Some(ty) = ctx.params.get(&match_place.name).cloned() {
                ctx.locals.insert(name.clone(), ty);
            } else {
                ctx.locals.insert(name.clone(), MirType::I64);
            }
        }
        // v0.41: Nested patterns in enum bindings
        // v0.51.31: Added struct_name (empty for enum variant tuple-like fields)
        Pattern::EnumVariant { bindings, .. } => {
            // For enum variants with bindings, extract fields
            for (i, binding) in bindings.iter().enumerate() {
                let field_place = ctx.fresh_temp();
                // Use field access to extract (simplified - real impl needs tag/data extraction)
                // v0.51.23: field_index = i for enum variant tuple-like fields
                ctx.push_inst(MirInst::FieldAccess {
                    dest: field_place.clone(),
                    base: match_place.clone(),
                    field: format!("_{}", i), // Tuple-like access
                    field_index: i,
                    struct_name: String::new(), // Enum variants don't have struct names
                });
                // Recursively bind inner patterns
                bind_pattern_variables(&binding.node, &field_place, ctx);
            }
        }
        Pattern::Struct { name, fields } => {
            // For struct patterns, bind field patterns
            // v0.51.31: Added struct_name for field type lookup
            for (field_name, field_pattern) in fields {
                let field_place = ctx.fresh_temp();
                // v0.51.23: Compute field index from struct definition
                let field_index = ctx.field_index(name, &field_name.node);
                ctx.push_inst(MirInst::FieldAccess {
                    dest: field_place.clone(),
                    base: match_place.clone(),
                    field: field_name.node.clone(),
                    field_index,
                    struct_name: name.clone(),
                });
                // Recursively bind inner patterns
                bind_pattern_variables(&field_pattern.node, &field_place, ctx);
            }
        }
        Pattern::Wildcard | Pattern::Literal(_) | Pattern::Range { .. } | Pattern::Or(_) => {
            // No bindings for wildcards, literals, ranges, or or-patterns
            // Note: Or-patterns with bindings would need special handling
        }
        // v0.41: Binding pattern: name @ pattern
        Pattern::Binding { name, pattern } => {
            // Bind the name to the entire value
            let binding_place = Place::new(name.clone());
            ctx.push_inst(MirInst::Copy {
                dest: binding_place.clone(),
                src: match_place.clone(),
            });
            // Register the variable type
            if let Some(ty) = ctx.locals.get(&match_place.name).cloned() {
                ctx.locals.insert(name.clone(), ty);
            } else if let Some(ty) = ctx.params.get(&match_place.name).cloned() {
                ctx.locals.insert(name.clone(), ty);
            } else {
                ctx.locals.insert(name.clone(), MirType::I64);
            }
            // Recursively bind inner pattern
            bind_pattern_variables(&pattern.node, match_place, ctx);
        }
        // v0.42: Tuple pattern - bind each element
        Pattern::Tuple(patterns) => {
            for (i, elem_pattern) in patterns.iter().enumerate() {
                // Create a place for tuple element access (synthesized name)
                let elem_place = Place::new(format!("{}.{}", match_place.name, i));
                bind_pattern_variables(&elem_pattern.node, &elem_place, ctx);
            }
        }
        // v0.44: Array pattern - bind each element
        Pattern::Array(patterns) => {
            for (i, elem_pattern) in patterns.iter().enumerate() {
                // Create a place for array element access (synthesized name)
                let elem_place = Place::new(format!("{}[{}]", match_place.name, i));
                bind_pattern_variables(&elem_pattern.node, &elem_place, ctx);
            }
        }
        // v0.45: Array rest pattern - bind prefix and suffix elements
        Pattern::ArrayRest { prefix, suffix } => {
            // Bind prefix elements from the start
            for (i, elem_pattern) in prefix.iter().enumerate() {
                let elem_place = Place::new(format!("{}[{}]", match_place.name, i));
                bind_pattern_variables(&elem_pattern.node, &elem_place, ctx);
            }
            // Bind suffix elements from the end (negative indexing conceptually)
            // In MIR, we'd need to compute the actual indices at runtime based on array length
            // For now, use symbolic suffix indices
            for (i, elem_pattern) in suffix.iter().enumerate() {
                let elem_place = Place::new(format!("{}[end-{}]", match_place.name, suffix.len() - i));
                bind_pattern_variables(&elem_pattern.node, &elem_place, ctx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Param, Span, Spanned, Visibility};

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned {
            node,
            span: Span { start: 0, end: 0 },
        }
    }

    #[test]
    fn test_lower_simple_function() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("add".to_string()),
                type_params: vec![],
                params: vec![
                    Param {
                        name: spanned("a".to_string()),
                        ty: spanned(Type::I64),
                    },
                    Param {
                        name: spanned("b".to_string()),
                        ty: spanned(Type::I64),
                    },
                ],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::Binary {
                    left: Box::new(spanned(Expr::Var("a".to_string()))),
                    op: BinOp::Add,
                    right: Box::new(spanned(Expr::Var("b".to_string()))),
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        assert_eq!(mir.functions.len(), 1);

        let func = &mir.functions[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.blocks.len(), 1);

        // Should have one BinOp instruction and a Return terminator
        let block = &func.blocks[0];
        assert_eq!(block.instructions.len(), 1);
        assert!(matches!(block.instructions[0], MirInst::BinOp { .. }));
        assert!(matches!(block.terminator, Terminator::Return(_)));
    }

    #[test]
    fn test_lower_if_expression() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("max".to_string()),
                type_params: vec![],
                params: vec![
                    Param {
                        name: spanned("a".to_string()),
                        ty: spanned(Type::I64),
                    },
                    Param {
                        name: spanned("b".to_string()),
                        ty: spanned(Type::I64),
                    },
                ],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::If {
                    cond: Box::new(spanned(Expr::Binary {
                        left: Box::new(spanned(Expr::Var("a".to_string()))),
                        op: BinOp::Gt,
                        right: Box::new(spanned(Expr::Var("b".to_string()))),
                    })),
                    then_branch: Box::new(spanned(Expr::Var("a".to_string()))),
                    else_branch: Box::new(spanned(Expr::Var("b".to_string()))),
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have 4 blocks: entry, then, else, merge
        assert_eq!(func.blocks.len(), 4);

        // Entry block should end with a Branch
        assert!(matches!(
            func.blocks[0].terminator,
            Terminator::Branch { .. }
        ));
    }

    #[test]
    fn test_lower_let_binding() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::Let {
                    name: "x".to_string(),
                    mutable: false,
                    ty: None,
                    value: Box::new(spanned(Expr::IntLit(42))),
                    body: Box::new(spanned(Expr::Var("x".to_string()))),
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have the local 'x' registered
        assert!(func.locals.iter().any(|(name, _)| name == "x"));
    }


    #[test]
    fn test_lower_string_literal() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::Let {
                    name: "s".to_string(),
                    mutable: false,
                    ty: None,
                    value: Box::new(spanned(Expr::StringLit("hello".to_string()))),
                    body: Box::new(spanned(Expr::IntLit(0))),
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have the local 's' registered with String type
        assert!(func.locals.iter().any(|(name, ty)| name == "s" && *ty == MirType::String));
    }

    #[test]
    fn test_lower_while_loop() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![],
                ret_name: None,
                ret_ty: spanned(Type::Unit),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::While {
                    cond: Box::new(spanned(Expr::BoolLit(false))),
                    invariant: None,  // v0.37: No invariant in test
                    body: Box::new(spanned(Expr::Unit)),
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have multiple blocks for while loop: entry, cond, body, exit
        assert!(func.blocks.len() >= 3);
    }

    // v0.19.0: Struct MIR tests
    #[test]
    fn test_lower_struct_init() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::StructInit {
                    name: "Point".to_string(),
                    fields: vec![
                        (spanned("x".to_string()), spanned(Expr::IntLit(10))),
                        (spanned("y".to_string()), spanned(Expr::IntLit(20))),
                    ],
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have StructInit instruction
        assert!(func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::StructInit { struct_name, .. } if struct_name == "Point")
        }));
    }

    #[test]
    fn test_lower_field_access() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![Param {
                    name: spanned("p".to_string()),
                    ty: spanned(Type::Named("Point".to_string())),
                }],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::FieldAccess {
                    expr: Box::new(spanned(Expr::Var("p".to_string()))),
                    field: spanned("x".to_string()),
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have FieldAccess instruction
        assert!(func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::FieldAccess { field, .. } if field == "x")
        }));
    }

    // v0.19.1: Enum MIR tests
    #[test]
    fn test_lower_enum_variant() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::EnumVariant {
                    enum_name: "Option".to_string(),
                    variant: "Some".to_string(),
                    args: vec![spanned(Expr::IntLit(42))],
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have EnumVariant instruction
        assert!(func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::EnumVariant { enum_name, variant, .. }
                     if enum_name == "Option" && variant == "Some")
        }));
    }

    #[test]
    fn test_lower_enum_unit_variant() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::EnumVariant {
                    enum_name: "Option".to_string(),
                    variant: "None".to_string(),
                    args: vec![],
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have EnumVariant instruction with empty args
        assert!(func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::EnumVariant { enum_name, variant, args, .. }
                     if enum_name == "Option" && variant == "None" && args.is_empty())
        }));
    }

    // v0.19.2: Pattern Matching MIR tests
    #[test]
    fn test_lower_match_literal() {
        use crate::ast::MatchArm;

        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![Param {
                    name: spanned("x".to_string()),
                    ty: spanned(Type::I64),
                }],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::Match {
                    expr: Box::new(spanned(Expr::Var("x".to_string()))),
                    arms: vec![
                        MatchArm {
                            pattern: spanned(Pattern::Literal(LiteralPattern::Int(0))),
                            guard: None,
                            body: spanned(Expr::IntLit(100)),
                        },
                        MatchArm {
                            pattern: spanned(Pattern::Literal(LiteralPattern::Int(1))),
                            guard: None,
                            body: spanned(Expr::IntLit(200)),
                        },
                        MatchArm {
                            pattern: spanned(Pattern::Wildcard),
                            guard: None,
                            body: spanned(Expr::IntLit(999)),
                        },
                    ],
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have multiple blocks for match arms
        assert!(func.blocks.len() >= 4); // entry, arm0, arm1, arm2, default, merge

        // Should have a Switch terminator in entry block
        assert!(matches!(func.blocks[0].terminator, Terminator::Switch { .. }));

        // Should have PHI instruction in merge block
        let has_phi = func.blocks.iter().any(|block| {
            block.instructions.iter().any(|inst| matches!(inst, MirInst::Phi { .. }))
        });
        assert!(has_phi);
    }

    #[test]
    fn test_lower_match_var_binding() {
        use crate::ast::MatchArm;

        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![Param {
                    name: spanned("x".to_string()),
                    ty: spanned(Type::I64),
                }],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::Match {
                    expr: Box::new(spanned(Expr::Var("x".to_string()))),
                    arms: vec![
                        MatchArm {
                            pattern: spanned(Pattern::Var("n".to_string())),
                            guard: None,
                            body: spanned(Expr::Binary {
                                left: Box::new(spanned(Expr::Var("n".to_string()))),
                                op: BinOp::Mul,
                                right: Box::new(spanned(Expr::IntLit(2))),
                            }),
                        },
                    ],
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have blocks for match
        assert!(func.blocks.len() >= 2);

        // Should have Copy instruction for binding 'n'
        let has_copy = func.blocks.iter().any(|block| {
            block.instructions.iter().any(|inst| {
                matches!(inst, MirInst::Copy { dest, .. } if dest.name == "n")
            })
        });
        assert!(has_copy);
    }

    // v0.19.3: Array MIR tests
    #[test]
    fn test_lower_array_init() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::ArrayLit(vec![
                    spanned(Expr::IntLit(1)),
                    spanned(Expr::IntLit(2)),
                    spanned(Expr::IntLit(3)),
                ])),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have ArrayInit instruction
        assert!(func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::ArrayInit { elements, .. } if elements.len() == 3)
        }));
    }

    #[test]
    fn test_lower_array_index() {
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![Param {
                    name: spanned("arr".to_string()),
                    ty: spanned(Type::Array(Box::new(Type::I64), 3)), // [i64; 3]
                }],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::Index {
                    expr: Box::new(spanned(Expr::Var("arr".to_string()))),
                    index: Box::new(spanned(Expr::IntLit(0))),
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have IndexLoad instruction
        assert!(func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::IndexLoad { array, .. } if array.name == "arr")
        }));
    }

    #[test]
    fn test_lower_method_call() {
        // Test: obj.method(arg) should lower to call method(obj, arg)
        let program = Program {
            header: None,
            items: vec![Item::FnDef(FnDef {
                attributes: vec![],
                visibility: Visibility::Private,
                name: spanned("test".to_string()),
                type_params: vec![],
                params: vec![Param {
                    name: spanned("obj".to_string()),
                    ty: spanned(Type::I64), // Simplified for testing
                }],
                ret_name: None,
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                contracts: vec![],
                body: spanned(Expr::MethodCall {
                    receiver: Box::new(spanned(Expr::Var("obj".to_string()))),
                    method: "double".to_string(),
                    args: vec![spanned(Expr::IntLit(10))],
                }),
                span: Span { start: 0, end: 0 },
            })],
        };

        let mir = lower_program(&program);
        let func = &mir.functions[0];

        // Should have Call instruction with method name "double"
        let has_call = func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, MirInst::Call { func: f, args, .. }
                if f == "double" && args.len() == 2) // receiver + 1 arg
        });
        assert!(has_call, "Expected Call instruction for method 'double' with 2 args");
    }
}
