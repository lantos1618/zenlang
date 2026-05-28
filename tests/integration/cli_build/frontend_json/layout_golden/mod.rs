use std::path::Path;

use super::golden_support::{emit_checked_json, fixture, write_subject};
mod generic;
mod subject;

fn emit_layout(path: &Path, description: &str) -> String {
    emit_checked_json("layout", "layout", path, description)
}

fn expected_fixture(path: &str) -> String {
    let expected_path = fixture(path);
    std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()))
}

fn assert_layout_matches_fixture(source_path: &Path, description: &str, fixture_stem: &str) {
    let actual = emit_layout(source_path, description);
    let fixture_path = format!("tests/fixtures/ir_json/layout_{fixture_stem}.golden.json");
    let expected = expected_fixture(&fixture_path);

    assert_eq!(actual.trim(), expected.trim());
}
