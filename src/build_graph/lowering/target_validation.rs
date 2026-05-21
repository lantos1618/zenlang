use crate::ast::Expression;

use super::dsl::{BuildTargetDslKind, BuildTargetField};
use super::BuildGraphError;

pub(super) fn validate_target_fields(
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
