use super::*;

mod intrinsics;
mod methods;

#[test]
fn typechecker_gate_enum_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/parser_enums/typechecker_gates.rs");
    let methods = read("tests/docs_truth/repo_hygiene/parser_enums/typechecker_gates/methods.rs");
    let intrinsics =
        read("tests/docs_truth/repo_hygiene/parser_enums/typechecker_gates/intrinsics.rs");

    assert!(
        root.lines().count() < 80,
        "typechecker_gates.rs should route focused enum hygiene guard modules"
    );
    for module_name in ["intrinsics", "methods"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "typechecker_gates.rs should include focused module: {module_name}"
        );
    }
    assert!(
        methods.contains("fn typechecker_gated_methods_use_owned_action_enum"),
        "gated method guards should live in typechecker_gates/methods.rs"
    );
    assert!(
        intrinsics.contains("fn typechecker_gated_intrinsics_use_owned_name_enum"),
        "gated intrinsic guards should live in typechecker_gates/intrinsics.rs"
    );
}
