pub mod symbols;
pub mod lexer;
pub mod parser;
pub mod core;
pub mod functions;
pub mod literals;
pub mod binary_ops;
pub mod control_flow;
pub mod pointers;
pub mod strings;
pub mod structs;
pub mod types;

// Re-export the symbols module for easier access
pub use symbols::{Symbol, SymbolTable};
pub use core::{Compiler, Type, StructTypeInfo};

// Expression codegen modules - now integrated into main Compiler impl
mod expr_codegen;
mod stmt_codegen; 