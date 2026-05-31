use crate::ast::typed::*;
use crate::ast::Statement;
use crate::error::{CompilerDiagnosticCode::*, Diagnostic, Span};

use super::{literal_coerced_type, type_display_pair, TypeChecker};

fn typed_assignment_statement(
    target: TypedExpression,
    value: TypedExpression,
    span: Span,
) -> TypedStatement {
    TypedStatement {
        kind: TypedStatementKind::Expression(TypedExpression {
            kind: TypedExprKind::Assign {
                target: Box::new(target),
                value: Box::new(value),
            },
            ty: Type::Void,
            span,
        }),
        span,
    }
}

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
                let var_type = match (ty, annotation_valid) {
                    (Some(t), true) => self.resolve_type(t),
                    (Some(_), false) => Type::Unknown,
                    (None, _) => typed_value.ty.clone(),
                };

                let typed_value = TypedExpression {
                    ty: literal_coerced_type(&var_type, &typed_value),
                    ..typed_value
                };

                if !self.types_compatible(&var_type, &typed_value.ty) {
                    let (expected, actual) = type_display_pair(&var_type, &typed_value.ty);
                    self.push_error(
                        E3072,
                        format!("variable `{name}` expects `{expected}`, found `{actual}`"),
                        *span,
                    );
                }

                if let (false, None, Some(target_info)) =
                    (*mutable, ty.as_ref(), self.lookup_var_info(name).cloned())
                {
                    self.check_assignment_target(
                        name,
                        &target_info.ty,
                        target_info.mutable,
                        &typed_value,
                        span,
                    );
                    return Ok(typed_assignment_statement(
                        TypedExpression {
                            kind: TypedExprKind::Variable(name.clone()),
                            ty: target_info.ty,
                            span: *span,
                        },
                        typed_value,
                        *span,
                    ));
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
                } else if !self.types_compatible(&typed_target.ty, &typed_value.ty) {
                    let (expected, actual) = type_display_pair(&typed_target.ty, &typed_value.ty);
                    self.push_error(
                        E3071,
                        format!("assignment expects `{expected}`, found `{actual}`"),
                        *span,
                    );
                }
                Ok(typed_assignment_statement(typed_target, typed_value, *span))
            }
            Statement::Expression { expr, span } => {
                let typed = self.check_expr(expr)?;
                Ok(TypedStatement {
                    kind: TypedStatementKind::Expression(typed),
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
        span: &Span,
    ) {
        if !mutable {
            self.push_error(
                E3070,
                format!("cannot assign to immutable variable `{}`", name),
                *span,
            );
        }

        if !self.types_compatible(target_ty, &value.ty) {
            let (expected, actual) = type_display_pair(target_ty, &value.ty);
            self.push_error(
                E3071,
                format!("assignment to `{name}` expects `{expected}`, found `{actual}`"),
                value.span,
            );
        }
    }
}
