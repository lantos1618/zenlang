use super::*;

mod expression_emission;
mod function_emission;
mod generated_c_support;

#[test]
fn codegen_c_hygiene_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/codegen_c.rs");
    let expression_emission =
        read("tests/docs_truth/repo_hygiene/codegen_c/expression_emission.rs");
    let function_emission = read("tests/docs_truth/repo_hygiene/codegen_c/function_emission.rs");
    let generated_c_support =
        read("tests/docs_truth/repo_hygiene/codegen_c/generated_c_support.rs");

    assert!(
        root.lines().count() < 80,
        "codegen_c.rs should route focused C-codegen hygiene guard modules"
    );
    for module_name in [
        "expression_emission",
        "function_emission",
        "generated_c_support",
    ] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "codegen_c.rs should include focused guard module: {module_name}"
        );
    }
    assert!(
        expression_emission
            .contains("fn codegen_c_expression_operator_spelling_lives_in_focused_helper"),
        "C expression helper guards should live in expression_emission.rs"
    );
    assert!(
        function_emission.contains("fn codegen_c_function_emission_lives_in_focused_helper"),
        "C function/type helper guards should live in function_emission.rs"
    );
    assert!(
        generated_c_support
            .contains("fn generated_c_test_support_splits_definition_and_call_scanning"),
        "generated-C support guards should live in generated_c_support.rs"
    );
}
