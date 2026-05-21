use super::super::*;

#[test]
fn lexer_string_interpolation_lives_in_focused_helper() {
    let strings = read("src/lexer/strings.rs");
    let interpolation = read("src/lexer/string_interpolation.rs");
    let lexer_module = read("src/lexer/mod.rs");

    assert!(
        strings.lines().count() < 160,
        "strings.rs should stay focused on literal string scanning"
    );
    assert!(
        !strings.contains("fn lex_interpolation_body"),
        "string interpolation body scanning should live in string_interpolation.rs"
    );
    assert!(
        interpolation.contains("fn lex_interpolation_body"),
        "string_interpolation.rs should scan interpolation bodies"
    );
    assert!(
        interpolation.contains("fn lex_next_no_skip"),
        "string_interpolation.rs should own no-skip token scanning for interpolation bodies"
    );
    assert!(
        lexer_module.contains("mod string_interpolation;"),
        "lexer module should include the focused string_interpolation helper"
    );
}

#[test]
fn lexer_number_scanning_lives_in_focused_helper() {
    let scan = read("src/lexer/scan.rs");
    let numbers = read("src/lexer/numbers.rs");
    let lexer_module = read("src/lexer/mod.rs");

    assert!(
        scan.lines().count() < 220,
        "scan.rs should stay focused on token dispatch and small token scanners"
    );
    assert!(
        !scan.contains("fn lex_prefixed_int"),
        "prefixed integer scanning should live in numbers.rs"
    );
    assert!(
        !scan.contains("fn eat_digits"),
        "digit scanning should live in numbers.rs"
    );
    assert!(
        numbers.contains("pub(super) fn lex_number"),
        "numbers.rs should own number token scanning"
    );
    assert!(
        numbers.contains("fn lex_prefixed_int"),
        "numbers.rs should own prefixed integer scanning"
    );
    assert!(
        lexer_module.contains("mod numbers;"),
        "lexer module should include the focused number scanning helper"
    );
}

#[test]
fn monomorphize_type_substitution_lives_in_focused_helper() {
    let monomorphize = read("src/typechecker/monomorphize.rs");
    let names = read("src/typechecker/monomorphize_names.rs");
    let substitution = read("src/typechecker/monomorphize_substitution.rs");
    let module = read("src/typechecker/mod.rs");

    assert!(
        monomorphize.lines().count() < 240,
        "monomorphize.rs should stay focused on callable specialization"
    );
    assert!(
        !monomorphize.contains("pub(crate) fn substitute_type"),
        "type substitution should live in monomorphize_substitution.rs"
    );
    assert!(
        names.contains("pub(crate) fn generic_function_mangled_name"),
        "monomorphize_names.rs should own generic callable mangling"
    );
    assert!(
        names.contains("pub(crate) fn mangle_generic_type_name"),
        "monomorphize_names.rs should own generic type mangling"
    );
    assert!(
        substitution.contains("pub(crate) fn substitute_type"),
        "monomorphize_substitution.rs should own generic AstType substitution"
    );
    assert!(
        module.contains("mod monomorphize_substitution;"),
        "typechecker module should include the focused monomorphize_substitution helper"
    );
    assert!(
        module.contains("mod monomorphize_names;"),
        "typechecker module should include the focused monomorphize_names helper"
    );
}

#[test]
fn monomorphize_specialized_type_ref_reconstruction_lives_in_focused_helper() {
    let specialized_types = read("src/typechecker/monomorphize_specialized_types.rs");
    let type_refs = read("src/typechecker/monomorphize_type_refs.rs");
    let specialized_type_refs = read("src/typechecker/monomorphize_specialized_type_refs.rs");
    let module = read("src/typechecker/mod.rs");

    assert!(
        specialized_types.lines().count() < 130,
        "monomorphize_specialized_types.rs should stay focused on emitting specialized type definitions"
    );
    assert!(
        !specialized_types.contains("fn generic_type_args_from_type"),
        "generic type-argument reconstruction should live in monomorphize_type_refs.rs"
    );
    for helper in [
        "ensure_specialized_type_refs",
        "ensure_specialized_type_refs_for_type",
    ] {
        assert!(
            !specialized_types.contains(&format!("fn {helper}")),
            "recursive specialized type-reference discovery should live in focused helper: {helper}"
        );
        assert!(
            specialized_type_refs.contains(&format!("fn {helper}")),
            "monomorphize_specialized_type_refs.rs should own helper: {helper}"
        );
    }
    assert!(
        type_refs.contains("pub(crate) fn generic_type_args_from_type"),
        "monomorphize_type_refs.rs should own generic type-argument reconstruction"
    );
    assert!(
        type_refs.contains("pub(crate) fn type_to_ast_ref"),
        "monomorphize_type_refs.rs should own Type-to-AstType reconstruction"
    );
    assert!(
        module.contains("mod monomorphize_type_refs;"),
        "typechecker module should include the focused monomorphize_type_refs helper"
    );
    assert!(
        module.contains("mod monomorphize_specialized_type_refs;"),
        "typechecker module should include focused specialized type-reference discovery"
    );
}

#[test]
fn monomorphize_generic_method_self_type_lives_in_focused_helper() {
    let monomorphize = read("src/typechecker/monomorphize.rs");
    let method_self = read("src/typechecker/monomorphize_method_self.rs");
    let type_args = read("src/typechecker/monomorphize_type_args.rs");
    let module = read("src/typechecker/mod.rs");

    assert!(
        monomorphize.lines().count() < 170,
        "monomorphize.rs should stay focused on callable specialization"
    );
    for helper in ["generic_method_self_type", "generic_receiver_self_type"] {
        assert!(
            !monomorphize.contains(&format!("fn {helper}")),
            "callable specialization should not own generic method Self helper: {helper}"
        );
        assert!(
            method_self.contains(&format!("fn {helper}")),
            "generic method Self reconstruction should live in focused helper: {helper}"
        );
    }
    assert!(
        module.contains("mod monomorphize_method_self;"),
        "typechecker module should include focused generic method Self helper"
    );
    for helper in [
        "reject_missing_generic_substitutions",
        "type_param_substitutions",
    ] {
        assert!(
            !monomorphize.contains(&format!("fn {helper}")),
            "callable specialization should not own generic type-argument helper: {helper}"
        );
        assert!(
            type_args.contains(&format!("fn {helper}")),
            "generic type-argument substitution diagnostics should live in focused helper: {helper}"
        );
    }
    assert!(
        module.contains("mod monomorphize_type_args;"),
        "typechecker module should include focused generic type-argument helper"
    );
}

#[test]
fn monomorphize_generic_aggregate_inference_lives_in_focused_helper() {
    let inference = read("src/typechecker/monomorphize_inference.rs");
    let generic_types = read("src/typechecker/monomorphize_inference_types.rs");
    let module = read("src/typechecker/mod.rs");

    assert!(
        inference.lines().count() < 160,
        "monomorphize_inference.rs should stay focused on inference entry points and recursive matching"
    );
    assert!(
        !inference.contains("fn match_generic_type_params"),
        "generic aggregate type inference should live in focused helper"
    );
    assert!(
        generic_types.contains("fn match_generic_type_params"),
        "focused generic aggregate inference helper should match struct and enum type params"
    );
    assert!(
        generic_types.contains("Type::Struct") && generic_types.contains("Type::Enum"),
        "focused generic aggregate inference helper should cover structs and enums"
    );
    assert!(
        module.contains("mod monomorphize_inference_types;"),
        "typechecker module should include focused generic aggregate inference helper"
    );
}
