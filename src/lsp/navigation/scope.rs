// Scope-related helper functions for references

use super::utils::find_function_range_from_doc;
use crate::ast::{Function, Statement};
use crate::lsp::types::{Document, SymbolScope};
use lsp_types::*;

/// Determine the scope of a symbol (local, module-level, or unknown)
pub fn determine_symbol_scope(
    doc: &Document,
    symbol_name: &str,
    position: Position,
) -> SymbolScope {
    if let Some(ast) = &doc.ast {
        for decl in ast {
            if let crate::ast::Declaration::Function(func) = decl {
                if let Some(func_range) = find_function_range_from_doc(doc, &func.name) {
                    if position.line >= func_range.start.line
                        && position.line <= func_range.end.line
                        && is_local_symbol_in_function(func, symbol_name)
                    {
                        return SymbolScope::Local {
                            function_name: func.name.clone(),
                        };
                    }
                }
            }
        }
    }

    if doc.symbols.contains_key(symbol_name) {
        return SymbolScope::ModuleLevel;
    }

    SymbolScope::Unknown
}

/// Check if a symbol is local to a function (parameter or local variable)
fn is_local_symbol_in_function(func: &Function, symbol_name: &str) -> bool {
    for (param_name, _param_type) in &func.args {
        if param_name == symbol_name {
            return true;
        }
    }
    is_symbol_in_statements(&func.body, symbol_name)
}

/// Check if a symbol is declared in the given statements
fn is_symbol_in_statements(statements: &[Statement], symbol_name: &str) -> bool {
    for stmt in statements {
        match stmt {
            Statement::VariableDeclaration { name, .. } if name == symbol_name => {
                return true;
            }
            Statement::Loop { body, .. } => {
                if is_symbol_in_statements(body, symbol_name) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}
