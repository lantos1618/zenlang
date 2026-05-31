use crate::ast::statements::Statement;
use crate::ast::types::{AstType, Param};
use crate::error::Span;
use serde::Serialize;

mod operators;
mod parts;
mod spans;
pub use operators::{BinaryOp, LoopControlAction, UnaryOp};
pub use parts::{MatchArm, StringPart};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expression {
    IntLiteral {
        value: i64,
        span: Span,
    },
    FloatLiteral {
        value: f64,
        span: Span,
    },
    StringLiteral {
        value: String,
        span: Span,
    },
    BoolLiteral {
        value: bool,
        span: Span,
    },

    Identifier {
        name: String,
        span: Span,
    },

    BinaryOp {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expression>,
        span: Span,
    },

    FunctionCall {
        name: String,
        module: Option<String>,
        type_args: Vec<AstType>,
        args: Vec<Expression>,
        span: Span,
    },
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        type_args: Vec<AstType>,
        args: Vec<Expression>,
        span: Span,
    },

    MemberAccess {
        object: Box<Expression>,
        field: String,
        span: Span,
    },
    IndexAccess {
        object: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },

    StructLiteral {
        name: String,
        type_args: Vec<AstType>,
        fields: Vec<(String, Expression)>,
        span: Span,
    },
    EnumVariant {
        enum_name: String,
        type_args: Vec<AstType>,
        variant: String,
        payload: Option<Box<Expression>>,
        span: Span,
    },
    ArrayLiteral {
        elements: Vec<Expression>,
        span: Span,
    },

    Match {
        scrutinee: Box<Expression>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Loop {
        body: Box<Expression>,
        control_label: Option<String>,
        span: Span,
    },
    LoopControl {
        action: LoopControlAction,
        target_label: String,
        span: Span,
    },
    If {
        condition: Box<Expression>,
        then_body: Box<Expression>,
        else_body: Option<Box<Expression>>,
        span: Span,
    },
    Block {
        statements: Vec<Statement>,
        expr: Option<Box<Expression>>,
        span: Span,
    },

    Closure {
        params: Vec<Param>,
        return_type: Option<AstType>,
        body: Box<Expression>,
        span: Span,
    },

    Cast {
        expr: Box<Expression>,
        target_type: AstType,
        span: Span,
    },

    StringInterpolation {
        parts: Vec<StringPart>,
        span: Span,
    },

    Defer {
        expr: Box<Expression>,
        span: Span,
    },

    /// `@await e` — suspend the enclosing `@async` function until the future `e`
    /// is ready, then evaluate to its inner value (ASYNC_PLAN.md milestone 1).
    Await {
        expr: Box<Expression>,
        span: Span,
    },
}

impl Expression {
    pub(crate) fn walk_type_refs(&self, on_type: &mut impl FnMut(&AstType, Span)) {
        match self {
            Expression::FunctionCall {
                type_args,
                args,
                span,
                ..
            } => {
                walk_type_refs(type_args, *span, on_type);
                walk_exprs(args, on_type);
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                receiver.walk_type_refs(on_type);
                walk_type_refs(type_args, *span, on_type);
                walk_exprs(args, on_type);
            }
            Expression::BinaryOp { left, right, .. }
            | Expression::IndexAccess {
                object: left,
                index: right,
                ..
            } => {
                left.walk_type_refs(on_type);
                right.walk_type_refs(on_type);
            }
            Expression::UnaryOp { operand, .. }
            | Expression::MemberAccess {
                object: operand, ..
            }
            | Expression::Loop { body: operand, .. }
            | Expression::Defer { expr: operand, .. }
            | Expression::Await { expr: operand, .. } => operand.walk_type_refs(on_type),
            Expression::StructLiteral {
                type_args,
                fields,
                span,
                ..
            } => {
                walk_type_refs(type_args, *span, on_type);
                for (_, value) in fields {
                    value.walk_type_refs(on_type);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload,
                span,
                ..
            } => {
                walk_type_refs(type_args, *span, on_type);
                walk_optional_expr(payload.as_deref(), on_type);
            }
            Expression::ArrayLiteral { elements, .. } => walk_exprs(elements, on_type),
            Expression::Match {
                scrutinee, arms, ..
            } => {
                scrutinee.walk_type_refs(on_type);
                for arm in arms {
                    walk_optional_expr(arm.guard.as_ref(), on_type);
                    arm.body.walk_type_refs(on_type);
                }
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                condition.walk_type_refs(on_type);
                then_body.walk_type_refs(on_type);
                walk_optional_expr(else_body.as_deref(), on_type);
            }
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    walk_statement_type_refs(statement, on_type);
                }
                walk_optional_expr(expr.as_deref(), on_type);
            }
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => {
                for param in params {
                    on_type(&param.ty, param.span);
                }
                if let Some(return_type) = return_type {
                    on_type(return_type, *span);
                }
                body.walk_type_refs(on_type);
            }
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                expr.walk_type_refs(on_type);
                on_type(target_type, *span);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let StringPart::Expr(expr) = part {
                        expr.walk_type_refs(on_type);
                    }
                }
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::LoopControl { .. } => {}
        }
    }
}

fn walk_exprs(exprs: &[Expression], on_type: &mut impl FnMut(&AstType, Span)) {
    for expr in exprs {
        expr.walk_type_refs(on_type);
    }
}

fn walk_optional_expr(expr: Option<&Expression>, on_type: &mut impl FnMut(&AstType, Span)) {
    if let Some(expr) = expr {
        expr.walk_type_refs(on_type);
    }
}

fn walk_type_refs(ast_types: &[AstType], span: Span, on_type: &mut impl FnMut(&AstType, Span)) {
    for ast_type in ast_types {
        on_type(ast_type, span);
    }
}

fn walk_statement_type_refs(statement: &Statement, on_type: &mut impl FnMut(&AstType, Span)) {
    match statement {
        Statement::VarDecl {
            ty, value, span, ..
        } => {
            if let Some(ty) = ty {
                on_type(ty, *span);
            }
            value.walk_type_refs(on_type);
        }
        Statement::Assignment { target, value, .. } => {
            target.walk_type_refs(on_type);
            value.walk_type_refs(on_type);
        }
        Statement::Expression { expr, .. } => expr.walk_type_refs(on_type),
    }
}
