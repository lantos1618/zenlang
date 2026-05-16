//! Integration tests for the Zen compiler pipeline.
//!
//! For each `.zen` file in `tests/zen/`, runs the full pipeline:
//! lex → parse → typecheck → C codegen → compile with cc → run → verify output.

#[path = "integration/support.rs"]
mod support;

use support::*;

#[test]
fn c_call_assertion_ignores_struct_return_definitions() {
    let c_source = r#"
typedef struct Box_Option_i32 Box_Option_i32;

Box_Option_i32 Box_copy_Option_i32(Box_Option_i32 self) {
    return self;
}
"#;

    assert!(
        !has_c_call_outside_signature(c_source, "Box_copy_Option_i32"),
        "definition-only generated C should not count as a call"
    );
}

#[test]
fn generated_c_call_definition_scan_reports_missing_generated_calls() {
    let c_source = r#"
#include <stdio.h>

int32_t inner_i32(int32_t value) {
    return value;
}

int32_t outer_i32(int32_t value) {
    printf("%d", value);
    missing_stmt_i32(value);
    return missing_i32(value) + inner_i32(value);
}
"#;

    assert_eq!(
        undefined_generated_c_calls(c_source),
        vec!["missing_stmt_i32".to_string(), "missing_i32".to_string()]
    );
}

#[test]
fn generated_c_definition_count_ignores_prototypes() {
    let c_source = r#"
int32_t inner_i32(int32_t value);

int32_t inner_i32(int32_t value) {
    return value;
}
"#;

    assert_c_function_definition_count(c_source, "inner_i32", 1);
}

// ── Individual test cases ───────────────────────────────────────────

#[test]
fn test_hello() {
    run_test("hello");
}

#[test]
fn test_arithmetic() {
    run_test("arithmetic");
}

#[test]
fn test_structs() {
    run_test("structs");
}

#[test]
fn test_enums() {
    run_test("enums");
}

#[test]
fn test_duplicate_enum_variant_names() {
    run_test("duplicate_enum_variant_names");
}

#[test]
fn test_ufc() {
    run_test("ufc");
}

#[test]
fn test_conditionals() {
    run_test("conditionals");
}

#[test]
fn test_loops() {
    run_test("loops");
}

#[test]
fn test_strings() {
    run_test("strings");
}

#[test]
fn test_functions() {
    run_test("functions");
}

#[test]
fn test_generic_identity() {
    run_test("generic_identity");
}

#[test]
fn test_generic_struct() {
    run_test("generic_struct");
}

#[test]
fn test_generic_enum_option() {
    run_test("generic_enum_option");
}

#[test]
fn test_generic_method() {
    run_test("generic_method");
}

#[test]
fn test_generic_method_self() {
    run_test("generic_method_self");
}

#[test]
fn test_generic_method_worklist() {
    run_test("generic_method_worklist");
}

#[test]
fn test_generic_method_nested_result() {
    run_test("generic_method_nested_result");
}

#[test]
fn test_type_impl_methods() {
    run_test("type_impl_methods");
}

#[test]
fn test_generic_result_enum() {
    run_test("generic_result_enum");
}

#[test]
fn test_generic_nested_result_enum() {
    run_test("generic_nested_result_enum");
}

#[test]
fn test_generic_vec() {
    run_test("generic_vec");
}

#[test]
fn test_generic_worklist() {
    run_test("generic_worklist");
}

#[test]
fn test_generic_worklist_dedup() {
    run_test("generic_worklist_dedup");
}

#[test]
fn test_generic_ufc_function() {
    run_test("generic_ufc_function");
}

#[test]
fn test_behavior_json_explicit_impl() {
    run_test("behavior_json_explicit_impl");
}

#[test]
fn test_behavior_default_method_dispatch() {
    run_test("behavior_default_method_dispatch");
}

#[test]
fn test_behavior_generic_default_method() {
    run_test("behavior_generic_default_method");
}

#[test]
fn test_behavior_inherited_default_method() {
    run_test("behavior_inherited_default_method");
}

#[test]
fn test_behavior_json_generic_dispatch() {
    run_test("behavior_json_generic_dispatch");
}

#[test]
fn test_behavior_json_generic_association() {
    run_test("behavior_json_generic_association");
}

#[test]
fn test_behavior_json_generic_bound() {
    run_test("behavior_json_generic_bound");
}

