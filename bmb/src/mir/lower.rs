//! AST to MIR lowering
//!
//! This module converts the high-level AST into MIR by:
//! - Flattening nested expressions into sequences of instructions
//! - Making control flow explicit through basic blocks
//! - Converting operators based on operand types

use crate::ast::{Attribute, BinOp, Expr, FnDef, Item, Program, Spanned, Type, UnOp};

use super::{
    Constant, LoweringContext, MirBinOp, MirExternFn, MirFunction, MirInst, MirProgram, MirType,
    MirUnaryOp, Operand, Place, Terminator,
};

/// Lower an entire program to MIR
pub fn lower_program(program: &Program) -> MirProgram {
    let functions = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::FnDef(fn_def) => Some(lower_function(fn_def)),
            // Type definitions, use statements, and extern fns don't produce MIR functions
            Item::StructDef(_) | Item::EnumDef(_) | Item::Use(_) | Item::ExternFn(_) => None,
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

    MirProgram {
        functions,
        extern_fns,
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
fn lower_function(fn_def: &FnDef) -> MirFunction {
    let mut ctx = LoweringContext::new();

    // Register parameters
    let params: Vec<(String, MirType)> = fn_def
        .params
        .iter()
        .map(|p| {
            let ty = ast_type_to_mir(&p.ty.node);
            ctx.params.insert(p.name.node.clone(), ty.clone());
            (p.name.node.clone(), ty)
        })
        .collect();

    let ret_ty = ast_type_to_mir(&fn_def.ret_ty.node);

    // Lower the function body
    let result = lower_expr(&fn_def.body, &mut ctx);

    // Finish with a return
    ctx.finish_block(Terminator::Return(Some(result)));

    // Collect locals
    let locals: Vec<(String, MirType)> = ctx.locals.clone().into_iter().collect();

    MirFunction {
        name: fn_def.name.node.clone(),
        params,
        ret_ty,
        locals,
        blocks: ctx.blocks,
    }
}

/// Lower an expression, returning the operand holding its result
fn lower_expr(expr: &Spanned<Expr>, ctx: &mut LoweringContext) -> Operand {
    match &expr.node {
        Expr::IntLit(n) => Operand::Constant(Constant::Int(*n)),

        Expr::FloatLit(f) => Operand::Constant(Constant::Float(*f)),

        Expr::BoolLit(b) => Operand::Constant(Constant::Bool(*b)),

        Expr::StringLit(s) => Operand::Constant(Constant::String(s.clone())),

        Expr::Unit => Operand::Constant(Constant::Unit),

        Expr::Var(name) => Operand::Place(Place::new(name.clone())),

        Expr::Binary { left, op, right } => {
            let lhs = lower_expr(left, ctx);
            let rhs = lower_expr(right, ctx);
            let dest = ctx.fresh_temp();

            // Determine the MIR operator based on operand types
            let lhs_ty = ctx.operand_type(&lhs);
            let mir_op = ast_binop_to_mir(*op, &lhs_ty);

            ctx.push_inst(MirInst::BinOp {
                dest: dest.clone(),
                op: mir_op,
                lhs,
                rhs,
            });

            Operand::Place(dest)
        }

        Expr::Unary { op, expr: inner } => {
            let src = lower_expr(inner, ctx);
            let dest = ctx.fresh_temp();

            let src_ty = ctx.operand_type(&src);
            let mir_op = ast_unop_to_mir(*op, &src_ty);

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
                    ctx.push_inst(MirInst::Copy {
                        dest: var_place,
                        src,
                    });
                }
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

        Expr::While { cond, body } => {
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

            // Body block
            ctx.start_block(body_label);
            let _ = lower_expr(body, ctx);
            ctx.finish_block(Terminator::Goto(cond_label));

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
                });
                Operand::Constant(Constant::Unit)
            } else {
                let dest = ctx.fresh_temp();
                ctx.push_inst(MirInst::Call {
                    dest: Some(dest.clone()),
                    func: func.clone(),
                    args: arg_ops,
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

        // v0.5: Struct and Enum expressions - basic stubs for now
        Expr::StructInit { .. } => {
            // TODO: Implement struct initialization in MIR
            Operand::Constant(Constant::Unit)
        }

        Expr::FieldAccess { .. } => {
            // TODO: Implement field access in MIR
            Operand::Constant(Constant::Unit)
        }

        Expr::EnumVariant { .. } => {
            // TODO: Implement enum variant construction in MIR
            Operand::Constant(Constant::Unit)
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
            // Basic implementation: evaluate first matching arm
            // TODO: Full pattern matching compilation
            if arms.is_empty() {
                return Operand::Constant(Constant::Unit);
            }

            // For now, just evaluate the expression and return first arm body
            // This is a simplified stub - full implementation requires jump tables
            let _match_val = lower_expr(expr, ctx);

            // Return first arm's body (simplified)
            if let Some(first_arm) = arms.first() {
                lower_expr(&first_arm.body, ctx)
            } else {
                Operand::Constant(Constant::Unit)
            }
        }

        // v0.5 Phase 5: References (simplified - just evaluate inner)
        Expr::Ref(inner) | Expr::RefMut(inner) => {
            lower_expr(inner, ctx)
        }

        Expr::Deref(inner) => {
            lower_expr(inner, ctx)
        }

        // v0.5 Phase 6: Arrays (simplified - return unit for now)
        Expr::ArrayLit(_elems) => {
            // TODO: Full array support with memory allocation
            Operand::Constant(Constant::Unit)
        }

        Expr::Index { expr, index } => {
            // Simplified: just evaluate both and return a placeholder
            let _arr = lower_expr(expr, ctx);
            let _idx = lower_expr(index, ctx);
            // TODO: Actual array indexing with bounds checking
            Operand::Constant(Constant::Int(0))
        }

        // v0.5 Phase 8: Method calls (simplified - evaluate receiver and args)
        Expr::MethodCall { receiver, method: _, args } => {
            let _recv = lower_expr(receiver, ctx);
            for arg in args {
                let _ = lower_expr(arg, ctx);
            }
            // TODO: Full method call support with runtime dispatch
            Operand::Constant(Constant::Int(0))
        }

        // v0.2: State references (handled during contract verification, not MIR)
        Expr::StateRef { expr, .. } => {
            // During MIR lowering, we just evaluate the expression
            // The .pre/.post semantics are handled by the SMT translator
            lower_expr(expr, ctx)
        }

        // v0.2: Refinement self-reference (translated to __it__ variable)
        Expr::It => Operand::Place(Place::new("__it__")),

        // v0.13.2: Try block - lower the body expression
        Expr::Try { body } => lower_expr(body, ctx),

        // v0.13.2: Question mark operator - lower the inner expression
        // Full error propagation semantics will be added with Result type support
        Expr::Question { expr: inner } => lower_expr(inner, ctx),
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
        Type::F64 => MirType::F64,
        Type::Bool => MirType::Bool,
        Type::String => MirType::String,
        Type::Unit => MirType::Unit,
        Type::Range(elem) => ast_type_to_mir(elem), // Range represented by its element type
        Type::Named(_) => MirType::I64, // Named types default to pointer-sized int for now
        // v0.13.1: Type variables are unresolved, treat as opaque (pointer-sized)
        Type::TypeVar(_) => MirType::I64,
        // v0.13.1: Generic types are treated as their container (pointer-sized for now)
        Type::Generic { .. } => MirType::I64,
        Type::Struct { .. } => MirType::I64, // Struct types treated as pointers
        Type::Enum { .. } => MirType::I64, // Enum types treated as tagged unions
        // v0.5 Phase 5: References are pointers
        Type::Ref(_) | Type::RefMut(_) => MirType::I64,
        // v0.5 Phase 6: Arrays are pointers to data
        Type::Array(_, _) => MirType::I64,
        // v0.2: Refined types use base type
        Type::Refined { base, .. } => ast_type_to_mir(base),
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
    }
}

/// Convert AST unary operator to MIR operator
fn ast_unop_to_mir(op: UnOp, ty: &MirType) -> MirUnaryOp {
    match (op, ty.is_float()) {
        (UnOp::Neg, false) => MirUnaryOp::Neg,
        (UnOp::Neg, true) => MirUnaryOp::FNeg,
        (UnOp::Not, _) => MirUnaryOp::Not,
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
}
