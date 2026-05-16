//! Integration tests for the Zen compiler pipeline.
//!
//! For each `.zen` file in `tests/zen/`, runs the full pipeline:
//! lex → parse → typecheck → C codegen → compile with cc → run → verify output.

use std::path::{Path, PathBuf};
use std::process::Command;

use zen::codegen::c::CBackend;
use zen::codegen::Backend;
use zen::error::FileTable;
use zen::module_system::ModuleSystem;
use zen::typechecker::TypeChecker;

/// Root of the test fixtures.
fn test_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/zen")
}

/// Run the full pipeline for a `.zen` file and return stdout of the compiled binary.
fn compile_to_c(zen_path: &Path) -> String {
    // 1. Load & parse
    let mut files = FileTable::new();
    let mut module_system = ModuleSystem::new();
    let graph = module_system
        .load_module_graph(zen_path, &mut files)
        .unwrap_or_else(|errs| {
            panic!(
                "load/parse error in {}:\n  {}",
                zen_path.display(),
                errs.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        });

    // 2. Resolve and typecheck the graph.
    let mut checker = TypeChecker::new();
    let typed = checker
        .check_module_graph_entry(&graph)
        .unwrap_or_else(|diags| {
            panic!(
                "typecheck error in {}:\n  {}",
                zen_path.display(),
                diags
                    .iter()
                    .map(|d| d.message.clone())
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        });

    // 4. Codegen
    let backend = CBackend;
    backend
        .generate(&typed)
        .unwrap_or_else(|e| panic!("codegen error in {}: {}", zen_path.display(), e))
}

fn compile_to_c_with_generated_call_check(zen_path: &Path) -> String {
    let c_source = compile_to_c(zen_path);
    assert_generated_c_calls_resolve_to_definitions(&c_source);
    c_source
}

/// Run the full pipeline for a `.zen` file and return stdout of the compiled binary.
fn compile_and_run(zen_path: &Path) -> String {
    let c_source = compile_to_c_with_generated_call_check(zen_path);

    // 4. Compile C → binary in a temp dir
    let tmp = tempfile::tempdir().expect("create temp dir");
    let c_path = tmp.path().join("out.c");
    let bin_path = tmp.path().join("out");

    std::fs::write(&c_path, &c_source).expect("write C source");

    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".into());
    let compile = Command::new(&cc)
        .args([
            c_path.to_str().unwrap(),
            "-o",
            bin_path.to_str().unwrap(),
            "-lm",
        ])
        .output()
        .expect("run cc");

    if !compile.status.success() {
        panic!(
            "cc failed for {}:\n--- C source ---\n{}\n--- stderr ---\n{}",
            zen_path.display(),
            c_source,
            String::from_utf8_lossy(&compile.stderr),
        );
    }

    // 5. Run binary
    let run = Command::new(&bin_path).output().expect("run binary");
    assert!(
        run.status.success(),
        "binary exited with {} for {}",
        run.status,
        zen_path.display()
    );

    String::from_utf8(run.stdout).expect("stdout is utf-8")
}

/// Read the expected output for a test name.
fn expected_output(name: &str) -> String {
    let path = test_dir()
        .join("expected")
        .join(format!("{}.expected", name));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

/// Run one end-to-end test by name.
fn run_test(name: &str) {
    let zen_path = test_dir().join(format!("{}.zen", name));
    let actual = compile_and_run(&zen_path);
    let expected = expected_output(name);
    assert_eq!(
        actual, expected,
        "\n--- test: {} ---\nexpected:\n{}\nactual:\n{}",
        name, expected, actual
    );
}

fn assert_c_function_definition(c_source: &str, name: &str) {
    let needle = format!(" {name}(");
    assert!(
        c_source
            .lines()
            .any(|line| line.trim_end().ends_with('{') && line.contains(&needle)),
        "expected generated C definition for `{name}`:\n{c_source}"
    );
}

fn assert_c_function_definition_count(c_source: &str, name: &str, expected: usize) {
    let actual = c_function_definitions(c_source)
        .iter()
        .filter(|definition| definition.as_str() == name)
        .count();
    assert_eq!(
        actual, expected,
        "expected {expected} generated C definitions for `{name}`, found {actual}:\n{c_source}"
    );
}

fn assert_c_call_resolves_to_definition(c_source: &str, name: &str) {
    assert_c_function_definition(c_source, name);
    assert!(
        has_c_call_outside_signature(c_source, name),
        "expected generated C call to `{name}` outside declarations/definitions:\n{c_source}"
    );
}

fn assert_generated_c_calls_resolve_to_definitions(c_source: &str) {
    let undefined = undefined_generated_c_calls(c_source);
    assert!(
        undefined.is_empty(),
        "generated C calls missing emitted definitions: {undefined:?}\n{c_source}"
    );
}

fn undefined_generated_c_calls(c_source: &str) -> Vec<String> {
    let definitions = c_function_definitions(c_source);
    let mut undefined = Vec::new();

    for line in c_source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("typedef ")
            || is_any_c_function_signature_line(trimmed)
        {
            continue;
        }

        for call in generated_c_calls_on_line(trimmed) {
            if !definitions.contains(&call) && !undefined.contains(&call) {
                undefined.push(call);
            }
        }
    }

    undefined
}

fn c_function_definitions(c_source: &str) -> Vec<String> {
    c_source
        .lines()
        .filter_map(|line| c_function_definition_name(line.trim()))
        .collect()
}

fn c_function_definition_name(trimmed: &str) -> Option<String> {
    if !trimmed.ends_with('{') {
        return None;
    }

    let paren = trimmed.find('(')?;
    let before = trimmed[..paren].trim_end();
    let name_start = before
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map_or(0, |idx| idx + 1);
    let name = &before[name_start..];

    if is_generated_c_function_name(name) {
        Some(name.to_string())
    } else {
        None
    }
}

fn is_any_c_function_signature_line(trimmed: &str) -> bool {
    if !(trimmed.ends_with(';') || trimmed.ends_with('{')) {
        return false;
    }

    let Some(paren) = trimmed.find('(') else {
        return false;
    };
    let before = &trimmed[..paren];
    let name_start = before
        .trim_end()
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map_or(0, |idx| idx + 1);
    let return_type = before[..name_start].trim();
    let name = before[name_start..].trim();

    !return_type.is_empty()
        && is_generated_c_function_name(name)
        && !before.contains('=')
        && !before.contains("return")
}

fn generated_c_calls_on_line(trimmed: &str) -> Vec<String> {
    let mut calls = Vec::new();
    let bytes = trimmed.as_bytes();
    let mut index = 0;

    while let Some(relative) = trimmed[index..].find('(') {
        let paren = index + relative;
        let mut start = paren;
        while start > 0 {
            let ch = bytes[start - 1] as char;
            if ch.is_ascii_alphanumeric() || ch == '_' {
                start -= 1;
            } else {
                break;
            }
        }

        let name = &trimmed[start..paren];
        if is_generated_c_function_name(name) && !calls.iter().any(|call| call == name) {
            calls.push(name.to_string());
        }

        index = paren + 1;
    }

    calls
}

fn is_generated_c_function_name(name: &str) -> bool {
    name.contains('_')
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic())
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn has_c_call_outside_signature(c_source: &str, name: &str) -> bool {
    let call = format!("{name}(");
    c_source.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.contains(&call) && !is_c_function_signature_line(trimmed, name)
    })
}