#[test]
fn test_behavior_json_generic_bound_ufcs() {
    run_test("behavior_json_generic_bound_ufcs");
}

#[test]
fn test_behavior_generic_parent_inheritance() {
    run_test("behavior_generic_parent_inheritance");
}

#[test]
fn test_behavior_inherited_generic_dispatch() {
    run_test("behavior_inherited_generic_dispatch");
}

#[path = "integration/generic_specializations.rs"]
mod generic_specializations;

#[path = "integration/cli_build.rs"]
mod cli_build;

#[test]
fn integration_frontend_helper_runs_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("bad_resolver_ref.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    missing_local
}
"#,
    )
    .expect("write test file");

    let panic = std::panic::catch_unwind(|| compile_to_c(&zen_path))
        .expect_err("compile_to_c should reject resolver errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'missing_local'"),
        "expected resolver diagnostic, panic={message}"
    );
}

#[test]
fn integration_frontend_helper_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    true
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    add(1, 2)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported module type errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, panic={message}"
    );
}

#[test]
fn imported_behavior_extends_requires_parent_methods() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let traits_path = tmp.path().join("traits.zen");
    std::fs::write(
        &traits_path,
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    )
    .expect("write traits module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ PrettyJson } = traits

Point: {
    x: i32
}

Point.implements(PrettyJson) {
    pretty = (value: Point) str {
        "point"
    }
}

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported inherited behavior requirements");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("implementation of `PrettyJson` is missing required method `encode`"),
        "expected inherited behavior method diagnostic, panic={message}"
    );
}

#[test]
fn imported_behavior_extends_imported_parent_requires_parent_methods() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let base_path = tmp.path().join("base.zen");
    std::fs::write(
        &base_path,
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}
"#,
    )
    .expect("write base module");

    let traits_path = tmp.path().join("traits.zen");
    std::fs::write(
        &traits_path,
        r#"
{ Json } = base

pub PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
    )
    .expect("write traits module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ PrettyJson } = traits

Point: {
    x: i32
}

Point.implements(PrettyJson) {
    pretty = (value: Point) str {
        "point"
    }
}

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject inherited imported parent requirements");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("implementation of `PrettyJson` is missing required method `encode`"),
        "expected imported parent behavior method diagnostic, panic={message}"
    );
}

#[test]
fn imported_behavior_extends_requires_transitive_parent_methods() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let traits_path = tmp.path().join("traits.zen");
    std::fs::write(
        &traits_path,
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub PrettyJson: behavior {
    pretty: (Self) str
}

pub FancyJson: behavior {
    fancy: (Self) str
}

PrettyJson.extends(Json<str>)
FancyJson.extends(PrettyJson)
"#,
    )
    .expect("write traits module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ FancyJson } = traits

Point: {
    x: i32
}

Point.implements(FancyJson) {
    pretty = (value: Point) str {
        "pretty"
    }

    fancy = (value: Point) str {
        "fancy"
    }
}

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path)).expect_err(
        "compile_to_c should reject transitive imported inherited behavior requirements",
    );
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("implementation of `FancyJson` is missing required method `encode`"),
        "expected transitive inherited behavior method diagnostic, panic={message}"
    );
}

#[test]
fn test_defer() {
    run_test("defer");
}

#[test]
fn test_defer_early_return() {
    run_test("defer_early_return");
}

#[test]
fn test_boolean_ops() {
    run_test("boolean_ops");
}

#[test]
fn test_nested_structs() {
    run_test("nested_structs");
}

#[test]
fn test_enum_match() {
    run_test("enum_match");
}

#[test]
fn test_mutability() {
    run_test("mutability");
}

#[test]
fn test_recursion() {
    run_test("recursion");
}

#[test]
fn test_nested_match() {
    run_test("nested_match");
}

#[test]
fn test_cast() {
    run_test("cast");
}

#[test]
fn test_multiple_defer() {
    run_test("multiple_defer");
}

