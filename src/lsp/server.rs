// Enhanced LSP Server for Zen Language
// Provides advanced IDE features with compiler integration

use lsp_server::{Connection, Message, Notification as ServerNotification, Request, Response};
use lsp_types::*;
use serde_json::Value;
use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use inkwell::context::Context;

// AST types used in type_inference.rs
use crate::ast::primitives;
use crate::compiler::Compiler;
use crate::error::{CompileError, Span};

use super::call_hierarchy::{
    handle_incoming_calls, handle_outgoing_calls, handle_prepare_call_hierarchy,
};
use super::code_action::handle_code_action;
use super::code_lens::handle_code_lens;
use super::completion::handle_completion as handle_completion_request;
use super::document_store::DocumentStore;
use super::formatting::handle_formatting;
use super::hover::handle_hover;
use super::inlay_hints::handle_inlay_hints;
use super::navigation::{
    handle_definition, handle_document_highlight, handle_references, handle_type_definition,
};
use super::rename::{handle_prepare_rename, handle_rename};
use super::semantic_tokens::handle_semantic_tokens;
use super::signature_help::handle_signature_help;
use super::symbols::{handle_document_symbols, handle_workspace_symbol};
use super::types::{AnalysisJob, AnalysisResult};
use super::utils::tokenize_with_lines;
use crate::lexer::Token;

/// Convert LSP position (line, character) to byte offset in content
fn position_to_byte_offset(content: &str, target_line: usize, target_char: usize) -> usize {
    let mut current_line = 0;
    let mut current_char = 0;

    for (byte_idx, ch) in content.char_indices() {
        if current_line == target_line && current_char == target_char {
            return byte_idx;
        }
        if ch == '\n' {
            current_line += 1;
            current_char = 0;
        } else {
            current_char += 1;
        }
    }

    // Clamp to content length for positions at end of file or beyond
    content.len()
}

/// Apply a text edit to document content, handling LSP Range positions
fn apply_text_edit(content: &str, range: &Range, new_text: &str) -> String {
    let (start_line, start_char) = (range.start.line as usize, range.start.character as usize);
    let (end_line, end_char) = (range.end.line as usize, range.end.character as usize);

    log::debug!(
        "[LSP] apply_text_edit: range=({}:{} -> {}:{}), new_text len={}, content len={}",
        start_line,
        start_char,
        end_line,
        end_char,
        new_text.len(),
        content.len()
    );

    let start_byte = position_to_byte_offset(content, start_line, start_char);
    let end_byte = position_to_byte_offset(content, end_line, end_char);

    log::debug!(
        "[LSP DEBUG] Converted positions: start_byte={}, end_byte={}",
        start_byte,
        end_byte
    );
    log::debug!(
        "[LSP DEBUG] Will replace: '{}'",
        &content[start_byte..end_byte.min(content.len())]
            .chars()
            .take(50)
            .collect::<String>()
    );

    // Ensure byte offsets are valid and ordered
    let start_byte = start_byte.min(content.len());
    let end_byte = end_byte.min(content.len());
    let (start_byte, end_byte) = if start_byte <= end_byte {
        (start_byte, end_byte)
    } else {
        log::debug!(
            "[LSP WARN] Start byte > end byte, swapping: {} > {}",
            start_byte,
            end_byte
        );
        (end_byte, start_byte)
    };

    // Apply the edit: replace the range with new_text
    let mut result =
        String::with_capacity(content.len() - (end_byte - start_byte) + new_text.len());
    result.push_str(&content[..start_byte]);
    result.push_str(new_text);
    result.push_str(&content[end_byte..]);

    log::debug!(
        "[LSP DEBUG] Result length: {} (was {})",
        result.len(),
        content.len()
    );
    result
}

// ============================================================================
// TYPES (moved to types.rs and document_store.rs)
// ============================================================================

