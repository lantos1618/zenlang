use lsp_server::{Request, Response};
use lsp_types::*;
use std::sync::{Arc, RwLock};

use crate::ast::looks_like_type_name;
use crate::ast::{Declaration, Expression, Statement, VariableDeclarationType};

use super::document_store::DocumentStore;
use super::utils::format_type;
use crate::type_context::TypeContext;

pub fn handle_inlay_hints(req: Request, store: &Arc<RwLock<DocumentStore>>) -> Response {
    let params: InlayHintParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(_) => return empty_response(req.id),
    };

    let store = match store.read() {
        Ok(s) => s,
        Err(_) => return empty_response(req.id),
    };

    let doc = match store.documents.get(&params.text_document.uri) {
        Some(d) => d,
        None => return empty_response(req.id),
    };

    let mut hints = Vec::new();
    let mut seen: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();

    // If we have TypeContext from the typechecker, use it for authoritative hints
    if let Some(type_ctx) = &doc.type_context {
        collect_hints_from_type_context(
            &doc.content,
            type_ctx,
            doc.ast.as_deref(),
            &mut hints,
            &mut seen,
        );
    }
    // When TypeContext is not available, we don't show type hints rather than
    // guessing with fragile string-based heuristics. The TypeContext should be
    // populated by background analysis for any actively edited file.

    crate::lsp::helpers::success_response_id(req.id, hints)
}

fn empty_response(id: lsp_server::RequestId) -> Response {
    crate::lsp::helpers::success_response_id(id, Vec::<InlayHint>::new())
}

/// Collect inlay hints using authoritative type data from the TypeContext.
/// Walks the AST to find variable declarations with inferred types, then
/// looks up their resolved types from the TypeContext.
fn collect_hints_from_type_context(
    content: &str,
    type_ctx: &TypeContext,
    ast: Option<&[Declaration]>,
    hints: &mut Vec<InlayHint>,
    seen: &mut std::collections::HashSet<(u32, u32)>,
) {
    let declarations = match ast {
        Some(decls) => decls,
        None => {
            // Fallback: no AST available, skip hint generation
            return;
        }
    };

    let lines: Vec<&str> = content.lines().collect();

    for decl in declarations {
        match decl {
            Declaration::Function(func) => {
                collect_hints_from_statements(
                    &lines, type_ctx, &func.name, &func.body, hints, seen,
                );
            }
            Declaration::ImplBlock(impl_block) => {
                for method in &impl_block.methods {
                    collect_hints_from_statements(
                        &lines,
                        type_ctx,
                        &method.name,
                        &method.body,
                        hints,
                        seen,
                    );
                }
            }
            _ => {}
        }
    }
}

/// Walk a list of statements (recursing into nested blocks/loops) and emit
/// inlay hints for variable declarations with inferred types.
fn collect_hints_from_statements(
    lines: &[&str],
    type_ctx: &TypeContext,
    func_name: &str,
    statements: &[Statement],
    hints: &mut Vec<InlayHint>,
    seen: &mut std::collections::HashSet<(u32, u32)>,
) {
    for stmt in statements {
        match stmt {
            Statement::VariableDeclaration {
                name,
                type_,
                initializer,
                declaration_type,
                span,
                ..
            } => {
                // Only show hints for inferred types (type_ is None)
                if type_.is_some() {
                    continue;
                }

                // Only inferred declaration types need hints
                let is_mutable =
                    matches!(declaration_type, VariableDeclarationType::InferredMutable);
                let is_inferred = matches!(
                    declaration_type,
                    VariableDeclarationType::InferredImmutable
                        | VariableDeclarationType::InferredMutable
                );
                if !is_inferred {
                    continue;
                }

                // Skip function/closure definitions
                if let Some(init) = initializer {
                    if matches!(init, Expression::Closure { .. }) {
                        continue;
                    }
                }

                // Skip PascalCase names (type/struct names)
                if looks_like_type_name(name) {
                    continue;
                }

                let span = match span {
                    Some(s) => s,
                    None => continue,
                };

                // Look up the resolved type from TypeContext
                let scoped_key = format!("{}::{}", func_name, name);
                let var_type = match type_ctx.variables.get(&scoped_key) {
                    Some(t) => t,
                    None => continue,
                };

                // AST spans are 1-based; LSP positions are 0-based
                let line_num = if span.line > 0 {
                    (span.line - 1) as u32
                } else {
                    0
                };
                let var_col = span.column as u32;

                let line = match lines.get(line_num as usize) {
                    Some(l) => *l,
                    None => continue,
                };

                let type_str = format_type(var_type);

                // Determine hint label and position based on declaration syntax
                let (hint_label, hint_char_pos) = if is_mutable {
                    // ::= syntax: insert type between :: and =
                    if let Some(dce_pos) = line.find("::=") {
                        (format!(" {} ", type_str), (dce_pos + 2) as u32)
                    } else {
                        continue;
                    }
                } else {
                    // Check source line to distinguish := from =
                    // Look for := after the variable name position
                    let after_var = &line[var_col as usize + name.len()..];
                    let trimmed_after = after_var.trim_start();
                    if trimmed_after.starts_with(":=") {
                        // := syntax: insert type between : and =
                        if let Some(colon_eq_pos) = line.find(":=") {
                            (format!(" {} ", type_str), (colon_eq_pos + 1) as u32)
                        } else {
                            continue;
                        }
                    } else {
                        // Plain = syntax: insert `: type` after variable name
                        (format!(": {} ", type_str), var_col + name.len() as u32)
                    }
                };

                let key = (line_num, hint_char_pos);
                if seen.contains(&key) {
                    continue;
                }
                seen.insert(key);

                hints.push(InlayHint {
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
                });
            }
            // Recurse into nested statement blocks
            Statement::Loop { body, .. } => {
                collect_hints_from_statements(lines, type_ctx, func_name, body, hints, seen);
            }
            Statement::Block { statements, .. } => {
                collect_hints_from_statements(lines, type_ctx, func_name, statements, hints, seen);
            }
            Statement::ComptimeBlock { statements, .. } => {
                collect_hints_from_statements(lines, type_ctx, func_name, statements, hints, seen);
            }
            _ => {}
        }
    }
}
