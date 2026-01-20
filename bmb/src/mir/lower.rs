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

/// Lower an entire program to MIR
pub fn lower_program(program: &Program) -> MirProgram {
    // v0.35.4: First pass - collect all function return types
    let mut func_return_types = std::collections::HashMap::new();
    for item in &program.items {
        if let Item::FnDef(fn_def) = item {
            let ret_ty = ast_type_to_mir(&fn_def.ret_ty.node);
            func_return_types.insert(fn_def.name.node.clone(), ret_ty);
        }
    }

    let functions = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::FnDef(fn_def) => Some(lower_function(fn_def, &func_return_types)),
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
fn lower_function(fn_def: &FnDef, func_return_types: &std::collections::HashMap<String, MirType>) -> MirFunction {
    let mut ctx = LoweringContext::new();

    // v0.35.4: Add user-defined function return types to context
    for (name, ty) in func_return_types {
        ctx.func_return_types.insert(name.clone(), ty.clone());
    }

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

    // v0.38: Extract contract facts for optimization
    let preconditions = extract_contract_facts(fn_def.pre.as_ref());
    let postconditions = extract_contract_facts(fn_def.post.as_ref());

    // v0.38.3: Extract @pure and @const attributes
    let is_pure = has_attribute(&fn_def.attributes, "pure");
    let is_const = has_attribute(&fn_def.attributes, "const");

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

        Expr::Var(name) => Operand::Place(Place::new(name.clone())),

        Expr::Binary { left, op, right } => {
            let lhs = lower_expr(left, ctx);
            let rhs = lower_expr(right, ctx);
            let dest = ctx.fresh_temp();

            // Determine the MIR operator based on operand types
            let lhs_ty = ctx.operand_type(&lhs);
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

        // v0.37: Invariant is for SMT verification, MIR lowering ignores it
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
                        "byte_at" | "len" | "strlen" | "cstr_byte_at" | "arg_count" => MirType::I64,
                        // Bool-returning runtime functions
                        "file_exists" | "cstr_eq" => MirType::Bool,
                        // Default to i64 for unknown functions
                        _ => MirType::I64,
                    }
                };
                ctx.locals.insert(dest.name.clone(), ret_ty);

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

        // v0.19.0: Struct initialization
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

            // Create destination for the field value
            let dest = ctx.fresh_temp();

            ctx.push_inst(MirInst::FieldAccess {
                dest: dest.clone(),
                base: base_place,
                field: field.node.clone(),
            });

            Operand::Place(dest)
        }

        // v0.43: Tuple field access (compile-time constant index)
        Expr::TupleField { expr, index } => {
            // Lower the tuple expression
            let tuple_op = lower_expr(expr, ctx);

            // Convert operand to place if needed
            let tuple_place = operand_to_place(tuple_op, ctx);

            // Create destination for the element value
            let dest = ctx.fresh_temp();

            // Use IndexLoad with constant index for tuple field access
            ctx.push_inst(MirInst::IndexLoad {
                dest: dest.clone(),
                array: tuple_place,
                index: Operand::Constant(Constant::Int(*index as i64)),
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
            let cases = compile_match_patterns(arms, &arm_labels, &default_label);

            // Close current block with switch terminator
            ctx.finish_block(Terminator::Switch {
                discriminant: Operand::Place(match_place.clone()),
                cases,
                default: default_label.clone(),
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

            // Generate default block (unreachable for exhaustive matches)
            ctx.start_block(default_label);
            ctx.finish_block(Terminator::Unreachable);

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

        Expr::Deref(inner) => {
            lower_expr(inner, ctx)
        }

        // v0.19.3: Array support
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
            ctx.push_inst(MirInst::ArrayInit {
                dest: dest.clone(),
                element_type,
                elements: mir_elements,
            });
            Operand::Place(dest)
        }

        // v0.42: Tuple expressions
        Expr::Tuple(elems) => {
            // Lower each element
            let mir_elements: Vec<Operand> = elems
                .iter()
                .map(|e| lower_expr(e, ctx))
                .collect();

            // For now, represent tuples using ArrayInit (simplified)
            // A proper implementation would use a struct-like aggregate
            let element_type = if !mir_elements.is_empty() {
                ctx.operand_type(&mir_elements[0])
            } else {
                MirType::I64
            };

            let dest = ctx.fresh_temp();
            ctx.push_inst(MirInst::ArrayInit {
                dest: dest.clone(),
                element_type,
                elements: mir_elements,
            });
            Operand::Place(dest)
        }

        Expr::Index { expr, index } => {
            // v0.19.3: Array indexing
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

            let idx = lower_expr(index, ctx);
            let dest = ctx.fresh_temp();
            ctx.push_inst(MirInst::IndexLoad {
                dest: dest.clone(),
                array: arr_place,
                index: idx,
            });
            Operand::Place(dest)
        }

        // v0.19.4: Method calls - static dispatch
        // Methods are lowered as function calls with receiver as first argument
        // The method name is prefixed with the receiver type for name mangling
        Expr::MethodCall { receiver, method, args } => {
            // Lower the receiver expression
            let recv_op = lower_expr(receiver, ctx);

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

        // Break - placeholder, full implementation requires control flow
        Expr::Break { value } => {
            match value {
                Some(v) => lower_expr(v, ctx),
                None => Operand::Constant(crate::mir::Constant::Unit),
            }
        }

        // Continue - placeholder
        Expr::Continue => {
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

        // v0.39: Type cast - lowered as Copy with type annotation
        // The actual cast semantics are handled by codegen
        Expr::Cast { expr, ty: _ } => {
            lower_expr(expr, ctx)
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
        // v0.5 Phase 5: References are pointers
        Type::Ref(_) | Type::RefMut(_) => MirType::I64,
        // v0.5 Phase 6: Arrays are pointers to data
        Type::Array(_, _) => MirType::I64,
        // v0.2: Refined types use base type
        Type::Refined { base, .. } => ast_type_to_mir(base),
        // v0.20.0: Fn types are function pointers (pointer-sized)
        Type::Fn { .. } => MirType::I64,
        // v0.31: Never type - unreachable code, use Unit
        Type::Never => MirType::Unit,
        // v0.37: Nullable type - convert inner type (for MIR, nullable is just a tagged union)
        Type::Nullable(inner) => ast_type_to_mir(inner),
        // v0.42: Tuple type - represent as struct-like aggregate
        Type::Tuple(_) => MirType::I64, // Simplified for now
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
/// Returns a list of (discriminant_value, target_label) pairs
fn compile_match_patterns(
    arms: &[MatchArm],
    arm_labels: &[String],
    default_label: &str,
) -> Vec<(i64, String)> {
    let mut cases = Vec::new();
    let mut has_wildcard = false;

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
                // Wildcard/var patterns catch all - they become the default case
                // For now, only the last one can be the default
                has_wildcard = true;
                // Add a fallthrough to this arm for the default path
                // We'll handle this by updating the default label
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
                        has_wildcard = true;
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

    // If we have a wildcard, we can still use the cases for non-wildcard matches
    // The default path will go to unreachable or the wildcard arm
    let _ = has_wildcard; // Silence unused warning for now
    let _ = default_label;

    cases
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
        Pattern::EnumVariant { bindings, .. } => {
            // For enum variants with bindings, extract fields
            for (i, binding) in bindings.iter().enumerate() {
                let field_place = ctx.fresh_temp();
                // Use field access to extract (simplified - real impl needs tag/data extraction)
                ctx.push_inst(MirInst::FieldAccess {
                    dest: field_place.clone(),
                    base: match_place.clone(),
                    field: format!("_{}", i), // Tuple-like access
                });
                // Recursively bind inner patterns
                bind_pattern_variables(&binding.node, &field_place, ctx);
            }
        }
        Pattern::Struct { fields, .. } => {
            // For struct patterns, bind field patterns
            for (field_name, field_pattern) in fields {
                let field_place = ctx.fresh_temp();
                ctx.push_inst(MirInst::FieldAccess {
                    dest: field_place.clone(),
                    base: match_place.clone(),
                    field: field_name.node.clone(),
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
