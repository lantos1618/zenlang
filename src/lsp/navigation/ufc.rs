// UFC (Uniform Function Call) method resolution helpers

use crate::intrinsics::well_known;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::type_query::TypeQuery;
use crate::lsp::types::UfcMethodInfo;
use crate::name_utils;
use crate::stdlib_types::stdlib_types;
use lsp_types::*;

/// Find UFC method call at the given position (e.g., receiver.method())
pub fn find_ufc_method_at_position(content: &str, position: Position) -> Option<UfcMethodInfo> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    let chars: Vec<char> = line.chars().collect();

    // Check if we're after a dot (UFC method call)
    if char_pos > 0 {
        let mut dot_pos = None;
        for i in (0..char_pos).rev() {
            if chars[i] == '.' {
                dot_pos = Some(i);
                break;
            } else if chars[i] == ' ' || chars[i] == '(' || chars[i] == ')' {
                break;
            }
        }

        if let Some(dot) = dot_pos {
            // Extract the method name after the dot
            let mut method_end = dot + 1;
            while method_end < chars.len()
                && (chars[method_end].is_alphanumeric() || chars[method_end] == '_')
            {
                method_end += 1;
            }
            let method_name: String = chars[(dot + 1)..method_end].iter().collect();

            // Extract the object/receiver before the dot
            let mut obj_start = dot;
            let mut paren_depth = 0;
            while obj_start > 0 {
                obj_start -= 1;
                match chars[obj_start] {
                    ')' => paren_depth += 1,
                    '(' => {
                        if paren_depth > 0 {
                            paren_depth -= 1;
                        } else {
                            break;
                        }
                    }
                    ' ' | '\t' | '\n' | '=' | '{' | '[' | ',' | ';' if paren_depth == 0 => {
                        obj_start += 1;
                        break;
                    }
                    _ => {}
                }
            }

            let receiver: String = chars[obj_start..dot].iter().collect();
            return Some(UfcMethodInfo {
                receiver: receiver.trim().to_string(),
                method_name,
            });
        }
    }
    None
}

use super::utils::find_stdlib_location;

/// Resolve a UFC method call to its definition location
pub fn resolve_ufc_method(
    method_info: &UfcMethodInfo,
    store: &DocumentStore,
    doc: &super::super::types::Document,
) -> Option<Location> {
    let receiver_type = {
        let tq = TypeQuery::new(doc);
        tq.find_variable_type(&method_info.receiver)
            .map(|t| name_utils::strip_generics(&t).to_string())
    };

    // First, try to find the method in stdlib_symbols
    if let Some(symbol) = store.stdlib_symbols.get(&method_info.method_name) {
        if let Some(def_uri) = &symbol.definition_uri {
            return Some(Location {
                uri: def_uri.clone(),
                range: symbol.range,
            });
        }
    }

    // Extract base type and generic parameters
    let (base_type, _generic_params) = if let Some(ref typ) = receiver_type {
        name_utils::parse_generic_type(typ)
    } else {
        (String::new(), Vec::new())
    };

    // Fallback: Try to find method by searching stdlib symbols with receiver type prefix
    if !base_type.is_empty() {
        let prefixed_name = format!("{}_{}", base_type.to_lowercase(), method_info.method_name);
        if let Some(symbol) = store.stdlib_symbols.get(&prefixed_name) {
            if let Some(def_uri) = &symbol.definition_uri {
                return Some(Location {
                    uri: def_uri.clone(),
                    range: symbol.range,
                });
            }
        }
    }

    let wk = well_known();

    // Use well_known for Option/Result (they have canonical names)
    let normalized_type = if wk.is_result(&base_type) {
        wk.result_name().to_string()
    } else if wk.is_option(&base_type) {
        wk.option_name().to_string()
    } else if base_type == "StaticString" || base_type == "str" {
        "String".to_string()
    } else {
        base_type.clone()
    };

    // Look up stdlib path dynamically from the registry
    let stdlib_reg = stdlib_types();
    if let Some(stdlib_path) = stdlib_reg.get_type_source_path(&normalized_type) {
        if let Some(location) = find_stdlib_location(
            stdlib_path,
            &method_info.method_name,
            Some(&normalized_type),
            store,
        ) {
            return Some(location);
        }
    }

    // Resolve type aliases via TypeContext (e.g., GPA -> Allocator)
    // instead of hardcoding individual alias mappings
    if let Some(type_ctx) = &doc.type_context {
        if let Some(aliased_type) = type_ctx.type_aliases.get(&normalized_type) {
            if let Some(alias_base) = aliased_type.base_name() {
                if let Some(stdlib_path) = stdlib_reg.get_type_source_path(alias_base) {
                    if let Some(location) = find_stdlib_location(
                        stdlib_path,
                        &method_info.method_name,
                        Some(alias_base),
                        store,
                    ) {
                        return Some(location);
                    }
                }
            }
        }
    }

    // Check if the method itself is defined on a known stdlib type by searching
    // the registry for any type that has this method (handles iter, loop, etc.)
    if let Some(stdlib_path) = stdlib_reg.find_method_source_path(&method_info.method_name) {
        if let Some(location) =
            find_stdlib_location(&stdlib_path, &method_info.method_name, None, store)
        {
            return Some(location);
        }
    }

    // Enhanced UFC function search - any function can be called as a method
    for (uri, doc) in &store.documents {
        if let Some(symbol) = doc.symbols.get(&method_info.method_name) {
            if matches!(
                symbol.kind,
                lsp_types::SymbolKind::FUNCTION | lsp_types::SymbolKind::METHOD
            ) {
                // Use TypeContext to check if the first param matches the receiver type
                if let Some(receiver_type) = &receiver_type {
                    if let Some(type_ctx) = &doc.type_context {
                        if let Some(params) = type_ctx.get_function_params(&method_info.method_name)
                        {
                            if let Some((_, first_param_type)) = params.first() {
                                if let Some(param_base) = first_param_type.base_name() {
                                    let recv_base = name_utils::strip_generics(receiver_type);
                                    if param_base == recv_base {
                                        return Some(Location {
                                            uri: uri.clone(),
                                            range: symbol.range,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                return Some(Location {
                    uri: uri.clone(),
                    range: symbol.range,
                });
            }
        }

        // Search for functions that might be UFC-callable
        for (name, symbol) in &doc.symbols {
            if name == &method_info.method_name
                && matches!(symbol.kind, lsp_types::SymbolKind::FUNCTION)
            {
                return Some(Location {
                    uri: uri.clone(),
                    range: symbol.range,
                });
            }

            if let Some(receiver_type) = &receiver_type {
                let prefixed_name = format!(
                    "{}_{}",
                    receiver_type.to_lowercase(),
                    method_info.method_name
                );
                if name == &prefixed_name && matches!(symbol.kind, lsp_types::SymbolKind::FUNCTION)
                {
                    return Some(Location {
                        uri: uri.clone(),
                        range: symbol.range,
                    });
                }
            }
        }
    }

    None
}
