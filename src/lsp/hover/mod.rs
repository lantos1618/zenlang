// Hover Module for Zen LSP
// Handles textDocument/hover requests

mod builtins;
mod expressions;
mod format_string;
mod imports;
mod inference;
mod patterns;
mod response;
mod structs;

use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;

use super::document_store::DocumentStore;
use super::helpers::{char_pos_to_byte_pos, type_context_to_lsp_location, zen_code_block};
use super::navigation::find_symbol_at_position;
use super::navigation::find_symbol_definition_in_content;
use super::types::*;
use super::utils::{format_symbol_kind, format_type};
use crate::ast::primitives;
use crate::ast::AstType;

pub use expressions::*;
pub use format_string::*;
pub use inference::infer_variable_type;
pub use structs::*;

// Re-export main handler
pub use handler::handle_hover;

mod handler {
    use super::*;

    /// Handle textDocument/hover requests
    pub fn handle_hover(
        req: Request,
        store: &std::sync::Arc<std::sync::Mutex<DocumentStore>>,
    ) -> Response {
        let params: HoverParams = match serde_json::from_value(req.params) {
            Ok(p) => p,
            Err(_) => {
                return Response {
                    id: req.id,
                    result: Some(Value::Null),
                    error: None,
                };
            }
        };

        let store = match store.lock() {
            Ok(s) => s,
            Err(_) => {
                return Response {
                    id: req.id,
                    result: Some(Value::Null),
                    error: None,
                };
            }
        };

        if let Some(doc) = store
            .documents
            .get(&params.text_document_position_params.text_document.uri)
        {
            let position = params.text_document_position_params.position;

            if let Some(symbol_name) = find_symbol_at_position(&doc.content, position) {
                let request_id = req.id.clone();

                // PRIORITY 1: Check if we're hovering on a format string field access (e.g., ${person.name})
                // This MUST happen before string literal check, otherwise we'll show "String literal" for expressions inside ${...}
                if let Some(format_hover) = format_string::get_format_string_field_hover(
                    &doc.content,
                    position,
                    &symbol_name,
                    &doc.symbols,
                    &store,
                ) {
                    return create_hover_response_from_string(request_id.clone(), format_hover);
                }

                // Check if we're hovering on a method call (e.g., io.println)
                if let Some(response) = handle_method_call_hover(
                    &doc.content,
                    position,
                    &symbol_name,
                    &store,
                    request_id.clone(),
                ) {
                    return response;
                }

                // Check if we're hovering over a pattern match variable
                if let Some(pattern_hover) = patterns::get_pattern_match_hover(
                    &doc.content,
                    position,
                    &symbol_name,
                    &doc.symbols,
                    &store.stdlib_symbols,
                    &store.documents,
                ) {
                    return create_hover_response_from_string(request_id.clone(), pattern_hover);
                }

                // Check if we're hovering over an enum variant definition
                if let Some(enum_hover) =
                    patterns::get_enum_variant_hover(&doc.content, position, &symbol_name)
                {
                    return create_hover_response_from_string(request_id.clone(), enum_hover);
                }

                // Check for symbol info in current document
                if let Some(response) =
                    handle_symbol_hover(&symbol_name, doc, &store, position, request_id.clone())
                {
                    return response;
                }

                // PRIORITY: Use TypeContext from the typechecker (authoritative type info)
                if let Some(response) =
                    handle_type_context_hover(&symbol_name, doc, position, request_id.clone())
                {
                    return response;
                }

                // Check stdlib symbols
                if let Some(symbol_info) = store.stdlib_symbols.get(&symbol_name) {
                    return response::create_hover_response(request_id.clone(), symbol_info, None);
                }

                // Check other open documents - limit search for performance
                if let Some(response) =
                    handle_other_documents_hover(&symbol_name, &store, request_id.clone())
                {
                    return response;
                }

                // Check for imports (e.g., { io } = @std)
                if let Some(response) = handle_import_hover(
                    &symbol_name,
                    &doc.content,
                    doc.ast.as_deref(),
                    position,
                    &store,
                    request_id.clone(),
                ) {
                    return response;
                }

                // Check for built-in types FIRST (before fallback text search)
                if let Some(builtin_text) = builtins::get_builtin_hover_text(&symbol_name) {
                    return create_hover_response_from_string(request_id.clone(), builtin_text);
                }

                // Try to infer type from variable assignment in the current file
                if let Some(response) =
                    handle_inferred_type_hover(&symbol_name, doc, &store, request_id.clone())
                {
                    return response;
                }

                // Find definition location with context
                if let Some(response) = handle_definition_hover(
                    &symbol_name,
                    &doc.content,
                    &params.text_document_position_params.text_document.uri,
                    request_id.clone(),
                ) {
                    return response;
                }

                if let Some(response) = handle_lightweight_type_inference(
                    &symbol_name,
                    &doc.content,
                    position,
                    request_id.clone(),
                ) {
                    return response;
                }

                return create_hover_response_from_string(request_id, zen_code_block(&symbol_name));
            }
            // If no symbol found at position (empty line, whitespace, etc.), return null
        }

        Response {
            id: req.id,
            result: Some(Value::Null),
            error: None,
        }
    }

