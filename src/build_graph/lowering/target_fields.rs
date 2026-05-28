use crate::ast::Expression;

use super::dsl::{BuildTargetDslKind, BuildTargetField};
use super::{unsupported_build_script, BuildGraphError};

pub(super) fn common_target_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
) -> Result<(Vec<String>, Vec<String>), BuildGraphError> {
    Ok((
        optional_string_array_field(kind, fields, BuildTargetField::Dependencies)?
            .unwrap_or_default(),
        optional_string_array_field(kind, fields, BuildTargetField::Features)?.unwrap_or_default(),
    ))
}

pub(super) fn required_string_field(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
    field: BuildTargetField,
) -> Result<String, BuildGraphError> {
    optional_string_field(kind, fields, field)?.ok_or_else(|| {
        unsupported_build_script(format!("missing required field `{field}` in `{kind}` build target"))
    })
}

pub(super) fn required_one_of_string_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
    options: [BuildTargetField; 2],
) -> Result<String, BuildGraphError> {
    let [first, second] = options;
    if let Some(value) = optional_string_field(kind, fields, first)? {
        return Ok(value);
    }
    if let Some(value) = optional_string_field(kind, fields, second)? {
        return Ok(value);
    }

    Err(unsupported_build_script(format!(
        "missing required field `{first}` or `{second}` in `{kind}` build target"
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
        _ => Err(unsupported_build_script(format!(
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
        unsupported_build_script(format!("missing required field `{field}` in `{kind}` build target"))
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
        return Err(unsupported_build_script(format!(
            "field `{field}` in `{kind}` build target must be an array of strings"
        )));
    };
    let mut values = Vec::with_capacity(elements.len());
    for element in elements {
        let Expression::StringLiteral { value, .. } = element else {
            return Err(unsupported_build_script(format!(
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