#[test]
fn test_multi_file_imports() {
    let zen_path = test_dir().join("multi_file/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "37\n");
}

#[test]
fn test_multi_file_generic_imports() {
    let zen_path = test_dir().join("multi_file_generic/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "42\n7\n5\n9\n");
}

#[test]
fn test_multi_file_generic_imported_type_dependency_imports() {
    let zen_path = test_dir().join("multi_file_generic_imported_type_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "73\n");
}

#[test]
fn test_multi_file_generic_imported_worklist_chain_imports() {
    let zen_path = test_dir().join("multi_file_generic_imported_worklist_chain/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "83\n");
}

#[test]
fn test_multi_file_generic_imported_transitive_dependency_imports() {
    let zen_path = test_dir().join("multi_file_generic_imported_transitive_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "89\n");
}

#[test]
fn test_multi_file_type_impl_imports() {
    let zen_path = test_dir().join("multi_file_type_impl/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "34\n");
}

#[test]
fn test_multi_file_type_impl_imported_type_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_impl_imported_type_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "61\n");
}

#[test]
fn test_multi_file_type_impl_return_enum_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_impl_return_enum_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "101\n");
}

#[test]
fn test_multi_file_type_method_imports() {
    let zen_path = test_dir().join("multi_file_type_method/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "13\n");
}

#[test]
fn test_multi_file_type_method_worklist_imports() {
    let zen_path = test_dir().join("multi_file_type_method_worklist/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "31\n");
}

#[test]
fn test_multi_file_type_method_method_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_method_method_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "47\n");
}

#[test]
fn test_multi_file_type_method_imported_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_method_imported_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "59\n");
}

#[test]
fn test_multi_file_type_method_return_enum_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_method_return_enum_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "97\n");
}

#[test]
fn test_multi_file_type_method_nested_result_dependency_imports() {
    let zen_path = test_dir().join("multi_file_type_method_nested_result_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "109\n7\n");
}

#[test]
fn imported_type_method_worklist_helpers_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
inner<T> = (value: T) T {
    value
}

pub Box<T>: {
    value: T
}

pub Box.get_inner<T> = (self: Box<T>) T {
    inner(self.value)
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    inner<i32>(1)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct calls to unimported helpers");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'inner'")
            || message.contains("undefined function `inner`"),
        "expected unimported helper diagnostic, panic={message}"
    );
}

#[test]
fn imported_type_method_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
pub Box<T>: {
    value: T
}

Box.inner<T> = (self: Box<T>) T {
    self.value
}

pub Box.get_inner<T> = (self: Box<T>) T {
    self.inner<T>()
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    box = Box<i32> { value: 47 }
    box.inner<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct calls to unimported methods");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("type `Box_i32` has no method `inner`")
            || message.contains("type `Box` has no method `inner`"),
        "expected unimported method diagnostic, panic={message}"
    );
}

#[test]
fn imported_type_method_imported_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
pub inner<T> = (value: T) T {
    value
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ inner } = helper

pub Box<T>: {
    value: T
}

pub Box.get_inner<T> = (self: Box<T>) T {
    inner(self.value)
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    inner<i32>(59)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct calls to source-module imports");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'inner'")
            || message.contains("undefined function `inner`"),
        "expected unimported helper diagnostic, panic={message}"
    );
}

#[test]
fn imported_type_impl_imported_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
pub Holder<T>: {
    value: T
}

pub Holder.get<T> = (self: Holder<T>) T {
    self.value
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ Holder } = helper

pub Box<T>: {
    value: T
}

Box.impl = {
    pub get_held<T> = (self: Box<T>) T {
        holder = Holder<T> { value: self.value }
        holder.get<T>()
    }
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    holder = Holder<i32> { value: 61 }
    holder.get<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct source-module imported type use");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown type symbol 'Holder'")
            || message.contains("unknown type `Holder`")
            || message.contains("unknown generic type `Holder`")
            || message.contains("type `Holder_i32` has no method `get`"),
        "expected unimported helper type or method diagnostic, panic={message}"
    );
}

#[test]
fn imported_generic_function_imported_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
pub Holder<T>: {
    value: T
}

pub Holder.get<T> = (self: Holder<T>) T {
    self.value
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ Holder } = helper

pub get_held<T> = (value: T) T {
    holder = Holder<T> { value: value }
    holder.get<T>()
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ get_held } = model

main = () i32 {
    holder = Holder<i32> { value: 73 }
    holder.get<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct source-module imported type use");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown type symbol 'Holder'")
            || message.contains("unknown type `Holder`")
            || message.contains("unknown generic type `Holder`")
            || message.contains("type `Holder_i32` has no method `get`"),
        "expected unimported helper type or method diagnostic, panic={message}"
    );
}

