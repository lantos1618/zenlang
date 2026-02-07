// Compile-time value types and control flow signals

use crate::ast::{self, AstType, Declaration, Expression, Pattern, Statement};
use crate::error::{CompileError, Result};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::meta;

/// Control flow signals for the comptime interpreter.
/// These are NOT errors — they represent normal loop/function control flow
/// (break, continue, return) that must propagate through the call stack.
#[derive(Debug, Clone, PartialEq)]
pub enum ComptimeControlFlow {
    Break,
    Continue,
    Return(ComptimeValue),
}

impl fmt::Display for ComptimeControlFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComptimeControlFlow::Break => write!(f, "break"),
            ComptimeControlFlow::Continue => write!(f, "continue"),
            ComptimeControlFlow::Return(v) => write!(f, "return {}", v),
        }
    }
}

/// Result type for statements that may produce control flow signals.
/// Ok(None) = statement completed, no value
/// Ok(Some(val)) = statement produced a value (expression statement, return)
/// Err(ControlFlow) = break/continue/return signal propagating up
/// Err(Error) = actual compile error
pub type StmtResult = std::result::Result<Option<ComptimeValue>, ComptimeSignal>;

/// A signal from a comptime statement — either a real error or a control flow event.
#[derive(Debug, Clone)]
pub enum ComptimeSignal {
    Error(CompileError),
    Flow(ComptimeControlFlow),
}

impl From<CompileError> for ComptimeSignal {
    fn from(e: CompileError) -> Self {
        ComptimeSignal::Error(e)
    }
}

impl ComptimeSignal {
    /// Convert back to a CompileError, treating unhandled control flow as an error.
    pub fn into_error(self) -> CompileError {
        match self {
            ComptimeSignal::Error(e) => e,
            ComptimeSignal::Flow(cf) => {
                CompileError::ComptimeError(format!("Unhandled control flow: {}", cf), None)
            }
        }
    }
}

// Sentinel prefixes for tunneling control flow through CompileError boundaries.
// When control flow signals need to pass through evaluate_expression (which returns
// Result<_, CompileError>), we encode them as ComptimeError with these prefixes
// and decode them back in execute_statement/execute_loop.
const FLOW_BREAK: &str = "\x00__flow_break__";
const FLOW_CONTINUE: &str = "\x00__flow_continue__";

/// Encode a control flow signal as a CompileError for tunneling through expression evaluation.
pub fn flow_to_error(cf: &ComptimeControlFlow) -> CompileError {
    match cf {
        ComptimeControlFlow::Break => CompileError::ComptimeError(FLOW_BREAK.to_string(), None),
        ComptimeControlFlow::Continue => {
            CompileError::ComptimeError(FLOW_CONTINUE.to_string(), None)
        }
        ComptimeControlFlow::Return(v) => {
            CompileError::ComptimeError(format!("Unexpected return in expression: {}", v), None)
        }
    }
}

/// Try to decode a CompileError back into a control flow signal.
pub fn error_to_flow(e: &CompileError) -> Option<ComptimeControlFlow> {
    if let CompileError::ComptimeError(msg, _) = e {
        if msg == FLOW_BREAK {
            return Some(ComptimeControlFlow::Break);
        }
        if msg == FLOW_CONTINUE {
            return Some(ComptimeControlFlow::Continue);
        }
    }
    None
}

// AST node wrapper for compile-time introspection.
// This enables Zen programs to walk and inspect the AST via meta.type_info().
#[derive(Debug, Clone)]
pub enum ASTNodeValue {
    Expression(Expression),
    Statement(Statement),
    Declaration(Declaration),
    Type(AstType),
    Pattern(Pattern),
    Program(ast::Program),
}