/// Helper to extract span, severity, and error code from a CompileError
fn extract_error_info(error: &CompileError) -> (Option<Span>, DiagnosticSeverity, &'static str) {
    use CompileError::*;
    match error {
        // Errors with spans
        ParseError(_, span) => (span.clone(), DiagnosticSeverity::ERROR, "parse-error"),
        SyntaxError(_, span) => (span.clone(), DiagnosticSeverity::ERROR, "syntax-error"),
        TypeError(_, span) => (span.clone(), DiagnosticSeverity::ERROR, "type-error"),
        TypeMismatch { span, .. } => (span.clone(), DiagnosticSeverity::ERROR, "type-mismatch"),
        UndeclaredVariable(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            "undeclared-variable",
        ),
        UndeclaredFunction(_, span) => (
            span.clone(),
            DiagnosticSeverity::ERROR,
            "undeclared-function",
        ),
        UnexpectedToken { span, .. } => {
            (span.clone(), DiagnosticSeverity::ERROR, "unexpected-token")
        }
        InvalidPattern(_, span) => (span.clone(), DiagnosticSeverity::ERROR, "invalid-pattern"),
        InvalidSyntax { span, .. } => (span.clone(), DiagnosticSeverity::ERROR, "invalid-syntax"),
        DuplicateDeclaration {
            duplicate_location, ..
        } => (
            duplicate_location.clone(),
            DiagnosticSeverity::ERROR,
            "duplicate-declaration",
        ),
        ImportError(_, span) => (span.clone(), DiagnosticSeverity::ERROR, "import-error"),
        FFIError(_, span) => (span.clone(), DiagnosticSeverity::ERROR, "ffi-error"),
        InvalidLoopCondition(_, span) => (span.clone(), DiagnosticSeverity::ERROR, "invalid-loop"),
        InternalError(_, span) => (span.clone(), DiagnosticSeverity::ERROR, "internal-error"),
        // Warnings
        MissingTypeAnnotation(_, span) => {
            (span.clone(), DiagnosticSeverity::WARNING, "missing-type")
        }
        MissingReturnStatement(_, span) => {
            (span.clone(), DiagnosticSeverity::WARNING, "missing-return")
        }
        UnsupportedFeature(_, span) => (
            span.clone(),
            DiagnosticSeverity::WARNING,
            "unsupported-feature",
        ),
        // Errors without spans
        FileNotFound(_, _) => (None, DiagnosticSeverity::ERROR, "file-not-found"),
        ComptimeError(_) => (None, DiagnosticSeverity::ERROR, "comptime-error"),
        BuildError(_) => (None, DiagnosticSeverity::ERROR, "build-error"),
        FileError(_) => (None, DiagnosticSeverity::ERROR, "file-error"),
        CyclicDependency(_) => (None, DiagnosticSeverity::ERROR, "cyclic-dependency"),
    }
}

/// Convert span to LSP Range (Position pair)
fn span_to_lsp_range(span: Option<Span>) -> Range {
    if let Some(span) = span {
        let line = if span.line > 0 {
            span.line as u32 - 1
        } else {
            0
        };
        Range {
            start: Position {
                line,
                character: span.column as u32,
            },
            end: Position {
                line,
                character: (span.column + (span.end - span.start).max(1)) as u32,
            },
        }
    } else {
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        }
    }
}

// Convert CompileError to LSP Diagnostic (standalone function for reuse)
fn compile_error_to_diagnostic(error: CompileError) -> Diagnostic {
    let (span, severity, code) = extract_error_info(&error);
    let (message, hint) = enhance_error_message(&error);

    Diagnostic {
        range: span_to_lsp_range(span),
        severity: Some(severity),
        code: Some(lsp_types::NumberOrString::String(code.to_string())),
        code_description: get_error_documentation(code),
        source: Some("zen-compiler".to_string()),
        message: if let Some(h) = hint {
            format!("{}\n\nHint: {}", message, h)
        } else {
            message
        },
        related_information: None,
        tags: get_diagnostic_tags(&error),
        data: None,
    }
}

