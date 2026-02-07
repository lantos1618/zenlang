// Meta-programming introspection for Zen
// Enables Zen programs to walk and inspect AST nodes at compile time.
//
// Split into components:
//   helpers.rs  - Shared builder functions (field_info, ast_expr, etc.)
//   fields.rs   - AST field extraction (expression_fields, statement_fields, etc.)
//   variants.rs - Variant name constants (meta.Expression.BinaryOp, etc.)

use crate::error::Result;
use std::collections::HashMap;

use super::values::{ASTNodeValue, ComptimeValue};

mod fields;
pub mod helpers;
mod variants;

// Re-export variant constructors for use in init_builtins
pub use variants::{
    declaration_variants, expression_variants, pattern_variants, statement_variants, type_variants,
};

// ---------------------------------------------------------------------------
// meta.variant_name(node) -> String
// ---------------------------------------------------------------------------

pub fn variant_name(node: &ASTNodeValue) -> String {
    match node {
        ASTNodeValue::Expression(expr) => expr.variant_name().to_string(),
        ASTNodeValue::Statement(stmt) => stmt.variant_name().to_string(),
        ASTNodeValue::Declaration(decl) => decl.variant_name().to_string(),
        ASTNodeValue::Type(ty) => ty.variant_name().to_string(),
        ASTNodeValue::Pattern(pat) => pat.variant_name().to_string(),
        ASTNodeValue::Program(_) => "Program".to_string(),
    }
}

// ---------------------------------------------------------------------------
// meta.fields(node) -> []FieldInfo
// ---------------------------------------------------------------------------

pub fn fields(node: &ASTNodeValue) -> Result<Vec<ComptimeValue>> {
    match node {
        ASTNodeValue::Expression(expr) => fields::expression_fields(expr),
        ASTNodeValue::Statement(stmt) => fields::statement_fields(stmt),
        ASTNodeValue::Declaration(decl) => fields::declaration_fields(decl),
        ASTNodeValue::Type(ty) => fields::type_fields(ty),
        ASTNodeValue::Pattern(pat) => fields::pattern_fields(pat),
        ASTNodeValue::Program(prog) => Ok(vec![
            helpers::field_info(
                "declarations",
                ComptimeValue::Array(
                    prog.declarations
                        .iter()
                        .map(|d| helpers::ast_node(ASTNodeValue::Declaration(d.clone())))
                        .collect(),
                ),
            ),
            helpers::field_info(
                "statements",
                ComptimeValue::Array(
                    prog.statements
                        .iter()
                        .map(|s| helpers::ast_node(ASTNodeValue::Statement(s.clone())))
                        .collect(),
                ),
            ),
        ]),
    }
}

// ---------------------------------------------------------------------------
// meta.type_info(node) -> TypeInfo
// ---------------------------------------------------------------------------

pub fn type_info(node: &ASTNodeValue) -> Result<ComptimeValue> {
    let vname = variant_name(node);
    let flds = fields(node)?;

    Ok(ComptimeValue::Struct {
        name: "TypeInfo".to_string(),
        fields: HashMap::from([
            ("variant".to_string(), ComptimeValue::String(vname)),
            ("fields".to_string(), ComptimeValue::Array(flds)),
            (
                "kind".to_string(),
                ComptimeValue::String(
                    match node {
                        ASTNodeValue::Expression(_) => "Expression",
                        ASTNodeValue::Statement(_) => "Statement",
                        ASTNodeValue::Declaration(_) => "Declaration",
                        ASTNodeValue::Type(_) => "Type",
                        ASTNodeValue::Pattern(_) => "Pattern",
                        ASTNodeValue::Program(_) => "Program",
                    }
                    .to_string(),
                ),
            ),
        ]),
    })
}

// ---------------------------------------------------------------------------
// meta.children(node) -> []ASTNode
// ---------------------------------------------------------------------------

pub fn children(node: &ASTNodeValue) -> Result<Vec<ComptimeValue>> {
    let flds = fields(node)?;
    let mut result = Vec::new();

    for f in &flds {
        if let ComptimeValue::Struct { fields, .. } = f {
            if let Some(value) = fields.get("value") {
                collect_ast_nodes(value, &mut result);
            }
        }
    }

    Ok(result)
}

fn collect_ast_nodes(value: &ComptimeValue, out: &mut Vec<ComptimeValue>) {
    match value {
        ComptimeValue::ASTNode(_) => out.push(value.clone()),
        ComptimeValue::Array(items) => {
            for item in items {
                collect_ast_nodes(item, out);
            }
        }
        ComptimeValue::Struct { fields, .. } => {
            for v in fields.values() {
                collect_ast_nodes(v, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
