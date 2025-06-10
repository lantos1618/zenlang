use thiserror::Error;
use inkwell::builder::BuilderError;
use inkwell::support::LLVMString;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Undefined function: {0}")]
    UndefinedFunction(String),
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: String,
        actual: String,
    },
    #[error("Invalid function type: {0}")]
    InvalidFunctionType(String),
    #[error("Invalid binary operation: {op} between {left} and {right}")]
    InvalidBinaryOperation {
        op: String,
        left: String,
        right: String,
    },
    #[error("Invalid pattern matching: {0}")]
    InvalidPatternMatching(String),
    #[error("LLVM error: {0}")]
    LLVMError(String),
    #[error("Internal compiler error: {0}")]
    InternalError(String),
    #[error("Invalid pointer operation: {0}")]
    InvalidPointerOperation(String),
    #[error("Invalid loop condition: {0}")]
    InvalidLoopCondition(String),
}

impl From<BuilderError> for CompileError {
    fn from(err: BuilderError) -> Self {
        CompileError::LLVMError(err.to_string())
    }
}

impl From<String> for CompileError {
    fn from(err: String) -> Self {
        CompileError::InternalError(err)
    }
}

impl From<LLVMString> for CompileError {
    fn from(err: LLVMString) -> Self {
        CompileError::LLVMError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CompileError>; 