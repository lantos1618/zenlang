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
                let var_type = if let Some(t) = ty {
                    self.resolve_type(t)
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

                // If `name = expr` form (not ::= or :=) and the variable already
                // exists in scope, treat as reassignment instead of new binding.
                if !*mutable && ty.is_none() && self.lookup_var(name).is_some() {
                    return Ok(TypedStatement {
                        kind: TypedStatementKind::Expression(TypedExpression {
                            kind: TypedExprKind::Assign {
                                target: Box::new(TypedExpression {
                                    kind: TypedExprKind::Variable(name.clone()),
                                    ty: var_type,
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

                self.define_var(name, var_type.clone());
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
}
