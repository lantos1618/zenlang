use super::*;

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
    let single_file_tests = std::fs::read_to_string("tests/integration/single_file_fixtures.rs")
        .expect("read single-file runtime tests");
    let runtime_tests = std::fs::read_to_string("tests/integration/runtime_fixtures.rs")
        .expect("read runtime tests");
    let multi_file_tests = std::fs::read_to_string("tests/integration/multi_file_fixtures.rs")
        .expect("read multi-file runtime tests");

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
