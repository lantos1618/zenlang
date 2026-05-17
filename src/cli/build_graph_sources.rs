use std::collections::BTreeSet;
use std::path::Path;
use std::process;

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

pub(super) fn check_build_graph_sources(base_dir: &Path, graph: &zen::build_graph::BuildGraph) {
    let mut source_paths = BTreeSet::new();
    for target in graph.targets() {
        for source in target.sources() {
            source_paths.insert(base_dir.join(source));
        }
    }

    check_source_paths(source_paths);
}

pub(super) fn validate_graph_only_library_sources(
    base_dir: &Path,
    graph: &zen::build_graph::BuildGraph,
) {
    let mut checked_sources = BTreeSet::new();
    for target in graph.targets().iter().filter(|target| {
        matches!(
            target.kind(),
            zen::build_graph::BuildTargetKind::Library { .. }
        )
    }) {
        for source in target.sources() {
            let source_path = base_dir.join(source);
            if !source_path.exists() {
                eprintln!(
                    "build graph target `{}` source not found: {}",
                    target.name(),
                    source
                );
                process::exit(1);
            }
            checked_sources.insert(source_path);
        }
    }

    check_source_paths(checked_sources);
}

fn check_source_paths(source_paths: BTreeSet<std::path::PathBuf>) {
    for source_path in source_paths {
        let source_path = source_path.to_str().unwrap_or_else(|| {
            eprintln!("error: non-utf8 source path: {}", source_path.display());
            process::exit(1);
        });
        super::graph_frontend(source_path);
    }
}