// Value types that can exist at compile time
#[derive(Debug, Clone)]
pub enum ComptimeValue {
    // Primitive values
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),

    // Compound values
    Array(Vec<ComptimeValue>),
    Struct {
        name: String,
        fields: HashMap<String, ComptimeValue>,
    },

    // Type value (for type-level computations)
    Type(AstType),

    // Function value (for higher-order functions)
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        closure: super::Environment,
    },

    // AST node (for meta-programming / AST walking from Zen code)
    ASTNode(Rc<ASTNodeValue>),

    // Special values
    Void,
    Null,
}

// Manual PartialEq: compare structurally, Functions/ASTNodes compare by discriminant only
impl PartialEq for ComptimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ComptimeValue::I8(a), ComptimeValue::I8(b)) => a == b,
            (ComptimeValue::I16(a), ComptimeValue::I16(b)) => a == b,
            (ComptimeValue::I32(a), ComptimeValue::I32(b)) => a == b,
            (ComptimeValue::I64(a), ComptimeValue::I64(b)) => a == b,
            (ComptimeValue::U8(a), ComptimeValue::U8(b)) => a == b,
            (ComptimeValue::U16(a), ComptimeValue::U16(b)) => a == b,
            (ComptimeValue::U32(a), ComptimeValue::U32(b)) => a == b,
            (ComptimeValue::U64(a), ComptimeValue::U64(b)) => a == b,
            (ComptimeValue::F32(a), ComptimeValue::F32(b)) => a.to_bits() == b.to_bits(),
            (ComptimeValue::F64(a), ComptimeValue::F64(b)) => a.to_bits() == b.to_bits(),
            (ComptimeValue::Bool(a), ComptimeValue::Bool(b)) => a == b,
            (ComptimeValue::String(a), ComptimeValue::String(b)) => a == b,
            (ComptimeValue::Array(a), ComptimeValue::Array(b)) => a == b,
            (
                ComptimeValue::Struct {
                    name: n1,
                    fields: f1,
                },
                ComptimeValue::Struct {
                    name: n2,
                    fields: f2,
                },
            ) => n1 == n2 && f1 == f2,
            (ComptimeValue::Type(a), ComptimeValue::Type(b)) => a == b,
            (ComptimeValue::Void, ComptimeValue::Void) => true,
            (ComptimeValue::Null, ComptimeValue::Null) => true,
            _ => false,
        }
    }
}

impl fmt::Display for ComptimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComptimeValue::I8(n) => write!(f, "{}", n),
            ComptimeValue::I16(n) => write!(f, "{}", n),
            ComptimeValue::I32(n) => write!(f, "{}", n),
            ComptimeValue::I64(n) => write!(f, "{}", n),
            ComptimeValue::U8(n) => write!(f, "{}", n),
            ComptimeValue::U16(n) => write!(f, "{}", n),
            ComptimeValue::U32(n) => write!(f, "{}", n),
            ComptimeValue::U64(n) => write!(f, "{}", n),
            ComptimeValue::F32(n) => write!(f, "{}", n),
            ComptimeValue::F64(n) => write!(f, "{}", n),
            ComptimeValue::Bool(b) => write!(f, "{}", b),
            ComptimeValue::String(s) => write!(f, "{}", s),
            ComptimeValue::Null => write!(f, "null"),
            ComptimeValue::Void => write!(f, "void"),
            ComptimeValue::Array(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            ComptimeValue::Struct { name, fields } => {
                write!(f, "{}{{{} fields}}", name, fields.len())
            }
            ComptimeValue::Type(t) => write!(f, "{:?}", t),
            ComptimeValue::Function { name, .. } => write!(f, "<fn {}>", name),
            ComptimeValue::ASTNode(node) => write!(f, "<ASTNode::{}>", meta::variant_name(node)),
        }
    }
}

