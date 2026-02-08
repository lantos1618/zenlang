//! Semantic completion using TypeContext
//!
//! This module provides intelligent completion by querying the TypeContext
//! populated during document analysis. Falls back to text-based heuristics
//! when TypeContext is unavailable.

use crate::ast::{AstType, Expression};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::type_context::TypeContext;
use lsp_types::*;

use super::document_store::DocumentStore;
use super::types::Document;
use super::utils::format_type;

/// Get completions for a dot expression using semantic analysis.
/// Returns None if TypeContext isn't available, allowing fallback to heuristics.
pub fn get_semantic_dot_completions(
    doc: &Document,
    receiver_type: &AstType,
    store: &DocumentStore,
) -> Option<Vec<CompletionItem>> {
    let type_ctx = doc.type_context.as_ref()?;

    let mut completions = Vec::new();

    // Get the base type name for lookups
    let type_name = get_type_name(receiver_type);

    // 1. Add struct fields if receiver is a struct
    if let Some(fields) = type_ctx.get_struct_fields(&type_name) {
        for (field_name, field_type) in fields {
            completions.push(CompletionItem {
                label: field_name.clone(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(format!("{}: {}", field_name, format_type(field_type))),
                documentation: Some(Documentation::String(format!("Field of `{}`", type_name))),
                sort_text: Some(format!("0{}", field_name)), // Fields first
                ..Default::default()
            });
        }
    }

    // 2. Add methods from TypeContext
    add_methods_from_context(&type_name, type_ctx, receiver_type, &mut completions);

    // 3. Add methods from stdlib_types (for stdlib types like Vec, String)
    add_methods_from_stdlib(&type_name, store, &mut completions);

    // 4. Add UFC functions (free functions where first param matches receiver type)
    add_ufc_methods(&type_name, type_ctx, &mut completions);

    Some(completions)
}

/// Get the base type name from an AstType
fn get_type_name(ty: &AstType) -> String {
    // Use centralized base_name() method, fallback to Display
    ty.base_name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}", ty))
}

