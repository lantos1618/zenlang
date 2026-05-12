use crate::ast::patterns::Pattern;
use crate::ast::statements::Statement;
use crate::ast::types::{AstType, Param};
use crate::error::Span;

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // Logical
    And,
    Or,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

impl BinaryOp {
    /// Returns the operator symbol for display/error messages.
    pub fn symbol(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::ShiftLeft => "<<",
            BinaryOp::ShiftRight => ">>",
        }
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,    // -x
    Not,    // !x
    BitNot, // ~x
}

impl UnaryOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
            UnaryOp::BitNot => "~",
        }
    }
}

/// Parts of a string interpolation: `"Hello, ${name}!"` becomes
/// `[Literal("Hello, "), Expr(<name>), Literal("!")]`.
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Expr(Expression),
}

/// A match arm: `| pattern guard? { body }`.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
    pub span: Span,
}

/// Expression — the parser's output for any value-producing construct.
///
/// Every variant carries a `span: Span` for error reporting.
/// Key invariants:
/// - `FunctionCall` / `MethodCall` have `type_args` — generics are NEVER encoded in the name
/// - `FunctionCall` has `module: Option<String>` — module is NEVER encoded in the name
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // ─── Literals ────────────────────────────────────────────────────
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
    CharLiteral {
        value: char,
        span: Span,
    },

    // ─── Identifiers / Names ─────────────────────────────────────────
    Identifier {
        name: String,
        span: Span,
    },

    // ─── Operators ───────────────────────────────────────────────────
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

    // ─── Calls ───────────────────────────────────────────────────────
    /// Free function call: `add(1, 2)` or `io.println("hi")`.
    /// Module is NEVER encoded in the name — it goes in `module`.
    /// Generics are NEVER encoded in the name — they go in `type_args`.
    FunctionCall {
        name: String,
        module: Option<String>,
        type_args: Vec<AstType>,
        args: Vec<Expression>,
        span: Span,
    },
    /// Method call: `receiver.method(args)`.
    /// UFC (universal function call) may also parse into this.
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        type_args: Vec<AstType>,
        args: Vec<Expression>,
        span: Span,
    },

    // ─── Access ──────────────────────────────────────────────────────
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

    // ─── Composite Literals ──────────────────────────────────────────
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

    // ─── Control Flow ────────────────────────────────────────────────
    /// Pattern match: `expr ? | arm | arm ...`
    Match {
        scrutinee: Box<Expression>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    /// While loop: `expr ? { body }` (condition ? { body } repeats while true)
    WhileLoop {
        condition: Box<Expression>,
        body: Box<Expression>,
        span: Span,
    },
    /// Infinite loop: `loop { body }` or `loop(() { body })`
    Loop {
        body: Box<Expression>,
        span: Span,
    },
    /// If/else — desugared from `expr ? | true { } | false { }` by parser
    If {
        condition: Box<Expression>,
        then_body: Box<Expression>,
        else_body: Option<Box<Expression>>,
        span: Span,
    },
    /// Block expression: `{ stmts; expr? }`
    Block {
        statements: Vec<Statement>,
        expr: Option<Box<Expression>>,
        span: Span,
    },
    /// Return from function.
    Return {
        value: Option<Box<Expression>>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },

    // ─── Closures / Lambdas ──────────────────────────────────────────
    Closure {
        params: Vec<Param>,
        return_type: Option<AstType>,
        body: Box<Expression>,
        span: Span,
    },

    // ─── Type Operations ─────────────────────────────────────────────
    /// `cast(expr, TargetType)`
    Cast {
        expr: Box<Expression>,
        target_type: AstType,
        span: Span,
    },

    // ─── Strings ─────────────────────────────────────────────────────
    /// `"Hello, ${name}!"`
    StringInterpolation {
        parts: Vec<StringPart>,
        span: Span,
    },

    // ─── Ranges ──────────────────────────────────────────────────────
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
        span: Span,
    },

    // ─── Defer ───────────────────────────────────────────────────────
    /// `@this.defer(expr)` — runs at scope exit in LIFO order
    Defer {
        expr: Box<Expression>,
        span: Span,
    },

    // ─── Error Recovery ──────────────────────────────────────────────
    /// Placeholder for parse errors — allows the parser to continue.
    Error {
        span: Span,
    },
}

impl Expression {
    /// Returns the span of this expression.
    pub fn span(&self) -> Span {
        match self {
            Expression::IntLiteral { span, .. }
            | Expression::FloatLiteral { span, .. }
            | Expression::StringLiteral { span, .. }
            | Expression::BoolLiteral { span, .. }
            | Expression::CharLiteral { span, .. }
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
            | Expression::If { span, .. }
            | Expression::Block { span, .. }
            | Expression::Return { span, .. }
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
