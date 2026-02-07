// LSP module for Zen Language Server

// Performance tuning constants for cross-document searches
// These limit how many documents we search to avoid blocking the LSP
pub mod search_limits {
    /// Maximum documents to search for definitions (high - definitions are important)
    pub const DEFINITION_SEARCH: usize = 50;
    /// Maximum documents to search for references (high - need completeness)
    pub const REFERENCES_SEARCH: usize = 50;
    /// Maximum documents to search for hover info (low - just need one match)
    pub const HOVER_SEARCH: usize = 10;
    /// Maximum documents to search for type inference (medium)
    pub const TYPE_INFERENCE_SEARCH: usize = 20;
    /// Maximum documents to search for enum variants (medium)
    pub const ENUM_SEARCH: usize = 30;
    /// Maximum documents for quick type lookups (low - fast path)
    pub const QUICK_TYPE_SEARCH: usize = 10;
    /// Maximum recursion depth for directory traversal (prevents stack overflow)
    pub const MAX_DIRECTORY_DEPTH: usize = 20;
    /// Maximum files to parse during workspace symbol search
    pub const MAX_FILES_TO_PARSE: usize = 100;
    /// Maximum iterations for loops that process unknown-length data (safety limit)
    pub const MAX_ITERATIONS: usize = 100;
    /// Timeout in milliseconds for LSP message polling (allows checking background results)
    pub const LSP_POLL_TIMEOUT_MS: u64 = 100;
}

// Submodules
pub mod analyzer;
pub mod call_hierarchy;
pub mod code_action;
pub mod code_lens;
pub mod completion;
pub mod document_store;
pub mod formatting;
pub mod helpers;
pub mod hover;
pub mod indexing;
pub mod inlay_hints;
pub mod navigation;
pub mod pattern_checking;
pub mod rename;
pub mod semantic_completion;
pub mod semantic_tokens;
pub mod server;
pub mod signature_help;
pub mod stdlib_resolver;
pub mod symbol_extraction;
pub mod symbols;

pub mod type_query;
pub mod types;
pub mod utils;

pub use server::ZenLanguageServer;

// Re-export commonly used types
pub use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DocumentSymbol, Hover, HoverContents, Location, MarkedString, Position, Range,
    SymbolInformation, SymbolKind,
};

// Re-export internal types for convenience
pub use document_store::DocumentStore;
pub use types::{AnalysisJob, AnalysisResult, Document, SymbolInfo, ZenCompletionContext};