/// Add methods from TypeContext (registered during type checking)
fn add_methods_from_context(
    type_name: &str,
    type_ctx: &TypeContext,
    receiver_type: &AstType,
    completions: &mut Vec<CompletionItem>,
) {
    // Look for methods registered as "TypeName.methodName"
    let prefix = format!("{}.", type_name);

    for (key, return_type) in &type_ctx.methods {
        if key.starts_with(&prefix) {
            let method_name = &key[prefix.len()..];

            // Get parameters if available
            let params_str = if let Some(params) = type_ctx.method_params.get(key) {
                params
                    .iter()
                    .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                "...".to_string()
            };

            // Specialize generic return types if possible
            let specialized_return = specialize_generic_type(return_type, receiver_type);

            completions.push(CompletionItem {
                label: method_name.to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some(format!(
                    "({}) -> {}",
                    params_str,
                    format_type(&specialized_return)
                )),
                documentation: Some(Documentation::String(format!("Method of `{}`", type_name))),
                sort_text: Some(format!("1{}", method_name)), // Methods after fields
                insert_text: Some(format!("{}($0)", method_name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
    }
}

/// Add methods from stdlib_types registry
fn add_methods_from_stdlib(
    type_name: &str,
    store: &DocumentStore,
    completions: &mut Vec<CompletionItem>,
) {
    // Look for stdlib methods in the symbol table
    let prefix = format!("{}.", type_name);

    for (key, symbol) in &store.stdlib_symbols {
        if key.starts_with(&prefix) {
            let method_name = &key[prefix.len()..];

            // Skip if already added
            if completions.iter().any(|c| c.label == method_name) {
                continue;
            }

            completions.push(CompletionItem {
                label: method_name.to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: symbol.detail.clone(),
                documentation: symbol.documentation.clone().map(Documentation::String),
                sort_text: Some(format!("2{}", method_name)), // Stdlib methods last
                insert_text: Some(format!("{}($0)", method_name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
    }
}

/// Add UFC (Uniform Function Call) methods - free functions whose first param matches the type
fn add_ufc_methods(type_name: &str, type_ctx: &TypeContext, completions: &mut Vec<CompletionItem>) {
    for (func_name, func_type) in &type_ctx.functions {
        // Skip if no parameters
        if func_type.params.is_empty() {
            continue;
        }

        // Check if first parameter type matches the receiver
        let (first_param_name, first_param_type) = &func_type.params[0];
        let first_param_type_name = get_type_name(first_param_type);

        if first_param_type_name == type_name || is_self_param(first_param_name) {
            // Skip if already added as a method
            if completions.iter().any(|c| c.label == *func_name) {
                continue;
            }

            // Format remaining parameters (skip self)
            let other_params: Vec<String> = func_type
                .params
                .iter()
                .skip(1)
                .map(|(name, ty)| format!("{}: {}", name, format_type(ty)))
                .collect();

            completions.push(CompletionItem {
                label: func_name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!(
                    "({}) -> {}  [UFC]",
                    other_params.join(", "),
                    format_type(&func_type.return_type)
                )),
                documentation: Some(Documentation::String(
                    "Uniform Function Call - free function callable as method".to_string(),
                )),
                sort_text: Some(format!("3{}", func_name)), // UFC methods lowest priority
                insert_text: Some(format!("{}($0)", func_name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
    }
}

/// Check if a parameter name indicates self
fn is_self_param(name: &str) -> bool {
    matches!(name, "self" | "this")
}

/// Specialize generic type parameters based on concrete receiver type.
/// E.g., if receiver is Vec<User> and return type is Option<T>, return Option<User>
fn specialize_generic_type(return_type: &AstType, receiver_type: &AstType) -> AstType {
    // Extract type args from receiver
    let type_args = match receiver_type {
        AstType::Generic { type_args, .. } => type_args,
        _ => return return_type.clone(),
    };

    if type_args.is_empty() {
        return return_type.clone();
    }

    // Simple substitution: replace T with first type arg, E with second, etc.
    substitute_type_params(return_type, type_args)
}

/// Substitute type parameters in a type
fn substitute_type_params(ty: &AstType, substitutions: &[AstType]) -> AstType {
    match ty {
        AstType::Generic { name, type_args } => {
            // If it's a single-letter generic like "T", substitute
            if name.len() == 1 {
                if let Some(first_char) = name.chars().next() {
                    if first_char.is_uppercase() && first_char >= 'T' {
                        let index = (first_char as usize) - ('T' as usize);
                        if index < substitutions.len() {
                            return substitutions[index].clone();
                        }
                    }
                }
            }

            // Recursively substitute in type args
            let new_args: Vec<AstType> = type_args
                .iter()
                .map(|arg| substitute_type_params(arg, substitutions))
                .collect();

            AstType::Generic {
                name: name.clone(),
                type_args: new_args,
            }
        }
        _ => ty.clone(),
    }
}

/// Resolve receiver expression to AstType using TypeContext.
/// This is the semantic alternative to text-based type inference.
pub fn resolve_receiver_type(
    receiver_expr: &str,
    doc: &Document,
    _position: Position,
) -> Option<AstType> {
    let type_ctx = doc.type_context.as_ref()?;
    let receiver = receiver_expr.trim();

    // Try to parse the receiver expression using the parser
    let lexer = Lexer::new(receiver);
    let mut parser = Parser::new(lexer);
    if let Ok(expr) = parser.parse_expression() {
        if let Some(resolved) = resolve_type_from_parsed_expr(&expr, type_ctx, doc) {
            return Some(resolved);
        }
    }

    // Fallback: check if it's a known variable in any scope
    // TypeContext stores variables as "scope::varname"
    for (key, var_type) in &type_ctx.variables {
        if key.ends_with(&format!("::{}", receiver)) {
            return Some(var_type.clone());
        }
    }

    // Check local symbols for type info
    if let Some(symbol) = doc.symbols.get(receiver) {
        if let Some(type_info) = &symbol.type_info {
            return Some(type_info.clone());
        }
    }

    None
}

/// Resolve type from a parsed AST expression using TypeContext
fn resolve_type_from_parsed_expr(
    expr: &Expression,
    type_ctx: &TypeContext,
    doc: &Document,
) -> Option<AstType> {
    match expr {
        // Simple identifier - look up in variables
        Expression::Identifier(name) => {
            // Check TypeContext variables (stored as "scope::varname")
            for (key, var_type) in &type_ctx.variables {
                if key.ends_with(&format!("::{}", name)) || key == name {
                    return Some(var_type.clone());
                }
            }
            // Check document symbols
            if let Some(symbol) = doc.symbols.get(name) {
                return symbol.type_info.clone();
            }
            None
        }

        // Function call - get return type
        Expression::FunctionCall { name, .. } => type_ctx.get_function_return_type(name),

        // Method call - resolve receiver type, then get method return type
        Expression::MethodCall { object, method, .. } => {
            if let Some(receiver_type) = resolve_type_from_parsed_expr(object, type_ctx, doc) {
                let type_name = get_type_name(&receiver_type);
                // Check for constructor methods first
                if let Some(ret) = type_ctx.get_constructor_type(&type_name, method) {
                    return Some(ret);
                }
                // Check regular methods
                if let Some(ret) = type_ctx.get_method_return_type(&type_name, method) {
                    return Some(ret);
                }
            }
            None
        }

        // Struct literal - the type is the struct name
        Expression::StructLiteral { name, .. } => Some(AstType::Generic {
            name: name.clone(),
            type_args: vec![],
        }),

        // Field access - resolve base type, then look up field type
        Expression::StructField { struct_, field } => {
            if let Some(receiver_type) = resolve_type_from_parsed_expr(struct_, type_ctx, doc) {
                let type_name = get_type_name(&receiver_type);
                if let Some(fields) = type_ctx.get_struct_fields(&type_name) {
                    for (fname, ftype) in fields {
                        if *fname == *field {
                            return Some(ftype.clone());
                        }
                    }
                }
            }
            None
        }

        // Literals
        Expression::Integer32(_) | Expression::Integer64(_) => Some(AstType::I32),
        Expression::Float32(_) | Expression::Float64(_) => Some(AstType::F64),
        Expression::String(_) => Some(AstType::StaticString),
        Expression::Boolean(_) => Some(AstType::Bool),

        _ => None,
    }
}
