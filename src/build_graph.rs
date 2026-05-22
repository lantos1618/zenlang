use std::collections::BTreeSet;
use std::fmt;

use serde::Serialize;

mod dependency_order;
mod graph;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildGraphInput {
    pub targets: Vec<BuildTargetInput>,
    pub declared_host_effects: Vec<HostEffect>,
    pub used_host_effects: Vec<HostEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildTargetInput {
    pub name: String,
    pub kind: BuildTargetKind,
    pub sources: Vec<String>,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BuildTargetKind {
    Executable {
        root_source_file: String,
        out_dir: String,
    },
    Test {
        root_source_file: String,
    },
    Library {
        exports: Vec<String>,
    },
}

impl BuildTargetKind {
    pub fn diagnostic_name(&self) -> &'static str {
        match self {
            Self::Executable { .. } => "executable",
            Self::Test { .. } => "test",
            Self::Library { .. } => "library",
        }
    }
}

impl fmt::Display for BuildTargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.diagnostic_name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HostEffect {
    ReadEnv(String),
    ReadFile(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildGraph {
    targets: Vec<BuildTarget>,
    declared_host_effects: Vec<HostEffect>,
    used_host_effects: Vec<HostEffect>,
}

#[derive(Serialize)]
struct BuildGraphJson<'a> {
    format: &'static str,
    schema_version: u32,
    semantic_status: &'static str,
    targets: &'a [BuildTarget],
    declared_host_effects: &'a [HostEffect],
    used_host_effects: &'a [HostEffect],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildTarget {
    name: String,
    kind: BuildTargetKind,
    sources: Vec<String>,
    dependencies: Vec<String>,
    features: Vec<String>,
}

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

impl BuildGraph {
    pub fn targets(&self) -> &[BuildTarget] {
        &self.targets
    }
}

impl BuildTarget {
    pub fn is_executable(&self) -> bool {
        matches!(self.kind, BuildTargetKind::Executable { .. })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &BuildTargetKind {
        &self.kind
    }

    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    pub fn features(&self) -> &[String] {
        &self.features
    }
}

impl fmt::Display for HostEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadEnv(name) => write!(f, "read env `{name}`"),
            Self::ReadFile(path) => write!(f, "read file `{path}`"),
        }
    }
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

fn sorted_unique<T>(values: Vec<T>) -> Vec<T>
where
    T: Ord,
{
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

include!("build_graph/lowering.rs");

#[cfg(test)]
#[path = "build_graph/lowering_tests.rs"]
mod lowering_tests;
