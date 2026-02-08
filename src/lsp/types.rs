// LSP Types and Data Structures

use crate::ast::Program;
use crate::ast::{AstType, Declaration};
use crate::lexer::Token;
use crate::type_context::TypeContext;
use lsp_types::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,
    pub version: i32,
    pub content: String,
    pub content_hash: u64,
    pub tokens: Vec<Token>,
    pub ast: Option<Vec<Declaration>>,
    pub diagnostics: Vec<Diagnostic>,
    pub symbols: HashMap<String, SymbolInfo>,
    pub last_analysis: Option<Instant>,
    /// Type context from TypeChecker - the source of truth for semantic analysis.
    /// Populated during full document analysis for intelligent completions.
    pub type_context: Option<Arc<TypeContext>>,
}

/// Fast hash for content comparison (using FNV-1a algorithm)
pub fn hash_content(content: &str) -> u64 {
    const FNV_PRIME: u64 = 0x00000100000001B3;
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;

    let mut hash = FNV_OFFSET;
    for byte in content.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

impl Document {
    /// Get lines as an iterator - avoids allocation
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.content.lines()
    }

    /// Get a specific line by index
    pub fn get_line(&self, index: usize) -> Option<&str> {
        self.content.lines().nth(index)
    }
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub selection_range: Range,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub type_info: Option<AstType>,
    pub definition_uri: Option<Url>,
    pub references: Vec<Range>,
    pub enum_variants: Option<Vec<String>>, // For enums: list of variant names
    pub params: Option<Vec<(String, AstType)>>, // For functions/methods: structured parameter list
}

#[derive(Debug, Clone)]
pub struct UfcMethodInfo {
    pub receiver: String,
    pub method_name: String,
}

#[derive(Debug)]
pub enum ZenCompletionContext {
    General,
    UfcMethod {
        receiver_type: String,
    },
    ModulePath {
        base: String,
    },
    /// Inside a struct literal: `Point { x: 1, ▊` - suggests remaining fields
    StructLiteral {
        struct_name: String,
    },
    /// After `|` in pattern matching: `expr ? | ▊` - suggests enum variants
    PatternMatch {
        matched_type: String,
    },
}

#[derive(Debug)]
pub enum SymbolScope {
    Local { function_name: String },
    ModuleLevel,
    Unknown,
}

// Background analysis job
#[derive(Debug, Clone)]
pub struct AnalysisJob {
    pub uri: Url,
    pub version: i32,
    pub content: String,
    pub content_hash: u64,
    pub program: Program,
}

// Background analysis result
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub uri: Url,
    pub version: i32,
    pub diagnostics: Vec<Diagnostic>,
    /// TypeContext extracted during analysis - enables semantic completions
    pub type_context: Option<Arc<TypeContext>>,
}
