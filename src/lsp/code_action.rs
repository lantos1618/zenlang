// Code Action Module for Zen LSP
// Handles textDocument/codeAction requests

use lsp_server::{Request, Response};
use lsp_types::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::document_store::DocumentStore;
use super::helpers::{success_response, try_lock, try_parse_params};

// ============================================================================
// PUBLIC HANDLER FUNCTION
// ============================================================================

pub fn handle_code_action(req: Request, store: &Arc<Mutex<DocumentStore>>) -> Response {
    let params: CodeActionParams = match try_parse_params(&req) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let mut actions = Vec::new();
    let store = match try_lock(store.as_ref(), &req) {
        Ok(s) => s,
        Err(_) => return success_response(&req, Vec::<CodeActionOrCommand>::new()),
    };

    if let Some(doc) = store.documents.get(&params.text_document.uri) {
        // Check diagnostics in the requested range
        for diagnostic in &params.context.diagnostics {
            if diagnostic.message.contains("requires an allocator") {
                // Create a code action to add get_default_allocator()
                actions.push(create_allocator_fix_action(
                    diagnostic,
                    &params.text_document.uri,
                    &doc.content,
                ));
            }

            // Add code action for string conversions
            if diagnostic.message.contains("type mismatch")
                && (diagnostic.message.contains("StaticString")
                    || diagnostic.message.contains("String"))
            {
                if let Some(action) = create_string_conversion_action(
                    diagnostic,
                    &params.text_document.uri,
                    &doc.content,
                ) {
                    actions.push(action);
                }
            }

            if diagnostic.message.contains("Result") && diagnostic.message.contains("unwrap") {
                if let Some(action) = create_error_handling_action(
                    diagnostic,
                    &params.text_document.uri,
                    &doc.content,
                ) {
                    actions.push(action);
                }
            }
        }

        // Add refactoring code actions (not tied to diagnostics)
        // Extract variable - only if there's a selection
        if params.range.start != params.range.end {
            if let Some(action) = create_extract_variable_action(
                &params.range,
                &params.text_document.uri,
                &doc.content,
            ) {
                actions.push(action);
            }

            // Extract function - only if there's a multi-line selection or complex expression
            if let Some(action) = create_extract_function_action(
                &params.range,
                &params.text_document.uri,
                &doc.content,
            ) {
                actions.push(action);
            }
        }

        if let Some(action) = create_add_import_action(&params.text_document.uri, &doc.content) {
            actions.push(action);
        }
    }

    success_response(&req, actions)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_allocator_fix_action(diagnostic: &Diagnostic, uri: &Url, content: &str) -> CodeAction {
    // Extract the line content
    let lines: Vec<&str> = content.lines().collect();
    let line_content = if (diagnostic.range.start.line as usize) < lines.len() {
        lines[diagnostic.range.start.line as usize]
    } else {
        ""
    };

    // Determine if we need to add allocator parameter or insert new call
    let (new_text, edit_range) = if line_content.contains("()") {
        // Empty parentheses - add allocator as first parameter
        (
            "get_default_allocator()".to_string(),
            Range {
                start: Position {
                    line: diagnostic.range.start.line,
                    character: diagnostic.range.end.character - 1, // Before closing paren
                },
                end: Position {
                    line: diagnostic.range.start.line,
                    character: diagnostic.range.end.character - 1,
                },
            },
        )
    } else if line_content.contains("(") {
        // Has parameters - add allocator as additional parameter
        (
            ", get_default_allocator()".to_string(),
            Range {
                start: Position {
                    line: diagnostic.range.end.line,
                    character: diagnostic.range.end.character - 1, // Before closing paren
                },
                end: Position {
                    line: diagnostic.range.end.line,
                    character: diagnostic.range.end.character - 1,
                },
            },
        )
    } else {
        // No parentheses - add full call
        ("(get_default_allocator())".to_string(), diagnostic.range)
    };

    let text_edit = TextEdit {
        range: edit_range,
        new_text,
    };

    let workspace_edit = WorkspaceEdit {
        changes: Some({
            let mut changes = HashMap::new();
            changes.insert(uri.clone(), vec![text_edit]);
            changes
        }),
        document_changes: None,
        change_annotations: None,
    };

    CodeAction {
        title: "Add get_default_allocator()".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit: Some(workspace_edit),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    }
}

fn create_string_conversion_action(
    diagnostic: &Diagnostic,
    uri: &Url,
    content: &str,
) -> Option<CodeAction> {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = diagnostic.range.start.line as usize;
    let line = lines.get(line_idx)?;

    let start_char = diagnostic.range.start.character as usize;
    let end_char = (diagnostic.range.end.character as usize).min(line.len());

    // Extract the text at the diagnostic range
    let selected = if start_char < line.len() && start_char < end_char {
        &line[start_char..end_char]
    } else {
        return None;
    };

    // Determine conversion direction and create actual edit
    let (title, new_text) = if diagnostic.message.contains("expected StaticString") {
        // If it's a String, call .as_static() - common pattern
        (
            "Convert to StaticString",
            format!("{}.as_static()", selected),
        )
    } else if diagnostic.message.contains("expected String") {
        // If it's a StaticString (literal), wrap with String.from()
        ("Convert to String", format!("String.from({})", selected))
    } else {
        return None;
    };

    let text_edit = TextEdit {
        range: diagnostic.range,
        new_text,
    };

    let workspace_edit = WorkspaceEdit {
        changes: Some({
            let mut changes = HashMap::new();
            changes.insert(uri.clone(), vec![text_edit]);
            changes
        }),
        document_changes: None,
        change_annotations: None,
    };

    Some(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit: Some(workspace_edit),
        command: None,
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}

fn create_error_handling_action(
    diagnostic: &Diagnostic,
    uri: &Url,
    content: &str,
) -> Option<CodeAction> {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = diagnostic.range.start.line as usize;
    let line = lines.get(line_idx)?;

    // Look for .unwrap() and suggest .raise() instead (Zen's error propagation)
    if let Some(unwrap_pos) = line.find(".unwrap()") {
        let text_edit = TextEdit {
            range: Range {
                start: Position {
                    line: diagnostic.range.start.line,
                    character: unwrap_pos as u32,
                },
                end: Position {
                    line: diagnostic.range.start.line,
                    character: (unwrap_pos + 9) as u32, // ".unwrap()" is 9 chars
                },
            },
            new_text: ".raise()".to_string(),
        };

        let workspace_edit = WorkspaceEdit {
            changes: Some({
                let mut changes = HashMap::new();
                changes.insert(uri.clone(), vec![text_edit]);
                changes
            }),
            document_changes: None,
            change_annotations: None,
        };

        return Some(CodeAction {
            title: "Replace .unwrap() with .raise() for error propagation".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(workspace_edit),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        });
    }

    None
}

fn create_extract_variable_action(range: &Range, uri: &Url, content: &str) -> Option<CodeAction> {
    // Extract the selected text
    let lines: Vec<&str> = content.lines().collect();
    let mut selected_text = String::new();

    if range.start.line == range.end.line {
        // Single line selection
        if let Some(line) = lines.get(range.start.line as usize) {
            let start_char = range.start.character as usize;
            let end_char = (range.end.character as usize).min(line.len());
            if start_char < line.len() && start_char < end_char {
                selected_text = line[start_char..end_char].to_string();
            }
        }
    } else {
        // Multi-line selection
        for line_idx in range.start.line..=range.end.line {
            if let Some(line) = lines.get(line_idx as usize) {
                if line_idx == range.start.line {
                    selected_text.push_str(&line[range.start.character as usize..]);
                } else if line_idx == range.end.line {
                    selected_text.push_str(&line[..range.end.character as usize]);
                } else {
                    selected_text.push_str(line);
                }
                if line_idx < range.end.line {
                    selected_text.push('\n');
                }
            }
        }
    }

    // Skip if selection is empty or just whitespace
    if selected_text.trim().is_empty() {
        return None;
    }

    // Skip if selection looks like a variable name already (simple heuristic)
    if selected_text
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_')
    {
        return None;
    }

    // Generate variable name based on selection
    let var_name = generate_variable_name(&selected_text);

    // Find the beginning of the current statement to insert the variable declaration
    let insert_line = range.start.line;
    let indent = if let Some(line) = lines.get(insert_line as usize) {
        line.chars()
            .take_while(|c| c.is_whitespace())
            .collect::<String>()
    } else {
        "    ".to_string()
    };

    // Create two edits:
    // 1. Insert variable declaration before the current line
    // 2. Replace selected expression with variable name
    let declaration = format!("{}{} = {};\n", indent, var_name, selected_text.trim());

    let changes = vec![
        // Insert variable declaration
        TextEdit {
            range: Range {
                start: Position {
                    line: insert_line,
                    character: 0,
                },
                end: Position {
                    line: insert_line,
                    character: 0,
                },
            },
            new_text: declaration,
        },
        // Replace selected expression with variable name
        TextEdit {
            range: *range,
            new_text: var_name.clone(),
        },
    ];

    let workspace_edit = WorkspaceEdit {
        changes: Some({
            let mut change_map = HashMap::new();
            change_map.insert(uri.clone(), changes);
            change_map
        }),
        document_changes: None,
        change_annotations: None,
    };

    Some(CodeAction {
        title: format!("Extract to variable '{}'", var_name),
        kind: Some(CodeActionKind::REFACTOR_EXTRACT),
        diagnostics: None,
        edit: Some(workspace_edit),
        command: None,
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}

fn create_extract_function_action(range: &Range, uri: &Url, content: &str) -> Option<CodeAction> {
    // Extract the selected text
    let lines: Vec<&str> = content.lines().collect();
    let mut selected_text = String::new();

    if range.start.line == range.end.line {
        // Single line selection - only extract if it's a substantial expression
        if let Some(line) = lines.get(range.start.line as usize) {
            let start_char = range.start.character as usize;
            let end_char = (range.end.character as usize).min(line.len());
            if start_char < line.len() && start_char < end_char {
                selected_text = line[start_char..end_char].to_string();
            }
        }
        // For single-line, only suggest if it's a complex expression (contains operators or calls)
        if !selected_text.contains('(')
            && !selected_text.contains('+')
            && !selected_text.contains('-')
            && !selected_text.contains('*')
        {
            return None;
        }
    } else {
        // Multi-line selection
        for line_idx in range.start.line..=range.end.line {
            if let Some(line) = lines.get(line_idx as usize) {
                if line_idx == range.start.line {
                    selected_text.push_str(&line[range.start.character as usize..]);
                } else if line_idx == range.end.line {
                    selected_text.push_str(&line[..range.end.character as usize]);
                } else {
                    selected_text.push_str(line);
                }
                if line_idx < range.end.line {
                    selected_text.push('\n');
                }
            }
        }
    }

    // Skip if selection is empty or just whitespace
    if selected_text.trim().is_empty() {
        return None;
    }

    // Generate function name based on selected code
    let func_name = generate_function_name(&selected_text);

    // Find appropriate indentation
    let base_indent = if let Some(line) = lines.get(range.start.line as usize) {
        line.chars()
            .take_while(|c| c.is_whitespace())
            .collect::<String>()
    } else {
        "".to_string()
    };

    // Create function with proper Zen formatting (name = () type { body })
    let func_body_indent = format!("{}    ", base_indent);
    let formatted_body: Vec<String> = selected_text
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", func_body_indent, line.trim())
            }
        })
        .collect();

    // Return type detection placeholder - could be smarter with AST analysis
    let return_type = "void";

    let new_function = format!(
        "{}{} = () {} {{\n{}\n{}}}\n\n",
        base_indent,
        func_name,
        return_type,
        formatted_body.join("\n"),
        base_indent
    );

    // Find where to insert the new function (before the current function)
    let insert_line = find_function_start(content, range.start.line);

    let mut changes = Vec::new();

    // Insert new function
    changes.push(TextEdit {
        range: Range {
            start: Position {
                line: insert_line,
                character: 0,
            },
            end: Position {
                line: insert_line,
                character: 0,
            },
        },
        new_text: new_function,
    });

    // Replace selected code with function call
    changes.push(TextEdit {
        range: *range,
        new_text: format!("{}()", func_name),
    });

    let workspace_edit = WorkspaceEdit {
        changes: Some({
            let mut change_map = HashMap::new();
            change_map.insert(uri.clone(), changes);
            change_map
        }),
        document_changes: None,
        change_annotations: None,
    };

    Some(CodeAction {
        title: format!("Extract to function '{}'", func_name),
        kind: Some(CodeActionKind::REFACTOR_EXTRACT),
        diagnostics: None,
        edit: Some(workspace_edit),
        command: None,
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}

fn find_function_start(content: &str, from_line: u32) -> u32 {
    // Find the start of the enclosing function by looking backwards
    // Zen uses: name = (params) return_type { }
    let lines: Vec<&str> = content.lines().collect();
    for i in (0..=from_line).rev() {
        if let Some(line) = lines.get(i as usize) {
            let trimmed = line.trim_start();
            // Match Zen function syntax: identifier = (...) type {
            if trimmed.contains(" = (") && trimmed.contains('{') {
                return i;
            }
        }
    }
    // If no function found, insert at the beginning
    0
}

fn generate_function_name(code: &str) -> String {
    // Generate a descriptive function name based on the code content
    let code_trimmed = code.trim();

    // If it contains a method call, use that as a hint
    if let Some(dot_pos) = code_trimmed.find('.') {
        if let Some(paren_pos) = code_trimmed[dot_pos..].find('(') {
            let method_part = &code_trimmed[dot_pos + 1..dot_pos + paren_pos];
            if !method_part.is_empty()
                && method_part.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                return format!("do_{}", method_part);
            }
        }
    }

    // If it contains specific keywords, use them as hints
    if code_trimmed.contains("loop") {
        return "process_loop".to_string();
    }
    if code_trimmed.contains("println") || code_trimmed.contains("print") {
        return "print_output".to_string();
    }
    if code_trimmed.contains("push") {
        return "add_items".to_string();
    }
    if code_trimmed.contains("get") {
        return "get_value".to_string();
    }

    // Default name
    "extracted_fn".to_string()
}

fn generate_variable_name(expression: &str) -> String {
    // Simple heuristic to generate a variable name from an expression
    let expr_trimmed = expression.trim();

    // If it's a method call, use the method name
    if let Some(dot_pos) = expr_trimmed.rfind('.') {
        if let Some(method_end) = expr_trimmed[dot_pos + 1..].find('(') {
            let method_name = &expr_trimmed[dot_pos + 1..dot_pos + 1 + method_end];
            return format!("{}_result", method_name);
        }
    }

    // If it's a function call, use the function name
    if let Some(paren_pos) = expr_trimmed.find('(') {
        let func_name = expr_trimmed[..paren_pos].trim();
        if !func_name.is_empty() && func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return format!("{}_result", func_name);
        }
    }

    // If it's a binary operation, try to infer from operands
    for op in ["==", "!=", "<=", ">=", "<", ">", "+", "-", "*", "/", "%"] {
        if expr_trimmed.contains(op) {
            return "result".to_string();
        }
    }

    // Default fallback
    "extracted_value".to_string()
}

fn create_add_import_action(uri: &Url, content: &str) -> Option<CodeAction> {
    let needs_io =
        content.contains("io.") && !content.contains("{ io }") && !content.contains("{io}");
    let needs_allocator = (content.contains("get_default_allocator")
        || content.contains("GPA")
        || content.contains("AsyncPool"))
        && !content.contains("@std");

    if !needs_io && !needs_allocator {
        return None;
    }

    let import_statement = if needs_io && needs_allocator {
        "{ io, GPA, AsyncPool } = @std\n"
    } else if needs_io {
        "{ io } = @std\n"
    } else {
        "{ GPA, AsyncPool } = @std\n"
    };

    let text_edit = TextEdit {
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
        new_text: import_statement.to_string(),
    };

    let workspace_edit = WorkspaceEdit {
        changes: Some({
            let mut changes = HashMap::new();
            changes.insert(uri.clone(), vec![text_edit]);
            changes
        }),
        document_changes: None,
        change_annotations: None,
    };

    Some(CodeAction {
        title: format!("Add import: {}", import_statement.trim()),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(workspace_edit),
        command: None,
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}
