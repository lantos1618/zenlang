use super::*;

#[test]
fn parser_function_and_top_level_binding_forms_live_in_focused_helper() {
    let declarations = read("src/parser/declarations.rs");
    let function_forms = read("src/parser/declarations/function_forms.rs");

    for helper in [
        "parse_function_def",
        "parse_function_signature_and_body",
        "parse_param_list",
        "parse_const_decl",
        "parse_var_decl_toplevel",
    ] {
        assert!(
            !declarations.contains(&format!("fn {helper}")),
            "parser declaration dispatch should not own function/top-level binding helper: {helper}"
        );
        assert!(
            function_forms.contains(&format!("fn {helper}")),
            "function and top-level binding parsing should live in the focused helper: {helper}"
        );
    }

    assert!(
        declarations.contains("mod function_forms;"),
        "parser declaration dispatch should load the focused function-forms helper"
    );
}
