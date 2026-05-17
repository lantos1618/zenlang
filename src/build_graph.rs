use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::Serialize;

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
    pub fn from_input(input: BuildGraphInput) -> Result<Self, BuildGraphError> {
        if input.targets.is_empty() {
            return Err(BuildGraphError::MissingTargets);
        }

        let declared_host_effects = sorted_unique(input.declared_host_effects);
        let used_host_effects = sorted_unique(input.used_host_effects);
        let declared_set: BTreeSet<_> = declared_host_effects.iter().cloned().collect();
        for effect in &used_host_effects {
            if !declared_set.contains(effect) {
                return Err(BuildGraphError::UndeclaredHostEffect(effect.clone()));
            }
        }

        let mut target_names = BTreeSet::new();
        let mut targets = Vec::with_capacity(input.targets.len());
        for target in input.targets {
            if target.name.is_empty() {
                return Err(BuildGraphError::EmptyTargetName);
            }
            if !target_names.insert(target.name.clone()) {
                return Err(BuildGraphError::DuplicateTargetName(target.name));
            }
            targets.push(BuildTarget {
                name: target.name,
                kind: target.kind,
                sources: sorted_unique(target.sources),
                dependencies: sorted_unique(target.dependencies),
                features: sorted_unique(target.features),
            });
        }
        for target in &targets {
            for dependency in &target.dependencies {
                if dependency == &target.name {
                    return Err(BuildGraphError::SelfTargetDependency(target.name.clone()));
                }
                if !target_names.contains(dependency) {
                    return Err(BuildGraphError::UnknownTargetDependency {
                        target: target.name.clone(),
                        dependency: dependency.clone(),
                    });
                }
            }
        }
        targets.sort_by(|left, right| left.name.cmp(&right.name));

        let graph = Self {
            targets,
            declared_host_effects,
            used_host_effects,
        };
        graph.targets_in_dependency_order()?;
        Ok(graph)
    }

    pub fn targets(&self) -> &[BuildTarget] {
        &self.targets
    }

    pub fn targets_in_dependency_order(&self) -> Result<Vec<&BuildTarget>, BuildGraphError> {
        let targets_by_name: BTreeMap<&str, &BuildTarget> = self
            .targets
            .iter()
            .map(|target| (target.name.as_str(), target))
            .collect();
        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        let mut ordered = Vec::with_capacity(self.targets.len());

        for target in &self.targets {
            visit_target(
                target.name.as_str(),
                &targets_by_name,
                &mut visiting,
                &mut visited,
                &mut ordered,
            )?;
        }

        Ok(ordered)
    }

    pub fn canonical_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
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

fn visit_target<'a>(
    name: &'a str,
    targets_by_name: &BTreeMap<&'a str, &'a BuildTarget>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
    ordered: &mut Vec<&'a BuildTarget>,
) -> Result<(), BuildGraphError> {
    if visited.contains(name) {
        return Ok(());
    }
    if !visiting.insert(name) {
        return Err(BuildGraphError::CyclicTargetDependency(name.to_string()));
    }

    let target =
        targets_by_name
            .get(name)
            .ok_or_else(|| BuildGraphError::UnknownTargetDependency {
                target: name.to_string(),
                dependency: name.to_string(),
            })?;
    for dependency in target.dependencies() {
        if !targets_by_name.contains_key(dependency.as_str()) {
            return Err(BuildGraphError::UnknownTargetDependency {
                target: target.name().to_string(),
                dependency: dependency.clone(),
            });
        }
        visit_target(
            dependency.as_str(),
            targets_by_name,
            visiting,
            visited,
            ordered,
        )?;
    }

    visiting.remove(name);
    visited.insert(name);
    ordered.push(*target);
    Ok(())
}

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
