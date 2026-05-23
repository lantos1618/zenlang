use super::super::*;

#[test]
fn imported_generic_diagnostics_pin_stable_codes() {
    let support = read("tests/integration/frontend_diagnostics/support.rs");
    let arity = read("tests/integration/frontend_diagnostics/imported_generic_arity.rs");
    let calls = read("tests/integration/frontend_diagnostics/imported_generic_calls.rs");

    assert!(
        support.contains("fn frontend_diagnostics("),
        "frontend diagnostics tests should inspect real Diagnostic values"
    );
    assert!(
        support.contains("fn assert_diagnostic_code_and_message("),
        "frontend diagnostics tests should share a code-plus-message assertion helper"
    );

    for source in [arity.as_str(), calls.as_str()] {
        assert!(
            !source.contains("compile_to_c_panic_message"),
            "imported generic diagnostics should not rely on panic-message substrings"
        );
        assert!(
            source.contains("frontend_diagnostics(&main_path)"),
            "imported generic diagnostics should collect real Diagnostic values"
        );
    }

    for (source, code) in [
        (arity.as_str(), "E5001"),
        (calls.as_str(), "E5000"),
        (calls.as_str(), "E5001"),
        (calls.as_str(), "E5002"),
        (calls.as_str(), "E6004"),
    ] {
        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            normalized.contains(&format!(
                r#"assert_diagnostic_code_and_message(&diagnostics,"{code}""#
            )),
            "imported generic diagnostics should pin diagnostic code {code}"
        );
    }
}
