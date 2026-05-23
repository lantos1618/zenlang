use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    let path = repo_root().join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn generated_c_fixture_block<'a>(source: &'a str, fixture: &str) -> &'a str {
    let fixture_start = source
        .find(fixture)
        .unwrap_or_else(|| panic!("missing generated-C fixture block: {fixture}"));
    let start = [
        "let c_source = compile_to_c_with_generated_call_check",
        "let c_source = compile_to_c_with_specialization_check",
        "compile_to_c_with_specialization_check",
    ]
    .iter()
    .filter_map(|marker| source[..fixture_start].rfind(marker))
    .max()
    .unwrap_or(fixture_start);
    let tail = &source[start..];
    let next_block = [
        "let c_source = compile_to_c_with_generated_call_check",
        "let c_source = compile_to_c_with_specialization_check",
        "compile_to_c_with_specialization_check",
    ]
    .iter()
    .filter_map(|marker| tail[fixture.len()..].find(marker))
    .min()
    .map(|offset| fixture.len() + offset)
    .unwrap_or(tail.len());

    &tail[..next_block]
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
