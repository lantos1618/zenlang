// Shared helpers for meta AST introspection

use crate::ast::{AstType, Expression, FieldValue, Pattern, Statement};
use std::collections::HashMap;
use std::rc::Rc;

use crate::comptime::values::ComptimeValue;
use crate::comptime::ASTNodeValue;

pub fn field_info(name: &str, value: ComptimeValue) -> ComptimeValue {
    ComptimeValue::Struct {
        name: "FieldInfo".to_string(),
        fields: HashMap::from([
            ("name".to_string(), ComptimeValue::String(name.to_string())),
            ("value".to_string(), value),
        ]),
    }
}

pub fn ast_node(value: ASTNodeValue) -> ComptimeValue {
    ComptimeValue::ASTNode(Rc::new(value))
}

pub fn ast_expr(e: Expression) -> ComptimeValue {
    ast_node(ASTNodeValue::Expression(e))
}

pub fn ast_stmt(s: Statement) -> ComptimeValue {
    ast_node(ASTNodeValue::Statement(s))
}

pub fn ast_type(t: AstType) -> ComptimeValue {
    ast_node(ASTNodeValue::Type(t))
}

pub fn ast_pattern(p: Pattern) -> ComptimeValue {
    ast_node(ASTNodeValue::Pattern(p))
}

/// Convert a `FieldValue` (from the ast layer) into a `ComptimeValue`.
pub fn field_value_to_comptime(fv: FieldValue) -> ComptimeValue {
    match fv {
        FieldValue::I8(v) => ComptimeValue::I8(v),
        FieldValue::I16(v) => ComptimeValue::I16(v),
        FieldValue::I32(v) => ComptimeValue::I32(v),
        FieldValue::I64(v) => ComptimeValue::I64(v),
        FieldValue::U8(v) => ComptimeValue::U8(v),
        FieldValue::U16(v) => ComptimeValue::U16(v),
        FieldValue::U32(v) => ComptimeValue::U32(v),
        FieldValue::U64(v) => ComptimeValue::U64(v),
        FieldValue::F32(v) => ComptimeValue::F32(v),
        FieldValue::F64(v) => ComptimeValue::F64(v),
        FieldValue::Bool(v) => ComptimeValue::Bool(v),
        FieldValue::String(v) => ComptimeValue::String(v),
        FieldValue::Array(arr) => {
            ComptimeValue::Array(arr.into_iter().map(field_value_to_comptime).collect())
        }
        FieldValue::Struct { name, fields } => ComptimeValue::Struct {
            name,
            fields: fields
                .into_iter()
                .map(|(k, v)| (k, field_value_to_comptime(v)))
                .collect(),
        },
        FieldValue::Expr(e) => ast_expr(*e),
        FieldValue::Stmt(s) => ast_stmt(*s),
        FieldValue::Decl(d) => ast_node(ASTNodeValue::Declaration(*d)),
        FieldValue::Type(t) => ast_type(*t),
        FieldValue::Pat(p) => ast_pattern(*p),
        FieldValue::Null => ComptimeValue::Null,
    }
}
