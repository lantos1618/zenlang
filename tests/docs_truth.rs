use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    let path = repo_root().join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

#[path = "docs_truth/contributor_docs.rs"]
mod contributor_docs;
#[path = "docs_truth/phase_audit.rs"]
mod phase_audit;
#[path = "docs_truth/public_docs.rs"]
mod public_docs;
#[path = "docs_truth/repo_hygiene/mod.rs"]
mod repo_hygiene;
#[path = "docs_truth/v1_spec.rs"]
mod v1_spec;
