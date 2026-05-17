//! Expression checking — check_function and check_expr.
#![allow(clippy::result_large_err)]

mod aggregate_support;
mod call_support;
mod call_validation;
mod control_flow_support;
mod method_call_support;

use crate::ast::expressions::StringPart;
use crate::ast::typed::*;
use crate::ast::{AstType, Expression, Param};
use crate::error::{Diagnostic, Span};

use super::closures::collect_captures;
use super::monomorphize_inference::InferenceConflict;
use super::{BehaviorBound, TypeChecker};

impl TypeChecker {
    pub(crate) fn check_function(
        &mut self,
        name: &str,
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        _span: &Span,
    ) -> Result<TypedFunction, Diagnostic> {
        let ret_type = return_type
            .as_ref()
            .map(|t| self.resolve_type(t))
            .unwrap_or(Type::Void);

        self.current_return_type = Some(ret_type.clone());

        // Push function scope with params
        self.push_scope();
        let mut typed_params = Vec::new();
        for p in params {
            let ty = self.resolve_type(&p.ty);
            self.define_var(&p.name, ty.clone());
            typed_params.push(TypedParam {
                name: p.name.clone(),
                ty,
                span: p.span,
            });
        }

        let typed_body = self.check_expr(body)?;
        let body_block = match typed_body.kind {
            TypedExprKind::Block(block) => block,
            _ => TypedBlock {
                ty: typed_body.ty.clone(),
                span: typed_body.span,
                statements: Vec::new(),
                expr: Some(Box::new(typed_body)),
            },
        };

        self.pop_scope();
        self.current_return_type = None;

        if ret_type != Type::Void && ret_type != Type::Never {
            if let Some(expr) = &body_block.expr {
                if expr.ty != Type::Never && !self.types_compatible(&ret_type, &expr.ty) {
                    return Err(Diagnostic::error(
                        "E3030",
                        format!(
                            "return type mismatch: expected `{}`, found `{}`",
                            ret_type.display_name(),
                            expr.ty.display_name()
                        ),
                        expr.span,
                    ));
                }
            }

            if !self.block_satisfies_return(&body_block, &ret_type) {
                return Err(Diagnostic::error(
                    "E3031",
                    format!(
                        "function `{}` must return `{}` on all non-error paths",
                        name,
                        ret_type.display_name()
                    ),
                    *_span,
                ));
            }
        }

        // Collect defers accumulated during this function's body (LIFO order)
        let mut defers: Vec<TypedExpression> = self.pending_defers.drain(..).collect();
        defers.reverse();

        Ok(TypedFunction {
            name: name.to_string(),
            params: typed_params,
            return_type: ret_type,
            body: body_block,
            defers,
            span: *_span,
        })
    }

    pub(crate) fn check_expr(&mut self, expr: &Expression) -> Result<TypedExpression, Diagnostic> {
        match expr {
            Expression::IntLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::IntLiteral(*value),
                ty: Type::I32, // default int type
                span: *span,
            }),

