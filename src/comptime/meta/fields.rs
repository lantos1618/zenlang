use crate::ast::{AstFields, AstType, Declaration, Expression, FieldValue, Pattern, Statement};
use crate::error::Result;

use super::helpers::{field_info, field_value_to_comptime};
use crate::comptime::values::ComptimeValue;

fn fields_to_comptime(fields: Vec<(&str, FieldValue)>) -> Vec<ComptimeValue> {
    fields
        .into_iter()
        .map(|(name, val)| field_info(name, field_value_to_comptime(val)))
        .collect()
}

pub fn expression_fields(expr: &Expression) -> Result<Vec<ComptimeValue>> {
    Ok(fields_to_comptime(expr.ast_fields()))
}

pub fn statement_fields(stmt: &Statement) -> Result<Vec<ComptimeValue>> {
    Ok(fields_to_comptime(stmt.ast_fields()))
}

pub fn declaration_fields(decl: &Declaration) -> Result<Vec<ComptimeValue>> {
    Ok(fields_to_comptime(decl.ast_fields()))
}

pub fn type_fields(ty: &AstType) -> Result<Vec<ComptimeValue>> {
    Ok(fields_to_comptime(ty.ast_fields()))
}

pub fn pattern_fields(pat: &Pattern) -> Result<Vec<ComptimeValue>> {
    Ok(fields_to_comptime(pat.ast_fields()))
}
