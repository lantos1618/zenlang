use crate::ast::Expression;

use super::dsl::{BuildTargetDslIdent, BuildTargetDslKind, BuildTargetField};
use super::{BuildGraphError, BuildTargetInput, BuildTargetKind};

pub(super) fn build_target_from_builder_add(
    expr: &Expression,
) -> Result<Option<BuildTargetInput>, BuildGraphError> {
    let Expression::MethodCall {
        receiver,
        method,
        args,
        ..
    } = expr
    else {
        return Ok(None);
    };
    if method != BuildTargetDslIdent::Add.as_str()
        || !matches!(
            receiver.as_ref(),
            Expression::Identifier { name, .. } if name == BuildTargetDslIdent::Builder.as_str()
        )
    {
        return Ok(None);
    }
    let [arg] = args.as_slice() else {
        return Ok(None);
    };
    let Expression::StructLiteral { name, fields, .. } = arg else {
        return Ok(None);
    };
    let target = match name.parse::<BuildTargetDslKind>() {
        Ok(kind @ BuildTargetDslKind::Executable) => {
            validate_target_fields(kind, fields)?;
            Some(executable_target_from_fields(kind, fields)?)
        }
        Ok(kind @ BuildTargetDslKind::Test) => {
            validate_target_fields(kind, fields)?;
            Some(test_target_from_fields(kind, fields)?)
        }
        Ok(kind @ BuildTargetDslKind::Library) => {
            validate_target_fields(kind, fields)?;
            Some(library_target_from_fields(kind, fields)?)
        }
        Err(()) => {
            return Err(BuildGraphError::UnsupportedBuildScript(format!(
                "unsupported build target kind `{name}`; supported target kinds are {}",
                BuildTargetDslKind::supported_display_list()
            )));
        }
    };
    Ok(target)
}

fn validate_target_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
) -> Result<(), BuildGraphError> {
    let allowed = allowed_fields(kind);
    let mut seen = std::collections::BTreeSet::new();
    for (name, _) in fields {
        let field = name.parse::<BuildTargetField>().map_err(|()| {
            BuildGraphError::UnsupportedBuildScript(format!(
                "unknown field `{name}` in `{kind}` build target"
            ))
        })?;
        if !allowed.contains(&field) {
            return Err(BuildGraphError::UnsupportedBuildScript(format!(
                "unknown field `{name}` in `{kind}` build target"
            )));
        }
        if !seen.insert(field) {
            return Err(BuildGraphError::UnsupportedBuildScript(format!(
                "duplicate field `{name}` in `{kind}` build target"
            )));
        }
    }
    Ok(())
}

fn allowed_fields(kind: BuildTargetDslKind) -> &'static [BuildTargetField] {
    match kind {
        BuildTargetDslKind::Executable => &[
            BuildTargetField::Name,
            BuildTargetField::Main,
            BuildTargetField::RootSourceFile,
            BuildTargetField::OutDir,
            BuildTargetField::Dependencies,
            BuildTargetField::Features,
            BuildTargetField::Packages,
            BuildTargetField::Link,
        ],
        BuildTargetDslKind::Test => &[
            BuildTargetField::Name,
            BuildTargetField::Root,
            BuildTargetField::RootSourceFile,
            BuildTargetField::Dependencies,
            BuildTargetField::Features,
            BuildTargetField::Packages,
            BuildTargetField::Link,
        ],
        BuildTargetDslKind::Library => &[
            BuildTargetField::Name,
            BuildTargetField::Exports,
            BuildTargetField::Dependencies,
            BuildTargetField::Features,
            BuildTargetField::Packages,
            BuildTargetField::Link,
        ],
    }
}

fn executable_target_from_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
) -> Result<BuildTargetInput, BuildGraphError> {
    let target_name = required_string_field(kind, fields, BuildTargetField::Name)?;
    let root_source_file = required_one_of_string_fields(
        kind,
        fields,
        &[BuildTargetField::Main, BuildTargetField::RootSourceFile],
    )?;
    let out_dir = required_string_field(kind, fields, BuildTargetField::OutDir)?;
    let dependencies =
        optional_string_array_field(kind, fields, BuildTargetField::Dependencies)?
            .unwrap_or_default();
    let features =
        optional_string_array_field(kind, fields, BuildTargetField::Features)?.unwrap_or_default();

    Ok(BuildTargetInput {
        name: target_name,
        kind: BuildTargetKind::Executable {
            root_source_file: root_source_file.clone(),
            out_dir,
        },
        sources: vec![root_source_file],
        dependencies,
        features,
    })
}

fn test_target_from_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
) -> Result<BuildTargetInput, BuildGraphError> {
    let root_source_file = required_one_of_string_fields(
        kind,
        fields,
        &[BuildTargetField::Root, BuildTargetField::RootSourceFile],
    )?;
    let target_name = optional_string_field(kind, fields, BuildTargetField::Name)?
        .unwrap_or_else(|| target_name_from_root(&root_source_file));
    let dependencies =
        optional_string_array_field(kind, fields, BuildTargetField::Dependencies)?
            .unwrap_or_default();
    let features =
        optional_string_array_field(kind, fields, BuildTargetField::Features)?.unwrap_or_default();

    Ok(BuildTargetInput {
        name: target_name,
        kind: BuildTargetKind::Test {
            root_source_file: root_source_file.clone(),
        },
        sources: vec![root_source_file],
        dependencies,
        features,
    })
}

fn library_target_from_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
) -> Result<BuildTargetInput, BuildGraphError> {
    let target_name = required_string_field(kind, fields, BuildTargetField::Name)?;
    let exports = required_string_array_field(kind, fields, BuildTargetField::Exports)?;
    let dependencies =
        optional_string_array_field(kind, fields, BuildTargetField::Dependencies)?
            .unwrap_or_default();
    let features =
        optional_string_array_field(kind, fields, BuildTargetField::Features)?.unwrap_or_default();
    if exports.is_empty() {
        return Err(BuildGraphError::UnsupportedBuildScript(format!(
            "field `{}` in `{kind}` build target must contain at least one source",
            BuildTargetField::Exports
        )));
    }

    Ok(BuildTargetInput {
        name: target_name,
        kind: BuildTargetKind::Library {
            exports: exports.clone(),
        },
        sources: exports,
        dependencies,
        features,
    })
}

fn target_name_from_root(root: &str) -> String {
    std::path::Path::new(root)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("test")
        .to_string()
}

fn required_string_field(
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

fn required_one_of_string_fields(
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

fn optional_string_field(
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

fn required_string_array_field(
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

fn optional_string_array_field(
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
