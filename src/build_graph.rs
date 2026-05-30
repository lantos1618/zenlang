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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        link: Vec<String>,
        /// C headers to force-include — `#include`d into the generated C so the
        /// emitted `@extern` prototypes are checked against the real library
        /// declarations (a mismatch becomes a C "conflicting types" error).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        headers: Vec<String>,
    },
    Test {
        root_source_file: String,
    },
    Library {
        exports: Vec<String>,
    },
}

impl fmt::Display for BuildTargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Executable { .. } => "executable",
            Self::Test { .. } => "test",
            Self::Library { .. } => "library",
        })
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
    pub targets: Vec<BuildTarget>,
    pub declared_host_effects: Vec<HostEffect>,
    pub used_host_effects: Vec<HostEffect>,
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
    pub name: String,
    pub kind: BuildTargetKind,
    pub sources: Vec<String>,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
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

include!("build_graph/lowering/mod.rs");
