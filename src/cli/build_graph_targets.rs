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

pub(super) fn test_build_target(
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

pub(super) fn executable_build_target(
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
