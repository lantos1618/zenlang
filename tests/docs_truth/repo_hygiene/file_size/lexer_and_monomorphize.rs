use super::super::*;

mod lexer;
mod monomorphize_callables;
mod monomorphize_types;

#[test]
fn lexer_and_monomorphize_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/file_size/lexer_and_monomorphize.rs");
    let lexer = read("tests/docs_truth/repo_hygiene/file_size/lexer_and_monomorphize/lexer.rs");
    let monomorphize_callables = read(
        "tests/docs_truth/repo_hygiene/file_size/lexer_and_monomorphize/monomorphize_callables.rs",
    );
    let monomorphize_types = read(
        "tests/docs_truth/repo_hygiene/file_size/lexer_and_monomorphize/monomorphize_types.rs",
    );

    assert!(
        root.lines().count() < 80,
        "lexer_and_monomorphize.rs should route focused file-size guard modules"
    );
    for module_name in ["lexer", "monomorphize_callables", "monomorphize_types"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "lexer_and_monomorphize.rs should include focused guard module: {module_name}"
        );
    }
    assert!(
        lexer.contains("fn lexer_string_interpolation_lives_in_focused_helper"),
        "lexer guards should live in lexer_and_monomorphize/lexer.rs"
    );
    assert!(
        monomorphize_callables
            .contains("fn monomorphize_generic_method_self_type_lives_in_focused_helper"),
        "callable monomorphization guards should live in lexer_and_monomorphize/monomorphize_callables.rs"
    );
    assert!(
        monomorphize_types.contains(
            "fn monomorphize_specialized_type_ref_reconstruction_lives_in_focused_helper"
        ),
        "type monomorphization guards should live in lexer_and_monomorphize/monomorphize_types.rs"
    );
}
