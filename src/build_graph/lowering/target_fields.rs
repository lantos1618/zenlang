use crate::ast::Expression;

use super::dsl::{BuildTargetDslKind, BuildTargetField};
use super::BuildGraphError;

pub(super) fn required_string_field(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
    field: BuildTargetField,
) -> Result<String, BuildGraphError> {
    optional_string_field(kind, fields, field)?.ok_or_else(|| {
        BuildGraphError::UnsupportedBuildScript(format!(
            "missing required field `{field}` in `{kind}` build target"
        ))
    })
}

pub(super) fn required_one_of_string_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
    options: &[BuildTargetField],
) -> Result<String, BuildGraphError> {
    for field in options {
        if let Some(value) = optional_string_field(kind, fields, *field)? {
            return Ok(value);
        }
    }
    let names = options
        .iter()
        .map(|field| format!("`{field}`"))
        .collect::<Vec<_>>();
    let Some((last, rest)) = names.split_last() else {
        return Err(BuildGraphError::UnsupportedBuildScript(format!(
            "missing required source field in `{kind}` build target"
        )));
    };
    let display = if rest.is_empty() {
        last.clone()
    } else {
        format!("{} or {last}", rest.join(", "))
    };
    Err(BuildGraphError::UnsupportedBuildScript(format!(
        "missing required field {display} in `{kind}` build target"
    )))
}

pub(super) fn optional_string_field(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
    field: BuildTargetField,
) -> Result<Option<String>, BuildGraphError> {
    let Some(value) = field_value(fields, field) else {
        return Ok(None);
    };
    match value {
        Expression::StringLiteral { value, .. } => Ok(Some(value.clone())),
        _ => Err(BuildGraphError::UnsupportedBuildScript(format!(
            "field `{field}` in `{kind}` build target must be a string"
        ))),
    }
}

pub(super) fn required_string_array_field(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
    field: BuildTargetField,
) -> Result<Vec<String>, BuildGraphError> {
    optional_string_array_field(kind, fields, field)?.ok_or_else(|| {
        BuildGraphError::UnsupportedBuildScript(format!(
            "missing required field `{field}` in `{kind}` build target"
        ))
    })
}

pub(super) fn optional_string_array_field(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
    field: BuildTargetField,
) -> Result<Option<Vec<String>>, BuildGraphError> {
    let Some(value) = field_value(fields, field) else {
        return Ok(None);
    };
    let Expression::ArrayLiteral { elements, .. } = value else {
        return Err(BuildGraphError::UnsupportedBuildScript(format!(
            "field `{field}` in `{kind}` build target must be an array of strings"
        )));
    };
    let mut values = Vec::with_capacity(elements.len());
    for element in elements {
        let Expression::StringLiteral { value, .. } = element else {
            return Err(BuildGraphError::UnsupportedBuildScript(format!(
                "field `{field}` in `{kind}` build target must be an array of strings"
            )));
        };
        values.push(value.clone());
    }
    Ok(Some(values))
}

fn field_value(fields: &[(String, Expression)], field: BuildTargetField) -> Option<&Expression> {
    fields
        .iter()
        .find_map(|(name, value)| (name == field.as_str()).then_some(value))
}
