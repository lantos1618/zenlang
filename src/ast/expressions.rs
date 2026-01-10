//! Expression nodes in the AST

use super::patterns::Pattern;
use super::types::AstType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    StringConcat,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Integer8(i8),
    Integer16(i16),
    Integer32(i32),
    Integer64(i64),
    Unsigned8(u8),
    Unsigned16(u16),
    Unsigned32(u32),
    Unsigned64(u64),
    Float32(f32),
    Float64(f64),
    Boolean(bool),
    String(String),
    Identifier(String),
    Unit, // The unit value, similar to () in Rust or void in C
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    // Pattern matching with ? operator (no match keyword!)
    QuestionMatch {
        scrutinee: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    // Conditional expression for simple boolean patterns (expr ? { block })
    Conditional {
        scrutinee: Box<Expression>,
        arms: Vec<ConditionalArm>,
    },
    AddressOf(Box<Expression>),
    Dereference(Box<Expression>),
    PointerOffset {
        pointer: Box<Expression>,
        offset: Box<Expression>,
    },
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    StructField {
        struct_: Box<Expression>,
        field: String,
    },
    // New expressions for enhanced features
    ArrayLiteral(Vec<Expression>),
    ArrayIndex {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    EnumVariant {
        enum_name: String,
        variant: String,
        payload: Option<Box<Expression>>,
    },
    // Enum literal syntax: .Some(value), .None (without enum name)
    EnumLiteral {
        variant: String,
        payload: Option<Box<Expression>>,
    },
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
    // Pointer-specific operations for Zen spec
    PointerDereference(Box<Expression>), // .val operation
    PointerAddress(Box<Expression>),     // .addr operation
    CreateReference(Box<Expression>),    // .ref() method
    CreateMutableReference(Box<Expression>), // .mut_ref() method
    StringLength(Box<Expression>),
    // Option<T> constructors
    Some(Box<Expression>), // Some(value)
    None,                  // None (also accessible as "null")
    // String interpolation: "Hello ${name}!"
    StringInterpolation {
        parts: Vec<StringPart>,
    },
    // For comptime expressions
    Comptime(Box<Expression>),
    // Range expressions
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
    // Pattern matching expressions
    PatternMatch {
        scrutinee: Box<Expression>,
        arms: Vec<PatternArm>,
    },
    // @std reference
    StdReference,
    // @builtin reference (raw compiler intrinsics)
    BuiltinReference,
    // @this reference (current scope)
    ThisReference,
    // Method call with UFC (Uniform Function Call)
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    // Loop expression (returns value)
    Loop {
        body: Box<Expression>,
    },
    // Collection loop: collection.loop((item) { ... })
    CollectionLoop {
        collection: Box<Expression>,
        param: (String, Option<AstType>), // The loop parameter name and optional type
        index_param: Option<(String, Option<AstType>)>, // Optional index parameter and type
        body: Box<Expression>,
    },
    // Closure expression
    Closure {
        params: Vec<(String, Option<AstType>)>,
        return_type: Option<AstType>,
        body: Box<Expression>,
    },
    // Block expression - evaluates to the last expression or void
    Block(Vec<super::statements::Statement>),
    // Return expression - for early returns in pattern match arms
    Return(Box<Expression>),
    // Type cast expression: expr as Type
    TypeCast {
        expr: Box<Expression>,
        target_type: AstType,
    },
    // Error propagation: expr.raise()
    Raise(Box<Expression>),
    // Defer expression: @this.defer(expr)
    Defer(Box<Expression>),
    // Break expression for loops (can be used in expression contexts like pattern arms)
    Break {
        label: Option<String>,
        value: Option<Box<Expression>>, // Break can optionally return a value
    },
    // Continue expression for loops
    Continue {
        label: Option<String>,
    },
    // Collection constructors
    // Vec<T, size>() - Fixed-size vector constructor
    VecConstructor {
        element_type: AstType,
        size: usize,
        initial_values: Option<Vec<Expression>>, // Optional initial values
    },
    // DynVec<T>(allocator) or DynVec<T1, T2, ...>(allocator) - Dynamic vector constructor
    DynVecConstructor {
        element_types: Vec<AstType>,
        allocator: Box<Expression>,                // Allocator expression
        initial_capacity: Option<Box<Expression>>, // Optional initial capacity
    },
    // Array<T>() - Array constructor
    ArrayConstructor {
        element_type: AstType,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Interpolation(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>, // Optional guard condition
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>, // Optional guard condition using ->
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>, // Optional guard condition
    pub body: Expression,
}
