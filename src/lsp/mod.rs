use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tokio::sync::RwLock;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::Program;

#[derive(Debug)]
pub struct ZenLanguageServer {
    client: Client,
    documents: Arc<RwLock<HashMap<String, String>>>,
}

impl ZenLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn parse_document(&self, uri: &str) -> Result<Program> {
        let documents = self.documents.read().await;
        let content = documents.get(uri)
            .ok_or_else(|| tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::InvalidParams))?;
        
        let lexer = Lexer::new(content);
        let mut parser = Parser::new(lexer);
        
        parser.parse_program()
            .map_err(|_e| tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::ParseError))
    }

    async fn get_diagnostics(&self, uri: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        match self.parse_document(uri).await {
            Ok(_) => {
                // No errors, document is valid
            }
            Err(_) => {
                // Add a generic error diagnostic
                diagnostics.push(Diagnostic {
                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("zen".to_string()),
                    message: "Failed to parse document".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }
        
        diagnostics
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ZenLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "zen-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        ":".to_string(),
                        "=".to_string(),
                        "(".to_string(),
                    ]),
                    all_commit_characters: None,
                    completion_item: None,
                    work_done_progress_options: Default::default(),
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                    identifier: Some("zen".to_string()),
                    inter_file_dependencies: true,
                    workspace_diagnostics: true,
                    work_done_progress_options: Default::default(),
                })),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Zen Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        
        {
            let mut documents = self.documents.write().await;
            documents.insert(uri.clone(), text);
        }
        
        let diagnostics = self.get_diagnostics(&uri).await;
        self.client
            .publish_diagnostics(uri.parse().unwrap(), diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        
        // Update document content
        for change in params.content_changes {
            let text = change.text;
            let mut documents = self.documents.write().await;
            documents.insert(uri.clone(), text);
        }
        
        // Publish diagnostics
        let diagnostics = self.get_diagnostics(&uri).await;
        self.client
            .publish_diagnostics(uri.parse().unwrap(), diagnostics, None)
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let mut documents = self.documents.write().await;
        documents.remove(&uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let _position = params.text_document_position.position;
        
        // Get document content
        let documents = self.documents.read().await;
        let _content = documents.get(&uri)
            .ok_or_else(|| tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::InvalidParams))?;
        
        // Simple completion based on position
        let mut completions = Vec::new();
        
        // Add basic keywords
        completions.push(CompletionItem {
            label: "loop".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Loop construct".to_string()),
            ..Default::default()
        });
        
        completions.push(CompletionItem {
            label: "break".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Break from loop".to_string()),
            ..Default::default()
        });
        
        completions.push(CompletionItem {
            label: "continue".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Continue loop iteration".to_string()),
            ..Default::default()
        });
        
        completions.push(CompletionItem {
            label: "struct".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define a struct".to_string()),
            ..Default::default()
        });
        
        completions.push(CompletionItem {
            label: "enum".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define an enum".to_string()),
            ..Default::default()
        });
        
        completions.push(CompletionItem {
            label: "comptime".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Compile-time evaluation".to_string()),
            ..Default::default()
        });
        
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;
        
        // Get document content
        let documents = self.documents.read().await;
        let _content = documents.get(&uri)
            .ok_or_else(|| tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::InvalidParams))?;
        
        // Simple hover information
        let hover_text = format!("Position: {:?}", position);
        
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(hover_text)),
            range: None,
        }))
    }

    async fn goto_definition(&self, _params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        // For now, return None - we'll implement this later
        Ok(None)
    }
}

pub async fn run_lsp_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| ZenLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
} 