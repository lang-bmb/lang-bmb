//! AST to MIR lowering
//!
//! This module converts the high-level AST into MIR by:
//! - Flattening nested expressions into sequences of instructions
//! - Making control flow explicit through basic blocks
//! - Converting operators based on operand types

use crate::ast::{BinOp, Expr, FnDef, Item, Program, Spanned, Type, UnOp};

use super::{
    Constant, LoweringContext, MirBinOp, MirFunction, MirInst, MirProgram, MirType, MirUnaryOp,
    Operand, Place, Terminator,
};

/// Lower an entire program to MIR
pub fn lower_program(program: &Program) -> MirProgram {
    let functions = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::FnDef(fn_def) => Some(lower_function(fn_def)),
            Item::StructDef(_) | Item::EnumDef(_) => None, // Type definitions don't produce MIR functions
        })
        .collect();

    MirProgram { functions }
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

            // Result place for the if expression
            let result = ctx.fresh_temp();

            // Branch based on condition
            ctx.finish_block(Terminator::Branch {
                cond: cond_op,
                then_label: then_label.clone(),
                else_label: else_label.clone(),
            });

            // Then block
            ctx.start_block(then_label);
            let then_result = lower_expr(then_branch, ctx);
            let then_src = operand_to_place(then_result, ctx);
            ctx.push_inst(MirInst::Copy {
                dest: result.clone(),
                src: then_src,
            });
            ctx.finish_block(Terminator::Goto(merge_label.clone()));

            // Else block
            ctx.start_block(else_label);
            let else_result = lower_expr(else_branch, ctx);
            let else_src = operand_to_place(else_result, ctx);
            ctx.push_inst(MirInst::Copy {
                dest: result.clone(),
                src: else_src,
            });
            ctx.finish_block(Terminator::Goto(merge_label.clone()));

            // Merge block
            ctx.start_block(merge_label);

            Operand::Place(result)
        }

        Expr::Let {
            name,
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

        Expr::Call { func, args } => {
            // Lower arguments
            let arg_ops: Vec<Operand> = args.iter().map(|arg| lower_expr(arg, ctx)).collect();

            // For now, assume all calls return i64 (we'd need type info for better handling)
            let dest = ctx.fresh_temp();

            ctx.push_inst(MirInst::Call {
                dest: Some(dest.clone()),
                func: func.clone(),
                args: arg_ops,
            });

            Operand::Place(dest)
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
        Type::Unit => MirType::Unit,
        Type::Named(_) => MirType::I64, // Named types default to pointer-sized int for now
        Type::Struct { .. } => MirType::I64, // Struct types treated as pointers
        Type::Enum { .. } => MirType::I64, // Enum types treated as tagged unions
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
    use crate::ast::{Param, Span, Spanned};

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
                name: spanned("add".to_string()),
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
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
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
                name: spanned("max".to_string()),
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
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
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
                name: spanned("test".to_string()),
                params: vec![],
                ret_ty: spanned(Type::I64),
                pre: None,
                post: None,
                body: spanned(Expr::Let {
                    name: "x".to_string(),
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
}
