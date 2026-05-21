use std::fmt;

use super::HostEffect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildGraphError {
    EmptyTargetName,
    DuplicateTargetName(String),
    SelfTargetDependency(String),
    CyclicTargetDependency(String),
    UnknownTargetDependency { target: String, dependency: String },
    MissingTargets,
    UndeclaredHostEffect(HostEffect),
    MissingBuildFunction,
    UnsupportedBuildScript(String),
}

impl fmt::Display for BuildGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTargetName => f.write_str("build target name cannot be empty"),
            Self::DuplicateTargetName(name) => {
                write!(f, "duplicate build target name `{name}`")
            }
            Self::SelfTargetDependency(name) => {
                write!(f, "build target `{name}` cannot depend on itself")
            }
            Self::CyclicTargetDependency(name) => {
                write!(f, "build target dependency cycle includes `{name}`")
            }
            Self::UnknownTargetDependency { target, dependency } => {
                write!(
                    f,
                    "build target `{target}` depends on unknown target `{dependency}`"
                )
            }
            Self::MissingTargets => f.write_str("build graph must contain at least one target"),
            Self::UndeclaredHostEffect(effect) => {
                write!(f, "undeclared host effect: {effect}")
            }
            Self::MissingBuildFunction => f.write_str("build.zen is missing `build` function"),
            Self::UnsupportedBuildScript(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for BuildGraphError {}
