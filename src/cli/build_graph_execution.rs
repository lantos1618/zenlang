use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

use super::{
    executable_build_target, test_build_target, validate_non_executed_target_sources,
    BuildGraphExecutableTarget, BuildGraphTestTarget,
};

struct BuildGraphExecutionContext {
    graph: zen::build_graph::BuildGraph,
    base_dir: PathBuf,
}

#[derive(Clone, Copy)]
pub(super) enum BuildGraphExecutionKind {
    Executable,
    Test,
}

impl BuildGraphExecutionKind {
    pub(super) fn includes(self, kind: &zen::build_graph::BuildTargetKind) -> bool {
        matches!(
            (self, kind),
            (
                Self::Executable,
                zen::build_graph::BuildTargetKind::Executable { .. }
            ) | (Self::Test, zen::build_graph::BuildTargetKind::Test { .. })
        )
    }
}

pub(super) fn single_executable_build_target(path_str: &str) -> BuildGraphExecutableTarget {
    let context = load_execution_context(path_str, BuildGraphExecutionKind::Executable);
    let ordered_targets = dependency_ordered_targets(&context.graph);
    let executable_targets: Vec<_> = ordered_targets
        .into_iter()
        .filter(|target| {
            matches!(
                target.kind(),
                zen::build_graph::BuildTargetKind::Executable { .. }
            )
        })
        .collect();
    if executable_targets.len() != 1 {
        eprintln!(
            "build graph C emission supports exactly one target, found {}",
            executable_targets.len()
        );
        process::exit(1);
    }

    validate_non_executed_target_sources(
        &context.base_dir,
        &context.graph,
        BuildGraphExecutionKind::Executable,
    );
    executable_build_target(&context.base_dir, executable_targets[0])
        .expect("one executable target")
}

pub(super) fn test_build_targets(path_str: &str) -> Vec<BuildGraphTestTarget> {
    let context = load_execution_context(path_str, BuildGraphExecutionKind::Test);
    validate_non_executed_target_sources(
        &context.base_dir,
        &context.graph,
        BuildGraphExecutionKind::Test,
    );
    let ordered_targets = dependency_ordered_targets(&context.graph);
    let targets: Vec<_> = ordered_targets
        .into_iter()
        .filter_map(|target| test_build_target(&context.base_dir, target))
        .collect();
    if targets.is_empty() {
        eprintln!("build graph test execution requires at least one test target");
        process::exit(1);
    }
    targets
}

pub(super) fn executable_build_targets(path_str: &str) -> Vec<BuildGraphExecutableTarget> {
    let targets = collect_executable_build_targets(path_str);
    if targets.is_empty() {
        eprintln!("build graph execution requires at least one executable target");
        process::exit(1);
    }
    targets
}

fn collect_executable_build_targets(path_str: &str) -> Vec<BuildGraphExecutableTarget> {
    let context = load_execution_context(path_str, BuildGraphExecutionKind::Executable);
    validate_non_executed_target_sources(
        &context.base_dir,
        &context.graph,
        BuildGraphExecutionKind::Executable,
    );
    dependency_ordered_targets(&context.graph)
        .into_iter()
        .filter_map(|target| executable_build_target(&context.base_dir, target))
        .collect()
}

fn load_execution_context(
    path_str: &str,
    execution_kind: BuildGraphExecutionKind,
) -> BuildGraphExecutionContext {
    let graph = super::load_build_graph(path_str);
    let build_path = Path::new(path_str);
    let base_dir = build_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    validate_executed_dependency_targets(&graph, execution_kind);

    BuildGraphExecutionContext { graph, base_dir }
}

fn dependency_ordered_targets(
    graph: &zen::build_graph::BuildGraph,
) -> Vec<&zen::build_graph::BuildTarget> {
    match graph.targets_in_dependency_order() {
        Ok(targets) => targets,
        Err(err) => {
            eprintln!("build graph error: {}", err);
            process::exit(1);
        }
    }
}

fn validate_executed_dependency_targets(
    graph: &zen::build_graph::BuildGraph,
    execution_kind: BuildGraphExecutionKind,
) {
    let targets_by_name: HashMap<_, _> = graph
        .targets()
        .iter()
        .map(|target| (target.name(), target))
        .collect();

    for target in graph
        .targets()
        .iter()
        .filter(|target| execution_kind.includes(target.kind()))
    {
        for dependency in target.dependencies() {
            let Some(dependency_target) = targets_by_name.get(dependency.as_str()) else {
                continue;
            };
            if execution_kind.includes(dependency_target.kind())
                || matches!(
                    dependency_target.kind(),
                    zen::build_graph::BuildTargetKind::Library { .. }
                )
            {
                continue;
            }
            eprintln!(
                "build graph target `{}` depends on gated {} target `{}`",
                target.name(),
                dependency_target.kind(),
                dependency_target.name()
            );
            process::exit(1);
        }
    }
}
