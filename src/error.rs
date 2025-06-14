use thiserror::Error;
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
            CompileError::SyntaxError(msg, span) => write!(f, "Syntax Error: {}{}", msg, span.map_or("".to_string(), |s| format!(" at line {} column {}", s.line, s.column))),
            CompileError::UndeclaredVariable(name, span) => write!(f, "Undeclared variable: '{}'{}", name, span.map_or("".to_string(), |s| format!(" at line {} column {}", s.line, s.column))),
            CompileError::UndeclaredFunction(name, span) => write!(f, "Undeclared function: '{}'{}", name, span.map_or("".to_string(), |s| format!(" at line {} column {}", s.line, s.column))),
            CompileError::TypeMismatch { expected, found, span } => write!(f, "Type mismatch: Expected {}, found {}{}", expected, found, span.map_or("".to_string(), |s| format!(" at line {} column {}", s.line, s.column))),
            CompileError::InvalidLoopCondition(msg, span) => write!(f, "Invalid loop condition: {}{}", msg, span.map_or("".to_string(), |s| format!(" at line {} column {}", s.line, s.column))),
            CompileError::MissingReturnStatement(func_name, span) => write!(f, "Missing return statement in function '{}'{}", func_name, span.map_or("".to_string(), |s| format!(" at line {} column {}", s.line, s.column))),
            CompileError::InternalError(msg, span) => write!(f, "Internal Compiler Error: {}{}", msg, span.map_or("".to_string(), |s| format!(" at line {} column {}", s.line, s.column))),
            CompileError::UnsupportedFeature(msg, span) => write!(f, "Unsupported Feature: {}{}", msg, span.map_or("".to_string(), |s| format!(" at line {} column {}", s.line, s.column))),
        }
    }
}

impl std::error::Error for CompileError {}

pub type Result<T> = std::result::Result<T, CompileError>; 