use super::*;

#[test]
fn typechecker_binary_op_checking_lives_in_focused_helper() {
    let root = read("src/typechecker/resolve.rs");
    let binary_ops = read("src/typechecker/resolve_binary_ops.rs");
    let module = read("src/typechecker/mod.rs");

    for helper in [
        "check_binary_op",
        "check_arithmetic_binary_op",
        "check_logical_binary_op",
        "check_bitwise_binary_op",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "type resolution root should not own binary operator helper: {helper}"
        );
        assert!(
            binary_ops.contains(&format!("fn {helper}")),
            "binary operator checking should live in focused helper: {helper}"
        );
    }

    assert!(
        module.contains("mod resolve_binary_ops;"),
        "typechecker root should include focused binary operator checking module"
    );
}

#[test]
fn typechecker_type_resolution_uses_named_and_generic_helpers() {
    let root = read("src/typechecker/resolve.rs");

    for helper in [
        "resolve_named_type",
        "resolve_generic_type",
        "resolve_struct_type",
        "resolve_enum_type",
    ] {
        assert!(
            root.contains(&format!("fn {helper}")),
            "type resolution should route aggregate construction through focused helper: {helper}"
        );
    }

    let named_branch = root
        .split("AstType::Named(name) =>")
        .nth(1)
        .and_then(|tail| tail.split("AstType::Generic").next())
        .expect("expected named type branch before generic type branch");
    assert!(
        named_branch.contains("self.resolve_named_type(name)"),
        "named type branch should delegate to resolve_named_type"
    );

    let generic_branch = root
        .split("AstType::Generic { name, type_args } =>")
        .nth(1)
        .and_then(|tail| tail.split("AstType::Ptr").next())
        .expect("expected generic type branch before pointer branch");
    assert!(
        generic_branch.contains("self.resolve_generic_type(name, type_args)"),
        "generic type branch should delegate to resolve_generic_type"
    );
}

#[test]
fn generic_type_reference_walker_bounds_live_in_focused_helper() {
    let root = read("src/typechecker/generic_type_reference_walker.rs");
    let type_refs = read("src/typechecker/generic_type_reference_walker/type_refs.rs");

    assert!(
        root.lines().count() < 160,
        "generic_type_reference_walker.rs should stay focused on public traversal entry points"
    );
    assert!(
        root.contains("mod type_refs;"),
        "generic type-reference walker should include the focused type_refs helper"
    );
    assert!(
        !root.contains("fn validate_generic_type_ref_bounds_with_unknowns"),
        "recursive generic type-ref bound validation should live in type_refs.rs"
    );
    assert!(
        type_refs.contains("pub(super) fn validate_generic_type_ref_bounds_with_unknowns"),
        "type_refs.rs should own recursive generic type-ref bound validation"
    );
    assert!(
        type_refs.contains("fn is_known_named_type"),
        "type_refs.rs should own named type lookup for generic type-ref validation"
    );
}
