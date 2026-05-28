use crate::ast::{Declaration, Program};
mod dsl;
mod host_effects;
mod target_fields;
mod targets;
mod traversal;

use dsl::BUILD_FUNCTION_NAME;
use traversal::build_graph_input_from_body;

fn unsupported_build_script(message: impl Into<String>) -> BuildGraphError {
    BuildGraphError::UnsupportedBuildScript(message.into())
}

impl BuildGraph {
    pub fn from_build_program(program: &Program) -> Result<Self, BuildGraphError> {
        let build_body = program
            .declarations
            .iter()
            .find_map(|decl| match decl {
                Declaration::Function { name, body, .. } if name == BUILD_FUNCTION_NAME => {
                    Some(body)
                }
                _ => None,
            })
            .ok_or(BuildGraphError::MissingBuildFunction)?;

        Self::from_input(build_graph_input_from_body(build_body)?)
    }
}
