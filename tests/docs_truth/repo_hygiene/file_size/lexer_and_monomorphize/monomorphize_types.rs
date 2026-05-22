use super::*;

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
