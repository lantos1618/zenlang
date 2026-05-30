use crate::ast::Expression;

use super::dsl::{
    BuildTargetDslKind, BuildTargetField, BUILDER_ADD_METHOD, BUILDER_IDENT,
    SUPPORTED_TARGET_KINDS,
};
use super::target_fields::{
    common_target_fields, optional_link_array_field, optional_string_array_field, optional_string_field,
    required_one_of_string_fields, required_string_array_field, required_string_field,
};
use super::{unsupported_build_script, BuildGraphError, BuildTargetInput, BuildTargetKind};

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
    if method != BUILDER_ADD_METHOD
        || !matches!(
            receiver.as_ref(),
            Expression::Identifier { name, .. } if name == BUILDER_IDENT
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
    let kind = name.parse::<BuildTargetDslKind>().map_err(|()| {
        unsupported_build_script(format!(
            "unsupported build target kind `{name}`; supported target kinds are {}",
            SUPPORTED_TARGET_KINDS
        ))
    })?;
    validate_target_fields(kind, fields)?;
    Ok(Some(match kind {
        BuildTargetDslKind::Executable => executable_target_from_fields(kind, fields)?,
        BuildTargetDslKind::Test => test_target_from_fields(kind, fields)?,
        BuildTargetDslKind::Library => library_target_from_fields(kind, fields)?,
    }))
}

fn validate_target_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
) -> Result<(), BuildGraphError> {
    let allowed: &[BuildTargetField] = match kind {
        BuildTargetDslKind::Executable => &[
            BuildTargetField::Name,
            BuildTargetField::Main,
            BuildTargetField::RootSourceFile,
            BuildTargetField::OutDir,
            BuildTargetField::Dependencies,
            BuildTargetField::Features,
            BuildTargetField::Link,
            BuildTargetField::Headers,
        ],
        BuildTargetDslKind::Test => &[
            BuildTargetField::Name,
            BuildTargetField::Root,
            BuildTargetField::RootSourceFile,
            BuildTargetField::Dependencies,
            BuildTargetField::Features,
        ],
        BuildTargetDslKind::Library => &[
            BuildTargetField::Name,
            BuildTargetField::Exports,
            BuildTargetField::Dependencies,
            BuildTargetField::Features,
        ],
    };
    let mut seen = std::collections::BTreeSet::new();
    for (name, _) in fields {
        let field = name.parse::<BuildTargetField>().map_err(|()| {
            unsupported_build_script(format!("unknown field `{name}` in `{kind}` build target"))
        })?;
        // `link` is supported on Executable targets (see the allow-list below);
        // `packages` remains gated until the package driver exists.
        if matches!(field, BuildTargetField::Packages) {
            return Err(unsupported_build_script(format!(
                "unsupported field `{field}` in `{kind}` build target; package semantics are gated"
            )));
        }
        if !allowed.contains(&field) {
            return Err(unsupported_build_script(format!(
                "unknown field `{name}` in `{kind}` build target"
            )));
        }
        if !seen.insert(field) {
            return Err(unsupported_build_script(format!(
                "duplicate field `{name}` in `{kind}` build target"
            )));
        }
    }
    Ok(())
}

fn executable_target_from_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
) -> Result<BuildTargetInput, BuildGraphError> {
    let root_source_file = required_one_of_string_fields(
        kind,
        fields,
        [BuildTargetField::Main, BuildTargetField::RootSourceFile],
    )?;
    let (dependencies, features) = common_target_fields(kind, fields)?;
    let target_name = required_string_field(kind, fields, BuildTargetField::Name)?;
    let out_dir = required_string_field(kind, fields, BuildTargetField::OutDir)?;
    let link =
        optional_link_array_field(kind, fields, BuildTargetField::Link)?.unwrap_or_default();
    let headers =
        optional_string_array_field(kind, fields, BuildTargetField::Headers)?.unwrap_or_default();

    Ok(BuildTargetInput {
        name: target_name,
        kind: BuildTargetKind::Executable {
            root_source_file: root_source_file.clone(),
            out_dir,
            link,
            headers,
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
        [BuildTargetField::Root, BuildTargetField::RootSourceFile],
    )?;
    let (dependencies, features) = common_target_fields(kind, fields)?;
    let target_name = optional_string_field(kind, fields, BuildTargetField::Name)?
        .unwrap_or_else(|| {
            std::path::Path::new(&root_source_file)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .filter(|stem| !stem.is_empty())
                .unwrap_or("test")
                .to_string()
        });

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
    let (dependencies, features) = common_target_fields(kind, fields)?;
    if exports.is_empty() {
        return Err(unsupported_build_script(format!(
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
