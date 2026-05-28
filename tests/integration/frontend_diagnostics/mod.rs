mod behavior_extends;
mod generic_behavior_imports;
mod imported_generic_arity;
mod imported_generic_calls;
mod support;

use support::{assert_diagnostic_code_and_message, frontend_diagnostics_for_modules};

#[test]
fn integration_frontend_helper_runs_resolver_diagnostics() {
    let diagnostics = frontend_diagnostics_for_modules(
        &[],
        r#"
main = () i32 {
    missing_local
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E3500",
        "unknown value symbol 'missing_local'",
        "resolver",
    );
}

#[test]
fn integration_frontend_helper_reports_imported_module_type_diagnostics() {
    let diagnostics = frontend_diagnostics_for_modules(
        &[(
            "math.zen",
            r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}

pub broken = () i32 {
    true
}
"#,
        )],
        r#"
{ add } = math

main = () i32 {
    add(1, 2)
}
"#,
    );

    assert_diagnostic_code_and_message(
        &diagnostics,
        "E3030",
        "return type mismatch: expected `i32`, found `bool`",
        "imported module type",
    );
}
