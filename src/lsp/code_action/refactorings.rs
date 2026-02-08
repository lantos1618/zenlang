// Refactoring code actions (extract variable, extract function, etc.)

use lsp_types::*;
use std::collections::HashMap;

use super::utils::utf16_offset_to_byte_offset;
use crate::ast::Declaration;

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
            let start_byte = utf16_offset_to_byte_offset(line, range.start.character as usize);
            let end_byte = utf16_offset_to_byte_offset(line, range.end.character as usize);
            if start_byte < line.len() && start_byte < end_byte && end_byte <= line.len() {
                selected_text = line[start_byte..end_byte].to_string();
            }
        }
    } else {
        // Multi-line selection
        for line_idx in range.start.line..=range.end.line {
            if let Some(line) = lines.get(line_idx as usize) {
                if line_idx == range.start.line {
                    let start_byte =
                        utf16_offset_to_byte_offset(line, range.start.character as usize);
                    if start_byte <= line.len() {
                        selected_text.push_str(&line[start_byte..]);
                    }
                } else if line_idx == range.end.line {
                    let end_byte = utf16_offset_to_byte_offset(line, range.end.character as usize);
                    if end_byte <= line.len() {
                        selected_text.push_str(&line[..end_byte]);
                    }
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
    ast: Option<&Vec<Declaration>>,
) -> Option<CodeAction> {
    // Extract the selected text
    let lines: Vec<&str> = content.lines().collect();
    let mut selected_text = String::new();

    if range.start.line == range.end.line {
        // Single line selection - only extract if it's a substantial expression
        if let Some(line) = lines.get(range.start.line as usize) {
            let start_byte = utf16_offset_to_byte_offset(line, range.start.character as usize);
            let end_byte = utf16_offset_to_byte_offset(line, range.end.character as usize);
            if start_byte < line.len() && start_byte < end_byte && end_byte <= line.len() {
                selected_text = line[start_byte..end_byte].to_string();
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
                    let start_byte =
                        utf16_offset_to_byte_offset(line, range.start.character as usize);
                    if start_byte <= line.len() {
                        selected_text.push_str(&line[start_byte..]);
                    }
                } else if line_idx == range.end.line {
                    let end_byte = utf16_offset_to_byte_offset(line, range.end.character as usize);
                    if end_byte <= line.len() {
                        selected_text.push_str(&line[..end_byte]);
                    }
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
    let insert_line = find_function_start(ast, range.start.line);

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

/// Find the start line of the enclosing function using AST Declaration::Function spans.
/// Falls back to line 0 if AST is unavailable or no enclosing function is found.
fn find_function_start(ast: Option<&Vec<Declaration>>, from_line: u32) -> u32 {
    if let Some(declarations) = ast {
        let mut best_line: Option<u32> = None;
        for decl in declarations {
            if let Declaration::Function(f) = decl {
                // Find the line range of this function from its body statement spans.
                let (func_start, func_end) = function_body_line_range(&f.body);
                if let (Some(start), Some(end)) = (func_start, func_end) {
                    // Allow margin before the first body statement for the function header
                    let header_start = start.saturating_sub(2);
                    if from_line >= header_start && from_line <= end + 1 {
                        best_line =
                            Some(best_line.map_or(header_start, |b: u32| b.max(header_start)));
                    }
                }
            }
        }
        if let Some(line) = best_line {
            return line;
        }
    }
    0
}

/// Extract the line range (min, max) of statements in a function body using their spans.
fn function_body_line_range(body: &[crate::ast::Statement]) -> (Option<u32>, Option<u32>) {
    use crate::ast::Statement;
    let mut min_line = None;
    let mut max_line = None;
    for stmt in body {
        let span = match stmt {
            Statement::Expression { span, .. }
            | Statement::Return { span, .. }
            | Statement::VariableDeclaration { span, .. }
            | Statement::VariableAssignment { span, .. }
            | Statement::PointerAssignment { span, .. }
            | Statement::Loop { span, .. }
            | Statement::Break { span, .. }
            | Statement::Continue { span, .. }
            | Statement::ComptimeBlock { span, .. }
            | Statement::Defer { span, .. }
            | Statement::ThisDefer { span, .. }
            | Statement::DestructuringImport { span, .. }
            | Statement::Block { span, .. } => span.as_ref(),
            Statement::ModuleImport { .. } => None,
        };
        if let Some(s) = span {
            let line = s.line as u32;
            min_line = Some(min_line.map_or(line, |m: u32| m.min(line)));
            max_line = Some(max_line.map_or(line, |m: u32| m.max(line)));
        }
    }
    (min_line, max_line)
}

fn generate_function_name(code: &str) -> String {
    // Generate a descriptive function name from the selected code text.
    // This is text manipulation on a user-selected snippet, not structural parsing.
    let code_trimmed = code.trim();

    // If the code is primarily a method call like `obj.method(args)`, use the method name
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

    // Default name — avoid guessing from keywords like "loop", "get", "push"
    // which produces misleading names for unrelated code
    "extracted_fn".to_string()
}