            Expression::FloatLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::FloatLiteral(*value),
                ty: Type::F64, // default float type
                span: *span,
            }),

            Expression::StringLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::StringLiteral(value.clone()),
                ty: Type::Str,
                span: *span,
            }),

            Expression::BoolLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::BoolLiteral(*value),
                ty: Type::Bool,
                span: *span,
            }),

            Expression::Identifier { name, span } => {
                if name == "true" {
                    return Ok(TypedExpression {
                        kind: TypedExprKind::BoolLiteral(true),
                        ty: Type::Bool,
                        span: *span,
                    });
                }
                if name == "false" {
                    return Ok(TypedExpression {
                        kind: TypedExprKind::BoolLiteral(false),
                        ty: Type::Bool,
                        span: *span,
                    });
                }
                let ty = match self.lookup_var(name) {
                    Some(t) => t,
                    None => {
                        // Not a variable — only warn if it's not a known type, enum, function, or import
                        if !self.structs.contains_key(name.as_str())
                            && !self.enums.contains_key(name.as_str())
                            && !self.functions.contains_key(name.as_str())
                            && !self.is_import(name)
                        {
                            self.diagnostics.push(Diagnostic::error(
                                "E3040",
                                format!("undefined variable `{}`", name),
                                *span,
                            ));
                        }
                        Type::Unknown
                    }
                };
                Ok(TypedExpression {
                    kind: TypedExprKind::Variable(name.clone()),
                    ty,
                    span: *span,
                })
            }

            Expression::BinaryOp {
                op,
                left,
                right,
                span,
            } => {
                let left = self.check_expr(left)?;
                let right = self.check_expr(right)?;
                let ty = self.check_binary_op(*op, &left.ty, &right.ty, span)?;
                Ok(TypedExpression {
                    kind: TypedExprKind::BinaryOp {
                        op: *op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    ty,
                    span: *span,
                })
            }

            Expression::FunctionCall {
                name,
                module,
                type_args,
                args,
                span,
            } => self.check_function_call_expr(name, module, type_args, args, *span),

            Expression::MethodCall {
                receiver,
                method,
                type_args,
                args,
                span,
            } => self.check_method_call_expr(receiver, method, type_args, args, *span),

            Expression::MemberAccess {
                object,
                field,
                span,
            } => self.check_member_access_expr(object, field, *span),

            Expression::StructLiteral {
                name,
                type_args,
                fields,
                span,
            } => self.check_struct_literal_expr(name, type_args, fields, *span),

            Expression::EnumVariant {
                enum_name,
                type_args,
                variant,
                payload,
                span,
            } => self.check_enum_variant_expr(enum_name, type_args, variant, payload, *span),

            Expression::ArrayLiteral { elements, span } => {
                self.check_array_literal_expr(elements, *span)
            }

            Expression::Block {
                statements,
                expr,
                span,
            } => {
                self.push_scope();
                let mut typed_stmts = Vec::new();
                for stmt in statements {
                    typed_stmts.push(self.check_statement(stmt)?);
                }
                let typed_expr = match expr {
                    Some(e) => Some(Box::new(self.check_expr(e)?)),
                    None => None,
                };
                let ty = typed_expr
                    .as_ref()
                    .map(|e| e.ty.clone())
                    .unwrap_or(Type::Void);
                self.pop_scope();
                Ok(TypedExpression {
                    kind: TypedExprKind::Block(TypedBlock {
                        statements: typed_stmts,
                        expr: typed_expr,
                        ty: ty.clone(),
                        span: *span,
                    }),
                    ty,
                    span: *span,
                })
            }

            Expression::Return { value, span } => {
                let typed_val = match value {
                    Some(v) => Some(Box::new(self.check_expr(v)?)),
                    None => None,
                };
                // Check return type compatibility
                if let Some(ref expected) = self.current_return_type {
                    let actual = typed_val.as_ref().map(|v| &v.ty).unwrap_or(&Type::Void);
                    if !self.types_compatible(expected, actual) {
                        self.diagnostics.push(Diagnostic::error(
                            "E3030",
                            format!(
                                "return type mismatch: expected `{}`, found `{}`",
                                expected.display_name(),
                                actual.display_name()
                            ),
                            *span,
                        ));
                    }
                }
                Ok(TypedExpression {
                    kind: TypedExprKind::Return(typed_val),
                    ty: Type::Never,
                    span: *span,
                })
            }

            Expression::Break { span } => Ok(TypedExpression {
                kind: TypedExprKind::Break,
                ty: Type::Never,
                span: *span,
            }),

            Expression::Continue { span } => Ok(TypedExpression {
                kind: TypedExprKind::Continue,
                ty: Type::Never,
                span: *span,
            }),

            Expression::Match {
                scrutinee,
                arms,
                span,
            } => self.check_match_expr(scrutinee, arms, *span),

            Expression::If {
                condition,
                then_body,
                else_body,
                span,
            } => self.check_if_expr(condition, then_body, else_body, *span),

            Expression::WhileLoop {
                condition,
                body,
                span,
            } => self.check_while_loop_expr(condition, body, *span),

            Expression::Loop {
                body,
                control_label,
                span,
            } => self.check_loop_expr(body, control_label, *span),

            Expression::LoopControl {
                action,
                target_label,
                span,
            } => Ok(TypedExpression {
                kind: TypedExprKind::LoopControl {
                    action: *action,
                    label: target_label.clone(),
                },
                ty: Type::Never,
                span: *span,
            }),

            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                let typed_expr = self.check_expr(expr)?;
                let from_type = typed_expr.ty.clone();
                let to_type = self.resolve_type(target_type);
                Ok(TypedExpression {
                    kind: TypedExprKind::Cast {
                        expr: Box::new(typed_expr),
                        from_type,
                        to_type: to_type.clone(),
                    },
                    ty: to_type,
                    span: *span,
                })
            }

            Expression::StringInterpolation { parts, span } => {
                let mut typed_parts = Vec::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => {
                            typed_parts.push(TypedStringPart::Literal(s.clone()));
                        }
                        StringPart::Expr(e) => {
                            let typed = self.check_expr(e)?;
                            typed_parts.push(TypedStringPart::Expr(typed));
                        }
                    }
                }
                Ok(TypedExpression {
                    kind: TypedExprKind::StringInterpolation { parts: typed_parts },
                    ty: Type::Str,
                    span: *span,
                })
            }

            Expression::Defer { expr, span } => {
                // Type-check the deferred expression and collect it
                let typed_expr = self.check_expr(expr)?;
                self.pending_defers.push(typed_expr);
                // Defer itself doesn't produce a value
                Ok(TypedExpression {
                    kind: TypedExprKind::Block(TypedBlock {
                        statements: Vec::new(),
                        expr: None,
                        ty: Type::Void,
                        span: *span,
                    }),
                    ty: Type::Void,
                    span: *span,
                })
            }

            Expression::IndexAccess {
                object,
                index,
                span,
            } => self.check_index_access_expr(object, index, *span),

            Expression::Closure {
                params,
                return_type: _return_type,
                body,
                span,
            } => {
                // Collect variables visible before entering closure scope
                let outer_vars: std::collections::HashMap<String, Type> = self
                    .scopes
                    .iter()
                    .flat_map(|s| s.vars.iter())
                    .map(|(k, v)| (k.clone(), v.ty.clone()))
                    .collect();

                self.push_scope();
                let mut param_types = Vec::new();
                let mut param_names: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for p in params {
                    let ty = self.resolve_type(&p.ty);
                    self.define_var_with_mutability(&p.name, ty.clone(), p.mutable);
                    param_types.push(ty);
                    param_names.insert(p.name.clone());
                }
                let typed_body = self.check_expr(body)?;
                self.pop_scope();

                let ret_type = if let Some(rt) = _return_type {
                    self.resolve_type(rt)
                } else {
                    typed_body.ty.clone()
                };

                // Capture analysis: find variables in body that come from outer scopes
                let mut captures = Vec::new();
                let mut seen = std::collections::HashSet::new();
                collect_captures(
                    &typed_body,
                    &param_names,
                    &outer_vars,
                    &mut captures,
                    &mut seen,
                );

                let fn_name = format!("__closure_{}_{}", span.start, span.end);
                let env_type = if captures.is_empty() {
                    String::new()
                } else {
                    format!("__env_{}_{}", span.start, span.end)
                };

                let fn_type = Type::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                };

                Ok(TypedExpression {
                    kind: TypedExprKind::Closure {
                        fn_name,
                        env_type,
                        captures,
                    },
                    ty: fn_type,
                    span: *span,
                })
            }

            Expression::UnaryOp { op, operand, span } => {
                let typed = self.check_expr(operand)?;
                let ty = typed.ty.clone();
                Ok(TypedExpression {
                    kind: TypedExprKind::UnaryOp {
                        op: *op,
                        operand: Box::new(typed),
                    },
                    ty,
                    span: *span,
                })
            }

            // TODO: implement Range type
            Expression::Range {
                start, end, span, ..
            } => {
                let _typed_start = self.check_expr(start)?;
                let _typed_end = self.check_expr(end)?;
                Ok(TypedExpression {
                    kind: TypedExprKind::Error,
                    ty: Type::Unknown,
                    span: *span,
                })
            }

            // TODO: implement char literal type
            Expression::Error { span } | Expression::CharLiteral { span, .. } => {
                Ok(TypedExpression {
                    kind: TypedExprKind::Error,
                    ty: Type::Unknown,
                    span: *span,
                })
            }
        }
    }
}
