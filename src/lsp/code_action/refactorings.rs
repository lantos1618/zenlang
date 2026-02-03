// Refactoring code actions (extract variable, extract function, etc.)

use lsp_types::*;
use std::collections::HashMap;

// ============================================================================
// EXTRACT VARIABLE
// ============================================================================

pub fn create_extract_variable_action(
    range: &Range,
    uri: &Url,
    content: &str,
) -> Option<CodeAction> {
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

// ============================================================================
// EXTRACT FUNCTION
// ============================================================================

pub fn create_extract_function_action(
    range: &Range,
    uri: &Url,
    content: &str,
) -> Option<CodeAction> {
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