#[test]
fn imported_generic_function_transitive_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let helper_path = tmp.path().join("helper.zen");
    std::fs::write(
        &helper_path,
        r#"
inner<T> = (value: T) T {
    value
}

pub middle<T> = (value: T) T {
    inner(value)
}
"#,
    )
    .expect("write helper module");

    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
{ middle } = helper

pub outer<T> = (value: T) T {
    middle(value)
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ outer } = model

main = () i32 {
    middle<i32>(89)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct transitive helper calls");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'middle'")
            || message.contains("undefined function `middle`"),
        "expected unimported transitive helper diagnostic, panic={message}"
    );
}

#[test]
fn imported_function_signature_type_dependencies_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
pub Point: {
    x: i32
}

pub make_point = () Point {
    Point { x: 109 }
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ make_point } = model

main = () i32 {
    point = Point { x: 109 }
    point.x
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject direct signature dependency type use");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown type symbol 'Point'")
            || message.contains("unknown type `Point`")
            || message.contains("unknown struct `Point`"),
        "expected unimported signature dependency type diagnostic, panic={message}"
    );
}

#[test]
fn imported_private_type_impl_methods_are_not_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
pub Box<T>: {
    value: T
}

Box.impl = {
    get<T> = (self: Box<T>) T {
        self.value
    }
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Box } = model

main = () i32 {
    box = Box<i32> { value: 34 }
    box.get<i32>()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject private imported impl methods");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("type `Box_i32` has no method `get`"),
        "expected private imported impl method diagnostic, panic={message}"
    );
}

#[test]
fn imported_private_behavior_impl_methods_are_not_directly_visible() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let model_path = tmp.path().join("model.zen");
    std::fs::write(
        &model_path,
        r#"
Hidden: behavior {
    reveal: (Self) str
}

pub Point: {
    x: i32
}

Point.implements(Hidden) {
    reveal = (value: Point) str {
        "hidden"
    }
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Point } = model

main = () i32 {
    point = Point { x: 34 }
    point.reveal()
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject private imported behavior impl methods");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("type `Point` has no method `reveal`"),
        "expected private imported behavior impl method diagnostic, panic={message}"
    );
}

#[test]
fn test_multi_file_behavior_bound_imports() {
    let zen_path = test_dir().join("multi_file_behavior_bound/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "11\n");
}

#[test]
fn test_multi_file_behavior_inheritance_imports() {
    let zen_path = test_dir().join("multi_file_behavior_inheritance/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "encoded\npretty\nfancy\n");
}

#[test]
fn test_multi_file_imported_behavior_impls() {
    let zen_path = test_dir().join("multi_file_imported_behavior_impl/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "encoded\n");
}

#[test]
fn test_multi_file_imported_behavior_defaults() {
    let zen_path = test_dir().join("multi_file_imported_behavior_default/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "default-json\n");
}

#[test]
fn test_multi_file_imported_generic_behavior_defaults() {
    let zen_path = test_dir().join("multi_file_imported_generic_behavior_default/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "imported-default\n");
}

#[test]
fn test_multi_file_imported_impl_with_imported_behavior() {
    let zen_path = test_dir().join("multi_file_imported_impl_imported_behavior/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "encoded\npretty\n");
}

#[test]
fn test_multi_file_imported_child_parent_dispatch() {
    let zen_path = test_dir().join("multi_file_imported_child_parent_dispatch/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "encoded\npretty\n");
}

#[test]
fn test_multi_file_imported_behavior_requires() {
    let zen_path = test_dir().join("multi_file_imported_behavior_requires/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "required\n");
}

#[test]
fn test_multi_file_imported_function_imported_behavior_bound() {
    let zen_path = test_dir().join("multi_file_imported_function_imported_behavior_bound/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "97\n");
}

#[test]
fn test_multi_file_imported_function_return_type_dependency() {
    let zen_path = test_dir().join("multi_file_imported_function_return_type_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "101\n");
}

#[test]
fn test_multi_file_imported_function_param_type_dependency() {
    let zen_path = test_dir().join("multi_file_imported_function_param_type_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "127\n");
}

#[test]
fn test_multi_file_imported_function_imported_return_type_behavior() {
    let zen_path =
        test_dir().join("multi_file_imported_function_imported_return_type_behavior/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "113\n");
}

#[test]
fn test_multi_file_imported_generic_function_return_enum_dependency() {
    let zen_path =
        test_dir().join("multi_file_imported_generic_function_return_enum_dependency/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "107\n");
}

// ── Discovery test: all .zen files have matching .expected ──────────

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