/// Enhance error messages with helpful context and suggestions
fn enhance_error_message(error: &CompileError) -> (String, Option<String>) {
    use CompileError::*;
    match error {
        TypeMismatch {
            expected, found, ..
        } => {
            let base_msg = format!("Type mismatch: expected `{}`, found `{}`", expected, found);
            let hint = generate_type_mismatch_hint(expected, found);
            (base_msg, hint)
        }
        UndeclaredVariable(name, _) => {
            let base_msg = format!("Undeclared variable `{}`", name);
            let hint = Some(format!(
                "Make sure `{}` is defined before use, or check for typos in the variable name",
                name
            ));
            (base_msg, hint)
        }
        UndeclaredFunction(name, _) => {
            let base_msg = format!("Undeclared function `{}`", name);
            let hint = Some(format!(
                "Make sure `{}` is defined or imported. Use Ctrl+. for quick-fix suggestions",
                name
            ));
            (base_msg, hint)
        }
        MissingTypeAnnotation(msg, _) => {
            let hint = Some(
                "Zen requires explicit type annotations for function parameters and struct fields"
                    .to_string(),
            );
            (msg.clone(), hint)
        }
        MissingReturnStatement(msg, _) => {
            let hint = Some(
                "Add a return statement or ensure the last expression in the function matches the return type"
                    .to_string(),
            );
            (msg.clone(), hint)
        }
        ImportError(msg, _) => {
            let hint = if msg.contains("not found") {
                Some(
                    "Check the module path. Use @std for stdlib, @this for local modules"
                        .to_string(),
                )
            } else {
                None
            };
            (msg.clone(), hint)
        }
        ParseError(msg, _) | SyntaxError(msg, _) => {
            let hint = if msg.contains("expected") {
                Some("Check for missing brackets, parentheses, or punctuation".to_string())
            } else {
                None
            };
            (msg.clone(), hint)
        }
        InvalidPattern(msg, _) => {
            let hint =
                Some("Pattern matching uses: `expr ? | pattern1 { } | pattern2 { }`".to_string());
            (msg.clone(), hint)
        }
        DuplicateDeclaration { name, .. } => {
            let base_msg = format!("Duplicate declaration of `{}`", name);
            let hint = Some(format!(
                "`{}` is already defined in this scope. Rename one of the declarations",
                name
            ));
            (base_msg, hint)
        }
        _ => (format!("{}", error), None),
    }
}

/// Check if a type string represents a specific well-known type.
/// Handles both generic forms (e.g., "Option<i32>") and plain names (e.g., "Option").
/// Avoids false positives like "InvalidOption" matching "Option".
fn is_type_named(type_str: &str, type_name: &str) -> bool {
    // Check for exact match or generic form (e.g., "Option" or "Option<...")
    type_str == type_name
        || type_str.starts_with(&format!("{}<", type_name))
        || type_str.starts_with(&format!("{}(", type_name)) // Function type
}

/// Generate specific hints for type mismatch errors
fn generate_type_mismatch_hint(expected: &str, actual: &str) -> Option<String> {
    // String conversion hints - StaticString is an alias, check exact match or generic
    let expected_is_static = is_type_named(expected, "StaticString");
    let actual_is_static = is_type_named(actual, "StaticString");
    let expected_is_string = is_type_named(expected, "String");
    let actual_is_string = is_type_named(actual, "String");

    if expected_is_static && actual_is_string {
        return Some("Use `.as_static()` to convert String to StaticString".to_string());
    }
    if expected_is_string && actual_is_static {
        return Some("Use `String.from(value)` to convert StaticString to String".to_string());
    }

    // Optional handling - use precise type matching
    let expected_is_option = is_type_named(expected, "Option");
    let actual_is_option = is_type_named(actual, "Option");

    if expected_is_option && !actual_is_option {
        return Some(format!(
            "Wrap the value with `Some({})` to create an Option",
            actual
        ));
    }
    if !expected_is_option && actual_is_option {
        return Some(
            "Unwrap the Option with `.unwrap()` or handle with pattern matching".to_string(),
        );
    }

    // Result handling - use precise type matching
    let expected_is_result = is_type_named(expected, "Result");
    let actual_is_result = is_type_named(actual, "Result");

    if expected_is_result && !actual_is_result {
        return Some(format!(
            "Wrap the value with `Ok({})` to create a Result",
            actual
        ));
    }
    if !expected_is_result && actual_is_result {
        return Some(
            "Handle the Result with `.raise()` to propagate errors or `.unwrap()` to panic"
                .to_string(),
        );
    }

    // Allocator hints - use precise type matching
    if is_type_named(expected, "Allocator") {
        return Some("Pass `get_default_allocator()` or a specific allocator instance".to_string());
    }

    // Numeric type hints
    if is_numeric_type(expected) && is_numeric_type(actual) {
        return Some(format!(
            "Cast with `@as({}, value)` for explicit conversion",
            expected
        ));
    }

    // Pointer hints
    if expected.contains('*') && !actual.contains('*') {
        return Some("Use `&value` to get a pointer to the value".to_string());
    }
    if !expected.contains('*') && actual.contains('*') {
        return Some("Dereference with `value.*` to get the underlying value".to_string());
    }

    None
}

