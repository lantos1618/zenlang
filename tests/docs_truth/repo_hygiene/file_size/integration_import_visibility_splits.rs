use super::super::*;

#[test]
fn import_visibility_dependency_panic_capture_stays_shared() {
    let root = read("tests/integration/import_visibility_dependencies.rs");
    let imported_type =
        read("tests/integration/import_visibility_dependencies/imported_type_dependencies.rs");

    assert_eq!(
        root.matches("fn compile_error_message(").count(),
        1,
        "import visibility dependency tests should share compile-error panic capture"
    );
    assert_eq!(
        root.matches("std::panic::catch_unwind").count(),
        1,
        "compile-error panic capture should not be repeated across each dependency test"
    );
    assert_eq!(
        root.matches(".downcast_ref::<String>()").count(),
        1,
        "panic downcast boilerplate should live in the shared helper"
    );
    for test_name in [
        "imported_type_impl_imported_type_dependencies_are_not_directly_visible",
        "imported_generic_function_imported_type_dependencies_are_not_directly_visible",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "import_visibility_dependencies.rs should not own imported-type dependency test: {test_name}"
        );
        assert!(
            imported_type.contains(&format!("fn {test_name}")),
            "imported-type dependency visibility tests should live in focused helper: {test_name}"
        );
    }
    assert!(
        root.lines().count() < 150,
        "import_visibility_dependencies.rs should stay focused on fixture shape, not repeated panic plumbing"
    );
    assert!(
        root.contains("#[path = \"import_visibility_dependencies/imported_type_dependencies.rs\"]"),
        "import_visibility_dependencies.rs should include the focused imported_type_dependencies module by path"
    );
}
