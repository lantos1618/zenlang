use super::*;
use std::path::Path;

#[test]
fn all_zen_files_have_expected_output() {
    let dir = test_dir();
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read tests/zen") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("zen") {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let expected = dir.join("expected").join(format!("{}.expected", stem));
            if !expected.exists() {
                missing.push(stem.to_string());
            }
        }
    }
    assert!(
        missing.is_empty(),
        "missing .expected files for: {:?}",
        missing
    );
}

#[test]
fn all_expected_outputs_are_exercised_by_runtime_tests() {
    let dir = test_dir();
    let expected_dir = dir.join("expected");
    let single_file_tests = runtime_test_sources([
        "tests/integration/single_file_fixtures.rs",
        "tests/integration/single_file_fixtures",
    ]);
    let runtime_tests = runtime_test_sources(["tests/integration/runtime_fixtures.rs"]);
    let multi_file_tests = runtime_test_sources([
        "tests/integration/multi_file_fixtures.rs",
        "tests/integration/multi_file_phase5_fixtures.rs",
    ]);

    let mut uncovered = Vec::new();
    for entry in std::fs::read_dir(&expected_dir).expect("read tests/zen/expected") {
        let entry = entry.expect("expected dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("expected") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_str().unwrap();
        let run_test_call = format!("run_test(\"{stem}\")");
        let multi_file_path = format!("{stem}/main.zen");
        if !single_file_tests.contains(&run_test_call)
            && !runtime_tests.contains(&run_test_call)
            && !multi_file_tests.contains(&multi_file_path)
        {
            uncovered.push(stem.to_string());
        }
    }

    uncovered.sort();
    assert!(
        uncovered.is_empty(),
        "expected fixtures without runtime coverage: {uncovered:?}"
    );
}

fn runtime_test_sources<const N: usize>(paths: [&str; N]) -> String {
    let mut sources = String::new();
    for path in paths {
        append_runtime_test_sources(Path::new(path), &mut sources);
    }
    sources
}

fn append_runtime_test_sources(path: &Path, sources: &mut String) {
    if !path.exists() {
        return;
    }

    if path.is_file() {
        sources.push_str(
            &std::fs::read_to_string(path)
                .unwrap_or_else(|err| panic!("read runtime test source {path:?}: {err}")),
        );
        sources.push('\n');
        return;
    }

    let mut entries = std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read runtime test source dir {path:?}: {err}"))
        .map(|entry| entry.expect("read runtime test source dir entry").path())
        .collect::<Vec<_>>();
    entries.sort();

    for entry in entries {
        if entry.is_dir() || entry.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            append_runtime_test_sources(&entry, sources);
        }
    }
}
