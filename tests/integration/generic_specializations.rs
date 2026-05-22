use super::support::*;
use std::path::PathBuf;

#[path = "generic_specializations/behavior_bounds.rs"]
mod behavior_bounds;
#[path = "generic_specializations/enum_generated_c.rs"]
mod enum_generated_c;
#[path = "generic_specializations/method_worklist_generated_c.rs"]
mod method_worklist_generated_c;
#[path = "generic_specializations/multifile_generated_c.rs"]
mod multifile_generated_c;

#[test]
fn generic_specializations_emit_each_generated_c_definition_once() {
    for fixture in generic_specialization_fixture_paths() {
        let c_source = compile_to_c_with_generated_call_check(&test_dir().join(&fixture));
        assert_generated_c_function_definitions_are_unique(&c_source);
    }
}

fn generic_specialization_fixture_paths() -> Vec<PathBuf> {
    let root = test_dir();
    let mut fixtures = Vec::new();

    for entry in std::fs::read_dir(&root).expect("read tests/zen fixtures") {
        let entry = entry.expect("read tests/zen fixture entry");
        let path = entry.path();
        if !path.is_file() || path.extension().is_none_or(|ext| ext != "zen") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        if stem.contains("generic") || stem == "type_impl_methods" {
            fixtures.push(relative_fixture_path(&root, path));
        }
    }

    for entry in std::fs::read_dir(&root).expect("read tests/zen fixture directories") {
        let entry = entry.expect("read tests/zen fixture directory entry");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if dir_name.contains("generic")
            || dir_name.starts_with("multi_file_type_impl")
            || dir_name.starts_with("multi_file_type_method")
        {
            let main = path.join("main.zen");
            if main.exists() {
                fixtures.push(relative_fixture_path(&root, main));
            }
        }
    }

    fixtures.sort();
    fixtures
}

fn relative_fixture_path(root: &std::path::Path, path: PathBuf) -> PathBuf {
    path.strip_prefix(root)
        .expect("fixture path should be under tests/zen")
        .to_path_buf()
}
