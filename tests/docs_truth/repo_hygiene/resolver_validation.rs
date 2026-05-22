use super::*;

#[test]
fn resolver_behavior_association_validation_lives_in_focused_helper() {
    let declaration_validation = read("src/resolver/declaration_validation.rs");
    let behavior_associations =
        read("src/resolver/declaration_validation/behavior_associations.rs");

    for validation_helper in [
        "validate_requires_declaration",
        "validate_derive_declaration",
        "validate_behavior_extends_declaration",
    ] {
        assert!(
            !declaration_validation.contains(&format!("fn {validation_helper}")),
            "main resolver declaration validation should not own behavior-association helper: {validation_helper}"
        );
        assert!(
            behavior_associations.contains(&format!("fn {validation_helper}")),
            "behavior association declaration validation should live in the focused helper: {validation_helper}"
        );
    }

    for dispatch_call in [
        "self.validate_requires_declaration",
        "self.validate_derive_declaration",
        "self.validate_behavior_extends_declaration",
    ] {
        assert!(
            declaration_validation.contains(dispatch_call),
            "main resolver declaration validation should dispatch behavior-association validation through helper: {dispatch_call}"
        );
    }
}

#[test]
fn resolver_type_declaration_validation_lives_in_focused_helper() {
    let declaration_validation = read("src/resolver/declaration_validation.rs");
    let type_declarations = read("src/resolver/declaration_validation/type_declarations.rs");

    for validation_helper in [
        "validate_struct_declaration",
        "validate_enum_declaration",
        "validate_behavior_declaration",
    ] {
        assert!(
            !declaration_validation.contains(&format!("fn {validation_helper}")),
            "main resolver declaration validation should not own type declaration helper: {validation_helper}"
        );
        assert!(
            type_declarations.contains(&format!("fn {validation_helper}")),
            "type declaration validation should live in focused helper: {validation_helper}"
        );
    }

    assert!(
        declaration_validation.lines().count() < 250,
        "declaration_validation.rs should stay a dispatcher plus small declaration-specific checks"
    );

    for dispatch_call in [
        "self.validate_struct_declaration",
        "self.validate_enum_declaration",
        "self.validate_behavior_declaration",
    ] {
        assert!(
            declaration_validation.contains(dispatch_call),
            "main resolver declaration validation should dispatch type declaration validation through helper: {dispatch_call}"
        );
    }
}

#[test]
fn resolver_impl_block_validation_lives_in_focused_helper() {
    let declaration_validation = read("src/resolver/declaration_validation.rs");
    let impl_blocks = read("src/resolver/declaration_validation/impl_blocks.rs");

    for owned_detail in [
        "table.record_behavior_impl",
        "duplicate behavior implementation",
        "BehaviorRefMetadata",
    ] {
        assert!(
            !declaration_validation.contains(owned_detail),
            "main resolver declaration validation should not own impl-block detail: {owned_detail}"
        );
        assert!(
            impl_blocks.contains(owned_detail),
            "resolver impl-block validation should live in focused helper: {owned_detail}"
        );
    }

    assert!(
        declaration_validation.contains("self.validate_impl_block_declaration"),
        "main resolver declaration validation should dispatch impl-block validation through helper"
    );
    assert!(
        impl_blocks.contains("pub(super) fn validate_impl_block_declaration"),
        "impl-block helper should own the resolver impl-block validation entry point"
    );
    assert!(
        declaration_validation.lines().count() < 180,
        "declaration_validation.rs should stay compact after impl-block validation split"
    );
}