fn is_numeric_type(type_name: &str) -> bool {
    // Use centralized primitive definitions
    primitives::is_numeric_type_name(type_name)
}

/// Get documentation URL for error codes
fn get_error_documentation(code: &str) -> Option<CodeDescription> {
    // Could point to actual documentation once available
    let url = match code {
        "type-mismatch" => "https://zenlang.dev/docs/errors#type-mismatch",
        "undeclared-variable" => "https://zenlang.dev/docs/errors#undeclared-variable",
        "missing-type" => "https://zenlang.dev/docs/errors#missing-type-annotation",
        _ => return None,
    };

    // For now, return None since docs don't exist yet
    // Once docs exist, uncomment:
    // Some(CodeDescription { href: Url::parse(url).ok()? })
    let _ = url;
    None
}

/// Get diagnostic tags (deprecated, unnecessary)
fn get_diagnostic_tags(error: &CompileError) -> Option<Vec<DiagnosticTag>> {
    use CompileError::*;
    match error {
        // Mark unused warnings with the UNNECESSARY tag
        MissingTypeAnnotation(msg, _) if msg.contains("unused") => {
            Some(vec![DiagnosticTag::UNNECESSARY])
        }
        // Could add DEPRECATED tag for deprecated API usage
        _ => None,
    }
}

// DocumentStore implementation moved to document_store.rs
// Using the one from document_store module

// ============================================================================
// ENHANCED LSP SERVER
// ============================================================================

pub struct ZenLanguageServer {
    connection: Connection,
    store: Arc<Mutex<DocumentStore>>,
    capabilities: ServerCapabilities,
}

