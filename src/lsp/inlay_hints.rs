use lsp_server::{Request, Response};
use lsp_types::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::ast::{AstType, Declaration, Expression, Statement};
use crate::stdlib_types::stdlib_types;

use super::document_store::DocumentStore;
use super::type_inference::get_base_type_name;
use super::types::Document;
use super::utils::format_type;

pub fn handle_inlay_hints(req: Request, store: &Arc<Mutex<DocumentStore>>) -> Response {
    let params: InlayHintParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(_) => return empty_response(req.id),
    };

    let store = match store.lock() {
        Ok(s) => s,
        Err(_) => return empty_response(req.id),
    };

    let doc = match store.documents.get(&params.text_document.uri) {
        Some(d) => d,
        None => return empty_response(req.id),
    };

    let mut hints = Vec::new();
    let mut seen: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();

    collect_hints_from_content(&doc.content, &mut hints, &mut seen);

    if let Some(ast) = &doc.ast {
        for decl in ast {
            if let Declaration::Function(func) = decl {
                collect_hints_from_statements(
                    &func.body,
                    &doc.content,
                    doc,
                    &store,
                    &mut hints,
                    &mut seen,
                );
            }
        }
    }

    Response {
        id: req.id,
        result: serde_json::to_value(hints).ok(),
        error: None,
    }
}

fn empty_response(id: lsp_server::RequestId) -> Response {
    Response {
        id,
        result: Some(serde_json::to_value(Vec::<InlayHint>::new()).unwrap_or(Value::Null)),
        error: None,
    }
}

fn collect_hints_from_content(
    content: &str,
    hints: &mut Vec<InlayHint>,
    seen: &mut std::collections::HashSet<(u32, u32)>,
) {
    let mut in_struct_def = false;
    let mut brace_depth: i32 = 0;
    let mut pending_struct_def = false; // True when we saw "Name:" and expect "{" next

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Track whether this line should be skipped (starts or is inside struct/enum def)
        let mut skip_this_line = in_struct_def;

        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            // Check for pending struct def (previous line was "Name:", now we see "{")
            if pending_struct_def && trimmed.starts_with('{') {
                in_struct_def = true;
                brace_depth = 0;
                skip_this_line = true;
                pending_struct_def = false;
            }

            // Check for struct definition start: "Name: {" or "Name:" (with brace on next line)
            if let Some(colon_pos) = trimmed.find(':') {
                let before = trimmed[..colon_pos].trim();
                let after = trimmed[colon_pos + 1..].trim();

                // If it looks like a struct definition (PascalCase: { ... } or PascalCase:)
                if !before.is_empty()
                    && before.chars().next().unwrap().is_uppercase()
                    && !before.contains('=')
                    && !before.contains(' ')
                {
                    if after.starts_with('{') || after.contains('{') {
                        // Brace on same line: "Person: {"
                        in_struct_def = true;
                        brace_depth = 0;
                        skip_this_line = true;
                        pending_struct_def = false;
                    } else if after.is_empty() {
                        // Brace might be on next line: "Person:"
                        pending_struct_def = true;
                        skip_this_line = true; // Skip the definition line too
                    }
                }
            } else {
                // Line doesn't have a colon - clear pending if it's not just a brace
                if !trimmed.starts_with('{') {
                    pending_struct_def = false;
                }
            }

            // Track brace depth for struct definitions
            if in_struct_def {
                for ch in trimmed.chars() {
                    if ch == '{' {
                        brace_depth += 1;
                    } else if ch == '}' {
                        brace_depth -= 1;
                        if brace_depth <= 0 {
                            in_struct_def = false;
                        }
                    }
                }
            }
        }

        // Skip lines inside struct/enum definitions
        if skip_this_line {
            continue;
        }

        if let Some(hint) = try_create_hint_for_line(line, line_num as u32, seen) {
            hints.push(hint);
        }
    }
}

