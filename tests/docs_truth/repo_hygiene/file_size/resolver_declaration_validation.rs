use super::super::*;

#[test]
fn resolver_callable_impl_block_validation_stays_split_by_responsibility() {
    let root = read("src/resolver/declaration_validation/callables.rs");
    let impl_blocks = read("src/resolver/declaration_validation/callables/impl_blocks.rs");

    assert!(
        root.lines().count() < 170,
        "callables.rs should stay focused on function and method declaration validation"
    );
    assert!(
        root.contains("mod impl_blocks;"),
        "callables.rs should include the focused impl-block validation helper"
    );
    assert!(
        root.contains("pub(super) use impl_blocks::ImplBlockDeclarationValidation;"),
        "callables.rs should re-export the impl-block validation input from the focused helper"
    );

    for helper in [
        "struct ImplBehaviorAssociationValidation",
        "fn validate_impl_block_declaration",
        "fn validate_impl_type_name",
        "fn validate_impl_behavior_association",
        "fn validate_impl_methods",
    ] {
        assert!(
            !root.contains(helper),
            "callables.rs should not own impl-block validation helper: {helper}"
        );
        assert!(
            impl_blocks.contains(helper),
            "impl-block validation helper should live in impl_blocks.rs: {helper}"
        );
    }
}
