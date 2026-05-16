use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

pub(super) struct BuildGraphExecutableTarget {
    pub(super) name: String,
    pub(super) root_source_file: String,
    pub(super) root_path: PathBuf,
    pub(super) out_dir: PathBuf,
}

pub(super) struct BuildGraphTestTarget {
    pub(super) name: String,
    pub(super) root_source_file: String,
    pub(super) root_path: PathBuf,
    pub(super) out_dir: PathBuf,
}

#[derive(Clone, Copy)]
enum BuildGraphExecutionKind {
    Executable,
    Test,
}

impl BuildGraphExecutionKind {
    fn includes(self, kind: &zen::build_graph::BuildTargetKind) -> bool {
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
    let mut targets = executable_build_targets(path_str);
    if targets.len() != 1 {
        eprintln!(
            "build graph C emission supports exactly one target, found {}",
            targets.len()
        );
        process::exit(1);
    }

    targets.pop().expect("one target")
}

pub(super) fn test_build_targets(path_str: &str) -> Vec<BuildGraphTestTarget> {
    let graph = super::load_build_graph(path_str);
    validate_executed_dependency_targets(&graph, BuildGraphExecutionKind::Test);
    let build_path = Path::new(path_str);
    let base_dir = build_path.parent().unwrap_or_else(|| Path::new("."));
    let ordered_targets = match graph.targets_in_dependency_order() {
        Ok(targets) => targets,
        Err(err) => {
            eprintln!("build graph error: {}", err);
            process::exit(1);
        }
    };
    let targets: Vec<_> = ordered_targets
        .into_iter()
        .filter_map(|target| test_build_target(base_dir, target))
        .collect();
    if targets.is_empty() {
        eprintln!("build graph test execution requires at least one test target");
        process::exit(1);
    }
    targets
}

pub(super) fn executable_build_targets(path_str: &str) -> Vec<BuildGraphExecutableTarget> {
    let graph = super::load_build_graph(path_str);
    validate_executed_dependency_targets(&graph, BuildGraphExecutionKind::Executable);
    let build_path = Path::new(path_str);
    let base_dir = build_path.parent().unwrap_or_else(|| Path::new("."));
    let ordered_targets = match graph.targets_in_dependency_order() {
        Ok(targets) => targets,
        Err(err) => {
            eprintln!("build graph error: {}", err);
            process::exit(1);
        }
    };
    let targets: Vec<_> = ordered_targets
        .into_iter()
        .filter_map(|target| executable_build_target(base_dir, target))
        .collect();
    if targets.is_empty() {
        eprintln!("build graph execution requires at least one executable target");
        process::exit(1);
    }
    targets
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
            if execution_kind.includes(dependency_target.kind()) {
                continue;
            }
            eprintln!(
                "build graph target `{}` depends on gated {} target `{}`",
                target.name(),
                build_target_kind_name(dependency_target.kind()),
                dependency_target.name()
            );
            process::exit(1);
        }
    }
}

pub(super) fn validate_build_graph_sources(base_dir: &Path, graph: &zen::build_graph::BuildGraph) {
    for target in graph.targets() {
        for source in target.sources() {
            if !base_dir.join(source).exists() {
                eprintln!(
                    "build graph target `{}` source not found: {}",
                    target.name(),
                    source
                );
                process::exit(1);
            }
        }
    }
}

fn build_target_kind_name(kind: &zen::build_graph::BuildTargetKind) -> &'static str {
    match kind {
        zen::build_graph::BuildTargetKind::Executable { .. } => "executable",
        zen::build_graph::BuildTargetKind::Test { .. } => "test",
        zen::build_graph::BuildTargetKind::Library { .. } => "library",
    }
}

fn test_build_target(
    base_dir: &Path,
    target: &zen::build_graph::BuildTarget,
) -> Option<BuildGraphTestTarget> {
    let zen::build_graph::BuildTargetKind::Test { root_source_file } = target.kind() else {
        return None;
    };
    let root_path = base_dir.join(root_source_file);
    if !root_path.exists() {
        eprintln!(
            "build graph target `{}` root source not found: {}",
            target.name(),
            root_source_file
        );
        process::exit(1);
    }

    Some(BuildGraphTestTarget {
        name: target.name().to_string(),
        root_source_file: root_source_file.clone(),
        root_path,
        out_dir: base_dir.join("build").join("tests"),
    })
}

fn executable_build_target(
    base_dir: &Path,
    target: &zen::build_graph::BuildTarget,
) -> Option<BuildGraphExecutableTarget> {
    let zen::build_graph::BuildTargetKind::Executable {
        root_source_file,
        out_dir,
    } = target.kind()
    else {
        return None;
    };
    let root_path = base_dir.join(root_source_file);
    if !root_path.exists() {
        eprintln!(
            "build graph target `{}` root source not found: {}",
            target.name(),
            root_source_file
        );
        process::exit(1);
    }

    Some(BuildGraphExecutableTarget {
        name: target.name().to_string(),
        root_source_file: root_source_file.clone(),
        root_path,
        out_dir: base_dir.join(out_dir),
    })
}
