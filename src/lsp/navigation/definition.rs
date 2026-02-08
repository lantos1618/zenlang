// Definition handler - go-to-definition

use super::imports::{find_import_info, find_import_info_from_ast};
use super::struct_fields::find_struct_field_definition;
use super::ufc::{find_ufc_method_at_position, resolve_ufc_method};
use super::utils::{find_symbol_at_position, find_symbol_definition_in_content};
use crate::lsp::document_store::DocumentStore;
use crate::lsp::helpers::{
    null_response, type_context_to_lsp_location, with_document, HasDocumentUri,
};
use lsp_server::{Request, Response};
use lsp_types::*;

fn definition_response(req: &Request, location: Location) -> Response {
    Response {
        id: req.id.clone(),
        result: Some(
            serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                .unwrap_or(serde_json::Value::Null),
        ),
        error: None,
    }
}

fn origin_location() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}

pub fn handle_definition(
    req: Request,
    store: &std::sync::Arc<std::sync::RwLock<DocumentStore>>,
) -> Response {
    with_document::<GotoDefinitionParams, _>(&req, store, |doc, params, store_guard| {
        let doc_uri = params.document_uri();
        let position = params.text_document_position_params.position;

        let location = resolve_ufc_method_definition(&doc.content, position, doc, store_guard)
            .or_else(|| find_struct_field_definition(&doc.content, position, doc, store_guard))
            .or_else(|| {
                resolve_symbol_definition(&doc.content, position, doc, doc_uri, store_guard)
            });

        match location {
            Some(loc) => definition_response(&req, loc),
            None => null_response(&req),
        }
    })
}

fn resolve_ufc_method_definition(
    content: &str,
    position: Position,
    doc: &crate::lsp::types::Document,
    store: &DocumentStore,
) -> Option<Location> {
    let method_info = find_ufc_method_at_position(content, position)?;
    resolve_ufc_method(&method_info, store, doc)
}

fn resolve_symbol_definition(
    content: &str,
    position: Position,
    doc: &crate::lsp::types::Document,
    doc_uri: &Url,
    store: &DocumentStore,
) -> Option<Location> {
    let symbol_name = find_symbol_at_position(content, position)?;
    log::debug!("[LSP] Go-to-definition for: '{}'", symbol_name);

    resolve_std_qualified_name(&symbol_name, store)
        .or_else(|| resolve_qualified_name(&symbol_name, position, doc, store))
        .or_else(|| resolve_type_context_definition(&symbol_name, doc, doc_uri))
        .or_else(|| resolve_std_import_ref(&symbol_name, content, position, store))
        .or_else(|| resolve_imported_symbol(&symbol_name, position, doc, store))
        .or_else(|| resolve_local_variable(&symbol_name, content, doc_uri))
        .or_else(|| resolve_document_symbol(&symbol_name, doc, doc_uri))
        .or_else(|| resolve_workspace_symbol(&symbol_name, store))
        .or_else(|| resolve_other_documents(&symbol_name, doc_uri, store))
        .or_else(|| resolve_workspace_search(&symbol_name, store))
}

fn resolve_std_qualified_name(symbol_name: &str, store: &DocumentStore) -> Option<Location> {
    if !symbol_name.starts_with("@std.") {
        return None;
    }
    let parts: Vec<&str> = symbol_name.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    let module_name = parts[1];
    let symbol_name_in_module = parts[2..].join(".");

    for (uri, stdlib_doc) in &store.documents {
        let uri_path = uri.path();
        if uri_path.contains("stdlib")
            && (uri_path.ends_with(&format!("{}/{}.zen", module_name, module_name))
                || uri_path.contains(&format!("{}/{}", module_name, module_name)))
        {
            if let Some(symbol_info) = stdlib_doc.symbols.get(&symbol_name_in_module) {
                return Some(Location {
                    uri: uri.clone(),
                    range: symbol_info.range,
                });
            }

            return Some(Location {
                uri: uri.clone(),
                range: origin_location(),
            });
        }
    }
    None
}

fn resolve_qualified_name(
    symbol_name: &str,
    position: Position,
    doc: &crate::lsp::types::Document,
    store: &DocumentStore,
) -> Option<Location> {
    if symbol_name.starts_with('@') || !symbol_name.contains('.') {
        return None;
    }
    let parts: Vec<&str> = symbol_name.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let module_alias = parts[0];
    let member_name = parts[1..].join(".");
    log::debug!(
        "[LSP] Qualified name: module_alias={}, member_name={}",
        module_alias,
        member_name
    );

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

        let module_path = if import_info.source == "@std" {
            format!("@std.{}", module_alias)
        } else {
            import_info.source.clone()
        };

        if let Some(loc) = resolve_member_in_module(&module_path, &member_name, module_alias, store)
        {
            return Some(loc);
        }

        for (uri, stdlib_doc) in &store.documents {
            let uri_path = uri.path();
            if uri_path.contains("stdlib")
                && (uri_path.ends_with(&format!("{}/{}.zen", module_alias, module_alias))
                    || uri_path.contains(&format!("{}/{}", module_alias, module_alias)))
            {
                if let Some(symbol_info) = stdlib_doc.symbols.get(&member_name) {
                    return Some(Location {
                        uri: uri.clone(),
                        range: symbol_info.range,
                    });
                }
            }
        }
    } else {
        log::debug!(
            "[LSP] No import found for {}, trying @std.{}",
            module_alias,
            module_alias
        );
        let module_path = format!("@std.{}", module_alias);
        if let Some(loc) = resolve_member_in_module(&module_path, &member_name, module_alias, store)
        {
            return Some(loc);
        }
    }

    None
}

