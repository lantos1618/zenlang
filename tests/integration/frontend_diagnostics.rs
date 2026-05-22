use super::*;

#[path = "frontend_diagnostics/behavior_extends/mod.rs"]
mod behavior_extends;
#[path = "frontend_diagnostics/generic_behavior_imports.rs"]
mod generic_behavior_imports;
#[path = "frontend_diagnostics/imported_generic_arity.rs"]
mod imported_generic_arity;
#[path = "frontend_diagnostics/support.rs"]
mod support;

use support::{compile_to_c_panic_message, write_tmp_module};

#[test]
fn integration_frontend_helper_runs_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = write_tmp_module(
        tmp.path(),
        "bad_resolver_ref.zen",
        r#"
main = () i32 {
    missing_local
}
"#,
    );
    let message = compile_to_c_panic_message(&zen_path);

    assert!(
        message.contains("unknown value symbol 'missing_local'"),
        "expected resolver diagnostic, panic={message}"
    );
}

#[test]
fn integration_frontend_helper_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_tmp_module(
        tmp.path(),
        "math.zen",
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    true
}
"#,
    );
    let main_path = write_tmp_module(
        tmp.path(),
        "main.zen",
        r#"
{ add } = math

main = () i32 {
    add(1, 2)
}
"#,
    );
    let message = compile_to_c_panic_message(&main_path);

    assert!(
        message.contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, panic={message}"
    );
}
