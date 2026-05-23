use crate::ast::Expression;

use super::dsl::{BuildTargetDslIdent, BuildTargetDslKind, BuildTargetField};
use super::target_fields::{
    common_target_fields, optional_string_field, required_one_of_string_fields,
    required_string_array_field, required_string_field, TargetCommonFields,
};
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
        if field.is_package_link_semantics() {
            return Err(BuildGraphError::UnsupportedBuildScript(format!(
                "unsupported field `{field}` in `{kind}` build target; package/link semantics are gated"
            )));
        }
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
    }
}

fn executable_target_from_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
) -> Result<BuildTargetInput, BuildGraphError> {
    let (root_source_file, common) = single_source_fields(
        kind,
        fields,
        &[BuildTargetField::Main, BuildTargetField::RootSourceFile],
    )?;
    let target_name = required_string_field(kind, fields, BuildTargetField::Name)?;
    let out_dir = required_string_field(kind, fields, BuildTargetField::OutDir)?;

    Ok(single_source_target(
        target_name,
        BuildTargetKind::Executable {
            root_source_file: root_source_file.clone(),
            out_dir,
        },
        root_source_file,
        common,
    ))
}

fn test_target_from_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
) -> Result<BuildTargetInput, BuildGraphError> {
    let (root_source_file, common) = single_source_fields(
        kind,
        fields,
        &[BuildTargetField::Root, BuildTargetField::RootSourceFile],
    )?;
    let target_name = optional_string_field(kind, fields, BuildTargetField::Name)?
        .unwrap_or_else(|| target_name_from_root(&root_source_file));

    Ok(single_source_target(
        target_name,
        BuildTargetKind::Test {
            root_source_file: root_source_file.clone(),
        },
        root_source_file,
        common,
    ))
}

fn single_source_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
    root_fields: &[BuildTargetField],
) -> Result<(String, TargetCommonFields), BuildGraphError> {
    Ok((
        required_one_of_string_fields(kind, fields, root_fields)?,
        common_target_fields(kind, fields)?,
    ))
}

fn library_target_from_fields(
    kind: BuildTargetDslKind,
    fields: &[(String, Expression)],
) -> Result<BuildTargetInput, BuildGraphError> {
    let target_name = required_string_field(kind, fields, BuildTargetField::Name)?;
    let exports = required_string_array_field(kind, fields, BuildTargetField::Exports)?;
    let common = common_target_fields(kind, fields)?;
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
        dependencies: common.dependencies,
        features: common.features,
    })
}

fn single_source_target(
    name: String,
    kind: BuildTargetKind,
    root_source_file: String,
    common: TargetCommonFields,
) -> BuildTargetInput {
    BuildTargetInput {
        name,
        kind,
        sources: vec![root_source_file],
        dependencies: common.dependencies,
        features: common.features,
    }
}

fn target_name_from_root(root: &str) -> String {
    std::path::Path::new(root)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("test")
        .to_string()
}