fn resolve_type_context_definition(
    symbol_name: &str,
    doc: &crate::lsp::types::Document,
    doc_uri: &Url,
) -> Option<Location> {
    let type_ctx = doc.type_context.as_ref()?;
    let def_loc = type_ctx.get_location(symbol_name)?;
    log::debug!(
        "[LSP] TypeContext hit for '{}' at line {}",
        symbol_name,
        def_loc.line
    );
    type_context_to_lsp_location(def_loc, doc_uri)
}

fn resolve_member_in_module(
    module_path: &str,
    member_name: &str,
    module_alias: &str,
    store: &DocumentStore,
) -> Option<Location> {
    let file_path = store.stdlib_resolver.resolve_module_path(module_path)?;
    log::debug!(
        "[LSP] Resolved module path {} to file: {:?}",
        module_path,
        file_path
    );

    let module_uri = Url::from_file_path(&file_path).ok()?;

    if let Some(symbol_info) = store.stdlib_symbols.get(member_name) {
        if let Some(def_uri) = &symbol_info.definition_uri {
            if def_uri.path().contains(&format!("/{}/", module_alias)) {
                log::debug!(
                    "[LSP] Found {} in stdlib_symbols from {}",
                    member_name,
                    def_uri.path()
                );
                return Some(Location {
                    uri: def_uri.clone(),
                    range: symbol_info.range,
                });
            }
        }
    }

    if let Some(module_doc) = store.documents.get(&module_uri) {
        if let Some(symbol_info) = module_doc.symbols.get(member_name) {
            return Some(Location {
                uri: module_uri,
                range: symbol_info.range,
            });
        }
    }

    None
}

fn resolve_std_import_ref(
    symbol_name: &str,
    content: &str,
    position: Position,
    store: &DocumentStore,
) -> Option<Location> {
    let line = content.lines().nth(position.line as usize).unwrap_or("");
    if !((symbol_name == "@std"
        || symbol_name == "std"
        || (symbol_name.starts_with("@std") && symbol_name.contains('.')))
        && line.contains('=')
        && (line.contains("@std") || line.contains("= @std")))
    {
        return None;
    }

    let module_path = if symbol_name == "@std" || symbol_name == "std" {
        "@std"
    } else {
        symbol_name
    };

    let file_path = store.stdlib_resolver.resolve_module_path(module_path)?;
    let uri = Url::from_file_path(&file_path).ok()?;

    if store.documents.contains_key(&uri) {
        return Some(Location {
            uri,
            range: origin_location(),
        });
    }

    if let Some(workspace_root) = &store.workspace_root {
        if let Ok(workspace_path) = workspace_root.to_file_path() {
            if file_path.strip_prefix(&workspace_path).is_ok() {
                if let Ok(uri) = Url::from_file_path(&file_path) {
                    return Some(Location {
                        uri,
                        range: origin_location(),
                    });
                }
            }
        }
    }

    None
}

fn resolve_imported_symbol(
    symbol_name: &str,
    position: Position,
    doc: &crate::lsp::types::Document,
    store: &DocumentStore,
) -> Option<Location> {
    let import_info = doc
        .ast
        .as_ref()
        .and_then(|ast| find_import_info_from_ast(ast, symbol_name))
        .or_else(|| find_import_info(&doc.content, symbol_name, position));
    let import_info = import_info?;

    if !import_info.source.starts_with("@std") {
        return None;
    }

    log::debug!(
        "[LSP] Found import: {} from {}",
        symbol_name,
        import_info.source
    );

    let module_name = if import_info.source == "@std" {
        symbol_name.to_string()
    } else if import_info.source == "@std.types" {
        "types".to_string()
    } else {
        import_info
            .source
            .strip_prefix("@std.")
            .unwrap_or("io")
            .to_string()
    };

    if import_info.source == "@std" && symbol_name == module_name {
        return resolve_module_import(symbol_name, &module_name, store);
    }

    resolve_symbol_from_stdlib(symbol_name, &import_info.source, store)
}

