//! # Abstract Syntax Tree
//!
//! The `ast` module defines the data structures that represent the code in a structured way.
//! The parser will produce these structures, and the compiler will consume them.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstType {
    Int8,
    Int32,
    Int64,
    Float,
    String,
    Void,
    Pointer(Box<AstType>),
    Function {
        args: Vec<AstType>,
        return_type: Box<AstType>,
    },
    Struct {
        name: String,
        fields: Vec<(String, AstType)>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    StringConcat,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer8(i8),
    Integer16(i16),
    Integer32(i32),
    Integer64(i64),
    Float(f64),
    String(String),
    Identifier(String),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    Conditional {
        scrutinee: Box<Expression>,
        arms: Vec<(Expression, Expression)>, // (pattern, body)
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
    StringLength(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Return(Expression),
    VariableDeclaration {
        name: String,
        type_: AstType,
        initializer: Option<Expression>,
    },
    VariableAssignment {
        name: String,
        value: Expression,
    },
    PointerAssignment {
        pointer: Expression,
        value: Expression,
    },
    Loop {
        condition: Expression,
        body: Vec<Statement>,
    },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub body: Vec<Statement>,
}

// For C FFI support
#[derive(Debug, Clone)]
pub struct ExternalFunction {
    pub name: String,
    pub args: Vec<AstType>,  // Just types, no names for external functions
    pub return_type: AstType,
    pub is_varargs: bool,  // For functions like printf
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Function(Function),
    ExternalFunction(ExternalFunction),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

// Convenience methods for backward compatibility
impl Program {
    pub fn from_functions(functions: Vec<Function>) -> Self {
        Self {
            declarations: functions.into_iter().map(Declaration::Function).collect(),
        }
    }

    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.declarations.iter().filter_map(|decl| match decl {
            Declaration::Function(f) => Some(f),
            _ => None,
        })
    }
} 