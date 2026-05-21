use super::super::*;

#[test]
fn compiler_intrinsic_registry_lives_in_focused_helper() {
    let root = read("src/intrinsics.rs");
    let registry = read("src/intrinsics/registry.rs");

    assert!(
        root.lines().count() < 130,
        "intrinsics.rs should stay focused on module recognition and public lookup APIs"
    );
    assert!(
        root.contains("mod registry;"),
        "intrinsics.rs should include the focused registry builder"
    );
    assert!(
        !root.contains("fn build_intrinsics"),
        "intrinsics.rs should not own compiler intrinsic registry construction"
    );
    assert!(
        registry.contains("fn build_intrinsics"),
        "registry.rs should own compiler intrinsic registry construction"
    );
    assert!(
        registry.contains("macro_rules! intrinsic"),
        "registry.rs should own compact intrinsic registration syntax"
    );
}