fn resolve_module_import(
    symbol_name: &str,
    module_name: &str,
    store: &DocumentStore,
) -> Option<Location> {
    if let Some(symbol_info) = store.stdlib_symbols.get(symbol_name) {
        if let Some(uri) = &symbol_info.definition_uri {
            return Some(Location {
                uri: uri.clone(),
                range: symbol_info.range,
            });
        }
    }

    for uri in store.documents.keys() {
        let uri_path = uri.path();
        if uri_path.contains("stdlib")
            && (uri_path.ends_with(&format!("{}/{}.zen", module_name, module_name))
                || uri_path.contains(&format!("{}/{}", module_name, module_name)))
        {
            return Some(Location {
                uri: uri.clone(),
                range: origin_location(),
            });
        }
    }

    if let Some(workspace_root) = &store.workspace_root {
        if let Ok(workspace_path) = workspace_root.to_file_path() {
            let module_file = workspace_path
                .join("stdlib")
                .join(module_name)
                .join(format!("{}.zen", module_name));
            if module_file.exists() {
                if let Ok(uri) = Url::from_file_path(&module_file) {
                    return Some(Location {
                        uri,
                        range: origin_location(),
                    });
                }
            }
        }
    }

    None
}

fn resolve_symbol_from_stdlib(
    symbol_name: &str,
    source: &str,
    store: &DocumentStore,
) -> Option<Location> {
    if let Some(symbol_info) = store.stdlib_symbols.get(symbol_name) {
        if let Some(uri) = &symbol_info.definition_uri {
            return Some(Location {
                uri: uri.clone(),
                range: symbol_info.range,
            });
        }
    }

    if let Some(file_path) = store.stdlib_resolver.resolve_module_path(source) {
        if let Ok(module_uri) = Url::from_file_path(&file_path) {
            if let Some(stdlib_doc) = store.documents.get(&module_uri) {
                if let Some(symbol_info) = stdlib_doc.symbols.get(symbol_name) {
                    return Some(Location {
                        uri: module_uri,
                        range: symbol_info.range,
                    });
                }
            }

            return Some(Location {
                uri: module_uri,
                range: origin_location(),
            });
        }
    }

    for (uri, stdlib_doc) in &store.documents {
        let uri_path = uri.path();
        if uri_path.contains("stdlib") {
            if let Some(symbol_info) = stdlib_doc.symbols.get(symbol_name) {
                return Some(Location {
                    uri: uri.clone(),
                    range: symbol_info.range,
                });
            }
        }
    }

    None
}

fn resolve_local_variable(symbol_name: &str, content: &str, doc_uri: &Url) -> Option<Location> {
    if symbol_name.contains('.')
        || symbol_name.starts_with('@')
        || !symbol_name.chars().next().is_some_and(|c| c.is_lowercase())
    {
        return None;
    }

    let range = find_symbol_definition_in_content(content, symbol_name)?;
    log::debug!(
        "[LSP] Found local variable '{}' definition at line {}",
        symbol_name,
        range.start.line
    );
    Some(Location {
        uri: doc_uri.clone(),
        range,
    })
}

fn resolve_document_symbol(
    symbol_name: &str,
    doc: &crate::lsp::types::Document,
    doc_uri: &Url,
) -> Option<Location> {
    let symbol_info = doc.symbols.get(symbol_name)?;
    log::debug!(
        "[LSP] Found in document symbols at line {}",
        symbol_info.range.start.line
    );
    Some(Location {
        uri: doc_uri.clone(),
        range: symbol_info.range,
    })
}

fn resolve_workspace_symbol(symbol_name: &str, store: &DocumentStore) -> Option<Location> {
    let symbol_info = store.workspace_symbols.get(symbol_name)?;
    let uri = symbol_info.definition_uri.as_ref()?;
    Some(Location {
        uri: uri.clone(),
        range: symbol_info.range,
    })
}

fn resolve_other_documents(
    symbol_name: &str,
    current_uri: &Url,
    store: &DocumentStore,
) -> Option<Location> {
    let mut test_match: Option<(Url, Range)> = None;

    for (uri, other_doc) in store
        .documents
        .iter()
        .take(crate::lsp::search_limits::DEFINITION_SEARCH)
    {
        if uri == current_uri {
            continue;
        }

        if let Some(symbol_info) = other_doc.symbols.get(symbol_name) {
            let uri_str = uri.as_str();
            let is_test = uri_str.contains("/tests/")
                || uri_str.contains("_test.zen")
                || uri_str.contains("test_");

            if is_test {
                test_match = Some((uri.clone(), symbol_info.range));
            } else {
                return Some(Location {
                    uri: uri.clone(),
                    range: symbol_info.range,
                });
            }
        }
    }

    test_match.map(|(uri, range)| Location { uri, range })
}

fn resolve_workspace_search(symbol_name: &str, store: &DocumentStore) -> Option<Location> {
    let (uri, symbol_info) = store.search_workspace_for_symbol(symbol_name)?;
    Some(Location {
        uri: uri.clone(),
        range: symbol_info.range,
    })
}
