// Enhanced LSP Server for Zen Language
// Provides advanced IDE features with compiler integration

use lsp_server::{Connection, Message, Request, Response, ResponseError, ErrorCode, Notification as ServerNotification};
use lsp_types::*;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::path::PathBuf;
use std::fs;

// Import from our submodules
use super::types::{Document, SymbolInfo, AnalysisJob, AnalysisResult};
use super::document_store::DocumentStore;
use super::utils::{compile_error_to_diagnostic, format_type, format_symbol_kind, symbol_kind_to_completion_kind};
use super::completion;
use super::navigation;
use super::symbols;
use super::formatting;

use crate::ast::{Declaration, AstType, Expression, Statement, Program};
use crate::lexer::{Lexer, Token};
use crate::parser::Parser;
use crate::compiler::Compiler;

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
                            document_selector: Some(vec![
                                DocumentFilter {
                                    language: Some("zen".to_string()),
                                    scheme: None,
                                    pattern: None,
                                }
                            ]),
                        },
                        semantic_tokens_options: SemanticTokensOptions {
                            work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None },
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
                            range: Some(true),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                        static_registration_options: StaticRegistrationOptions { id: None },
                    }
                )
            ),
            ..ServerCapabilities::default()
        };

        let store = Arc::new(Mutex::new(DocumentStore::new()));

        Ok(Self {
            connection,
            store,
            capabilities,
        })
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        eprintln!("[LSP] Starting Zen Language Server");

        let (sender, receiver) = mpsc::channel();

        // Set up background analysis
        {
            let mut store = self.store.lock().unwrap();
            store.set_analysis_sender(sender.clone());
        }

        // Spawn background analysis thread
        let analysis_store = Arc::clone(&self.store);
        let analysis_connection = self.connection.sender.clone();
        thread::spawn(move || {
            Self::background_analysis_worker(receiver, analysis_store, analysis_connection);
        });

        let server_capabilities = serde_json::to_value(&self.capabilities).unwrap();
        let initialization_params = self.connection.initialize(server_capabilities)?;

        eprintln!("[LSP] Server initialized with params: {:#?}", initialization_params);

        // Extract workspace root from initialization params
        if let Ok(init_params) = serde_json::from_value::<InitializeParams>(initialization_params) {
            if let Some(workspace_folders) = init_params.workspace_folders {
                if let Some(first_folder) = workspace_folders.first() {
                    let workspace_uri = first_folder.uri.clone();
                    eprintln!("[LSP] Setting workspace root: {}", workspace_uri);

                    // Clone what we need before spawning the thread
                    let store_clone = Arc::clone(&self.store);

                    // Spawn background thread for workspace indexing
                    thread::spawn(move || {
                        eprintln!("[LSP] Starting background workspace indexing...");

                        // Index workspace without holding the lock
                        if let Ok(root_path) = workspace_uri.to_file_path() {
                            let workspace_symbols = DocumentStore::index_workspace_files(&root_path);

                            // Now acquire lock just to merge the symbols
                            if let Ok(mut store) = store_clone.lock() {
                                eprintln!("[LSP] Merging {} workspace symbols into store", workspace_symbols.len());

                                // Merge workspace symbols (don't overwrite stdlib)
                                for (name, symbol) in workspace_symbols {
                                    if !store.stdlib_symbols.contains_key(&name) {
                                        store.workspace_symbols.insert(name, symbol);
                                    }
                                }

                                store.set_workspace_root(workspace_uri);
                                eprintln!("[LSP] Background workspace indexing complete");
                            }
                        }
                    });
                }
            } else if let Some(root_uri) = init_params.root_uri {
                eprintln!("[LSP] Setting workspace root from root_uri: {}", root_uri);
                let store_clone = Arc::clone(&self.store);

                thread::spawn(move || {
                    eprintln!("[LSP] Starting background workspace indexing...");

                    if let Ok(root_path) = root_uri.to_file_path() {
                        let workspace_symbols = DocumentStore::index_workspace_files(&root_path);

                        if let Ok(mut store) = store_clone.lock() {
                            eprintln!("[LSP] Merging {} workspace symbols into store", workspace_symbols.len());

                            for (name, symbol) in workspace_symbols {
                                if !store.stdlib_symbols.contains_key(&name) {
                                    store.workspace_symbols.insert(name, symbol);
                                }
                            }

                            store.set_workspace_root(root_uri);
                            eprintln!("[LSP] Background workspace indexing complete");
                        }
                    }
                });
            }
        }

        self.main_loop()?;

        Ok(())
    }

    fn background_analysis_worker(
        receiver: mpsc::Receiver<AnalysisJob>,
        store: Arc<Mutex<DocumentStore>>,
        connection: lsp_server::Sender,
    ) {
        eprintln!("[LSP] Background analysis worker started");

        while let Ok(job) = receiver.recv() {
            eprintln!("[LSP] Background analysis for {}", job.uri);

            // Run full compiler analysis (expensive)
            let mut diagnostics = Vec::new();

            let mut compiler = Compiler::new();
            if let Err(errors) = compiler.compile_program(&job.program) {
                for error in errors {
                    diagnostics.push(compile_error_to_diagnostic(error));
                }
            }

            let result = AnalysisResult {
                uri: job.uri.clone(),
                version: job.version,
                diagnostics: diagnostics.clone(),
            };

            // Update diagnostics in store
            if let Ok(mut store) = store.lock() {
                if let Some(doc) = store.documents.get_mut(&result.uri) {
                    if doc.version == result.version {
                        doc.diagnostics = result.diagnostics.clone();
                    }
                }
            }

            // Send diagnostics to client
            let notification = ServerNotification {
                method: "textDocument/publishDiagnostics".to_string(),
                params: serde_json::to_value(PublishDiagnosticsParams {
                    uri: result.uri,
                    diagnostics: result.diagnostics,
                    version: Some(result.version),
                }).unwrap(),
            };

            let _ = connection.send(Message::Notification(notification));
        }

        eprintln!("[LSP] Background analysis worker stopped");
    }

    fn main_loop(&self) -> Result<(), Box<dyn Error>> {
        eprintln!("[LSP] Entering main loop");

        for msg in &self.connection.receiver {
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    self.handle_request(req)?;
                }
                Message::Notification(not) => {
                    self.handle_notification(not)?;
                }
                Message::Response(resp) => {
                    eprintln!("[LSP] Received response: {:?}", resp);
                }
            }
        }

        Ok(())
    }

    fn handle_request(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let method = req.method.as_str();

        match method {
            "textDocument/completion" => self.handle_completion(req),
            "textDocument/hover" => self.handle_hover(req),
            "textDocument/signatureHelp" => self.handle_signature_help(req),
            "textDocument/definition" => self.handle_goto_definition(req),
            "textDocument/typeDefinition" => self.handle_type_definition(req),
            "textDocument/references" => self.handle_references(req),
            "textDocument/documentHighlight" => self.handle_document_highlight(req),
            "textDocument/documentSymbol" => self.handle_document_symbols(req),
            "workspace/symbol" => self.handle_workspace_symbols(req),
            "textDocument/codeAction" => self.handle_code_actions(req),
            "textDocument/codeLens" => self.handle_code_lens(req),
            "textDocument/formatting" => self.handle_formatting(req),
            "textDocument/rangeFormatting" => self.handle_range_formatting(req),
            "textDocument/rename" => self.handle_rename(req),
            "textDocument/prepareRename" => self.handle_prepare_rename(req),
            "textDocument/foldingRange" => self.handle_folding_range(req),
            "textDocument/inlayHint" => self.handle_inlay_hints(req),
            "textDocument/semanticTokens/full" => self.handle_semantic_tokens_full(req),
            "textDocument/semanticTokens/range" => self.handle_semantic_tokens_range(req),
            "callHierarchy/incomingCalls" => self.handle_incoming_calls(req),
            "callHierarchy/outgoingCalls" => self.handle_outgoing_calls(req),
            "textDocument/prepareCallHierarchy" => self.handle_prepare_call_hierarchy(req),
            _ => {
                eprintln!("[LSP] Unhandled request: {}", method);
                let response = Response {
                    id: req.id,
                    result: None,
                    error: Some(ResponseError {
                        code: ErrorCode::MethodNotFound as i32,
                        message: format!("Method not found: {}", method),
                        data: None,
                    }),
                };
                self.connection.sender.send(Message::Response(response))?;
                Ok(())
            }
        }
    }

    fn handle_notification(&self, not: ServerNotification) -> Result<(), Box<dyn Error>> {
        let method = not.method.as_str();

        match method {
            "textDocument/didOpen" => self.handle_did_open(not),
            "textDocument/didChange" => self.handle_did_change(not),
            "textDocument/didSave" => self.handle_did_save(not),
            "textDocument/didClose" => self.handle_did_close(not),
            _ => {
                eprintln!("[LSP] Unhandled notification: {}", method);
                Ok(())
            }
        }
    }

    // ============================================================================
    // NOTIFICATION HANDLERS
    // ============================================================================

    fn handle_did_open(&self, not: ServerNotification) -> Result<(), Box<dyn Error>> {
        let params: DidOpenTextDocumentParams = serde_json::from_value(not.params)?;
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        let content = params.text_document.text;

        eprintln!("[LSP] Document opened: {}", uri);

        let diagnostics = {
            let mut store = self.store.lock().unwrap();
            store.open(uri.clone(), version, content)
        };

        self.publish_diagnostics(uri, diagnostics, Some(version))?;
        Ok(())
    }

    fn handle_did_change(&self, not: ServerNotification) -> Result<(), Box<dyn Error>> {
        let params: DidChangeTextDocumentParams = serde_json::from_value(not.params)?;
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().next() {
            let diagnostics = {
                let mut store = self.store.lock().unwrap();
                store.update(uri.clone(), version, change.text)
            };

            self.publish_diagnostics(uri, diagnostics, Some(version))?;
        }

        Ok(())
    }

    fn handle_did_save(&self, not: ServerNotification) -> Result<(), Box<dyn Error>> {
        let params: DidSaveTextDocumentParams = serde_json::from_value(not.params)?;
        eprintln!("[LSP] Document saved: {}", params.text_document.uri);
        Ok(())
    }

    fn handle_did_close(&self, not: ServerNotification) -> Result<(), Box<dyn Error>> {
        let params: DidCloseTextDocumentParams = serde_json::from_value(not.params)?;
        eprintln!("[LSP] Document closed: {}", params.text_document.uri);
        Ok(())
    }

    fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>, version: Option<i32>) -> Result<(), Box<dyn Error>> {
        let params = PublishDiagnosticsParams {
            uri,
            diagnostics,
            version,
        };

        let notification = ServerNotification {
            method: "textDocument/publishDiagnostics".to_string(),
            params: serde_json::to_value(params)?,
        };

        self.connection.sender.send(Message::Notification(notification))?;
        Ok(())
    }

    // ============================================================================
    // REQUEST HANDLERS - Delegated to submodules
    // ============================================================================

    fn handle_completion(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = {
            let store = self.store.lock().unwrap();
            completion::handle_completion(req, &store)
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_hover(&self, req: Request) -> Result<(), Box<dyn Error>> {
        // TODO: Move to navigation module
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(None::<Hover>)?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_signature_help(&self, req: Request) -> Result<(), Box<dyn Error>> {
        // TODO: Move to navigation module
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(None::<SignatureHelp>)?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_goto_definition(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = navigation::handle_definition(req, &self.store);
        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_type_definition(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = navigation::handle_type_definition(req, &self.store);
        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_references(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = navigation::handle_references(req, &self.store);
        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_document_highlight(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = navigation::handle_document_highlight(req, &self.store);
        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_document_symbols(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = {
            let store = self.store.lock().unwrap();
            symbols::handle_document_symbols(req, &store)
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_workspace_symbols(&self, req: Request) -> Result<(), Box<dyn Error>> {
        // TODO: Move to symbols module or implement
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(Vec::<SymbolInformation>::new())?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_code_actions(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(Vec::<CodeAction>::new())?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_code_lens(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(Vec::<CodeLens>::new())?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_formatting(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = formatting::handle_formatting(req, Arc::clone(&self.store));
        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_range_formatting(&self, req: Request) -> Result<(), Box<dyn Error>> {
        // TODO: Move to formatting module
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(Vec::<TextEdit>::new())?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_rename(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(None::<WorkspaceEdit>)?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_prepare_rename(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(None::<PrepareRenameResponse>)?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_folding_range(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(Vec::<FoldingRange>::new())?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_inlay_hints(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(Vec::<InlayHint>::new())?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_semantic_tokens_full(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(None::<SemanticTokensResult>)?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_semantic_tokens_range(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(None::<SemanticTokensRangeResult>)?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_prepare_call_hierarchy(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(None::<Vec<CallHierarchyItem>>)?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_incoming_calls(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(Vec::<CallHierarchyIncomingCall>::new())?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }

    fn handle_outgoing_calls(&self, req: Request) -> Result<(), Box<dyn Error>> {
        let response = Response {
            id: req.id,
            result: Some(serde_json::to_value(Vec::<CallHierarchyOutgoingCall>::new())?),
            error: None,
        };

        self.connection.sender.send(Message::Response(response))?;
        Ok(())
    }
}
