// Quick fix code actions for diagnostics

use lsp_types::*;
use std::collections::HashMap;

use super::utils::extract_symbol_from_diagnostic;

// ============================================================================
// ALLOCATOR FIX
// ============================================================================

pub fn create_allocator_fix_action(
    diagnostic: &Diagnostic,
    uri: &Url,
    content: &str,
) -> CodeAction {
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

// ============================================================================
// STRING CONVERSION
// ============================================================================

pub fn create_string_conversion_action(
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

// ============================================================================
// ERROR HANDLING
// ============================================================================

pub fn create_error_handling_action(
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

// ============================================================================
// UNUSED VARIABLE QUICK-FIX
// ============================================================================

pub fn create_unused_variable_fix(
    diagnostic: &Diagnostic,
    uri: &Url,
    content: &str,
) -> Option<CodeAction> {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = diagnostic.range.start.line as usize;
    let line = lines.get(line_idx)?;

    let start_char = diagnostic.range.start.character as usize;
    let end_char = (diagnostic.range.end.character as usize).min(line.len());

    // Extract variable name
    let var_name = if start_char < line.len() && start_char < end_char {
        &line[start_char..end_char]
    } else {
        // Try to extract from diagnostic message
        let extracted = extract_symbol_from_diagnostic(&diagnostic.message);
        if extracted.is_empty() {
            return None;
        }
        return create_underscore_prefix_action(diagnostic, uri, &extracted);
    };

    create_underscore_prefix_action(diagnostic, uri, var_name)
}

fn create_underscore_prefix_action(
    diagnostic: &Diagnostic,
    uri: &Url,
    var_name: &str,
) -> Option<CodeAction> {
    // Skip if already prefixed with underscore
    if var_name.starts_with('_') {
        return None;
    }

    let new_name = format!("_{}", var_name);

    let text_edit = TextEdit {
        range: diagnostic.range,
        new_text: new_name.clone(),
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
        title: format!("Prefix with underscore: _{}", var_name),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit: Some(workspace_edit),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}
