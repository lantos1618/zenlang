//! Statement checking.
#![allow(clippy::result_large_err)]

mod bindings;

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
            } => self.check_var_decl_statement(name, ty, value, *mutable, span),
            Statement::Assignment {
                target,
                value,
                span,
            } => self.check_assignment_statement(target, value, span),
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
