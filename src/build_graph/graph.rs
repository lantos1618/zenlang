use std::collections::BTreeSet;

use super::*;

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

    pub fn canonical_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&BuildGraphJson {
            format: "zen.build_graph.v0",
            schema_version: 0,
            semantic_status: "deterministic",
            targets: &self.targets,
            declared_host_effects: &self.declared_host_effects,
            used_host_effects: &self.used_host_effects,
        })
    }
}