fn try_create_hint_for_line(
    line: &str,
    line_num: u32,
    seen: &mut std::collections::HashSet<(u32, u32)>,
) -> Option<InlayHint> {
    let trimmed = line.trim();

    if trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with('{')
        || trimmed.starts_with('}')
        || trimmed.starts_with('|')
        || trimmed.contains("= (")
        || trimmed.contains("= @")
    {
        return None;
    }

    let (_var_name, hint_pos, has_explicit_type, is_mutable, is_colon_eq, rhs) =
        parse_var_decl(line)?;

    if has_explicit_type {
        return None;
    }

    let key = (line_num, hint_pos);
    if seen.contains(&key) {
        return None;
    }

    let inferred = infer_type(&rhs)?;
    seen.insert(key);

    // Format hint based on declaration type:
    // - Mutable (::=): show ":: Type" after variable name → "name :: Type = value"
    // - Colon-eq (:=): show " Type" after the colon → "name: Type = value"
    // - Immutable (=): show ": Type" after variable name → "name: Type = value"
    let (hint_label, hint_char_pos) = if is_mutable {
        // ::= → name :: Type = value
        // Position hint between :: and = in the ::= operator
        let dce_pos = line.find("::=")?;
        (format!(" {} ", inferred), (dce_pos + 2) as u32)
    } else if is_colon_eq {
        // := → name: Type = value
        // Position hint right after the colon (before the =)
        // Find the := in the line and position after the :
        let colon_eq_pos = line.find(":=")?;
        (format!(" {} ", inferred), (colon_eq_pos + 1) as u32)
    } else {
        // = → name: Type = value (with space before =)
        (format!(": {} ", inferred), hint_pos)
    };

    Some(InlayHint {
        position: Position {
            line: line_num,
            character: hint_char_pos,
        },
        label: InlayHintLabel::String(hint_label),
        kind: Some(InlayHintKind::TYPE),
        text_edits: None,
        tooltip: None,
        padding_left: None,
        padding_right: None,
        data: None,
    })
}

/// Parse a variable declaration line.
/// Returns: (var_name, hint_position, has_explicit_type, is_mutable, is_colon_eq, rhs)
fn parse_var_decl(line: &str) -> Option<(String, u32, bool, bool, bool, String)> {
    let trimmed = line.trim();

    if trimmed.is_empty() || trimmed.starts_with("//") {
        return None;
    }

    // Detect declaration type and operator:
    // - ::= means mutable with inferred type
    // - := means immutable with inferred type (shorthand)
    // - = means immutable with inferred type
    // is_mutable: true for ::=, false for := and =
    // is_colon_eq: true for :=, false for others
    let (eq_pos, eq_char_count, is_mutable, is_colon_eq) = if let Some(pos) = trimmed.find(" ::= ")
    {
        (pos, 5, true, false)
    } else if let Some(pos) = trimmed.find("::=") {
        (pos, 3, true, false)
    } else if let Some(pos) = trimmed.find(" := ") {
        (pos, 4, false, true)
    } else if let Some(pos) = trimmed.find(":=") {
        (pos, 2, false, true)
    } else if let Some(pos) = trimmed.find(" = ") {
        (pos, 3, false, false)
    } else {
        return None;
    };

    let before_eq = &trimmed[..eq_pos];
    let after_eq = &trimmed[eq_pos + eq_char_count..];

    // Check for explicit type annotation
    let has_explicit_type = if before_eq.contains("::") {
        // Mutable with explicit type: "name :: Type"
        let parts: Vec<&str> = before_eq.splitn(2, "::").collect();
        parts.len() == 2 && !parts[1].trim().is_empty()
    } else if before_eq.contains(':') {
        // Immutable with explicit type: "name: Type"
        let colon_pos = before_eq.find(':')?;
        let after_colon = before_eq[colon_pos + 1..].trim();
        !after_colon.is_empty()
    } else {
        false
    };

    // Extract variable name
    let var_name = if before_eq.contains("::") {
        before_eq.split("::").next()?.trim()
    } else if before_eq.contains(':') {
        before_eq.split(':').next()?.trim()
    } else {
        before_eq.trim()
    };

    if var_name.is_empty()
        || var_name.contains('(')
        || var_name.contains(')')
        || var_name.contains('.')
        || var_name.contains(' ')
        || var_name.chars().next()?.is_uppercase()
    {
        return None;
    }

    let var_start = line.find(var_name)?;
    let var_end_pos = (var_start + var_name.len()) as u32;

    Some((
        var_name.to_string(),
        var_end_pos,
        has_explicit_type,
        is_mutable,
        is_colon_eq,
        after_eq.trim().to_string(),
    ))
}

fn infer_type(rhs: &str) -> Option<String> {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::well_known::well_known;

    let rhs = rhs.trim();

    // Try parsing the expression using the real parser
    let lexer = Lexer::new(rhs);
    let mut parser = Parser::new(lexer);
    if let Ok(expr) = parser.parse_expression() {
        if let Some(type_name) = infer_type_from_expr(&expr) {
            return Some(type_name);
        }
    }

    // Fallback for simple literals that might not parse in isolation
    if rhs.starts_with('"') || rhs.starts_with('\'') {
        return Some("StaticString".to_string());
    }

    if rhs == "true" || rhs == "false" {
        return Some("bool".to_string());
    }

    // Check for None variant
    let wk = well_known();
    if rhs == wk.none_name() {
        return Some(wk.option_name().to_string());
    }

    None
}

