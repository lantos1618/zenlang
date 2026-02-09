// Signature Help Module for Zen LSP
// Handles textDocument/signatureHelp requests

use lsp_server::{Request, Response};
use lsp_types::*;
use std::sync::{Arc, RwLock};

use super::document_store::DocumentStore;
use super::helpers::{
    char_pos_to_byte_pos, null_response, success_response, try_parse_params, try_read,
};
use super::types::SymbolInfo;
use crate::ast::{Declaration, Expression, Statement};
use crate::lsp::utils::format_type;
use crate::type_context::TypeContext;

// ============================================================================
// PUBLIC HANDLER FUNCTION
// ============================================================================

pub fn handle_signature_help(req: Request, store: &Arc<RwLock<DocumentStore>>) -> Response {
    let params: SignatureHelpParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let empty_help = SignatureHelp {
        signatures: vec![],
        active_signature: None,
        active_parameter: None,
    };

    let store = match try_read(store.as_ref(), &req) {
        Ok(s) => s,
        Err(_) => return success_response(&req, empty_help),
    };

    let doc = match store
        .documents
        .get(&params.text_document_position_params.text_document.uri)
    {
        Some(d) => d,
        None => return null_response(&req),
    };

    // Find function call at cursor position
    let position = params.text_document_position_params.position;
    let function_call = find_function_call_at_position(doc.ast.as_deref(), &doc.content, position);

    log::debug!(
        "[LSP] Signature help at {}:{} - function_call: {:?}",
        position.line,
        position.character,
        function_call
    );

    let signature_help = match function_call {
        Some((function_name, active_param)) => {
            log::debug!(
                "[LSP] Looking for function '{}' in {} doc symbols",
                function_name,
                doc.symbols.len()
            );

            let mut signature_info = None;

            // PRIORITY: Use TypeContext for authoritative parameter info
            if let Some(type_ctx) = &doc.type_context {
                signature_info = find_function_in_type_context(type_ctx, &function_name);
            }

            // Check document symbols
            if signature_info.is_none() {
                if let Some(symbol) = doc.symbols.get(&function_name) {
                    log::debug!("[LSP] Found '{}' in document symbols", function_name);
                    signature_info = Some(create_signature_info(symbol));
                }
            }

            // Check stdlib symbols if not found
            if signature_info.is_none() {
                if let Some(symbol) = store.stdlib_symbols.get(&function_name) {
                    signature_info = Some(create_signature_info(symbol));
                }
            }

            // Check workspace symbols if not found
            if signature_info.is_none() {
                if let Some(symbol) = store.workspace_symbols.get(&function_name) {
                    signature_info = Some(create_signature_info(symbol));
                }
            }

            match signature_info {
                Some(sig_info) => SignatureHelp {
                    signatures: vec![sig_info],
                    active_signature: Some(0),
                    active_parameter: Some(active_param as u32),
                },
                None => SignatureHelp {
                    signatures: vec![],
                    active_signature: None,
                    active_parameter: None,
                },
            }
        }
        None => SignatureHelp {
            signatures: vec![],
            active_signature: None,
            active_parameter: None,
        },
    };

    Response {
        id: req.id,
        result: serde_json::to_value(signature_help).ok(),
        error: None,
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn find_function_call_at_position(
    ast: Option<&[Declaration]>,
    content: &str,
    position: Position,
) -> Option<(String, usize)> {
    // Convert LSP 0-based line to 1-based line used by AST spans
    let cursor_line = position.line as usize + 1;
    let cursor_col = position.character as usize;

    // Primary path: walk the AST to find the call at the cursor
    if let Some(declarations) = ast {
        if let Some(result) =
            find_call_in_declarations(declarations, content, cursor_line, cursor_col)
        {
            return Some(result);
        }
    }

    // Fallback: text-based approach when AST is not available or didn't find a match
    find_function_call_from_text(content, position)
}

/// Walk all declarations looking for a function/method call at the cursor position.
fn find_call_in_declarations(
    declarations: &[Declaration],
    content: &str,
    cursor_line: usize,
    cursor_col: usize,
) -> Option<(String, usize)> {
    for decl in declarations {
        let result = match decl {
            Declaration::Function(func) => {
                find_call_in_statements(&func.body, content, cursor_line, cursor_col)
            }
            Declaration::Struct(s) => {
                // Walk methods inside struct definitions
                for method in &s.methods {
                    if let Some(r) =
                        find_call_in_statements(&method.body, content, cursor_line, cursor_col)
                    {
                        return Some(r);
                    }
                }
                None
            }
            Declaration::Enum(e) => {
                for method in &e.methods {
                    if let Some(r) =
                        find_call_in_statements(&method.body, content, cursor_line, cursor_col)
                    {
                        return Some(r);
                    }
                }
                None
            }
            Declaration::ImplBlock(imp) => {
                for method in &imp.methods {
                    if let Some(r) =
                        find_call_in_statements(&method.body, content, cursor_line, cursor_col)
                    {
                        return Some(r);
                    }
                }
                None
            }
            Declaration::TraitImplementation(ti) => {
                for method in &ti.methods {
                    if let Some(r) =
                        find_call_in_statements(&method.body, content, cursor_line, cursor_col)
                    {
                        return Some(r);
                    }
                }
                None
            }
            Declaration::ComptimeBlock(stmts) => {
                find_call_in_statements(stmts, content, cursor_line, cursor_col)
            }
            Declaration::Constant { value, .. } => {
                find_call_in_expression(value, content, cursor_line, cursor_col)
            }
            _ => None,
        };
        if result.is_some() {
            return result;
        }
    }
    None
}

/// Walk statements looking for a function/method call at the cursor position.
fn find_call_in_statements(
    stmts: &[Statement],
    content: &str,
    cursor_line: usize,
    cursor_col: usize,
) -> Option<(String, usize)> {
    for stmt in stmts {
        let result = match stmt {
            Statement::Expression { expr, .. } => {
                find_call_in_expression(expr, content, cursor_line, cursor_col)
            }
            Statement::Return { expr, .. } => {
                find_call_in_expression(expr, content, cursor_line, cursor_col)
            }
            Statement::VariableDeclaration { initializer, .. } => {
                if let Some(init) = initializer {
                    find_call_in_expression(init, content, cursor_line, cursor_col)
                } else {
                    None
                }
            }
            Statement::VariableAssignment { value, .. } => {
                find_call_in_expression(value, content, cursor_line, cursor_col)
            }
            Statement::PointerAssignment { pointer, value, .. } => {
                find_call_in_expression(pointer, content, cursor_line, cursor_col)
                    .or_else(|| find_call_in_expression(value, content, cursor_line, cursor_col))
            }
            Statement::Loop { kind, body, .. } => {
                if let crate::ast::LoopKind::Condition(cond) = kind {
                    if let Some(r) = find_call_in_expression(cond, content, cursor_line, cursor_col)
                    {
                        return Some(r);
                    }
                }
                find_call_in_statements(body, content, cursor_line, cursor_col)
            }
            Statement::ComptimeBlock { statements, .. } => {
                find_call_in_statements(statements, content, cursor_line, cursor_col)
            }
            Statement::Defer { statement, .. } => find_call_in_statements(
                std::slice::from_ref(statement.as_ref()),
                content,
                cursor_line,
                cursor_col,
            ),
            Statement::ThisDefer { expr, .. } => {
                find_call_in_expression(expr, content, cursor_line, cursor_col)
            }
            Statement::DestructuringImport { source, .. } => {
                find_call_in_expression(source, content, cursor_line, cursor_col)
            }
            Statement::Block { statements, .. } => {
                find_call_in_statements(statements, content, cursor_line, cursor_col)
            }
            _ => None,
        };
        if result.is_some() {
            return result;
        }
    }
    None
}

/// Recursively search an expression tree for a FunctionCall or MethodCall at the cursor.
///
/// Returns the innermost (most deeply nested) call that contains the cursor,
/// so `foo(bar(cursor))` returns `bar`, not `foo`.
fn find_call_in_expression(
    expr: &Expression,
    content: &str,
    cursor_line: usize,
    cursor_col: usize,
) -> Option<(String, usize)> {
    match expr {
        Expression::FunctionCall {
            name, args, span, ..
        } => {
            // First recurse into args to find a more deeply nested call
            for arg in args {
                if let Some(result) = find_call_in_expression(arg, content, cursor_line, cursor_col)
                {
                    return Some(result);
                }
            }
            // Then check if the cursor is inside THIS call
            if let Some(s) = span {
                if cursor_is_in_call(s, content, cursor_line, cursor_col) {
                    let active = count_args_before_cursor(args, content, cursor_line, cursor_col);
                    return Some((name.clone(), active));
                }
            }
            None
        }
        Expression::MethodCall {
            object,
            method,
            args,
            span,
            ..
        } => {
            // Recurse into the receiver object
            if let Some(result) = find_call_in_expression(object, content, cursor_line, cursor_col)
            {
                return Some(result);
            }
            // Recurse into args
            for arg in args {
                if let Some(result) = find_call_in_expression(arg, content, cursor_line, cursor_col)
                {
                    return Some(result);
                }
            }
            // Check if cursor is inside THIS method call
            if let Some(s) = span {
                if cursor_is_in_call(s, content, cursor_line, cursor_col) {
                    let active = count_args_before_cursor(args, content, cursor_line, cursor_col);
                    return Some((method.clone(), active));
                }
            }
            None
        }
        // Recurse into sub-expressions for all other variants
        Expression::BinaryOp { left, right, .. } => {
            find_call_in_expression(left, content, cursor_line, cursor_col)
                .or_else(|| find_call_in_expression(right, content, cursor_line, cursor_col))
        }
        Expression::QuestionMatch { scrutinee, arms } => {
            if let Some(r) = find_call_in_expression(scrutinee, content, cursor_line, cursor_col) {
                return Some(r);
            }
            for arm in arms {
                if let Some(r) =
                    find_call_in_expression(&arm.body, content, cursor_line, cursor_col)
                {
                    return Some(r);
                }
                if let Some(guard) = &arm.guard {
                    if let Some(r) =
                        find_call_in_expression(guard, content, cursor_line, cursor_col)
                    {
                        return Some(r);
                    }
                }
            }
            None
        }
        Expression::Conditional { scrutinee, arms } => {
            if let Some(r) = find_call_in_expression(scrutinee, content, cursor_line, cursor_col) {
                return Some(r);
            }
            for arm in arms {
                if let Some(r) =
                    find_call_in_expression(&arm.body, content, cursor_line, cursor_col)
                {
                    return Some(r);
                }
            }
            None
        }
        Expression::PatternMatch { scrutinee, arms } => {
            if let Some(r) = find_call_in_expression(scrutinee, content, cursor_line, cursor_col) {
                return Some(r);
            }
            for arm in arms {
                if let Some(r) =
                    find_call_in_expression(&arm.body, content, cursor_line, cursor_col)
                {
                    return Some(r);
                }
                if let Some(guard) = &arm.guard {
                    if let Some(r) =
                        find_call_in_expression(guard, content, cursor_line, cursor_col)
                    {
                        return Some(r);
                    }
                }
            }
            None
        }
        Expression::MemberAccess { object, .. } => {
            find_call_in_expression(object, content, cursor_line, cursor_col)
        }
        Expression::ArrayIndex { array, index } => {
            find_call_in_expression(array, content, cursor_line, cursor_col)
                .or_else(|| find_call_in_expression(index, content, cursor_line, cursor_col))
        }
        Expression::ArrayLiteral(elems) => {
            for elem in elems {
                if let Some(r) = find_call_in_expression(elem, content, cursor_line, cursor_col) {
                    return Some(r);
                }
            }
            None
        }
        Expression::StructLiteral { fields, .. } => {
            for (_, val) in fields {
                if let Some(r) = find_call_in_expression(val, content, cursor_line, cursor_col) {
                    return Some(r);
                }
            }
            None
        }
        Expression::StructField { struct_, .. } => {
            find_call_in_expression(struct_, content, cursor_line, cursor_col)
        }
        Expression::Some(inner)
        | Expression::AddressOf(inner)
        | Expression::Dereference(inner)
        | Expression::PointerDereference(inner)
        | Expression::PointerAddress(inner)
        | Expression::CreateReference(inner)
        | Expression::CreateMutableReference(inner)
        | Expression::StringLength(inner)
        | Expression::Comptime(inner)
        | Expression::Raise(inner)
        | Expression::Defer(inner)
        | Expression::Return(inner) => {
            find_call_in_expression(inner, content, cursor_line, cursor_col)
        }
        Expression::PointerOffset { pointer, offset } => {
            find_call_in_expression(pointer, content, cursor_line, cursor_col)
                .or_else(|| find_call_in_expression(offset, content, cursor_line, cursor_col))
        }
        Expression::EnumVariant { payload, .. } | Expression::EnumLiteral { payload, .. } => {
            if let Some(p) = payload {
                find_call_in_expression(p, content, cursor_line, cursor_col)
            } else {
                None
            }
        }
        Expression::StringInterpolation { parts } => {
            for part in parts {
                if let crate::ast::StringPart::Interpolation(e) = part {
                    if let Some(r) = find_call_in_expression(e, content, cursor_line, cursor_col) {
                        return Some(r);
                    }
                }
            }
            None
        }
        Expression::Range { start, end, .. } => {
            find_call_in_expression(start, content, cursor_line, cursor_col)
                .or_else(|| find_call_in_expression(end, content, cursor_line, cursor_col))
        }
        Expression::Loop { body } => {
            find_call_in_expression(body, content, cursor_line, cursor_col)
        }
        Expression::CollectionLoop {
            collection, body, ..
        } => find_call_in_expression(collection, content, cursor_line, cursor_col)
            .or_else(|| find_call_in_expression(body, content, cursor_line, cursor_col)),
        Expression::Closure { body, .. } => {
            find_call_in_expression(body, content, cursor_line, cursor_col)
        }
        Expression::Block(stmts) => {
            find_call_in_statements(stmts, content, cursor_line, cursor_col)
        }
        Expression::Break { value, .. } => {
            if let Some(v) = value {
                find_call_in_expression(v, content, cursor_line, cursor_col)
            } else {
                None
            }
        }
        Expression::DynVecConstructor {
            allocator,
            initial_capacity,
            ..
        } => {
            if let Some(r) = find_call_in_expression(allocator, content, cursor_line, cursor_col) {
                return Some(r);
            }
            if let Some(cap) = initial_capacity {
                return find_call_in_expression(cap, content, cursor_line, cursor_col);
            }
            None
        }
        Expression::VecConstructor { initial_values, .. } => {
            if let Some(vals) = initial_values {
                for v in vals {
                    if let Some(r) = find_call_in_expression(v, content, cursor_line, cursor_col) {
                        return Some(r);
                    }
                }
            }
            None
        }
        // Leaf expressions with no sub-expressions
        _ => None,
    }
}

/// Check whether the cursor is positioned inside a function/method call's argument list.
///
/// The span marks the start of the call expression (where the function name begins).
/// We scan forward in the source text from the span's start position to find the `(`,
/// then check if the cursor is between that `(` and the matching `)`.
fn cursor_is_in_call(
    span: &crate::error::Span,
    content: &str,
    cursor_line: usize,
    cursor_col: usize,
) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    // span.line is 1-based
    let span_line_idx = span.line.saturating_sub(1);
    if span_line_idx >= lines.len() {
        return false;
    }

    // Find the byte offset of the span start in the content
    let span_byte_start = lines[..span_line_idx]
        .iter()
        .map(|l| l.len() + 1) // +1 for newline
        .sum::<usize>()
        + span.column;

    // Find the opening paren `(` starting from the span position
    let content_bytes = content.as_bytes();
    let mut paren_pos = None;
    for (i, &byte) in content_bytes.iter().enumerate().skip(span_byte_start) {
        if byte == b'(' {
            paren_pos = Some(i);
            break;
        }
        // Stop if we hit a line boundary too far away (not part of this call)
        if byte == b';' || byte == b'{' {
            break;
        }
    }

    let open_paren = match paren_pos {
        Some(p) => p,
        None => return false,
    };

    // Convert open_paren byte offset to (line, col)
    let (paren_line, paren_col) = byte_offset_to_line_col(content, open_paren);

    // Cursor must be after the `(`
    if cursor_line < paren_line || (cursor_line == paren_line && cursor_col <= paren_col) {
        return false;
    }

    // Find the matching close paren
    let mut depth = 1;
    let mut i = open_paren + 1;
    while i < content_bytes.len() && depth > 0 {
        match content_bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        if depth > 0 {
            i += 1;
        }
    }

    if depth != 0 {
        // Unmatched paren - the call is still being typed, cursor is inside
        return true;
    }

    // i points to the closing paren
    let (close_line, close_col) = byte_offset_to_line_col(content, i);

    // Cursor must be before or at the closing `)`
    cursor_line < close_line || (cursor_line == close_line && cursor_col <= close_col)
}

/// Convert a byte offset in content to a (line, col) pair where line is 1-based.
fn byte_offset_to_line_col(content: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 0;
    for (i, ch) in content.bytes().enumerate() {
        if i == byte_offset {
            return (line, col);
        }
        if ch == b'\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Count how many args have spans that start at or before the cursor position.
/// This gives us the 0-based active parameter index.
///
/// If args don't have usable spans, falls back to counting commas in the source text
/// between the call's opening paren and the cursor.
fn count_args_before_cursor(
    args: &[Expression],
    content: &str,
    cursor_line: usize,
    cursor_col: usize,
) -> usize {
    // Try span-based counting: count args whose spans start at or before cursor
    let mut counted = 0;
    let mut any_span = false;

    for arg in args {
        if let Some(span) = get_expression_span(arg) {
            any_span = true;
            // If this arg's span starts at or before the cursor, the cursor is
            // at or past this argument
            if span.line < cursor_line || (span.line == cursor_line && span.column <= cursor_col) {
                counted += 1;
            }
        }
    }

    if any_span {
        // counted is the number of args that start at or before cursor.
        // The active parameter index is counted - 1 (if we're inside the last
        // started arg) or counted (if cursor is between args/after a comma).
        // The safest approach: active_param = max(0, counted - 1) when counted > 0,
        // else 0. But more precisely, if cursor is past the last arg start,
        // we're on that arg. The index is counted-1.
        if counted > 0 {
            counted - 1
        } else {
            0
        }
    } else {
        // Fallback: count commas in source between the approximate call start and cursor.
        // This is a small, contained text operation.
        count_commas_before_cursor(content, cursor_line, cursor_col)
    }
}

/// Get the span from an expression, if it has one directly or can be inferred.
fn get_expression_span(expr: &Expression) -> Option<&crate::error::Span> {
    match expr {
        Expression::FunctionCall { span, .. } | Expression::MethodCall { span, .. } => {
            span.as_ref()
        }
        _ => None,
    }
}

/// Count commas at depth 0 on the current line up to the cursor position.
/// Used as a fallback when arg spans are not available.
fn count_commas_before_cursor(content: &str, cursor_line: usize, cursor_col: usize) -> usize {
    let lines: Vec<&str> = content.lines().collect();

    // Build context from up to 5 lines before cursor to handle multi-line calls
    let mut context = String::new();
    let mut line_offset = 0;
    let cursor_line_0 = cursor_line.saturating_sub(1); // convert to 0-based

    let start_line = cursor_line_0.saturating_sub(5);
    for i in start_line..=cursor_line_0 {
        if i >= lines.len() {
            break;
        }
        if i == cursor_line_0 {
            line_offset = context.len();
        }
        context.push_str(lines[i]);
        context.push(' ');
    }

    let current_line = lines.get(cursor_line_0).unwrap_or(&"");
    let char_byte_offset = char_pos_to_byte_pos(current_line, cursor_col);
    let cursor_pos = (line_offset + char_byte_offset).min(context.len());

    // Walk backwards to find opening paren
    let context_bytes = context.as_bytes();
    let mut paren_count = 0i32;
    let mut current_pos = cursor_pos;

    while current_pos > 0 {
        let byte = context_bytes[current_pos - 1];
        if byte == b')' {
            paren_count += 1;
        } else if byte == b'(' {
            if paren_count == 0 {
                break;
            }
            paren_count -= 1;
        }
        current_pos -= 1;
    }

    if current_pos == 0 {
        return 0;
    }

    // Count commas between opening paren and cursor
    let inside = &context[current_pos..cursor_pos];
    let mut active_param = 0;
    let mut depth = 0i32;
    for ch in inside.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => active_param += 1,
            _ => {}
        }
    }

    active_param
}

/// Text-based fallback for finding function calls when no AST is available.
fn find_function_call_from_text(content: &str, position: Position) -> Option<(String, usize)> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    // Build context string from current line and potential previous lines for multi-line calls
    let mut context = String::new();
    let mut line_offset = 0;

    // Look back up to 5 lines for multi-line function calls
    let start_line = position.line.saturating_sub(5);
    for i in start_line..=position.line {
        if i as usize >= lines.len() {
            break;
        }
        if i == position.line {
            line_offset = context.len();
        }
        context.push_str(lines[i as usize]);
        context.push(' ');
    }

    // Convert character position to byte position for the current line
    let current_line = lines.get(position.line as usize).unwrap_or(&"");
    let char_byte_offset = char_pos_to_byte_pos(current_line, position.character as usize);
    let cursor_pos = (line_offset + char_byte_offset).min(context.len());

    // Find the function call - look backwards from cursor for opening paren
    let mut paren_count = 0;
    let mut current_pos = cursor_pos;
    let context_bytes = context.as_bytes();

    // Move to the nearest opening paren (using bytes - parens are ASCII)
    while current_pos > 0 {
        let byte = context_bytes[current_pos - 1];
        if byte == b')' {
            paren_count += 1;
        } else if byte == b'(' {
            if paren_count == 0 {
                break;
            }
            paren_count -= 1;
        }
        current_pos -= 1;
    }

    if current_pos == 0 {
        return None; // No opening paren found
    }

    // Extract function name before the opening paren using the Zen parser.
    let before_paren = context[..current_pos - 1].trim();
    let expr_text = before_paren
        .rsplit_once(['=', ';', '{', ',', '('])
        .map(|(_, after)| after.trim())
        .unwrap_or(before_paren);

    let lexer = crate::lexer::Lexer::new(expr_text);
    let mut parser = crate::parser::Parser::new(lexer);
    let function_name = if let Ok(expr) = parser.parse_expression() {
        match expr {
            Expression::Identifier(name) => name,
            Expression::FunctionCall {
                module: Some(m),
                name,
                ..
            } => format!("{}.{}", m, name),
            Expression::FunctionCall { name, .. } => name,
            Expression::MethodCall { method, .. } => method,
            Expression::MemberAccess { member, .. } => member,
            _ => return None,
        }
    } else {
        // Fallback: split on whitespace/delimiters and take the last dotted segment
        before_paren
            .split(|c: char| c.is_whitespace() || c == '=' || c == ',' || c == ';' || c == '{')
            .next_back()?
            .trim()
            .rsplit('.')
            .next()?
            .to_string()
    };

    // Count parameters by counting commas at paren_depth = 0
    let inside_parens = &context[current_pos..cursor_pos];
    let mut active_param = 0;
    let mut depth = 0;

    for ch in inside_parens.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => active_param += 1,
            _ => {}
        }
    }

    Some((function_name, active_param))
}

