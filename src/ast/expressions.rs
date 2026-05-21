use crate::ast::statements::Statement;
use crate::ast::types::{AstType, Param};
use crate::error::Span;
use serde::Serialize;

mod expression_parts;
mod operators;
mod span;
pub use expression_parts::{MatchArm, StringPart};
pub use operators::{BinaryOp, LoopControlAction, UnaryOp};

/// Expression — the parser's output for any value-producing construct.
///
/// Every variant carries a `span: Span` for error reporting.
/// Key invariants:
/// - `FunctionCall` / `MethodCall` have `type_args` — generics are NEVER encoded in the name
/// - `FunctionCall` has `module: Option<String>` — module is NEVER encoded in the name
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    /// Infinite loop: `loop { body }`, `loop(() { body })`, or `loop((l) { body })`.
    Loop {
        body: Box<Expression>,
        control_label: Option<String>,
        span: Span,
    },
    /// `l.done()`, `l.next()`, `done(l)`, or `next(l)` inside `loop((l) { ... })`.
    LoopControl {
        action: LoopControlAction,
        target_label: String,
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
