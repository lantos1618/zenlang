// Document lifecycle management (open, update, close)
use super::super::analyzer;
use super::super::types::{hash_content, AnalysisJob};
use super::DocumentStore;
use crate::ast::Program;
use lsp_types::*;
use std::time::Instant;

impl DocumentStore {
    pub fn open(&mut self, uri: Url, version: i32, content: String) -> Vec<Diagnostic> {
        let content_hash = hash_content(&content);
        let tokens = self.tokenize(&content);
        let ast = self.parse(&content);

        let symbols = if let Some(ref ast_decls) = ast {
            self.extract_symbols_from_ast(ast_decls, &content)
        } else {
            std::collections::HashMap::new()
        };

        let doc = super::super::types::Document {
            uri: uri.clone(),
            version,
            content: content.clone(),
            content_hash,
            tokens,
            ast: ast.clone(),
            diagnostics: Vec::new(),
            symbols,
            last_analysis: Some(Instant::now()),
            type_context: None, // Populated during background analysis
        };

        self.documents.insert(uri.clone(), doc);

        if let Some(ast_decls) = ast {
            if let Some(sender) = &self.analysis_sender {
                let job = AnalysisJob {
                    uri,
                    version,
                    content,
                    content_hash,
                    program: Program {
                        declarations: ast_decls,
                        statements: vec![],
                    },
                };
                let _ = sender.send(job);
            }
        }

        Vec::new()
    }

    pub fn update(&mut self, uri: Url, version: i32, content: String) -> Vec<Diagnostic> {
        const DEBOUNCE_MS: u128 = 300;

        let new_hash = hash_content(&content);

        // Check if content actually changed (hash-based)
        let content_changed = self
            .documents
            .get(&uri)
            .map(|doc| doc.content_hash != new_hash)
            .unwrap_or(true);

        // Skip full re-analysis if content unchanged AND we already have TypeContext
        if !content_changed {
            if let Some(doc) = self.documents.get(&uri) {
                let has_type_context = doc.type_context.is_some();
                let cached_diagnostics = doc.diagnostics.clone();
                if let Some(doc) = self.documents.get_mut(&uri) {
                    doc.version = version;
                }
                if has_type_context {
                    return cached_diagnostics;
                }
            }
        }

        let should_run_analysis = self
            .documents
            .get(&uri)
            .and_then(|doc| doc.last_analysis)
            .map(|last| last.elapsed().as_millis() >= DEBOUNCE_MS)
            .unwrap_or(true);

        // Quick diagnostics from TypeChecker (always run for immediate feedback)
        let diagnostics = self.analyze_document(&content, !should_run_analysis);

        let tokens = self.tokenize(&content);
        let ast = self.parse(&content);
        let symbols = self.extract_symbols(&content);

        // Send to background thread for full analysis if enabled and debounced
        if should_run_analysis {
            if let Some(ast_decls) = &ast {
                if let Some(sender) = &self.analysis_sender {
                    let job = AnalysisJob {
                        uri: uri.clone(),
                        version,
                        content: content.clone(),
                        content_hash: new_hash,
                        program: Program {
                            declarations: ast_decls.clone(),
                            statements: vec![],
                        },
                    };
                    let _ = sender.send(job);
                }
            }
        }

        if let Some(doc) = self.documents.get_mut(&uri) {
            doc.version = version;
            doc.content = content;
            doc.content_hash = new_hash;
            doc.tokens = tokens;
            doc.ast = ast;
            doc.diagnostics = diagnostics.clone(); // Need clone for return value
            doc.symbols = symbols;
            if should_run_analysis {
                doc.last_analysis = Some(Instant::now());
            }
        }

        diagnostics
    }

    pub(super) fn analyze_document(
        &self,
        content: &str,
        skip_expensive_analysis: bool,
    ) -> Vec<Diagnostic> {
        analyzer::analyze_document(content, skip_expensive_analysis, &self.documents)
    }
}
