use std::path::{Path, PathBuf};

pub(super) struct BuildGraphTarget {
    pub(super) name: String,
    pub(super) root_source_file: String,
    pub(super) root_path: PathBuf,
    pub(super) out_dir: PathBuf,
    /// System libraries to link (from the `link:` field), e.g. `["SDL3"]`.
    pub(super) link: Vec<String>,
    /// C headers to force-include (from `headers:`), e.g. `["SDL3/SDL.h"]`.
    pub(super) headers: Vec<String>,
}

pub(super) fn test_build_target(
    base_dir: &Path,
    target: &zen::build_graph::BuildTarget,
) -> Option<BuildGraphTarget> {
    let zen::build_graph::BuildTargetKind::Test { root_source_file } = &target.kind else {
        return None;
    };
    let root_path =
        super::require_target_source_path(base_dir, target, root_source_file, "root source");

    Some(BuildGraphTarget {
        name: target.name.clone(),
        root_source_file: root_source_file.clone(),
        root_path,
        out_dir: base_dir.join("build").join("tests"),
        link: Vec::new(),
        headers: Vec::new(),
    })
}

pub(super) fn executable_build_target(
    base_dir: &Path,
    target: &zen::build_graph::BuildTarget,
) -> Option<BuildGraphTarget> {
    let zen::build_graph::BuildTargetKind::Executable {
        root_source_file,
        out_dir,
        link,
        headers,
    } = &target.kind
    else {
        return None;
    };
    let root_path =
        super::require_target_source_path(base_dir, target, root_source_file, "root source");

    Some(BuildGraphTarget {
        name: target.name.clone(),
        root_source_file: root_source_file.clone(),
        root_path,
        out_dir: base_dir.join(out_dir),
        link: link.clone(),
        headers: headers.clone(),
    })
}
