use std::collections::BTreeSet;

use serde::Serialize;

mod dependency_order;
mod errors;
mod formatting;
mod json;

pub use errors::BuildGraphError;

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
