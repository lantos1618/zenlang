// Code Lens Module for Zen LSP
// Handles textDocument/codeLens requests

use lsp_server::{Request, Response};
use lsp_types::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::ast::Declaration;

use super::document_store::DocumentStore;
use super::helpers::{null_response, success_response, try_lock, try_parse_params};

// ============================================================================
// PUBLIC HANDLER FUNCTION
// ============================================================================

pub fn handle_code_lens(req: Request, store: &Arc<Mutex<DocumentStore>>) -> Response {
    let params: CodeLensParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let store = match try_lock(store.as_ref(), &req) {
        Ok(s) => s,
        Err(_) => return success_response(&req, Vec::<CodeLens>::new()),
    };

    let doc = match store.documents.get(&params.text_document.uri) {
        Some(d) => d,
        None => return null_response(&req),
    };

    // Debug: log to file
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/zen-lsp-codelens.log")
    {
        use std::io::Write;
        let _ = writeln!(f, "=== CodeLens request PID={} ===", std::process::id());
        let _ = writeln!(f, "URI: {}", params.text_document.uri);
    }

    let mut lenses = Vec::new();
    // Track which functions we've already processed to prevent duplicates from AST
    let mut processed_functions: HashSet<String> = HashSet::new();

    // Generate code lenses for all functions from the parsed AST
    if let Some(ast) = &doc.ast {
        for decl in ast.iter() {
            if let Declaration::Function(func) = decl {
                let func_name = &func.name;

                // Skip if we've already processed this function (handles potential AST duplicates)
                if processed_functions.contains(func_name) {
                    continue;
                }
                processed_functions.insert(func_name.clone());

                // Get line from symbol info if available, otherwise fall back to content search
                let line_num = doc
                    .symbols
                    .get(func_name)
                    .map(|s| s.range.start.line as usize)
                    .or_else(|| find_function_line(&doc.content, func_name));

                if let Some(line) = line_num {
                    let range = Range {
                        start: Position {
                            line: line as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line as u32,
                            character: 0,
                        },
                    };

                    // main and build get both Run and Build buttons
                    if func_name == "main" || func_name == "build" {
                        lenses.push(CodeLens {
                            range,
                            command: Some(Command {
                                title: "▶ Run".to_string(),
                                command: "zen.run".to_string(),
                                arguments: Some(vec![
                                    serde_json::to_value(&params.text_document.uri).unwrap(),
                                    serde_json::to_value(func_name).unwrap(),
                                    serde_json::to_value(line).unwrap(),
                                ]),
                            }),
                            data: None,
                        });

                        lenses.push(CodeLens {
                            range,
                            command: Some(Command {
                                title: "🔨 Build".to_string(),
                                command: "zen.build".to_string(),
                                arguments: Some(vec![
                                    serde_json::to_value(&params.text_document.uri).unwrap(),
                                    serde_json::to_value(func_name).unwrap(),
                                    serde_json::to_value(line).unwrap(),
                                ]),
                            }),
                            data: None,
                        });
                    } else if is_test_function(func_name) {
                        // Test functions get "Run Test" button
                        lenses.push(CodeLens {
                            range,
                            command: Some(Command {
                                title: "▶ Run Test".to_string(),
                                command: "zen.runTest".to_string(),
                                arguments: Some(vec![
                                    serde_json::to_value(&params.text_document.uri).unwrap(),
                                    serde_json::to_value(func_name).unwrap(),
                                ]),
                            }),
                            data: None,
                        });
                    }
                    // Regular functions don't get Run buttons - can't run them in isolation
                }
            }
        }
    }

    // Final deduplication: remove any lenses with same line and title
    let mut seen_lens: HashSet<(u32, String)> = HashSet::new();
    let lenses: Vec<CodeLens> = lenses
        .into_iter()
        .filter(|lens| {
            let line = lens.range.start.line;
            let title = lens
                .command
                .as_ref()
                .map(|c| c.title.clone())
                .unwrap_or_default();
            let key = (line, title);
            if seen_lens.contains(&key) {
                false
            } else {
                seen_lens.insert(key);
                true
            }
        })
        .collect();

    // Debug: log final lenses
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/zen-lsp-codelens.log")
    {
        use std::io::Write;
        let _ = writeln!(f, "Returning {} lenses:", lenses.len());
        for lens in &lenses {
            if let Some(cmd) = &lens.command {
                let _ = writeln!(f, "  - Line {}: {}", lens.range.start.line, cmd.title);
            }
        }
        let _ = writeln!(f);
    }

    success_response(&req, lenses)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn is_test_function(name: &str) -> bool {
    name.starts_with("test_") || name.ends_with("_test") || name.contains("_test_")
}

fn find_function_line(content: &str, func_name: &str) -> Option<usize> {
    for (line_num, line) in content.lines().enumerate() {
        // Look for function definition: "func_name = ("
        // Must have func_name before '=' and '(' after '='
        if let Some(eq_pos) = line.find('=') {
            let before_eq = &line[..eq_pos];
            let after_eq = &line[eq_pos..];

            // Check if the function name is in the part before '='
            // and '(' appears after '='
            if before_eq.contains(func_name) && after_eq.contains('(') {
                // Verify it's the EXACT function name (not a substring like "compute_main" for "main")
                let trimmed = before_eq.trim();
                // Only exact match - don't use ends_with which could match "compute_main" for "main"
                if trimmed == func_name {
                    return Some(line_num);
                }
            }
        }
    }
    None
}