impl ZenLanguageServer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (connection, _io_threads) = Connection::stdio();

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(vec![
                    ".".to_string(),
                    ":".to_string(),
                    "@".to_string(),
                    "?".to_string(),
                ]),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                all_commit_characters: None,
                completion_item: None,
            }),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                retrigger_characters: None,
                work_done_progress_options: WorkDoneProgressOptions::default(),
            }),
            definition_provider: Some(OneOf::Left(true)),
            type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
            references_provider: Some(OneOf::Left(true)),
            document_highlight_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(false),
            }),
            document_formatting_provider: Some(OneOf::Left(true)),
            document_range_formatting_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Right(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            })),
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            inlay_hint_provider: Some(OneOf::Left(true)),
            call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                    SemanticTokensRegistrationOptions {
                        text_document_registration_options: TextDocumentRegistrationOptions {
                            document_selector: Some(vec![DocumentFilter {
                                language: Some("zen".to_string()),
                                scheme: None,
                                pattern: None,
                            }]),
                        },
                        semantic_tokens_options: SemanticTokensOptions {
                            work_done_progress_options: WorkDoneProgressOptions {
                                work_done_progress: None,
                            },
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::NAMESPACE,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::CLASS,
                                    SemanticTokenType::ENUM,
                                    SemanticTokenType::INTERFACE,
                                    SemanticTokenType::STRUCT,
                                    SemanticTokenType::TYPE_PARAMETER,
                                    SemanticTokenType::PARAMETER,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::PROPERTY,
                                    SemanticTokenType::ENUM_MEMBER,
                                    SemanticTokenType::EVENT,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::METHOD,
                                    SemanticTokenType::MACRO,
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::MODIFIER,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::REGEXP,
                                    SemanticTokenType::OPERATOR,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DECLARATION,
                                    SemanticTokenModifier::DEFINITION,
                                    SemanticTokenModifier::READONLY,
                                    SemanticTokenModifier::STATIC,
                                    SemanticTokenModifier::DEPRECATED,
                                    SemanticTokenModifier::ABSTRACT,
                                    SemanticTokenModifier::ASYNC,
                                    SemanticTokenModifier::MODIFICATION,
                                    SemanticTokenModifier::DOCUMENTATION,
                                    SemanticTokenModifier::DEFAULT_LIBRARY,
                                ],
                            },
                            full: Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
                            range: Some(false),
                        },
                        static_registration_options: StaticRegistrationOptions { id: None },
                    },
                ),
            ),
            ..Default::default()
        };

        Ok(Self {
            connection,
            store: Arc::new(Mutex::new(DocumentStore::new())),
            capabilities,
        })
    }

    pub fn run(mut self) -> Result<(), Box<dyn Error>> {
        // Starting Enhanced Zen Language Server

        let server_capabilities = serde_json::to_value(&self.capabilities)?;
        let initialization_params = self.connection.initialize(server_capabilities)?;

        if let Ok(params) = serde_json::from_value::<InitializeParams>(initialization_params) {
            // Try workspace_folders first (preferred), fall back to root_uri for compatibility
            #[allow(deprecated)]
            if let Some(folders) = params.workspace_folders {
                if let Some(first_folder) = folders.first() {
                    if let Ok(mut s) = self.store.lock() {
                        s.set_workspace_root(first_folder.uri.clone());
                    }
                }
            } else if let Some(root_uri) = params.root_uri {
                if let Ok(mut s) = self.store.lock() {
                    s.set_workspace_root(root_uri);
                }
            }
        }

        // Start background analysis thread
        let (analysis_tx, analysis_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();

        // Give the analysis sender to the document store
        if let Ok(mut s) = self.store.lock() {
            s.set_analysis_sender(analysis_tx);
        }

        // Spawn background analysis worker
        let _analysis_thread = thread::spawn(move || {
            Self::background_analysis_worker(analysis_rx, result_tx);
        });

        // Zen LSP initialized with enhanced capabilities

        // Start main loop with result receiver for async diagnostics
        self.main_loop_with_background(result_rx)?;

        // Zen Language Server shutting down
        Ok(())
    }

    fn background_analysis_worker(
        job_rx: Receiver<AnalysisJob>,
        result_tx: Sender<AnalysisResult>,
    ) {
        // Background analysis worker started

        // Create LLVM context and compiler (reused for all analyses)
        let context = Context::create();
        let compiler = Compiler::new(&context);

        while let Ok(job) = job_rx.recv() {
            // Analyzing document

            let errors = compiler.analyze_for_diagnostics(&job.program);

            // Analysis complete

            // Convert compiler errors to LSP diagnostics using the shared function
            let diagnostics: Vec<Diagnostic> = errors
                .into_iter()
                .map(compile_error_to_diagnostic)
                .collect();

            // Run TypeChecker to get TypeContext for semantic features
            // Must load modules and pass them to TypeChecker for type alias resolution
            let type_context = {
                use crate::ast::Declaration;
                use crate::module_system::ModuleSystem;
                use crate::typechecker::TypeChecker;

                // Load modules exactly like analyzer.rs does
                let mut module_system = ModuleSystem::new();
                for decl in &job.program.declarations {
                    if let Declaration::ModuleImport { module_path, .. } = decl {
                        let _ = module_system.load_module(module_path);
                    }
                }

                // Merge programs to include all imports
                let merged_program = module_system.merge_programs(job.program.clone());

                // Create TypeChecker with module information
                let mut type_checker = TypeChecker::new();
                type_checker.with_stdlib_modules(module_system.get_modules());

                // Type-check the MERGED program, not the original
                type_checker
                    .check_program(&merged_program)
                    .ok()
                    .map(std::sync::Arc::new)
            };

            let result = AnalysisResult {
                uri: job.uri,
                version: job.version,
                diagnostics,
                type_context,
            };

            // Send result back (ignore if receiver disconnected)
            let _ = result_tx.send(result);
        }

        // Background analysis worker stopped
    }

    fn main_loop_with_background(
        &mut self,
        result_rx: Receiver<AnalysisResult>,
    ) -> Result<(), Box<dyn Error>> {
        loop {
            // Check for background analysis results (non-blocking)
            match result_rx.try_recv() {
                Ok(result) => {
                    // Store TypeContext in document for semantic features
                    if let Some(type_ctx) = &result.type_context {
                        if let Ok(mut store) = self.store.lock() {
                            if let Some(doc) = store.documents.get_mut(&result.uri) {
                                doc.type_context = Some(type_ctx.clone());
                            }
                        }
                    }
                    // Publishing background diagnostics
                    self.publish_diagnostics(result.uri, result.diagnostics)?;
                }
                Err(TryRecvError::Empty) => {
                    // No results ready, continue
                }
                Err(TryRecvError::Disconnected) => {
                    // Background analysis thread disconnected
                    break;
                }
            }

            // Handle LSP messages (with timeout to check background results)
            use crate::lsp::search_limits::LSP_POLL_TIMEOUT_MS;
            use std::time::Duration;
            let timeout = Duration::from_millis(LSP_POLL_TIMEOUT_MS);

            match self.connection.receiver.recv_timeout(timeout) {
                Ok(msg) => match msg {
                    Message::Request(req) => {
                        if self.connection.handle_shutdown(&req)? {
                            return Ok(());
                        }
                        self.handle_request(req)?;
                    }
                    Message::Notification(notif) => {
                        self.handle_notification(notif)?;
                    }
                    Message::Response(_) => {}
                },
                Err(err) => {
                    // Handle connection errors
                    // If it's a timeout, continue; if disconnected, exit
                    if err.is_timeout() {
                        // Timeout is expected, continue loop
                    } else {
                        // Connection closed or other error, exit gracefully
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_request(&self, req: Request) -> Result<(), Box<dyn Error>> {
        // Handling request - optimized: avoid unnecessary clones by matching on method first
        let method = req.method.as_str();
        let response = match method {
            "textDocument/hover" => self.handle_hover(req),
            "textDocument/completion" => self.handle_completion(req),
            "textDocument/definition" => self.handle_definition(req),
            "textDocument/typeDefinition" => self.handle_type_definition(req),
            "textDocument/references" => self.handle_references(req),
            "textDocument/documentHighlight" => self.handle_document_highlight(req),
            "textDocument/documentSymbol" => self.handle_document_symbols(req),
            "textDocument/formatting" => self.handle_formatting(req),
            "textDocument/rename" => self.handle_rename(req),
            "textDocument/prepareRename" => self.handle_prepare_rename(req),
            "textDocument/codeAction" => self.handle_code_action(req),
            "textDocument/semanticTokens/full" => self.handle_semantic_tokens(req),
            "textDocument/signatureHelp" => self.handle_signature_help(req),
            "textDocument/inlayHint" => self.handle_inlay_hints(req),
            "textDocument/codeLens" => self.handle_code_lens(req),
            "textDocument/foldingRange" => self.handle_folding_range(req),
            "workspace/symbol" => self.handle_workspace_symbol(req),
            "textDocument/prepareCallHierarchy" => self.handle_prepare_call_hierarchy(req),
            "callHierarchy/incomingCalls" => self.handle_incoming_calls(req),
            "callHierarchy/outgoingCalls" => self.handle_outgoing_calls(req),
            _ => Response {
                id: req.id,
                result: Some(Value::Null),
                error: None,
            },
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_notification(&self, notif: ServerNotification) -> Result<(), Box<dyn Error>> {
        match notif.method.as_str() {
            "textDocument/didOpen" => {
                let params: DidOpenTextDocumentParams = serde_json::from_value(notif.params)?;
                let diagnostics = match self.store.lock() {
                    Ok(mut s) => s.open(
                        params.text_document.uri.clone(),
                        params.text_document.version,
                        params.text_document.text,
                    ),
                    Err(_) => Vec::new(),
                };
                self.publish_diagnostics(params.text_document.uri, diagnostics)?;

                // Notify client to refresh semantic tokens when document opens
                self.request_semantic_tokens_refresh()?;
            }
            "textDocument/didChange" => {
                let params: DidChangeTextDocumentParams = serde_json::from_value(notif.params)?;

                // Handle all content changes (LSP spec allows array)
                // IMPORTANT: Changes must be applied sequentially, each to the updated document state
                let diagnostics = match self.store.lock() {
                    Ok(mut s) => {
                        // Get the current document content (or start fresh if document doesn't exist)
                        let mut current_content = s
                            .documents
                            .get(&params.text_document.uri)
                            .map(|doc| doc.content.clone())
                            .unwrap_or_default();

                        // Apply each change sequentially to the current content
                        for change in &params.content_changes {
                            current_content = if let Some(range) = &change.range {
                                // Incremental change: apply the edit to the current document content
                                log::debug!("[LSP] Applying incremental change: range=({}:{} -> {}:{}), text='{}'",
                                         range.start.line, range.start.character,
                                         range.end.line, range.end.character,
                                         change.text.chars().take(20).collect::<String>());
                                apply_text_edit(&current_content, range, &change.text)
                            } else {
                                // Full document replacement (no range field)
                                log::debug!("[LSP] Applying full document replacement");
                                change.text.clone()
                            };
                        }

                        // Update the document with the final content after all changes
                        s.update(
                            params.text_document.uri.clone(),
                            params.text_document.version,
                            current_content,
                        )
                    }
                    Err(e) => {
                        log::debug!("[LSP] Error locking store for didChange: {:?}", e);
                        Vec::new()
                    }
                };

                if let Err(e) =
                    self.publish_diagnostics(params.text_document.uri.clone(), diagnostics)
                {
                    log::debug!("[LSP] Error publishing diagnostics: {:?}", e);
                }

                // Notify client to refresh semantic tokens when document changes
                if let Err(e) = self.request_semantic_tokens_refresh() {
                    log::debug!("[LSP] Error requesting semantic tokens refresh: {:?}", e);
                }
            }
            "initialized" => {
                // Client initialized
            }
            _ => {}
        }
        Ok(())
    }

    fn publish_diagnostics(
        &self,
        uri: Url,
        diagnostics: Vec<Diagnostic>,
    ) -> Result<(), Box<dyn Error>> {
        // Publishing diagnostics (previously logged individually)

        let params = PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        };

        let notification = ServerNotification {
            method: "textDocument/publishDiagnostics".to_string(),
            params: serde_json::to_value(params)?,
        };

        self.connection
            .sender
            .send(Message::Notification(notification))?;
        Ok(())
    }

    /// Request the client to refresh semantic tokens when document changes
    fn request_semantic_tokens_refresh(&self) -> Result<(), Box<dyn Error>> {
        let notification = ServerNotification {
            method: "workspace/semanticTokens/refresh".to_string(),
            params: serde_json::Value::Null,
        };
        self.connection
            .sender
            .send(Message::Notification(notification))?;
        Ok(())
    }

    fn handle_hover(&self, req: Request) -> Response {
        handle_hover(req, &self.store)
    }

    fn handle_completion(&self, req: Request) -> Response {
        handle_completion_request(req, &self.store)
    }

    fn handle_definition(&self, req: Request) -> Response {
        handle_definition(req, &self.store)
    }

    fn handle_type_definition(&self, req: Request) -> Response {
        handle_type_definition(req, &self.store)
    }

    // extract_type_name moved to navigation.rs

    fn handle_document_highlight(&self, req: Request) -> Response {
        handle_document_highlight(req, &self.store)
    }

    // Navigation helper functions moved to navigation.rs

    fn handle_references(&self, req: Request) -> Response {
        handle_references(req, &self.store)
    }

    fn handle_document_symbols(&self, req: Request) -> Response {
        handle_document_symbols(req, &self.store)
    }

    fn handle_formatting(&self, req: Request) -> Response {
        handle_formatting(req, Arc::clone(&self.store))
    }

    fn handle_rename(&self, req: Request) -> Response {
        handle_rename(req, &self.store)
    }

    fn handle_prepare_rename(&self, req: Request) -> Response {
        handle_prepare_rename(req, &self.store)
    }

    fn handle_signature_help(&self, req: Request) -> Response {
        handle_signature_help(req, &self.store)
    }

    fn handle_inlay_hints(&self, req: Request) -> Response {
        handle_inlay_hints(req, &self.store)
    }

    fn handle_code_lens(&self, req: Request) -> Response {
        handle_code_lens(req, &self.store)
    }

    fn handle_folding_range(&self, req: Request) -> Response {
        let params: FoldingRangeParams = match serde_json::from_value(req.params) {
            Ok(p) => p,
            Err(_) => {
                return Response {
                    id: req.id,
                    result: Some(Value::Null),
                    error: None,
                };
            }
        };

        let store = match self.store.lock() {
            Ok(s) => s,
            Err(_) => {
                return Response {
                    id: req.id,
                    result: Some(
                        serde_json::to_value(Vec::<FoldingRange>::new()).unwrap_or(Value::Null),
                    ),
                    error: None,
                };
            }
        };

        let mut ranges = Vec::new();

        if let Some(doc) = store.documents.get(&params.text_document.uri) {
            // Use lexer-based tokenization to correctly handle braces in strings/comments
            let tokens = tokenize_with_lines(&doc.content);
            let mut brace_stack: Vec<(u32, FoldingRangeKind)> = Vec::new();

            for token_info in tokens {
                match &token_info.token {
                    Token::Symbol('{') => {
                        brace_stack.push((token_info.line, FoldingRangeKind::Region));
                    }
                    Token::Symbol('}') => {
                        if let Some((start_line, kind)) = brace_stack.pop() {
                            if token_info.line > start_line {
                                ranges.push(FoldingRange {
                                    start_line,
                                    start_character: None,
                                    end_line: token_info.line,
                                    end_character: None,
                                    kind: Some(kind),
                                    collapsed_text: None,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Response {
            id: req.id,
            result: Some(serde_json::to_value(ranges).unwrap_or(Value::Null)),
            error: None,
        }
    }

    fn handle_code_action(&self, req: Request) -> Response {
        handle_code_action(req, &self.store)
    }

    // Code action helper functions moved to code_action.rs

    fn handle_semantic_tokens(&self, req: Request) -> Response {
        handle_semantic_tokens(req, &self.store)
    }

    // generate_semantic_tokens moved to semantic_tokens.rs

    // Type inference functions moved to type_inference.rs

    // Type inference and completion helper functions moved to type_inference.rs and completion.rs

    // Signature help helper functions moved to signature_help.rs

    // Inlay hints helper functions moved to inlay_hints.rs

    fn handle_workspace_symbol(&self, req: Request) -> Response {
        handle_workspace_symbol(req, &self.store)
    }

    fn handle_prepare_call_hierarchy(&self, req: Request) -> Response {
        handle_prepare_call_hierarchy(req, &self.store)
    }

    fn handle_incoming_calls(&self, req: Request) -> Response {
        handle_incoming_calls(req, &self.store)
    }

    fn handle_outgoing_calls(&self, req: Request) -> Response {
        handle_outgoing_calls(req, &self.store)
    }

    // Call hierarchy helper functions moved to call_hierarchy.rs
    // Code action helper functions moved to code_action.rs

    // Hover helper functions moved to hover.rs

    // Reference finding functions moved to navigation.rs
    // Workspace utilities moved to workspace.rs
}