impl ComptimeValue {
    /// Convert a compile-time value to an AST expression
    pub fn to_expression(&self) -> Result<Expression> {
        match self {
            ComptimeValue::I8(v) => Ok(Expression::Integer8(*v)),
            ComptimeValue::I16(v) => Ok(Expression::Integer16(*v)),
            ComptimeValue::I32(v) => Ok(Expression::Integer32(*v)),
            ComptimeValue::I64(v) => Ok(Expression::Integer64(*v)),
            ComptimeValue::U8(v) => Ok(Expression::Unsigned8(*v)),
            ComptimeValue::U16(v) => Ok(Expression::Unsigned16(*v)),
            ComptimeValue::U32(v) => Ok(Expression::Unsigned32(*v)),
            ComptimeValue::U64(v) => Ok(Expression::Unsigned64(*v)),
            ComptimeValue::F32(v) => Ok(Expression::Float32(*v)),
            ComptimeValue::F64(v) => Ok(Expression::Float64(*v)),
            ComptimeValue::Bool(v) => Ok(Expression::Boolean(*v)),
            ComptimeValue::String(v) => Ok(Expression::String(v.clone())),
            ComptimeValue::Array(values) => {
                let exprs: Result<Vec<_>> = values.iter().map(|v| v.to_expression()).collect();
                Ok(Expression::ArrayLiteral(exprs?))
            }
            ComptimeValue::ASTNode(node) => match node.as_ref() {
                ASTNodeValue::Expression(e) => Ok(e.clone()),
                other => Err(CompileError::ComptimeError(
                    format!(
                        "Cannot convert {:?} AST node to runtime expression",
                        std::mem::discriminant(other)
                    ),
                    None,
                )),
            },
            ComptimeValue::Void => Ok(Expression::Unit),
            other => Err(CompileError::ComptimeError(
                format!("Cannot convert {} to expression", other),
                None,
            )),
        }
    }

    /// Get the type of a compile-time value
    pub fn get_type(&self) -> AstType {
        match self {
            ComptimeValue::I8(_) => AstType::I8,
            ComptimeValue::I16(_) => AstType::I16,
            ComptimeValue::I32(_) => AstType::I32,
            ComptimeValue::I64(_) => AstType::I64,
            ComptimeValue::U8(_) => AstType::U8,
            ComptimeValue::U16(_) => AstType::U16,
            ComptimeValue::U32(_) => AstType::U32,
            ComptimeValue::U64(_) => AstType::U64,
            ComptimeValue::F32(_) => AstType::F32,
            ComptimeValue::F64(_) => AstType::F64,
            ComptimeValue::Bool(_) => AstType::Bool,
            ComptimeValue::String(_) => crate::ast::resolve_string_struct_type(),
            ComptimeValue::Array(v) => {
                if v.is_empty() {
                    AstType::Slice(Box::new(AstType::Void))
                } else {
                    AstType::Slice(Box::new(v[0].get_type()))
                }
            }
            ComptimeValue::Struct { name, .. } => AstType::Struct {
                name: name.clone(),
                fields: vec![],
            },
            ComptimeValue::Type(_) => AstType::Generic {
                name: "Type".to_string(),
                type_args: vec![],
            },
            ComptimeValue::Void => AstType::Void,
            ComptimeValue::Null => AstType::ptr(AstType::Void),
            ComptimeValue::Function { .. } => AstType::Generic {
                name: "Function".to_string(),
                type_args: vec![],
            },
            ComptimeValue::ASTNode(node) => {
                let variant = match node.as_ref() {
                    ASTNodeValue::Expression(_) => "Expression",
                    ASTNodeValue::Statement(_) => "Statement",
                    ASTNodeValue::Declaration(_) => "Declaration",
                    ASTNodeValue::Type(_) => "Type",
                    ASTNodeValue::Pattern(_) => "Pattern",
                    ASTNodeValue::Program(_) => "Program",
                };
                AstType::Generic {
                    name: "ASTNode".to_string(),
                    type_args: vec![AstType::Struct {
                        name: variant.to_string(),
                        fields: vec![],
                    }],
                }
            }
        }
    }
}
