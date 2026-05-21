use crate::ast::{Declaration, Program};

#[path = "lowering/dsl.rs"]
mod dsl;
#[path = "lowering/host_effects.rs"]
mod host_effects;
#[path = "lowering/target_fields.rs"]
mod target_fields;
#[path = "lowering/targets.rs"]
mod targets;
#[path = "lowering/traversal.rs"]
mod traversal;

use dsl::{BuildTargetDslIdent, HostEffectResultVariant};
#[cfg(test)]
use dsl::{BuildTargetDslKind, BuildTargetField};
use traversal::{BuildProgramLowering, BuildTargetAddContext};

impl BuildGraph {
    pub fn from_build_program(program: &Program) -> Result<Self, BuildGraphError> {
        let build_body = program
            .declarations
            .iter()
            .find_map(|decl| match decl {
                Declaration::Function { name, body, .. }
                    if name == BuildTargetDslIdent::Build.as_str() =>
                {
                    Some(body)
                }
                _ => None,
            })
            .ok_or(BuildGraphError::MissingBuildFunction)?;

        let mut lowering = BuildProgramLowering::default();
        lowering.collect_expr(build_body, BuildTargetAddContext::StaticGraphBody);
        Self::from_input(lowering.into_input()?)
    }
}
