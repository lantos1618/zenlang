use super::support::*;
use std::path::PathBuf;
mod behavior_bounds;
mod enum_generated_c;
mod method_worklist_generated_c;
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
        if path.is_file() && path.extension().is_some_and(|ext| ext == "zen") {
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("");
            if stem.contains("generic") || stem == "type_impl_methods" {
                fixtures.push(relative_fixture_path(&root, path));
            }
        } else if path.is_dir()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("generic") || name.starts_with("multi_file"))
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

fn assert_fixture_specialization(
    fixture: &str,
    required_snippets: &[&str],
    single_definition_calls: &[&str],
    forbidden_snippets: &[&str],
) -> String {
    let c_source = compile_to_c_with_generated_call_check(&test_dir().join(fixture));
    assert_generated_c_specialization(
        &c_source,
        required_snippets,
        single_definition_calls,
        forbidden_snippets,
    );
    c_source
}

fn assert_box_inner_dependency(fixture: &str, forbidden_snippets: &[&str]) {
    assert_fixture_specialization(
        fixture,
        &[
            "int32_t inner_i32(int32_t value)",
            "int32_t Box_get_inner_i32(Box_i32 self)",
            "inner_i32(self.value)",
            "Box_get_inner_i32(box)",
        ],
        &["inner_i32", "Box_get_inner_i32"],
        forbidden_snippets,
    );
}

fn assert_point_encode_dispatch(fixture: &str) {
    assert_fixture_specialization(
        fixture,
        &[
            "zen_str Point_encode(Point value)",
            "zen_str encode_Point(Point value)",
            "Point_encode(value)",
        ],
        &["Point_encode", "encode_Point"],
        &["T_encode"],
    );
}

fn assert_option_unwrap_or_method(fixture: &str, none_call: &str) {
    assert_fixture_specialization(
        fixture,
        &[
            "typedef struct Option_i32 Option_i32;",
            "int32_t Option_unwrap_or_i32(Option_i32 self, int32_t fallback)",
            "Option_unwrap_or_i32(some, 0LL)",
            none_call,
        ],
        &["Option_unwrap_or_i32"],
        &["Option_T", "T Option_unwrap_or", "Option_unwrap_or(some"],
    );
}

fn assert_result_unwrap_or_method(fixture: &str, err_call: &str) {
    assert_fixture_specialization(
        fixture,
        &[
            "typedef struct Result_i32_StaticString Result_i32_StaticString;",
            "int32_t Result_unwrap_or_i32_StaticString(Result_i32_StaticString self, int32_t fallback)",
            "Result_unwrap_or_i32_StaticString(ok, 0LL)",
            err_call,
        ],
        &["Result_unwrap_or_i32_StaticString"],
        &["Result_T", "T Result_unwrap_or", "Result_unwrap_or(err"],
    );
}

fn assert_result_unwrap_or_multi_specialization(fixture: &str, err_int_call: &str) {
    assert_fixture_specialization(
        fixture,
        &[
            "typedef struct Result_i32_StaticString Result_i32_StaticString;",
            "typedef struct Result_bool_StaticString Result_bool_StaticString;",
            "int32_t Result_unwrap_or_i32_StaticString(Result_i32_StaticString self, int32_t fallback)",
            "bool Result_unwrap_or_bool_StaticString(Result_bool_StaticString self, bool fallback)",
            "Result_unwrap_or_i32_StaticString(ok_int, 0LL)",
            err_int_call,
            "Result_unwrap_or_bool_StaticString(ok_bool, true)",
            "Result_unwrap_or_bool_StaticString(err_bool, true)",
        ],
        &[
            "Result_unwrap_or_i32_StaticString",
            "Result_unwrap_or_bool_StaticString",
        ],
        &["Result_T", "T Result_unwrap_or", "Result_unwrap_or(err"],
    );
}

fn relative_fixture_path(root: &std::path::Path, path: PathBuf) -> PathBuf {
    path.strip_prefix(root)
        .expect("fixture path should be under tests/zen")
        .to_path_buf()
}