    fn append_definition_location(
        hover_parts: &mut Vec<String>,
        symbol_name: &str,
        doc: &Document,
    ) {
        if let Some(type_ctx) = doc.type_context.as_ref() {
            if let Some(def_loc) = type_ctx.get_location(symbol_name) {
                if let Some(lsp_loc) = type_context_to_lsp_location(def_loc, &doc.uri) {
                    let path_display = lsp_loc
                        .uri
                        .to_file_path()
                        .ok()
                        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
                        .unwrap_or_else(|| "current file".to_string());
                    hover_parts.push(format!(
                        "**Defined at:** `{}:{}`",
                        path_display,
                        lsp_loc.range.start.line + 1
                    ));
                }
            }
        }
    }

    fn handle_type_context_hover(
        symbol_name: &str,
        doc: &Document,
        position: Position,
        request_id: lsp_server::RequestId,
    ) -> Option<Response> {
        let type_ctx = doc.type_context.as_ref()?;

        // 1. Check variables — TypeContext stores them as "function_name::var_name"
        //    We need to figure out which function scope the cursor is in.
        if let Some(ast) = &doc.ast {
            for decl in ast {
                if let crate::ast::Declaration::Function(func) = decl {
                    if let Some(func_range) = super::super::navigation::utils::find_function_range(
                        &doc.content,
                        &func.name,
                    ) {
                        if position.line >= func_range.start.line
                            && position.line <= func_range.end.line
                        {
                            // Check function parameters
                            for (param_name, param_type) in &func.args {
                                if param_name == symbol_name {
                                    let type_str = format_type(param_type);
                                    return Some(create_hover_response_from_string(
                                        request_id,
                                        format!(
                                            "```zen\n{}: {}\n```\n\n**Parameter** of `{}`",
                                            param_name, type_str, func.name
                                        ),
                                    ));
                                }
                            }

                            // Check scoped variables from TypeContext
                            if let Some(var_type) =
                                type_ctx.get_variable_type(&func.name, symbol_name)
                            {
                                let type_str = format_type(&var_type);
                                let mut hover_parts =
                                    vec![zen_code_block(&format!("{}: {}", symbol_name, type_str))];
                                hover_parts.push(format!("**Type:** `{}`", type_str));
                                return Some(create_hover_response_from_string(
                                    request_id,
                                    hover_parts.join("\n\n"),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // 2. Check functions
        if let Some(func_type) = type_ctx.functions.get(symbol_name) {
            let params_str = func_type
                .params
                .iter()
                .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let ret_str = format_type(&func_type.return_type);
            let mut hover_parts = vec![format!(
                "```zen\n{} = ({}) {}\n```",
                symbol_name, params_str, ret_str
            )];
            if func_type.is_external {
                hover_parts.push("**External function**".to_string());
            }
            hover_parts.push("**Kind:** Function".to_string());
            append_definition_location(&mut hover_parts, symbol_name, doc);
            return Some(create_hover_response_from_string(
                request_id,
                hover_parts.join("\n\n"),
            ));
        }

        // 3. Check structs
        if let Some(fields) = type_ctx.get_struct_fields(symbol_name) {
            let fields_str = fields
                .iter()
                .map(|(name, ty)| format!("    {}: {}", name, format_type(ty)))
                .collect::<Vec<_>>()
                .join("\n");
            let mut hover_parts =
                vec![zen_code_block(&format!("{}:\n{}", symbol_name, fields_str))];
            // Show behavior implementations
            if let Some(behaviors) = type_ctx.behavior_impls.get(symbol_name) {
                if !behaviors.is_empty() {
                    hover_parts.push(format!("**Implements:** {}", behaviors.join(", ")));
                }
            }
            hover_parts.push("**Kind:** Struct".to_string());
            append_definition_location(&mut hover_parts, symbol_name, doc);
            return Some(create_hover_response_from_string(
                request_id,
                hover_parts.join("\n\n"),
            ));
        }

        // 4. Check enums
        if let Some(variants) = type_ctx.get_enum_variants(symbol_name) {
            let variants_str = variants
                .iter()
                .map(|(name, payload)| {
                    if let Some(ty) = payload {
                        format!("    .{}({})", name, format_type(ty))
                    } else {
                        format!("    .{}", name)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            let mut hover_parts = vec![
                zen_code_block(&format!("{}:\n{}", symbol_name, variants_str)),
                "**Kind:** Enum".to_string(),
            ];
            append_definition_location(&mut hover_parts, symbol_name, doc);
            return Some(create_hover_response_from_string(
                request_id,
                hover_parts.join("\n\n"),
            ));
        }

        // 5. Check type aliases
        if let Some(aliased_type) = type_ctx.type_aliases.get(symbol_name) {
            let type_str = format_type(aliased_type);
            let mut hover_parts = vec![
                format!("```zen\n{} = {}\n```", symbol_name, type_str),
                format!("**Type alias** for `{}`", type_str),
            ];
            append_definition_location(&mut hover_parts, symbol_name, doc);
            return Some(create_hover_response_from_string(
                request_id,
                hover_parts.join("\n\n"),
            ));
        }

        // 6. Check methods ("Type.method" format) — handle dotted symbol names
        if symbol_name.contains('.') {
            if let Some(return_type) = type_ctx.methods.get(symbol_name) {
                let ret_str = format_type(return_type);
                let mut hover_parts =
                    vec![zen_code_block(&format!("{} -> {}", symbol_name, ret_str))];
                if let Some(params) = type_ctx.method_params.get(symbol_name) {
                    let params_str = params
                        .iter()
                        .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    hover_parts.insert(
                        0,
                        format!(
                            "```zen\n{} = ({}) {}\n```",
                            symbol_name, params_str, ret_str
                        ),
                    );
                    hover_parts.remove(1); // remove the simpler version
                }
                hover_parts.push("**Kind:** Method".to_string());
                append_definition_location(&mut hover_parts, symbol_name, doc);
                return Some(create_hover_response_from_string(
                    request_id,
                    hover_parts.join("\n\n"),
                ));
            }
        }

        None
    }

    fn handle_method_call_hover(
        content: &str,
        position: Position,
        symbol_name: &str,
        store: &DocumentStore,
        request_id: lsp_server::RequestId,
    ) -> Option<Response> {
        let line = content.lines().nth(position.line as usize).unwrap_or("");
        let char_pos = position.character as usize;
        let byte_pos = char_pos_to_byte_pos(line, char_pos);

        if let Some(dot_pos) = line[..byte_pos].rfind('.') {
            let before_dot = &line[..dot_pos];
            let after_dot = &line[dot_pos + 1..];

            if let Some(method_start) = after_dot.find(|c: char| c.is_alphanumeric() || c == '_') {
                let method_name = after_dot[method_start..]
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>();

                // symbol_name may include receiver: "gpa.default_gpa" vs just "default_gpa"
                let is_method_hover = method_name == symbol_name
                    || symbol_name.ends_with(&format!(".{}", method_name));

                if is_method_hover {
                    let receiver = before_dot
                        .trim_end()
                        .rsplit(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let full_method = format!("{}.{}", receiver, method_name);

                    if let Some(symbol_info) = store.stdlib_symbols.get(&full_method) {
                        return Some(response::create_hover_response(
                            request_id,
                            symbol_info,
                            None,
                        ));
                    }

                    if let Some(symbol_info) = store.stdlib_symbols.get(&method_name) {
                        return Some(response::create_hover_response(
                            request_id,
                            symbol_info,
                            None,
                        ));
                    }

                    if let Some(return_type) = crate::stdlib_types::stdlib_types()
                        .get_function_return_type(&receiver, &method_name)
                    {
                        let type_str = format_type(return_type);
                        let hover_text = format!(
                            "```zen\n{} = () {}\n```\n\n**Type:** `{}`\n\n**Module:** `{}`",
                            method_name, type_str, type_str, receiver
                        );
                        return Some(create_hover_response_from_string(request_id, hover_text));
                    }
                }
            }
        }
        None
    }

    fn handle_symbol_hover(
        symbol_name: &str,
        doc: &Document,
        store: &DocumentStore,
        position: Position,
        request_id: lsp_server::RequestId,
    ) -> Option<Response> {
        if let Some(symbol_info) = doc.symbols.get(symbol_name) {
            let mut hover_content = Vec::with_capacity(4);

            if let Some(detail) = &symbol_info.detail {
                hover_content.push(format!("```zen\n{}\n```", detail));
            }

            if let Some(doc) = &symbol_info.documentation {
                hover_content.push(doc.clone());
            }

            if let Some(type_info) = &symbol_info.type_info {
                if matches!(type_info, AstType::Struct { .. }) {
                    if let Some(type_name) = type_info.base_name() {
                        if let Some(struct_def) = store.find_struct_definition(type_name) {
                            hover_content.push(format!(
                                "```zen\n{}\n```",
                                structs::format_struct_definition(&struct_def)
                            ));
                        } else {
                            hover_content.push(format!("**Type:** `{}`", format_type(type_info)));
                        }
                    } else {
                        hover_content.push(format!("**Type:** `{}`", format_type(type_info)));
                    }
                } else {
                    hover_content.push(format!("**Type:** `{}`", format_type(type_info)));
                }
            } else if symbol_info.kind == SymbolKind::STRUCT {
                if let Some(struct_def) = store.find_struct_definition(symbol_name) {
                    hover_content.clear();
                    hover_content.push(format!(
                        "```zen\n{}\n```",
                        structs::format_struct_definition(&struct_def)
                    ));
                }
            }

            hover_content.push(format!(
                "**Kind:** {}",
                format_symbol_kind(symbol_info.kind)
            ));

            let contents = HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_content.join("\n\n"),
            });

            return Some(Response {
                id: request_id,
                result: Some(
                    serde_json::to_value(Hover {
                        contents,
                        range: Some(Range {
                            start: Position {
                                line: position.line,
                                character: position
                                    .character
                                    .saturating_sub(symbol_name.len() as u32),
                            },
                            end: Position {
                                line: position.line,
                                character: position.character + symbol_name.len() as u32,
                            },
                        }),
                    })
                    .unwrap_or(Value::Null),
                ),
                error: None,
            });
        }
        None
    }

    fn handle_other_documents_hover(
        symbol_name: &str,
        store: &DocumentStore,
        request_id: lsp_server::RequestId,
    ) -> Option<Response> {
        for (_uri, other_doc) in store
            .documents
            .iter()
            .take(crate::lsp::search_limits::HOVER_SEARCH)
        {
            if let Some(symbol_info) = other_doc.symbols.get(symbol_name) {
                let mut hover_content = Vec::with_capacity(3);

                if let Some(detail) = &symbol_info.detail {
                    hover_content.push(format!("```zen\n{}\n```", detail));
                }

                if let Some(type_info) = &symbol_info.type_info {
                    hover_content.push(format!("**Type:** `{}`", format_type(type_info)));
                }

                let contents = HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_content.join("\n\n"),
                });

                return Some(Response {
                    id: request_id,
                    result: Some(
                        serde_json::to_value(Hover {
                            contents,
                            range: None,
                        })
                        .unwrap_or(Value::Null),
                    ),
                    error: None,
                });
            }
        }
        None
    }

    fn handle_import_hover(
        symbol_name: &str,
        content: &str,
        ast: Option<&[crate::ast::Declaration]>,
        position: Position,
        store: &DocumentStore,
        request_id: lsp_server::RequestId,
    ) -> Option<Response> {
        // Prefer AST-based lookup, fall back to string parsing
        let import_info = ast
            .and_then(|ast| imports::find_import_info_from_ast(ast, symbol_name))
            .or_else(|| imports::find_import_info(content, symbol_name, position));
        if let Some(import_info) = import_info {
            let mut hover_content = Vec::with_capacity(4);
            hover_content.push(format!("```zen\n{}\n```", import_info.import_line));
            hover_content.push(format!("**Imported from:** `{}`", import_info.source));

            if let Some(symbol_info) = store.stdlib_symbols.get(symbol_name) {
                if let Some(detail) = &symbol_info.detail {
                    hover_content.insert(0, format!("```zen\n{}\n```", detail));
                }
                if let Some(def_uri) = &symbol_info.definition_uri {
                    if let Ok(path) = def_uri.to_file_path() {
                        hover_content.push(format!("**Definition:** `{}`", path.display()));
                    }
                }
                if let Some(type_info) = &symbol_info.type_info {
                    hover_content.push(format!("**Type:** `{}`", format_type(type_info)));
                }
            } else {
                hover_content.push("**Type:** Module/Namespace".to_string());
            }

            return Some(create_hover_response_from_string(
                request_id,
                hover_content.join("\n\n"),
            ));
        }
        None
    }

    fn handle_inferred_type_hover(
        symbol_name: &str,
        doc: &Document,
        store: &DocumentStore,
        request_id: lsp_server::RequestId,
    ) -> Option<Response> {
        if let Some(inferred_type) =
            inference::infer_variable_type(symbol_name, &doc.symbols, store)
        {
            let enhanced_type =
                if let Some(struct_name) = structs::extract_struct_name_from_type(&inferred_type) {
                    if let Some(struct_def) = store.find_struct_definition(&struct_name) {
                        format!(
                            "```zen\n{}\n```",
                            structs::format_struct_definition(&struct_def)
                        )
                    } else {
                        inferred_type
                    }
                } else {
                    inferred_type
                };

            return Some(create_hover_response_from_string(request_id, enhanced_type));
        }
        None
    }

    fn handle_definition_hover(
        symbol_name: &str,
        content: &str,
        uri: &lsp_types::Url,
        request_id: lsp_server::RequestId,
    ) -> Option<Response> {
        if let Some(range) = find_symbol_definition_in_content(content, symbol_name) {
            let line_text = content
                .lines()
                .nth(range.start.line as usize)
                .map(|l| l.trim())
                .unwrap_or("");
            let file_name = uri
                .path_segments()
                .and_then(|mut s| s.next_back())
                .unwrap_or("unknown");

            let mut hover_content = Vec::with_capacity(3);
            hover_content.push(format!("```zen\n{}\n```", line_text));
            hover_content.push(format!(
                "**Location:** `{}:{}`",
                file_name,
                range.start.line + 1
            ));

            if let Some(type_str) = inference::extract_type_from_line(line_text) {
                hover_content.push(format!("**Type:** `{}`", type_str));
            }

            return Some(create_hover_response_from_string(
                request_id,
                hover_content.join("\n\n"),
            ));
        }
        None
    }

    fn is_inside_format_expression(line: &str, byte_pos: usize) -> bool {
        // Find the nearest ${ before the cursor
        use crate::lsp::search_limits::MAX_ITERATIONS;
        let mut search_pos = 0;
        let mut iterations = 0;

        while iterations < MAX_ITERATIONS {
            iterations += 1;

            let search_range = search_pos..byte_pos.min(line.len());
            if search_range.is_empty() {
                break;
            }

            let dollar_pos = if let Some(pos) = line[search_range].rfind('$') {
                search_pos + pos
            } else {
                break;
            };

            // Check if it's escaped
            if dollar_pos > 0 && line.as_bytes()[dollar_pos - 1] == b'\\' {
                search_pos = dollar_pos.saturating_sub(1);
                continue;
            }

            // Check if it's ${...
            if dollar_pos + 1 < line.len() && line.as_bytes()[dollar_pos + 1] == b'{' {
                // Find the closing }
                if let Some(close_brace) = line[dollar_pos + 2..].find('}') {
                    let expr_end = dollar_pos + 2 + close_brace;
                    // Check if cursor is between ${ and }
                    if byte_pos > dollar_pos && byte_pos <= expr_end + 1 {
                        return true;
                    }
                } else {
                    // No closing brace found, but we're inside ${...
                    if byte_pos > dollar_pos {
                        return true;
                    }
                }
            }

            if dollar_pos == 0 {
                break;
            }
            search_pos = dollar_pos.saturating_sub(1);
        }

        false
    }

    /// Extract string literal if cursor is inside one (public for use in handler)
    fn extract_string_literal_from_line(line: &str, byte_pos: usize) -> Option<String> {
        let mut in_string = false;
        let mut string_start = 0;
        let mut escape_next = false;

        for (i, ch) in line.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                if !in_string {
                    // Start of string
                    in_string = true;
                    string_start = i;
                } else {
                    // End of string
                    // Include boundaries: if cursor is at or between the quotes
                    if byte_pos >= string_start && byte_pos <= i {
                        // We're inside this string literal (including on the quotes)
                        return Some(line[string_start..=i].to_string());
                    }
                    in_string = false;
                }
            }
        }

        // If we're still in a string at the end, check if cursor is in it
        if in_string && byte_pos >= string_start {
            return Some(line[string_start..].to_string());
        }

        None
    }

    fn handle_lightweight_type_inference(
        symbol_name: &str,
        content: &str,
        position: Position,
        request_id: lsp_server::RequestId,
    ) -> Option<Response> {
        let line = content.lines().nth(position.line as usize).unwrap_or("");
        let char_pos = position.character as usize;
        let byte_pos = char_pos_to_byte_pos(line, char_pos);
        let trimmed = symbol_name.trim();

        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            return Some(create_hover_response_from_string(
                request_id,
                format!("```zen\n{}\n```\n\n**Type:** `StaticString`", trimmed),
            ));
        }

        if trimmed.parse::<i64>().is_ok() {
            return Some(create_hover_response_from_string(
                request_id,
                format!("```zen\n{}\n```\n\n**Type:** `i32`", trimmed),
            ));
        }

        if trimmed.parse::<f64>().is_ok() && trimmed.contains('.') {
            return Some(create_hover_response_from_string(
                request_id,
                format!("```zen\n{}\n```\n\n**Type:** `f64`", trimmed),
            ));
        }

        // Use centralized boolean literal check
        if primitives::is_boolean_literal(trimmed) {
            return Some(create_hover_response_from_string(
                request_id,
                format!("```zen\n{}\n```\n\n**Type:** `bool`", trimmed),
            ));
        }

        if !is_inside_format_expression(line, byte_pos) {
            if let Some(string_literal) = extract_string_literal_from_line(line, byte_pos) {
                return Some(create_hover_response_from_string(
                    request_id,
                    format!(
                        "```zen\n{}\n```\n\n**Type:** `StaticString`",
                        string_literal
                    ),
                ));
            }
        }

        None
    }

    fn create_hover_response_from_string(id: lsp_server::RequestId, content: String) -> Response {
        let contents = HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        });

        Response {
            id,
            result: Some(
                serde_json::to_value(Hover {
                    contents,
                    range: None,
                })
                .unwrap_or(Value::Null),
            ),
            error: None,
        }
    }
}