/// Find function in TypeContext and create SignatureInformation from typechecker data
fn find_function_in_type_context(
    type_ctx: &TypeContext,
    function_name: &str,
) -> Option<SignatureInformation> {
    // Check direct function match
    if let Some(func_type) = type_ctx.functions.get(function_name) {
        let params_str = func_type
            .params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
            .collect::<Vec<_>>()
            .join(", ");
        let ret_str = format_type(&func_type.return_type);
        let label = format!("{} = ({}) {}", function_name, params_str, ret_str);

        let parameters: Vec<ParameterInformation> = func_type
            .params
            .iter()
            .map(|(name, ty)| ParameterInformation {
                label: lsp_types::ParameterLabel::Simple(format!("{}: {}", name, format_type(ty))),
                documentation: None,
            })
            .collect();

        return Some(SignatureInformation {
            label,
            documentation: None,
            parameters: if parameters.is_empty() {
                None
            } else {
                Some(parameters)
            },
            active_parameter: None,
        });
    }

    // Check methods — try "Type.method" patterns
    // The function_name from the call site might just be the method name,
    // so search method_params for any key ending with ".function_name"
    for (method_key, params) in &type_ctx.method_params {
        if method_key.ends_with(&format!(".{}", function_name)) {
            let ret_type = type_ctx.methods.get(method_key)?;
            let ret_str = format_type(ret_type);
            let params_str = params
                .iter()
                .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let label = format!("{} = ({}) {}", method_key, params_str, ret_str);

            let parameters: Vec<ParameterInformation> = params
                .iter()
                .map(|(name, ty)| ParameterInformation {
                    label: lsp_types::ParameterLabel::Simple(format!(
                        "{}: {}",
                        name,
                        format_type(ty)
                    )),
                    documentation: None,
                })
                .collect();

            return Some(SignatureInformation {
                label,
                documentation: None,
                parameters: if parameters.is_empty() {
                    None
                } else {
                    Some(parameters)
                },
                active_parameter: None,
            });
        }
    }

    None
}

/// Create SignatureInformation from a SymbolInfo, using structured params when available.
fn create_signature_info(symbol: &SymbolInfo) -> SignatureInformation {
    let label = symbol
        .detail
        .clone()
        .unwrap_or_else(|| format!("{}(...)", symbol.name));

    // Use structured params from SymbolInfo when available
    let parameters = if let Some(params) = &symbol.params {
        params
            .iter()
            .map(|(name, ty)| ParameterInformation {
                label: lsp_types::ParameterLabel::Simple(format!("{}: {}", name, format_type(ty))),
                documentation: None,
            })
            .collect()
    } else {
        vec![] // No structured params available — don't guess from text
    };

    SignatureInformation {
        label,
        documentation: symbol
            .documentation
            .as_ref()
            .map(|doc| Documentation::String(doc.clone())),
        parameters: if parameters.is_empty() {
            None
        } else {
            Some(parameters)
        },
        active_parameter: None,
    }
}
