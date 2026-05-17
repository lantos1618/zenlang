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
            executable_target_from_fields(fields)
        }
        Ok(kind @ BuildTargetDslKind::Test) => {
            validate_target_fields(kind, fields)?;
            test_target_from_fields(fields)
        }
        Ok(kind @ BuildTargetDslKind::Library) => {
            validate_target_fields(kind, fields)?;
            library_target_from_fields(fields)
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

fn executable_target_from_fields(fields: &[(String, Expression)]) -> Option<BuildTargetInput> {
    let target_name = string_field(fields, BuildTargetField::Name)?;
    let root_source_file = string_field(fields, BuildTargetField::Main)
        .or_else(|| string_field(fields, BuildTargetField::RootSourceFile))?;
    let out_dir = string_field(fields, BuildTargetField::OutDir)?;
    let dependencies =
        string_array_field(fields, BuildTargetField::Dependencies).unwrap_or_default();
    let features = string_array_field(fields, BuildTargetField::Features).unwrap_or_default();

    Some(BuildTargetInput {
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

fn test_target_from_fields(fields: &[(String, Expression)]) -> Option<BuildTargetInput> {
    let root_source_file = string_field(fields, BuildTargetField::Root)
        .or_else(|| string_field(fields, BuildTargetField::RootSourceFile))?;
    let target_name = string_field(fields, BuildTargetField::Name)
        .unwrap_or_else(|| target_name_from_root(&root_source_file));
    let dependencies =
        string_array_field(fields, BuildTargetField::Dependencies).unwrap_or_default();
    let features = string_array_field(fields, BuildTargetField::Features).unwrap_or_default();

    Some(BuildTargetInput {
        name: target_name,
        kind: BuildTargetKind::Test {
            root_source_file: root_source_file.clone(),
        },
        sources: vec![root_source_file],
        dependencies,
        features,
    })
}

fn library_target_from_fields(fields: &[(String, Expression)]) -> Option<BuildTargetInput> {
    let target_name = string_field(fields, BuildTargetField::Name)?;
    let exports = string_array_field(fields, BuildTargetField::Exports)?;
    let dependencies =
        string_array_field(fields, BuildTargetField::Dependencies).unwrap_or_default();
    let features = string_array_field(fields, BuildTargetField::Features).unwrap_or_default();
    if exports.is_empty() {
        return None;
    }

    Some(BuildTargetInput {
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

fn string_field(fields: &[(String, Expression)], field_name: BuildTargetField) -> Option<String> {
    fields.iter().find_map(|(name, value)| {
        (name == field_name.as_str()).then(|| match value {
            Expression::StringLiteral { value, .. } => Some(value.clone()),
            _ => None,
        })?
    })
}

fn string_array_field(
    fields: &[(String, Expression)],
    field_name: BuildTargetField,
) -> Option<Vec<String>> {
    fields.iter().find_map(|(name, value)| {
        if name != field_name.as_str() {
            return None;
        }
        let Expression::ArrayLiteral { elements, .. } = value else {
            return None;
        };
        elements
            .iter()
            .map(|element| match element {
                Expression::StringLiteral { value, .. } => Some(value.clone()),
                _ => None,
            })
            .collect()
    })
}
