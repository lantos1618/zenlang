use super::Expression;
use crate::error::Span;

impl Expression {
    /// Returns the span of this expression.
    pub fn span(&self) -> Span {
        match self {
            Expression::IntLiteral { span, .. }
            | Expression::FloatLiteral { span, .. }
            | Expression::StringLiteral { span, .. }
            | Expression::BoolLiteral { span, .. }
            | Expression::Identifier { span, .. }
            | Expression::BinaryOp { span, .. }
            | Expression::UnaryOp { span, .. }
            | Expression::FunctionCall { span, .. }
            | Expression::MethodCall { span, .. }
            | Expression::MemberAccess { span, .. }
            | Expression::IndexAccess { span, .. }
            | Expression::StructLiteral { span, .. }
            | Expression::EnumVariant { span, .. }
            | Expression::ArrayLiteral { span, .. }
            | Expression::Match { span, .. }
            | Expression::WhileLoop { span, .. }
            | Expression::Loop { span, .. }
            | Expression::LoopControl { span, .. }
            | Expression::If { span, .. }
            | Expression::Block { span, .. }
            | Expression::Break { span, .. }
            | Expression::Continue { span, .. }
            | Expression::Closure { span, .. }
            | Expression::Cast { span, .. }
            | Expression::StringInterpolation { span, .. }
            | Expression::Range { span, .. }
            | Expression::Defer { span, .. }
            | Expression::Error { span, .. } => *span,
        }
    }
}
