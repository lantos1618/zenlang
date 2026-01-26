// Definition handler - go-to-definition

use super::imports::{find_import_info, find_import_info_from_ast};
use super::ufc::{find_ufc_method_at_position, resolve_ufc_method};
use super::utils::{find_symbol_at_position, find_symbol_definition_in_content};
use crate::lsp::document_store::DocumentStore;
use crate::lsp::helpers::{null_response, try_parse_params};
use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;

/// Handle textDocument/definition requests
pub fn handle_definition(
    req: Request,
    store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
) -> Response {
    log::debug!("[LSP DEFINITION] Starting definition request");
    let params: GotoDefinitionParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => {
            log::debug!("[LSP DEFINITION] Failed to parse params");
            return resp;
        }
    };

    log::debug!("[LSP DEFINITION] Waiting for store lock...");
    let lock_start = std::time::Instant::now();
    let store = match store.lock() {
        Ok(s) => {
            log::debug!(
                "[LSP DEFINITION] Got store lock in {:?}",
                lock_start.elapsed()
            );
            s
        }
        Err(e) => {
            log::debug!("[LSP DEFINITION] Failed to get store lock: {:?}", e);
            return null_response(&req);
        }
    };

    if let Some(doc) = store
        .documents
        .get(&params.text_document_position_params.text_document.uri)
    {
        let position = params.text_document_position_params.position;

        // Check if we're on a UFC method call
        if let Some(method_info) = find_ufc_method_at_position(&doc.content, position) {
            if let Some(location) = resolve_ufc_method(&method_info, &store) {
                return Response {
                    id: req.id,
                    result: Some(
                        serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                            .unwrap_or(Value::Null),
                    ),
                    error: None,
                };
            }
        }

        // Find the symbol at the cursor position
        if let Some(symbol_name) = find_symbol_at_position(&doc.content, position) {
            log::debug!("[LSP] Go-to-definition for: '{}'", symbol_name);

            // Check if clicking on @std.text or similar module path
            if symbol_name.starts_with("@std.") {
                let parts: Vec<&str> = symbol_name.split('.').collect();
                if parts.len() >= 3 {
                    let module_name = parts[1];
                    let symbol_name_in_module = parts[2..].join(".");

                    for (uri, stdlib_doc) in &store.documents {
                        let uri_path = uri.path();
                        if uri_path.contains("stdlib")
                            && (uri_path.ends_with(&format!("{}/{}.zen", module_name, module_name))
                                || uri_path.contains(&format!("{}/{}", module_name, module_name)))
                        {
                            if let Some(symbol_info) =
                                stdlib_doc.symbols.get(&symbol_name_in_module)
                            {
                                return Response {
                                    id: req.id,
                                    result: Some(
                                        serde_json::to_value(GotoDefinitionResponse::Scalar(
                                            Location {
                                                uri: uri.clone(),
                                                range: symbol_info.range,
                                            },
                                        ))
                                        .unwrap_or(Value::Null),
                                    ),
                                    error: None,
                                };
                            }

                            let location = Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                            };
                            return Response {
                                id: req.id,
                                result: Some(
                                    serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                                        .unwrap_or(Value::Null),
                                ),
                                error: None,
                            };
                        }
                    }
                }
            }

            // Handle qualified names like "build.LibOptions" where "build" is an imported module
            if !symbol_name.starts_with('@') && symbol_name.contains('.') {
                let parts: Vec<&str> = symbol_name.split('.').collect();
                if parts.len() >= 2 {
                    let module_alias = parts[0];
                    let member_name = parts[1..].join(".");
                    log::debug!(
                        "[LSP] Qualified name: module_alias={}, member_name={}",
                        module_alias,
                        member_name
                    );

                    // Look for import of module_alias (e.g., "{ build } = @std")
                    // Prefer AST-based lookup, fall back to string parsing
                    let import_info = doc
                        .ast
                        .as_ref()
                        .and_then(|ast| find_import_info_from_ast(ast, module_alias))
                        .or_else(|| find_import_info(&doc.content, module_alias, position));
                    if let Some(import_info) = import_info {
                        log::debug!(
                            "[LSP] Found import for {}: source={}",
                            module_alias,
                            import_info.source
                        );

                        // Resolve the module source
                        let module_path = if import_info.source == "@std" {
                            // Module imported from @std (e.g., { build } = @std means @std.build)
                            format!("@std.{}", module_alias)
                        } else {
                            import_info.source.clone()
                        };

                        // Try to resolve and find the symbol
                        if let Some(file_path) =
                            store.stdlib_resolver.resolve_module_path(&module_path)
                        {
                            log::debug!(
                                "[LSP] Resolved module path {} to file: {:?}",
                                module_path,
                                file_path
                            );

                            if let Ok(module_uri) = Url::from_file_path(&file_path) {
                                // Check stdlib_symbols first (has pre-indexed symbols with URIs)
                                if let Some(symbol_info) = store.stdlib_symbols.get(&member_name) {
                                    // Verify this symbol is from the expected module
                                    if let Some(def_uri) = &symbol_info.definition_uri {
                                        if def_uri.path().contains(&format!("/{}/", module_alias)) {
                                            log::debug!(
                                                "[LSP] Found {} in stdlib_symbols from {}",
                                                member_name,
                                                def_uri.path()
                                            );
                                            return Response {
                                                id: req.id,
                                                result: Some(
                                                    serde_json::to_value(
                                                        GotoDefinitionResponse::Scalar(Location {
                                                            uri: def_uri.clone(),
                                                            range: symbol_info.range,
                                                        }),
                                                    )
                                                    .unwrap_or(Value::Null),
                                                ),
                                                error: None,
                                            };
                                        }
                                    }
                                }

                                // Check if we have this document open
                                if let Some(module_doc) = store.documents.get(&module_uri) {
                                    if let Some(symbol_info) = module_doc.symbols.get(&member_name)
                                    {
                                        return Response {
                                            id: req.id,
                                            result: Some(
                                                serde_json::to_value(
                                                    GotoDefinitionResponse::Scalar(Location {
                                                        uri: module_uri,
                                                        range: symbol_info.range,
                                                    }),
                                                )
                                                .unwrap_or(Value::Null),
                                            ),
                                            error: None,
                                        };
                                    }
                                }

                                // File exists but not indexed - search it directly
                                if file_path.exists() {
                                    log::debug!(
                                        "[LSP] Searching file directly for {}: {:?}",
                                        member_name,
                                        file_path
                                    );
                                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                                        if let Some(range) = find_symbol_definition_in_content(
                                            &content,
                                            &member_name,
                                        ) {
                                            log::debug!(
                                                "[LSP] Found {} at line {} in {:?}",
                                                member_name,
                                                range.start.line,
                                                file_path
                                            );
                                            return Response {
                                                id: req.id,
                                                result: Some(
                                                    serde_json::to_value(
                                                        GotoDefinitionResponse::Scalar(Location {
                                                            uri: module_uri,
                                                            range,
                                                        }),
                                                    )
                                                    .unwrap_or(Value::Null),
                                                ),
                                                error: None,
                                            };
                                        }
                                    }
                                }
                            }
                        }

                        // Fallback: search stdlib documents for the module and member
                        for (uri, stdlib_doc) in &store.documents {
                            let uri_path = uri.path();
                            if uri_path.contains("stdlib")
                                && (uri_path
                                    .ends_with(&format!("{}/{}.zen", module_alias, module_alias))
                                    || uri_path
                                        .contains(&format!("{}/{}", module_alias, module_alias)))
                            {
                                if let Some(symbol_info) = stdlib_doc.symbols.get(&member_name) {
                                    return Response {
                                        id: req.id,
                                        result: Some(
                                            serde_json::to_value(GotoDefinitionResponse::Scalar(
                                                Location {
                                                    uri: uri.clone(),
                                                    range: symbol_info.range,
                                                },
                                            ))
                                            .unwrap_or(Value::Null),
                                        ),
                                        error: None,
                                    };
                                }
                            }
                        }
                    } else {
                        // No import found - try resolving as @std.module_alias anyway
                        // This handles cases where the import format isn't detected
                        log::debug!(
                            "[LSP] No import found for {}, trying @std.{}",
                            module_alias,
                            module_alias
                        );
                        let module_path = format!("@std.{}", module_alias);
                        if let Some(file_path) =
                            store.stdlib_resolver.resolve_module_path(&module_path)
                        {
                            if let Ok(module_uri) = Url::from_file_path(&file_path) {
                                // Check stdlib_symbols
                                if let Some(symbol_info) = store.stdlib_symbols.get(&member_name) {
                                    if let Some(def_uri) = &symbol_info.definition_uri {
                                        if def_uri.path().contains(&format!("/{}/", module_alias)) {
                                            return Response {
                                                id: req.id,
                                                result: Some(
                                                    serde_json::to_value(
                                                        GotoDefinitionResponse::Scalar(Location {
                                                            uri: def_uri.clone(),
                                                            range: symbol_info.range,
                                                        }),
                                                    )
                                                    .unwrap_or(Value::Null),
                                                ),
                                                error: None,
                                            };
                                        }
                                    }
                                }

                                // Direct file search
                                if file_path.exists() {
                                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                                        if let Some(range) = find_symbol_definition_in_content(
                                            &content,
                                            &member_name,
                                        ) {
                                            return Response {
                                                id: req.id,
                                                result: Some(
                                                    serde_json::to_value(
                                                        GotoDefinitionResponse::Scalar(Location {
                                                            uri: module_uri,
                                                            range,
                                                        }),
                                                    )
                                                    .unwrap_or(Value::Null),
                                                ),
                                                error: None,
                                            };
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Check if clicking on @std or std in the import statement itself
            let line = doc
                .content
                .lines()
                .nth(position.line as usize)
                .unwrap_or("");
            if (symbol_name == "@std"
                || symbol_name == "std"
                || (symbol_name.starts_with("@std") && symbol_name.contains('.')))
                && line.contains('=')
                && (line.contains("@std") || line.contains("= @std"))
            {
                let module_path = if symbol_name == "@std" || symbol_name == "std" {
                    "@std"
                } else {
                    &symbol_name
                };

                if let Some(file_path) = store.stdlib_resolver.resolve_module_path(module_path) {
                    if let Ok(uri) = Url::from_file_path(&file_path) {
                        if let Some(_doc) = store.documents.get(&uri) {
                            let location = Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                            };
                            return Response {
                                id: req.id,
                                result: Some(
                                    serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                                        .unwrap_or(Value::Null),
                                ),
                                error: None,
                            };
                        } else if let Some(workspace_root) = &store.workspace_root {
                            if let Ok(workspace_path) = workspace_root.to_file_path() {
                                if file_path.strip_prefix(&workspace_path).is_ok() {
                                    if let Ok(uri) = Url::from_file_path(&file_path) {
                                        let location = Location {
                                            uri,
                                            range: Range {
                                                start: Position {
                                                    line: 0,
                                                    character: 0,
                                                },
                                                end: Position {
                                                    line: 0,
                                                    character: 0,
                                                },
                                            },
                                        };
                                        return Response {
                                            id: req.id,
                                            result: Some(
                                                serde_json::to_value(
                                                    GotoDefinitionResponse::Scalar(location),
                                                )
                                                .unwrap_or(Value::Null),
                                            ),
                                            error: None,
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Check if this is an imported symbol (e.g., { io } = @std)
            // Prefer AST-based lookup, fall back to string parsing
            let import_info = doc
                .ast
                .as_ref()
                .and_then(|ast| find_import_info_from_ast(ast, &symbol_name))
                .or_else(|| find_import_info(&doc.content, &symbol_name, position));
            if let Some(import_info) = import_info {
                log::debug!(
                    "[LSP] Found import: {} from {}",
                    symbol_name,
                    import_info.source
                );

                if import_info.source.starts_with("@std") {
                    let module_name = if import_info.source == "@std" {
                        symbol_name.clone()
                    } else if import_info.source == "@std.types" {
                        "types".to_string()
                    } else {
                        import_info
                            .source
                            .strip_prefix("@std.")
                            .unwrap_or("io")
                            .to_string()
                    };

                    // Module import - resolve to the module file
                    if import_info.source == "@std" && symbol_name == module_name {
                        if let Some(symbol_info) = store.stdlib_symbols.get(&symbol_name) {
                            if let Some(uri) = &symbol_info.definition_uri {
                                let location = Location {
                                    uri: uri.clone(),
                                    range: symbol_info.range,
                                };
                                return Response {
                                    id: req.id,
                                    result: Some(
                                        serde_json::to_value(GotoDefinitionResponse::Scalar(
                                            location,
                                        ))
                                        .unwrap_or(Value::Null),
                                    ),
                                    error: None,
                                };
                            }
                        }

                        for uri in store.documents.keys() {
                            let uri_path = uri.path();
                            if uri_path.contains("stdlib")
                                && (uri_path
                                    .ends_with(&format!("{}/{}.zen", module_name, module_name))
                                    || uri_path
                                        .contains(&format!("{}/{}", module_name, module_name)))
                            {
                                let location = Location {
                                    uri: uri.clone(),
                                    range: Range {
                                        start: Position {
                                            line: 0,
                                            character: 0,
                                        },
                                        end: Position {
                                            line: 0,
                                            character: 0,
                                        },
                                    },
                                };
                                return Response {
                                    id: req.id,
                                    result: Some(
                                        serde_json::to_value(GotoDefinitionResponse::Scalar(
                                            location,
                                        ))
                                        .unwrap_or(Value::Null),
                                    ),
                                    error: None,
                                };
                            }
                        }

                        if let Some(workspace_root) = &store.workspace_root {
                            if let Ok(workspace_path) = workspace_root.to_file_path() {
                                let module_file = workspace_path
                                    .join("stdlib")
                                    .join(&module_name)
                                    .join(format!("{}.zen", module_name));
                                if module_file.exists() {
                                    if let Ok(uri) = Url::from_file_path(&module_file) {
                                        let location = Location {
                                            uri,
                                            range: Range {
                                                start: Position {
                                                    line: 0,
                                                    character: 0,
                                                },
                                                end: Position {
                                                    line: 0,
                                                    character: 0,
                                                },
                                            },
                                        };
                                        return Response {
                                            id: req.id,
                                            result: Some(
                                                serde_json::to_value(
                                                    GotoDefinitionResponse::Scalar(location),
                                                )
                                                .unwrap_or(Value::Null),
                                            ),
                                            error: None,
                                        };
                                    }
                                }
                            }
                        }
                    } else {
                        // Try to find the symbol in stdlib_symbols first
                        if let Some(symbol_info) = store.stdlib_symbols.get(&symbol_name) {
                            if let Some(uri) = &symbol_info.definition_uri {
                                let location = Location {
                                    uri: uri.clone(),
                                    range: symbol_info.range,
                                };
                                return Response {
                                    id: req.id,
                                    result: Some(
                                        serde_json::to_value(GotoDefinitionResponse::Scalar(
                                            location,
                                        ))
                                        .unwrap_or(Value::Null),
                                    ),
                                    error: None,
                                };
                            }
                        }

                        // Use stdlib_resolver to find the actual file path
                        if let Some(file_path) = store
                            .stdlib_resolver
                            .resolve_module_path(&import_info.source)
                        {
                            if let Ok(module_uri) = Url::from_file_path(&file_path) {
                                // Check if we have this document open
                                if let Some(stdlib_doc) = store.documents.get(&module_uri) {
                                    if let Some(symbol_info) = stdlib_doc.symbols.get(&symbol_name)
                                    {
                                        return Response {
                                            id: req.id,
                                            result: Some(
                                                serde_json::to_value(
                                                    GotoDefinitionResponse::Scalar(Location {
                                                        uri: module_uri,
                                                        range: symbol_info.range,
                                                    }),
                                                )
                                                .unwrap_or(Value::Null),
                                            ),
                                            error: None,
                                        };
                                    }
                                }

                                // File exists but not indexed - search it directly
                                if file_path.exists() {
                                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                                        if let Some(range) = find_symbol_definition_in_content(
                                            &content,
                                            &symbol_name,
                                        ) {
                                            return Response {
                                                id: req.id,
                                                result: Some(
                                                    serde_json::to_value(
                                                        GotoDefinitionResponse::Scalar(Location {
                                                            uri: module_uri,
                                                            range,
                                                        }),
                                                    )
                                                    .unwrap_or(Value::Null),
                                                ),
                                                error: None,
                                            };
                                        }
                                    }
                                }
                            }
                        }

                        // Fallback: search all stdlib documents
                        for (uri, stdlib_doc) in &store.documents {
                            let uri_path = uri.path();
                            if uri_path.contains("stdlib") {
                                if let Some(symbol_info) = stdlib_doc.symbols.get(&symbol_name) {
                                    return Response {
                                        id: req.id,
                                        result: Some(
                                            serde_json::to_value(GotoDefinitionResponse::Scalar(
                                                Location {
                                                    uri: uri.clone(),
                                                    range: symbol_info.range,
                                                },
                                            ))
                                            .unwrap_or(Value::Null),
                                        ),
                                        error: None,
                                    };
                                }
                            }
                        }
                    }
                }
            }

            // Check local document symbols (from AST) - prioritize same file
            if let Some(symbol_info) = doc.symbols.get(&symbol_name) {
                log::debug!(
                    "[LSP] Found in document symbols at line {}",
                    symbol_info.range.start.line
                );
                let location = Location {
                    uri: params
                        .text_document_position_params
                        .text_document
                        .uri
                        .clone(),
                    range: symbol_info.range,
                };
                return Response {
                    id: req.id,
                    result: Some(
                        serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                            .unwrap_or(Value::Null),
                    ),
                    error: None,
                };
            }

            // Check workspace symbols (indexed from all files)
            if let Some(symbol_info) = store.workspace_symbols.get(&symbol_name) {
                if let Some(uri) = &symbol_info.definition_uri {
                    let location = Location {
                        uri: uri.clone(),
                        range: symbol_info.range,
                    };
                    return Response {
                        id: req.id,
                        result: Some(
                            serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                                .unwrap_or(Value::Null),
                        ),
                        error: None,
                    };
                }
            }

            // Search for the symbol in other open documents
            let mut test_match: Option<(Url, Range)> = None;
            let current_uri = &params.text_document_position_params.text_document.uri;

            for (uri, other_doc) in store
                .documents
                .iter()
                .take(crate::lsp::search_limits::DEFINITION_SEARCH)
            {
                if uri == current_uri {
                    continue;
                }

                if let Some(symbol_info) = other_doc.symbols.get(&symbol_name) {
                    let uri_str = uri.as_str();
                    let is_test = uri_str.contains("/tests/")
                        || uri_str.contains("_test.zen")
                        || uri_str.contains("test_");

                    if is_test {
                        test_match = Some((uri.clone(), symbol_info.range));
                    } else {
                        let location = Location {
                            uri: uri.clone(),
                            range: symbol_info.range,
                        };
                        return Response {
                            id: req.id,
                            result: Some(
                                serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                                    .unwrap_or(Value::Null),
                            ),
                            error: None,
                        };
                    }
                }
            }

            // If we only found test matches, use that
            if let Some((uri, range)) = test_match {
                let location = Location { uri, range };
                return Response {
                    id: req.id,
                    result: Some(
                        serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                            .unwrap_or(Value::Null),
                    ),
                    error: None,
                };
            }

            // Search the entire workspace for the symbol
            if let Some((uri, symbol_info)) = store.search_workspace_for_symbol(&symbol_name) {
                let location = Location {
                    uri: uri.clone(),
                    range: symbol_info.range,
                };
                return Response {
                    id: req.id,
                    result: Some(
                        serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                            .unwrap_or(Value::Null),
                    ),
                    error: None,
                };
            }

            // Fallback: text search within current document
            log::debug!("[LSP] Trying text search fallback for: '{}'", symbol_name);
            if let Some(range) = find_symbol_definition_in_content(&doc.content, &symbol_name) {
                log::debug!("[LSP] Found via text search at line {}", range.start.line);
                let location = Location {
                    uri: params
                        .text_document_position_params
                        .text_document
                        .uri
                        .clone(),
                    range,
                };
                return Response {
                    id: req.id,
                    result: Some(
                        serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                            .unwrap_or(Value::Null),
                    ),
                    error: None,
                };
            }

            log::debug!("[LSP] No definition found for: '{}'", symbol_name);
        }
    }

    Response {
        id: req.id,
        result: Some(Value::Null),
        error: None,
    }
}
