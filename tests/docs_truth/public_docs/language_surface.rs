use super::super::*;

#[test]
fn public_language_docs_and_examples_do_not_teach_return_keyword() {
    for path in [
        "README.md",
        "docs/learn_zen_in_y_minutes.md",
        "examples/01_hello_world.zen",
        "examples/02_variables_and_types.zen",
        "examples/03_pattern_matching.zen",
        "examples/04_structs_and_methods.zen",
        "examples/05_loops.zen",
        "examples/06_error_handling.zen",
        "examples/project/main.zen",
        "examples/project/math_utils.zen",
        "examples/project/test.zen",
        "examples/project/build.zen",
        "tests/nested_struct_field_access.zen",
    ] {
        let contents = read(path);
        assert!(
            !contents.contains("return "),
            "{path} still teaches the removed return keyword"
        );
    }
}
