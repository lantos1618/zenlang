use super::*;

mod association_declarations;
mod association_inheritance;
mod default_methods;
mod impl_methods;
mod impl_signatures;

#[test]
fn typechecker_behavior_impl_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/typechecker_behavior_impls.rs");
    let association_declarations = read(
        "tests/docs_truth/repo_hygiene/typechecker_behavior_impls/association_declarations.rs",
    );
    let association_inheritance =
        read("tests/docs_truth/repo_hygiene/typechecker_behavior_impls/association_inheritance.rs");
    let default_methods =
        read("tests/docs_truth/repo_hygiene/typechecker_behavior_impls/default_methods.rs");
    let impl_methods =
        read("tests/docs_truth/repo_hygiene/typechecker_behavior_impls/impl_methods.rs");
    let impl_signatures =
        read("tests/docs_truth/repo_hygiene/typechecker_behavior_impls/impl_signatures.rs");

    assert!(
        root.lines().count() < 80,
        "typechecker_behavior_impls.rs should route focused hygiene guard modules"
    );
    for module_name in [
        "association_declarations",
        "association_inheritance",
        "default_methods",
        "impl_methods",
        "impl_signatures",
    ] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "typechecker_behavior_impls.rs should include focused module: {module_name}"
        );
    }
    assert!(
        association_declarations
            .contains("fn behavior_association_declaration_tasks_live_in_focused_helper"),
        "association declaration guards should live in association_declarations.rs"
    );
    assert!(
        association_inheritance
            .contains("fn behavior_association_inheritance_lives_in_focused_helper"),
        "association inheritance guards should live in association_inheritance.rs"
    );
    assert!(
        default_methods.contains("fn behavior_default_method_synthesis_lives_in_focused_helper"),
        "default method guards should live in default_methods.rs"
    );
    assert!(
        impl_methods.contains("fn behavior_impl_method_validation_lives_in_focused_helper"),
        "impl method guards should live in impl_methods.rs"
    );
    assert!(
        impl_signatures.contains("fn behavior_impl_signature_collection_lives_in_focused_helper"),
        "impl signature guards should live in impl_signatures.rs"
    );
}
