use inkwell::builder::BuilderError;
use inkwell::support::LLVMString;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    SyntaxError(String, Option<Span>),
    UndeclaredVariable(String, Option<Span>),
    UndeclaredFunction(String, Option<Span>),
    TypeMismatch {
        expected: String,
        found: String,
        span: Option<Span>,
    },
    InvalidLoopCondition(String, Option<Span>),
    MissingReturnStatement(String, Option<Span>),
    InternalError(String, Option<Span>),
    UnsupportedFeature(String, Option<Span>),
    TypeError(String, Option<Span>),
    FileNotFound(String, Option<String>),
    ParseError(String, Option<Span>),
    ComptimeError(String),
}

impl From<BuilderError> for CompileError {
    fn from(err: BuilderError) -> Self {
        CompileError::InternalError(err.to_string(), None)
    }
}

impl From<String> for CompileError {
    fn from(err: String) -> Self {
        CompileError::InternalError(err, None)
    }
}

impl From<LLVMString> for CompileError {
    fn from(err: LLVMString) -> Self {
        CompileError::InternalError(err.to_string(), None)
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompileError::SyntaxError(msg, span) => write!(f, "Syntax Error: {}{}", msg, span.as_ref().map(|s| format!(" at line {} column {}", s.line, s.column)).unwrap_or_default()),
            CompileError::UndeclaredVariable(name, span) => write!(f, "Undeclared variable: '{}'{}", name, span.as_ref().map(|s| format!(" at line {} column {}", s.line, s.column)).unwrap_or_default()),
            CompileError::UndeclaredFunction(name, span) => write!(f, "Undeclared function: '{}'{}", name, span.as_ref().map(|s| format!(" at line {} column {}", s.line, s.column)).unwrap_or_default()),
            CompileError::TypeMismatch { expected, found, span } => write!(f, "Type mismatch: Expected {}, found {}{}", expected, found, span.as_ref().map(|s| format!(" at line {} column {}", s.line, s.column)).unwrap_or_default()),
            CompileError::InvalidLoopCondition(msg, span) => write!(f, "Invalid loop condition: {}{}", msg, span.as_ref().map(|s| format!(" at line {} column {}", s.line, s.column)).unwrap_or_default()),
            CompileError::MissingReturnStatement(func_name, span) => write!(f, "Missing return statement in function '{}'{}", func_name, span.as_ref().map(|s| format!(" at line {} column {}", s.line, s.column)).unwrap_or_default()),
            CompileError::InternalError(msg, span) => write!(f, "Internal Compiler Error: {}{}", msg, span.as_ref().map(|s| format!(" at line {} column {}", s.line, s.column)).unwrap_or_default()),
            CompileError::UnsupportedFeature(msg, _) => write!(f, "Unsupported feature: {}", msg),
            CompileError::TypeError(msg, span) => write!(f, "Type error: {}{}", msg, span.as_ref().map(|s| format!(" at line {} column {}", s.line, s.column)).unwrap_or_default()),
            CompileError::FileNotFound(path, detail) => write!(f, "File not found: {}{}", path, detail.as_ref().map(|d| format!(" ({})", d)).unwrap_or_default()),
            CompileError::ParseError(msg, span) => write!(f, "Parse error: {}{}", msg, span.as_ref().map(|s| format!(" at line {} column {}", s.line, s.column)).unwrap_or_default()),
            CompileError::ComptimeError(msg) => write!(f, "Compile-time error: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

pub type Result<T> = std::result::Result<T, CompileError>; 