/// Infer type from a parsed AST Expression
fn infer_type_from_expr(expr: &Expression) -> Option<String> {
    use crate::well_known::well_known;

    let wk = well_known();

    match expr {
        // Integer literals
        Expression::Integer8(_)
        | Expression::Integer16(_)
        | Expression::Integer32(_)
        | Expression::Integer64(_) => Some("i32".to_string()),

        // Unsigned integer literals
        Expression::Unsigned8(_)
        | Expression::Unsigned16(_)
        | Expression::Unsigned32(_)
        | Expression::Unsigned64(_) => Some("u64".to_string()),

        // Float literals
        Expression::Float32(_) | Expression::Float64(_) => Some("f64".to_string()),

        // Boolean
        Expression::Boolean(_) => Some("bool".to_string()),

        // String literal
        Expression::String(_) => Some("StaticString".to_string()),

        // Function call - check if it's a type constructor
        Expression::FunctionCall { name, .. } => {
            let base_name = get_base_type_name(name);
            // Check if stdlib has this as a struct (Vec, HashMap, etc.)
            if stdlib_types().get_struct_definition(&base_name).is_some() {
                return Some(base_name);
            }
            // Check for constructor methods
            if let Some(dot_pos) = name.rfind('.') {
                let receiver = &name[..dot_pos];
                let method = &name[dot_pos + 1..];
                if matches!(method, "new" | "init" | "create" | "default" | "from") {
                    return Some(receiver.to_string());
                }
                // Check stdlib for method return type
                if let Some(ret) = stdlib_types().get_method_return_type(receiver, method) {
                    return Some(format_type(ret));
                }
            }
            None
        }

        // Struct literal - the name IS the type
        Expression::StructLiteral { name, .. } => Some(name.clone()),

        // Enum variant construction
        Expression::EnumVariant {
            enum_name, variant, ..
        } => {
            if wk.is_option(enum_name) || variant == "Some" || variant == "None" {
                Some(wk.option_name().to_string())
            } else if wk.is_result(enum_name) || variant == "Ok" || variant == "Err" {
                Some(wk.result_name().to_string())
            } else {
                Some(enum_name.clone())
            }
        }

        // Enum literal (shorthand like .Some(x))
        Expression::EnumLiteral { variant, .. } => {
            if variant == "Some" || variant == "None" {
                Some(wk.option_name().to_string())
            } else if variant == "Ok" || variant == "Err" {
                Some(wk.result_name().to_string())
            } else {
                None
            }
        }

        // Array literal
        Expression::ArrayLiteral(_) => Some("Array".to_string()),

        // Method call - try to get return type from stdlib
        Expression::MethodCall { object, method, .. } => {
            if let Some(recv_type) = infer_type_from_expr(object) {
                if let Some(ret) = stdlib_types().get_method_return_type(&recv_type, method) {
                    return Some(format_type(ret));
                }
            }
            None
        }

        _ => None,
    }
}

