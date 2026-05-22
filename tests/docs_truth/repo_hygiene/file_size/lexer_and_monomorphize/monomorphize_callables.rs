use super::*;

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
