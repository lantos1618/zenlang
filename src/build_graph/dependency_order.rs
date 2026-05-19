use std::collections::{BTreeMap, BTreeSet};

use super::{BuildGraph, BuildGraphError, BuildTarget};

impl BuildGraph {
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
}

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