fn collect_hints_from_statements(
    statements: &[Statement],
    content: &str,
    doc: &Document,
    store: &DocumentStore,
    hints: &mut Vec<InlayHint>,
    seen: &mut std::collections::HashSet<(u32, u32)>,
) {
    for stmt in statements {
        match stmt {
            Statement::VariableDeclaration {
                name,
                type_,
                initializer,
                ..
            } => {
                if type_.is_none() {
                    if let Some(init) = initializer {
                        if let Some(inferred) = infer_expr_type(init, doc, store) {
                            if let Some(pos) = find_var_pos(content, name) {
                                let key = (pos.line, pos.character);
                                if !seen.contains(&key) {
                                    seen.insert(key);
                                    hints.push(InlayHint {
                                        position: pos,
                                        label: InlayHintLabel::String(format!(": {}", inferred)),
                                        kind: Some(InlayHintKind::TYPE),
                                        text_edits: None,
                                        tooltip: None,
                                        padding_left: None,
                                        padding_right: None,
                                        data: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Statement::Loop { body, .. } => {
                collect_hints_from_statements(body, content, doc, store, hints, seen);
            }
            _ => {}
        }
    }
}

fn find_var_pos(content: &str, var_name: &str) -> Option<Position> {
    let mut in_struct_def = false;
    let mut brace_depth: i32 = 0;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Track struct/enum definitions to skip them
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            // Detect struct definition start: "Name: {" or "Name<T>: {"
            if let Some(colon_pos) = trimmed.find(':') {
                let before = trimmed[..colon_pos].trim();
                let after = trimmed[colon_pos + 1..].trim();

                // Check for PascalCase type definition (might have generics like Name<T>)
                let base_name = get_base_type_name(before);
                if !base_name.is_empty()
                    && base_name.chars().next().unwrap().is_uppercase()
                    && !before.contains('=')
                    && (after.starts_with('{') || after.contains('{'))
                {
                    in_struct_def = true;
                    brace_depth = 0;
                }
            }

            // Track brace depth
            if in_struct_def {
                for ch in trimmed.chars() {
                    if ch == '{' {
                        brace_depth += 1;
                    } else if ch == '}' {
                        brace_depth -= 1;
                        if brace_depth <= 0 {
                            in_struct_def = false;
                        }
                    }
                }
            }
        }

        // Skip lines inside struct definitions
        if in_struct_def {
            continue;
        }

        // Look for variable declaration (must have = operator, not just :)
        if let Some(pos) = line.find(var_name) {
            let before = pos == 0
                || line
                    .as_bytes()
                    .get(pos - 1)
                    .map(|&b| b.is_ascii_whitespace())
                    .unwrap_or(true);
            let after = &line[pos + var_name.len()..].trim_start();

            // Only match if followed by = (with optional : for type annotation)
            // This excludes struct field definitions like "name: Type"
            if before
                && (after.starts_with('=')
                    || after.starts_with(":=")
                    || after.starts_with("::=")
                    || (after.starts_with(':') && after.contains('=')))
            {
                return Some(Position {
                    line: line_num as u32,
                    character: (pos + var_name.len()) as u32,
                });
            }
        }
    }
    None
}

fn infer_expr_type(expr: &Expression, doc: &Document, store: &DocumentStore) -> Option<String> {
    match expr {
        Expression::Integer32(_) => Some("i32".to_string()),
        Expression::Integer64(_) => Some("i64".to_string()),
        Expression::Float32(_) => Some("f32".to_string()),
        Expression::Float64(_) => Some("f64".to_string()),
        Expression::String(_) => Some("StaticString".to_string()),
        Expression::Boolean(_) => Some("bool".to_string()),

        Expression::BinaryOp { left, right, .. } => {
            let l = infer_expr_type(left, doc, store)?;
            let r = infer_expr_type(right, doc, store)?;
            Some(
                if l == "f64" || r == "f64" {
                    "f64"
                } else if l == "i64" || r == "i64" {
                    "i64"
                } else {
                    "i32"
                }
                .to_string(),
            )
        }

        Expression::FunctionCall { name, .. } => {
            if let Some(dot_pos) = name.rfind('.') {
                let receiver = &name[..dot_pos];
                let method = &name[dot_pos + 1..];

                if let Some(ret) = stdlib_types().get_function_return_type(receiver, method) {
                    return Some(format_type(ret));
                }
                if let Some(ret) = stdlib_types().get_method_return_type(receiver, method) {
                    return Some(format_type(ret));
                }
            }

            for symbols in [
                &doc.symbols,
                &store.stdlib_symbols,
                &store.workspace_symbols,
            ] {
                if let Some(sym) = symbols.get(name) {
                    if let Some(AstType::Function { return_type, .. }) = sym.type_info.as_ref() {
                        return Some(format_type(return_type));
                    }
                    if let Some(ref detail) = sym.detail {
                        if let Some(ret) = extract_return_type(detail) {
                            return Some(ret);
                        }
                    }
                }
            }
            None
        }

        Expression::StructLiteral { name, .. } => Some(name.clone()),

        Expression::MethodCall { object, method, .. } => {
            let recv_type = infer_expr_type(object, doc, store)?;
            if let Some(ret) = stdlib_types().get_method_return_type(&recv_type, method) {
                return Some(format_type(ret));
            }
            None
        }

        Expression::Identifier(name) => {
            if let Some(sym) = doc.symbols.get(name) {
                if let Some(ref ti) = sym.type_info {
                    return Some(format_type(ti));
                }
                if let Some(ref detail) = sym.detail {
                    if let Some(colon_pos) = detail.find(':') {
                        let type_part = detail[colon_pos + 1..].trim();
                        if !type_part.is_empty() {
                            return Some(type_part.to_string());
                        }
                    }
                }
            }
            find_variable_type_in_content(&doc.content, name)
        }

        _ => None,
    }
}

fn extract_return_type(sig: &str) -> Option<String> {
    let close = sig.rfind(')')?;
    let ret = sig[close + 1..].trim();
    if ret.is_empty() || ret == "{" {
        None
    } else {
        Some(ret.split_whitespace().next()?.to_string())
    }
}

fn find_variable_type_in_content(content: &str, var_name: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();

        let pattern = format!("{} = ", var_name);
        let pattern2 = format!("{} ::= ", var_name);

        if trimmed.starts_with(&pattern) || trimmed.starts_with(&pattern2) {
            let eq_pos = trimmed.find('=').unwrap();
            let rhs = trimmed[eq_pos + 1..].trim().trim_start_matches('=').trim();

            if let Some(dot_pos) = rhs.find('.') {
                let type_name = &rhs[..dot_pos];
                if type_name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    let method = rhs[dot_pos + 1..].split('(').next().unwrap_or("");
                    if matches!(
                        method,
                        "new" | "init" | "create" | "default" | "from" | "with_capacity"
                    ) {
                        return Some(type_name.to_string());
                    }
                }
            }
        }
    }
    None
}
