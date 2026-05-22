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
fn monomorphize_specialized_type_name_recovery_lives_in_focused_helper() {
    let specialized_types = read("src/typechecker/monomorphize_specialized_types.rs");
    let specialized_type_names = read("src/typechecker/monomorphize_specialized_type_names.rs");
    let module = read("src/typechecker/mod.rs");

    assert!(
        specialized_types.lines().count() < 210,
        "monomorphize_specialized_types.rs should stay focused on emitting specialized definitions"
    );
    assert!(
        !specialized_types.contains("fn generic_type_args_from_type"),
        "specialized type-name recovery should live in monomorphize_specialized_type_names.rs"
    );
    assert!(
        specialized_type_names.contains("fn generic_type_args_from_type"),
        "monomorphize_specialized_type_names.rs should recover generic args from specialized type names"
    );
    assert!(
        specialized_type_names.contains("pub(crate) fn type_to_ast_ref"),
        "monomorphize_specialized_type_names.rs should own Type-to-AstType recovery for monomorphization"
    );
    assert!(
        module.contains("mod monomorphize_specialized_type_names;"),
        "typechecker module should include the focused specialized type-name helper"
    );
}
