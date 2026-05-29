use serde_json::Value;
use std::path::Path;

use super::super::support::{assert_zen_success, run_zen};
use super::golden_support::write_subject as write_json_subject;
mod generic_symbols;
mod graph;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn write_subject(tmp: &tempfile::TempDir) -> std::path::PathBuf {
    write_json_subject(
        tmp,
        "math.zen",
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}
"#,
    );

    write_json_subject(
        tmp,
        "main.zen",
        r#"
{ add } = math

main = () i32 {
    add(20, 22)
}
"#,
    )
}

fn normalized_module_graph_json(mode: &str) -> String {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let main_path = write_subject(&tmp);
    normalized_json_for_path(mode, &main_path)
}

fn normalized_json_for_path(mode: &str, path: &Path) -> String {
    let args = ["emit-json", mode, path.to_str().unwrap()];
    let output = run_zen(&args);
    assert_zen_success(&args, &output);

    let mut json: Value =
        serde_json::from_slice(&output.stdout).expect("module graph JSON stdout is JSON");
    for module in json["modules"]
        .as_array_mut()
        .expect("module graph modules array")
    {
        let path = module["canonical_path"]
            .as_str()
            .expect("module canonical path");
        module["canonical_path"] = Path::new(path)
            .file_name()
            .expect("module file name")
            .to_string_lossy()
            .into_owned()
            .into();
    }

    serde_json::to_string_pretty(&json).expect("serialize normalized module graph JSON")
}

fn normalized_fixture(path: &str) -> String {
    let expected_path = fixture(path);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));
    let json: Value = serde_json::from_str(&expected)
        .unwrap_or_else(|err| panic!("parse {}: {err}", expected_path.display()));
    serde_json::to_string_pretty(&json).expect("serialize normalized fixture JSON")
}

fn assert_symbols_fixture_matches(source_path: &str, fixture_path: &str) {
    let actual = normalized_json_for_path("symbols", &fixture(source_path));
    if super::golden_support::maybe_bless(fixture_path, &actual) {
        return;
    }
    let expected = normalized_fixture(fixture_path);

    assert_eq!(actual.trim(), expected.trim());
}
