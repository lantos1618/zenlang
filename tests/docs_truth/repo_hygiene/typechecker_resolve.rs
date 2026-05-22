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
fn typechecker_type_compatibility_lives_in_focused_helper() {
    let root = read("src/typechecker/resolve.rs");
    let compatibility = read("src/typechecker/resolve/compatibility.rs");

    assert!(
        !root.contains("fn types_compatible("),
        "resolve.rs should not own type compatibility checking"
    );
    assert!(
        compatibility.contains("fn types_compatible("),
        "type compatibility checking should live in focused resolve helper"
    );
    assert!(
        root.contains("mod compatibility;"),
        "type resolution should load the focused compatibility helper"
    );
    assert!(
        root.lines().count() < 210,
        "resolve.rs should stay focused on AstType resolution and field lookup"
    );
}
