use std::collections::BTreeSet;
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
    MissingTargets,
    UndeclaredHostEffect(HostEffect),
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
        targets.sort_by(|left, right| left.name.cmp(&right.name));

        Ok(Self {
            targets,
            declared_host_effects,
            used_host_effects,
        })
    }

    pub fn targets(&self) -> &[BuildTarget] {
        &self.targets
    }

    pub fn canonical_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl BuildTarget {
    pub fn is_executable(&self) -> bool {
        matches!(self.kind, BuildTargetKind::Executable { .. })
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
            Self::MissingTargets => f.write_str("build graph must contain at least one target"),
            Self::UndeclaredHostEffect(effect) => {
                write!(f, "undeclared host effect: {effect}")
            }
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
