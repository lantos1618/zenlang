use super::*;

fn assert_multi_file_fixture(fixture: &str, expected: &str) {
    let zen_path = test_dir().join(fixture).join("main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, expected);
}

#[path = "multi_file_fixtures/basic.rs"]
mod basic;
#[path = "multi_file_fixtures/behavior_imports.rs"]
mod behavior_imports;
#[path = "multi_file_fixtures/function_dependencies.rs"]
mod function_dependencies;
#[path = "multi_file_fixtures/generic_imports.rs"]
mod generic_imports;
#[path = "multi_file_fixtures/type_impls.rs"]
mod type_impls;
#[path = "multi_file_fixtures/type_methods.rs"]
mod type_methods;
