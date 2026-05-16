use crate::ast::expressions::Expression;
use crate::ast::types::AstType;
use crate::error::Span;
use serde::Serialize;

/// Statement — things that don't produce a value (or whose value is discarded).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Statement {
    /// Variable declaration: `x = 42`, `y ::= 0`, `z: i32 = 10`
    VarDecl {
        name: String,
        ty: Option<AstType>,
        value: Expression,
        /// `true` for `::=` (mutable binding)
        mutable: bool,
        /// `true` for compile-time constants
        constant: bool,
        span: Span,
    },

    /// Assignment to an existing mutable variable: `x = x + 1`
    /// Target can be an identifier, member access, or index access.
    Assignment {
        target: Expression,
        value: Expression,
        span: Span,
    },

    /// Expression used as a statement (value discarded).
    Expression { expr: Expression, span: Span },

    /// Block of statements: `{ stmt; stmt; ... }`
    Block { stmts: Vec<Statement>, span: Span },
}

impl Statement {
    /// Returns the span of this statement.
    pub fn span(&self) -> Span {
        match self {
            Statement::VarDecl { span, .. }
            | Statement::Assignment { span, .. }
            | Statement::Expression { span, .. }
            | Statement::Block { span, .. } => *span,
        }
    }
}
