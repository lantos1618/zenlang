//! Statement checking.
#![allow(clippy::result_large_err)]

use crate::ast::typed::*;
use crate::ast::Statement;
use crate::error::Diagnostic;

use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_statement(
        &mut self,
        stmt: &Statement,
    ) -> Result<TypedStatement, Diagnostic> {
        match stmt {
            Statement::VarDecl {
                name,
                ty,
                value,
                mutable,
                span,
                ..
            } => {
                let typed_value = self.check_expr(value)?;
                let annotation_valid = ty
                    .as_ref()
                    .is_none_or(|t| self.generic_type_annotation_arities_valid(t));
                let var_type = if let Some(t) = ty {
                    if annotation_valid {
                        self.resolve_type(t)
                    } else {
                        Type::Unknown
                    }
                } else {
                    typed_value.ty.clone()
                };

                // Literal coercion: int/float literals adopt the declared type
                let typed_value = if (var_type.is_integer()
                    && matches!(typed_value.kind, TypedExprKind::IntLiteral(_)))
                    || (var_type.is_float()
                        && matches!(typed_value.kind, TypedExprKind::FloatLiteral(_)))
                {
                    TypedExpression {
                        ty: var_type.clone(),
                        ..typed_value
                    }
                } else {
                    typed_value
                };

                if !self.types_compatible(&var_type, &typed_value.ty) {
                    self.diagnostics.push(Diagnostic::error(
                        "E3072",
                        format!(
                            "variable `{}` expects `{}`, found `{}`",
                            name,
                            var_type.display_name(),
                            typed_value.ty.display_name()
                        ),
                        *span,
                    ));
                }

                // If `name = expr` form (not ::= or :=) and the variable already
                // exists in scope, treat as reassignment instead of new binding.
                if !*mutable && ty.is_none() && self.lookup_var_info(name).is_some() {
                    let target_info = self.lookup_var_info(name).cloned().expect("checked above");
                    self.check_assignment_target(
                        name,
                        &target_info.ty,
                        target_info.mutable,
                        &typed_value,
                        span,
                    );
                    return Ok(TypedStatement {
                        kind: TypedStatementKind::Expression(TypedExpression {
                            kind: TypedExprKind::Assign {
                                target: Box::new(TypedExpression {
                                    kind: TypedExprKind::Variable(name.clone()),
                                    ty: target_info.ty,
                                    span: *span,
                                }),
                                value: Box::new(typed_value),
                            },
                            ty: Type::Void,
                            span: *span,
                        }),
                        span: *span,
                    });
                }

                self.define_var_with_mutability(name, var_type.clone(), *mutable);
                Ok(TypedStatement {
                    kind: TypedStatementKind::VarDecl {
                        name: name.clone(),
                        ty: var_type,
                        value: typed_value,
                        mutable: *mutable,
                    },
                    span: *span,
                })
            }
            Statement::Assignment {
                target,
                value,
                span,
            } => {
                let typed_target = self.check_expr(target)?;
                let typed_value = self.check_expr(value)?;
                if let TypedExprKind::Variable(name) = &typed_target.kind {
                    if let Some(info) = self.lookup_var_info(name).cloned() {
                        self.check_assignment_target(
                            name,
                            &info.ty,
                            info.mutable,
                            &typed_value,
                            span,
                        );
                    }
                } else if typed_target.ty != Type::Unknown
                    && typed_value.ty != Type::Unknown
                    && !self.types_compatible(&typed_target.ty, &typed_value.ty)
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E3071",
                        format!(
                            "assignment expects `{}`, found `{}`",
                            typed_target.ty.display_name(),
                            typed_value.ty.display_name()
                        ),
                        *span,
                    ));
                }
                Ok(TypedStatement {
                    kind: TypedStatementKind::Expression(TypedExpression {
                        kind: TypedExprKind::Assign {
                            target: Box::new(typed_target),
                            value: Box::new(typed_value),
                        },
                        ty: Type::Void,
                        span: *span,
                    }),
                    span: *span,
                })
            }
            Statement::Expression { expr, span } => {
                let typed = self.check_expr(expr)?;
                Ok(TypedStatement {
                    kind: TypedStatementKind::Expression(typed),
                    span: *span,
                })
            }
            Statement::Block { stmts, span } => {
                self.push_scope();
                let mut typed_stmts = Vec::new();
                for s in stmts {
                    typed_stmts.push(self.check_statement(s)?);
                }
                self.pop_scope();
                // Return the last statement as an expression
                let _last = typed_stmts
                    .last()
                    .map(|s| match &s.kind {
                        TypedStatementKind::Expression(e) => e.ty.clone(),
                        _ => Type::Void,
                    })
                    .unwrap_or(Type::Void);
                Ok(TypedStatement {
                    kind: TypedStatementKind::Expression(TypedExpression {
                        kind: TypedExprKind::Block(TypedBlock {
                            statements: typed_stmts,
                            expr: None,
                            ty: _last,
                            span: *span,
                        }),
                        ty: Type::Void,
                        span: *span,
                    }),
                    span: *span,
                })
            }
        }
    }

    fn check_assignment_target(
        &mut self,
        name: &str,
        target_ty: &Type,
        mutable: bool,
        value: &TypedExpression,
        span: &crate::error::Span,
    ) {
        if !mutable {
            self.diagnostics.push(Diagnostic::error(
                "E3070",
                format!("cannot assign to immutable variable `{}`", name),
                *span,
            ));
        }

        if value.ty != Type::Unknown
            && *target_ty != Type::Unknown
            && !self.types_compatible(target_ty, &value.ty)
        {
            self.diagnostics.push(Diagnostic::error(
                "E3071",
                format!(
                    "assignment to `{}` expects `{}`, found `{}`",
                    name,
                    target_ty.display_name(),
                    value.ty.display_name()
                ),
                value.span,
            ));
        }
    }
}
