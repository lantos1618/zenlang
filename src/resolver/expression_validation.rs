use crate::ast::{Expression, TypeParam};
use crate::error::Diagnostic;

use super::symbol_table::ScopeStack;
use super::{Resolver, SymbolTable};

mod calls;
mod construct_dispatch;
mod dispatch;
mod traversal;

impl Resolver {
    pub(super) fn validate_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match expr {
            Expression::FunctionCall { .. }
            | Expression::Identifier { .. }
            | Expression::MethodCall { .. } => self.validate_call_expr_refs(
                table,
                type_params,
                expr,
                locals,
                allow_self_type,
                diagnostics,
            ),
            Expression::BinaryOp { .. }
            | Expression::UnaryOp { .. }
            | Expression::MemberAccess { .. }
            | Expression::IndexAccess { .. }
            | Expression::WhileLoop { .. }
            | Expression::If { .. }
            | Expression::Cast { .. }
            | Expression::StringInterpolation { .. }
            | Expression::Range { .. }
            | Expression::Defer { .. } => self.validate_traversal_expr_refs(
                table,
                type_params,
                expr,
                locals,
                allow_self_type,
                diagnostics,
            ),
            Expression::StructLiteral { .. }
            | Expression::EnumVariant { .. }
            | Expression::ArrayLiteral { .. }
            | Expression::Match { .. }
            | Expression::Loop { .. }
            | Expression::Block { .. }
            | Expression::Closure { .. } => self.validate_construct_expr_refs(
                table,
                type_params,
                expr,
                locals,
                allow_self_type,
                diagnostics,
            ),
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::LoopControl { .. }
            | Expression::Error { .. } => {}
        }
    }
}