fn is_c_function_signature_line(trimmed: &str, name: &str) -> bool {
    let needle = format!(" {name}(");
    let Some(call_start) = trimmed.find(&needle) else {
        return false;
    };
    let prefix = &trimmed[..call_start];
    !prefix.contains('=')
        && !prefix.contains("return")
        && (trimmed.ends_with(';') || trimmed.ends_with('{'))
}

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

#[test]
fn generic_specializations_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c_with_generated_call_check(&test_dir().join("generic_method.zen"));
    assert!(c_source.contains("int32_t Box_get_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_get_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_get_i32");
    assert!(!c_source.contains("Box_T"));
    assert!(!c_source.contains("T Box_get"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_method_self.zen"));
    assert!(c_source.contains("Box_i32 Box_copy_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_copy_i32(box)"));
    assert!(c_source.contains("Box_Option_i32 Box_copy_Option_i32(Box_Option_i32 self)"));
    assert!(c_source.contains("Box_copy_Option_i32(nested)"));
    assert!(c_source.contains("Option_i32 Option_copy_i32(Option_i32 self)"));
    assert!(c_source.contains("Option_copy_i32(option)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_copy_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_copy_Option_i32");
    assert_c_call_resolves_to_definition(&c_source, "Option_copy_i32");
    assert!(
        c_source
            .find("struct Option_i32")
            .expect("Option_i32 struct")
            < c_source
                .find("struct Box_Option_i32")
                .expect("Box_Option_i32 struct")
    );
    assert!(!c_source.contains("Box_copy(box"));
    assert!(!c_source.contains("Unknown"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_method_worklist.zen"));
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t Box_get_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("inner_i32(self.value)"));
    assert!(c_source.contains("Box_get_inner_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_inner_i32");
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("inner_T"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("generic_method_nested_result.zen"),
    );
    assert!(c_source.contains("typedef struct Result_Option_i32_str Result_Option_i32_str;"));
    assert!(c_source.contains("Result_Option_i32_str Box_wrap_result_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_wrap_result_i32(box)"));
    assert!(c_source.contains("unwrap_result_Option_i32_str(wrapped,"));
    assert!(c_source.contains("unwrap_option_i32(some, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_wrap_result_i32");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_result_Option_i32_str");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_option_i32");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T Box_wrap_result"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("type_impl_methods.zen"));
    assert!(c_source.contains("int32_t Point_get(Point self)"));
    assert!(c_source.contains("int32_t Point_keep_i32(Point self, int32_t value)"));
    assert!(c_source.contains("Point_get(point)"));
    assert!(c_source.contains("Point_keep_i32(point, 7LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_get");
    assert_c_call_resolves_to_definition(&c_source, "Point_keep_i32");
    assert!(!c_source.contains("T Point_keep"));
    assert!(!c_source.contains("Point_keep(point"));

    let c_source = compile_to_c_with_generated_call_check(&test_dir().join("generic_vec.zen"));
    assert!(c_source.contains("int32_t Vec_len_i32(Vec_i32 self)"));
    assert!(c_source.contains("int32_t Vec_len_str(Vec_str self)"));
    assert!(c_source.contains("Vec_len_i32(ints)"));
    assert!(c_source.contains("Vec_len_str(words)"));
    assert_c_call_resolves_to_definition(&c_source, "Vec_len_i32");
    assert_c_call_resolves_to_definition(&c_source, "Vec_len_str");
    assert!(!c_source.contains("Vec_T"));
    assert!(!c_source.contains("T Vec_len"));

    let c_source = compile_to_c_with_generated_call_check(&test_dir().join("generic_worklist.zen"));
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t outer_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "outer_i32");
    assert_c_function_definition_count(&c_source, "inner_i32", 1);
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("inner_T"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_worklist_dedup.zen"));
    assert!(c_source.contains("int32_t left_i32(int32_t value)"));
    assert!(c_source.contains("int32_t right_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "left_i32");
    assert_c_call_resolves_to_definition(&c_source, "right_i32");
    assert_c_function_definition_count(&c_source, "inner_i32", 1);
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("inner_T"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_enum_option.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("int32_t unwrap_or_i32(Option_i32 value, int32_t fallback)"));
    assert!(c_source.contains("Option_i32_Some"));
    assert!(c_source.contains("unwrap_or_i32(x, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T unwrap_or"));
    assert!(!c_source.contains("unwrap_or(x"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("duplicate_enum_variant_names.zen"),
    );
    assert!(c_source.contains("First_i32_Some"));
    assert!(c_source.contains("First_i32_None"));
    assert!(c_source.contains("Second_bool_Some"));
    assert!(c_source.contains("Second_bool_None"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_result_enum.zen"));
    assert!(c_source.contains("typedef struct Result_i32_str Result_i32_str;"));
    assert!(c_source.contains("int32_t unwrap_or_i32_str(Result_i32_str value, int32_t fallback)"));
    assert!(c_source.contains("Result_i32_str_Err"));
    assert!(c_source.contains("unwrap_or_i32_str(err, 9LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_or_i32_str");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T unwrap_or"));
    assert!(!c_source.contains("unwrap_or(err"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_nested_result_enum.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("typedef struct Result_Option_i32_str Result_Option_i32_str;"));
    assert!(c_source.contains(
        "Option_i32 unwrap_result_Option_i32_str(Result_Option_i32_str value, Option_i32 fallback)"
    ));
    assert!(c_source.contains("unwrap_result_Option_i32_str(ok,"));
    assert!(c_source.contains("unwrap_option_i32(some, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_result_Option_i32_str");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_option_i32");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T unwrap_result"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("multi_file_generic/main.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("typedef struct Result_i32_str Result_i32_str;"));
    assert!(c_source.contains("int32_t unwrap_option_i32(Option_i32 value, int32_t fallback)"));
    assert!(
        c_source.contains("int32_t unwrap_result_i32_str(Result_i32_str value, int32_t fallback)")
    );
    assert!(c_source.contains("unwrap_option_i32(some, 0LL)"));
    assert!(c_source.contains("unwrap_result_i32_str(err, 9LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_option_i32");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_result_i32_str");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T unwrap_option"));
    assert!(!c_source.contains("T unwrap_result"));
    assert!(!c_source.contains("unwrap_option(some"));
    assert!(!c_source.contains("unwrap_result(err"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_imported_type_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Holder_i32 Holder_i32;"));
    assert!(c_source.contains("int32_t Holder_get_i32(Holder_i32 self)"));
    assert!(c_source.contains("int32_t get_held_i32(int32_t value)"));
    assert!(c_source.contains("const Holder_i32 holder = (Holder_i32){ .value = value }"));
    assert!(c_source.contains("Holder_get_i32(holder)"));
    assert!(c_source.contains("get_held_i32(73LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Holder_get_i32");
    assert_c_call_resolves_to_definition(&c_source, "get_held_i32");
    assert!(!c_source.contains("Holder_T"));
    assert!(!c_source.contains("T Holder_get"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_imported_worklist_chain/main.zen"),
    );
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t middle_i32(int32_t value)"));
    assert!(c_source.contains("int32_t outer_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert!(c_source.contains("middle_i32(value)"));
    assert!(c_source.contains("outer_i32(83LL)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "middle_i32");
    assert_c_call_resolves_to_definition(&c_source, "outer_i32");
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("T middle"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_generic_imported_transitive_dependency/main.zen"),
    );
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t middle_i32(int32_t value)"));
    assert!(c_source.contains("int32_t outer_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert!(c_source.contains("middle_i32(value)"));
    assert!(c_source.contains("outer_i32(89LL)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "middle_i32");
    assert_c_call_resolves_to_definition(&c_source, "outer_i32");
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("T middle"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("multi_file_type_impl/main.zen"));
    assert!(c_source.contains("int32_t Box_get_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_get_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_get_i32");
    assert!(!c_source.contains("Box_T"));
    assert!(!c_source.contains("T Box_get"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_impl_imported_type_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Holder_i32 Holder_i32;"));
    assert!(c_source.contains("int32_t Holder_get_i32(Holder_i32 self)"));
    assert!(c_source.contains("int32_t Box_get_held_i32(Box_i32 self)"));
    assert!(c_source.contains("const Holder_i32 holder = (Holder_i32){ .value = self.value }"));
    assert!(c_source.contains("Holder_get_i32(holder)"));
    assert!(c_source.contains("Box_get_held_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Holder_get_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_held_i32");
    assert!(!c_source.contains("Holder_T"));
    assert!(!c_source.contains("T Holder_get"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_impl_return_enum_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("Option_i32 Box_wrap_i32(Box_i32 self)"));
    assert!(c_source.contains("int32_t Box_value_or_i32(Box_i32 self, int32_t fallback)"));
    assert!(c_source.contains("Box_wrap_i32(self)"));
    assert!(c_source.contains("Box_value_or_i32(box, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_wrap_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_value_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T Box_wrap"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("multi_file_type_method/main.zen"));
    assert!(c_source.contains("int32_t Point_keep_i32(Point self, int32_t value)"));
    assert!(c_source.contains("Point_keep_i32(point, 13LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_keep_i32");
    assert!(!c_source.contains("T Point_keep"));
    assert!(!c_source.contains("Point_keep(point"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_method_worklist/main.zen"),
    );
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t Box_get_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("inner_i32(self.value)"));
    assert!(c_source.contains("Box_get_inner_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_inner_i32");
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("inner_T"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_method_method_dependency/main.zen"),
    );
    assert!(c_source.contains("int32_t Box_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("int32_t Box_get_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_inner_i32(self)"));
    assert!(c_source.contains("Box_get_inner_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_inner_i32");
    assert!(!c_source.contains("T Box_inner"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_method_imported_dependency/main.zen"),
    );
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t Box_get_inner_i32(Box_i32 self)"));
    assert!(c_source.contains("inner_i32(self.value)"));
    assert!(c_source.contains("Box_get_inner_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_get_inner_i32");
    assert!(!c_source.contains("T inner"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_method_return_enum_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("Option_i32 Box_wrap_i32(Box_i32 self)"));
    assert!(c_source.contains("int32_t Box_value_or_i32(Box_i32 self, int32_t fallback)"));
    assert!(c_source.contains("Box_wrap_i32(self)"));
    assert!(c_source.contains("Box_value_or_i32(box, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_wrap_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_value_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T Box_wrap"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_type_method_nested_result_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("typedef struct Result_Option_i32_str Result_Option_i32_str;"));
    assert!(c_source.contains("Result_Option_i32_str Box_wrap_result_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_wrap_result_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_wrap_result_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("Result_Option_T"));
    assert!(!c_source.contains("T Box_wrap_result"));

    let c_source =
        compile_to_c_with_generated_call_check(&test_dir().join("generic_ufc_function.zen"));
    assert!(c_source.contains("int32_t id_i32(int32_t value)"));
    assert!(c_source.contains("id_i32(12LL)"));
    assert_c_call_resolves_to_definition(&c_source, "id_i32");
    assert!(!c_source.contains("id(12LL)"));
    assert!(!c_source.contains("T id"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("behavior_json_generic_bound_ufcs.zen"),
    );
    assert!(c_source.contains("Point Point_encode(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_behavior_bound/main.zen"),
    );
    assert!(c_source.contains("Point Point_encode(Point value)"));
    assert!(c_source.contains("Point encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_behavior_inheritance/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_behavior_impl/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_behavior_default/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_to_json(Point "));
    assert!(c_source.contains("zen_str render_Point(Point value)"));
    assert!(c_source.contains("Point_to_json(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_to_json");
    assert_c_call_resolves_to_definition(&c_source, "render_Point");
    assert!(!c_source.contains("T_to_json"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_generic_behavior_default/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point __arg0)"));
    assert!(c_source.contains("zen_str render_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "render_Point");
    assert!(!c_source.contains("Json_T"));
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("behavior_generic_default_method.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point __arg0)"));
    assert!(c_source.contains("Point_encode(point)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert!(!c_source.contains("Json_T"));
    assert!(!c_source.contains("T Point_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_impl_imported_behavior/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_child_parent_dispatch/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str render_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "render_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_behavior_requires/main.zen"),
    );
    assert!(c_source.contains("zen_str Point_encode(Point value)"));
    assert!(c_source.contains("zen_str encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_imported_behavior_bound/main.zen"),
    );
    assert!(c_source.contains("int32_t Point_encode(Point value)"));
    assert!(c_source.contains("int32_t encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_return_type_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Point"));
    assert!(c_source.contains("Point make_point(void)"));
    assert!(c_source.contains("int32_t Point_encode(Point value)"));
    assert!(c_source.contains("int32_t encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "make_point");
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_param_type_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Point"));
    assert!(c_source.contains("Point make_point(void)"));
    assert!(c_source.contains("int32_t Point_encode(Point value)"));
    assert!(c_source.contains("int32_t encode_point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert!(c_source.contains("encode_point(point)"));
    assert_c_call_resolves_to_definition(&c_source, "make_point");
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_function_imported_return_type_behavior/main.zen"),
    );
    assert!(c_source.contains("typedef struct Point"));
    assert!(c_source.contains("Point make_point(void)"));
    assert!(c_source.contains("int32_t Point_encode(Point value)"));
    assert!(c_source.contains("int32_t encode_Point(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert_c_call_resolves_to_definition(&c_source, "make_point");
    assert_c_call_resolves_to_definition(&c_source, "Point_encode");
    assert_c_call_resolves_to_definition(&c_source, "encode_Point");
    assert!(!c_source.contains("T_encode"));

    let c_source = compile_to_c_with_generated_call_check(
        &test_dir().join("multi_file_imported_generic_function_return_enum_dependency/main.zen"),
    );
    assert!(c_source.contains("typedef struct Option_i32"));
    assert!(c_source.contains("Option_i32 wrap_i32(int32_t value)"));
    assert!(c_source.contains("int32_t unwrap_i32(Option_i32 value, int32_t fallback)"));
    assert!(c_source.contains("wrap_i32(107LL)"));
    assert!(c_source.contains("unwrap_i32(value, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "wrap_i32");
    assert_c_call_resolves_to_definition(&c_source, "unwrap_i32");
    assert!(!c_source.contains("T wrap"));
    assert!(!c_source.contains("T unwrap"));
}

#[test]
fn check_command_runs_resolver_diagnostics() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unknown value symbol 'missing_local'"),
        "expected resolver diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_reports_imported_module_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    missing_dep_local
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

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", main_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("unknown value symbol 'missing_dep_local'"),
        "expected imported module resolver diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_reports_imported_module_type_diagnostics() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", main_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_deduplicates_typechecker_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let traits_path = tmp.path().join("traits.zen");
    std::fs::write(
        &traits_path,
        r#"
pub Json<T>: behavior {
    encode: (Self) T
}

pub Point: {
    x: i32
}
"#,
    )
    .expect("write traits module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ Json, Point } = traits

Point.requires(Json<str>)

main = () i32 {
    0
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", main_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let diagnostic = "type `Point` does not implement required behavior `Json_str`";
    assert!(
        stderr.contains(diagnostic),
        "expected missing behavior diagnostic, stderr={stderr}"
    );
    assert_eq!(
        stderr.matches(diagnostic).count(),
        1,
        "expected missing behavior diagnostic once, stderr={stderr}"
    );
}

#[test]
fn emit_command_reports_imported_module_type_diagnostics() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", main_path.to_str().unwrap()])
        .output()
        .expect("run zen emit");

    assert!(
        !output.status.success(),
        "zen emit unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn emit_json_ast_command_outputs_resolved_module_graph() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
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
    add(20, 22)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "ast", main_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json ast");

    assert!(
        output.status.success(),
        "zen emit-json ast failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("emit-json ast stdout is json");
    assert_eq!(json["format"], "zen.ast.v0");
    assert_eq!(json["entry_module"], 0);
    assert_eq!(json["modules"].as_array().expect("modules array").len(), 2);

    let entry = &json["modules"][0];
    assert_eq!(entry["id"], 0);
    assert_eq!(entry["imports"][0]["local_name"], "add");
    assert_eq!(entry["imports"][0]["source_symbol"], "add");
    assert!(
        entry["program"]["declarations"]
            .as_array()
            .expect("entry declarations")
            .iter()
            .any(|decl| decl["Function"]["name"] == "main"),
        "entry AST should contain main function: {entry}"
    );

    let imported = &json["modules"][1];
    assert_eq!(imported["id"], 1);
    assert!(
        imported["program"]["declarations"]
            .as_array()
            .expect("imported declarations")
            .iter()
            .any(|decl| decl["Function"]["name"] == "add"),
        "imported AST should contain add function: {imported}"
    );
}

#[test]
fn emit_json_typed_command_outputs_checked_program() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir().join("generic_method.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed");

    assert!(
        output.status.success(),
        "zen emit-json typed failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("emit-json typed stdout is json");
    assert_eq!(json["format"], "zen.typed.v0");

    let functions = json["program"]["functions"]
        .as_array()
        .expect("typed functions array");
    assert!(
        functions
            .iter()
            .any(|function| function["name"] == "Box.get_i32"),
        "typed JSON should contain specialized generic method: {json}"
    );
    assert!(
        functions.iter().any(|function| function["name"] == "main"),
        "typed JSON should contain main function: {json}"
    );

    let types = json["program"]["types"]
        .as_array()
        .expect("typed types array");
    assert!(
        types.iter().any(|ty| ty["name"] == "Box_i32"),
        "typed JSON should contain specialized generic type: {json}"
    );

    let serialized = String::from_utf8(output.stdout).expect("typed JSON is utf-8");
    assert!(!serialized.contains("Box_T"));
    assert!(!serialized.contains("T Box_get"));
}

#[test]
fn emit_json_symbols_command_outputs_module_symbol_tables() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
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
    add(20, 22)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "symbols", main_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json symbols");

    assert!(
        output.status.success(),
        "zen emit-json symbols failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("emit-json symbols stdout is json");
    assert_eq!(json["format"], "zen.symbols.v0");
    assert_eq!(json["entry_module"], 0);
    assert_eq!(json["modules"].as_array().expect("modules array").len(), 2);

    let entry_symbols = json["modules"][0]["symbols"]
        .as_array()
        .expect("entry symbols array");
    assert!(
        entry_symbols.iter().any(|symbol| {
            symbol["namespace"] == "Value"
                && symbol["name"] == "main"
                && symbol["return_type_name"] == "i32"
        }),
        "entry symbols should contain main value symbol: {json}"
    );
    assert!(
        entry_symbols.iter().any(|symbol| {
            symbol["namespace"] == "Import"
                && symbol["name"] == "add"
                && symbol["import_source"] == "math"
        }),
        "entry symbols should contain add import symbol: {json}"
    );

    let imported_symbols = json["modules"][1]["symbols"]
        .as_array()
        .expect("imported symbols array");
    assert!(
        imported_symbols.iter().any(|symbol| {
            symbol["namespace"] == "Value"
                && symbol["name"] == "add"
                && symbol["is_public"] == true
                && symbol["parameter_count"] == 2
                && symbol["return_type_name"] == "i32"
        }),
        "imported symbols should contain public add signature: {json}"
    );
}

#[test]
fn emit_json_diagnostics_command_outputs_machine_readable_errors() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("bad_type.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    true
}
"#,
    )
    .expect("write bad source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on errors: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("diagnostics stdout is json");
    assert_eq!(json["format"], "zen.diagnostics.v0");
    assert_eq!(json["files"].as_array().expect("files array").len(), 1);

    let diagnostic = &json["diagnostics"][0];
    assert_eq!(diagnostic["severity"], "error");
    assert_eq!(diagnostic["code"], "E3030");
    assert!(
        diagnostic["message"]
            .as_str()
            .expect("diagnostic message")
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "unexpected diagnostic payload: {diagnostic}"
    );

    let span = &diagnostic["span"];
    assert!(span["path"]
        .as_str()
        .expect("span path")
        .ends_with("bad_type.zen"));
    assert_eq!(span["line"], 3);
    assert_eq!(span["column"], 5);
}

#[test]
fn build_command_reports_imported_module_type_diagnostics() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build", main_path.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build");

    assert!(
        !output.status.success(),
        "zen build unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn build_command_rejects_build_zen_until_deterministic_graph_exists() {
    assert_build_zen_rejected(&["build"], "zen build build.zen");
}

#[test]
fn check_command_rejects_build_zen_until_deterministic_graph_exists() {
    assert_build_zen_rejected(&["check"], "zen check build.zen");
}

#[test]
fn emit_command_rejects_build_zen_until_deterministic_graph_exists() {
    assert_build_zen_rejected(&["emit"], "zen emit build.zen");
}

#[test]
fn direct_file_command_rejects_build_zen_until_deterministic_graph_exists() {
    assert_build_zen_rejected(&[], "zen build.zen");
}

#[test]
fn emit_json_build_graph_outputs_project_build_graph() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "build-graph", "examples/project/build.zen"])
        .output()
        .expect("run zen emit-json build-graph");

    assert!(
        output.status.success(),
        "emit-json build-graph failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("build graph json");
    assert_eq!(json["targets"][0]["name"], "myapp");
    assert_eq!(json["targets"][0]["kind"]["root_source_file"], "main.zen");
    assert_eq!(json["targets"][0]["kind"]["out_dir"], "build/");
}

#[test]
fn emit_json_build_graph_rejects_undeclared_host_effects() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "build-graph", build_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json build-graph");

    assert!(
        !output.status.success(),
        "emit-json build-graph unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("undeclared host effect: read env `ZEN_STD`"),
        "expected undeclared host effect diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_build_zen_rejected(prefix_args: &[&str], command_name: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write build.zen");

    let mut args = prefix_args.to_vec();
    args.push(build_path.to_str().unwrap());
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("run {command_name}: {err}"));

    assert!(
        !output.status.success(),
        "{command_name} unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "build.zen execution is gated until deterministic build graph support exists"
        ),
        "expected build.zen gated diagnostic for {command_name}, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

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
