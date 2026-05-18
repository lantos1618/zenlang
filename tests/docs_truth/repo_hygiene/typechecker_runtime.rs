use super::*;

#[test]
fn root_std_runtime_calls_use_owned_intrinsic_enum() {
    let call_validation = read("src/typechecker/expressions/call_validation.rs");
    let runtime_calls = read("src/typechecker/std_runtime_calls.rs");

    for forbidden in [
        r#"("io", "print")"#,
        r#"("io", "println")"#,
        r#"matches!((module, function)"#,
        r#"module == "io""#,
        r#"function == "print""#,
        r#"function == "println""#,
    ] {
        assert!(
            !call_validation.contains(forbidden),
            "root std runtime call checks should parse through StdRuntimeCall, not raw spelling checks: {forbidden}"
        );
    }

    for required in [
        "enum StdRuntimeCall",
        "const IO: &'static str = \"io\"",
        "const PRINT: &'static str = \"print\"",
        "const PRINTLN: &'static str = \"println\"",
        "const ALL: &[StdRuntimeCall]",
        "impl fmt::Display for StdRuntimeCall",
        ".find(|call| call.module() == module && call.function() == function)",
        "pub(super) fn parse_std_runtime_call",
    ] {
        assert!(
            runtime_calls.contains(required),
            "root std runtime spelling should live in StdRuntimeCall: {required}"
        );
    }
}
