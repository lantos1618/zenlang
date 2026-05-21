use super::*;

#[test]
fn typechecker_ast_behavior_collection_lives_in_focused_helper() {
    let ast_collection = read("src/typechecker/declaration_collection_ast.rs");
    let behavior_collection = read("src/typechecker/declaration_collection_ast_behaviors.rs");

    for helper in [
        "collect_behavior_declaration_tasks",
        "push_behavior_declaration_task",
        "collect_behavior_declarations_from_tasks",
        "collect_ast_behavior_declaration_signature",
        "collect_resolver_backed_behavior_declaration_stub",
        "validate_ast_precollection_tasks",
        "collect_ast_precollection_validation_tasks",
        "push_behavior_extends_replay_task",
    ] {
        assert!(
            !ast_collection.contains(&format!("fn {helper}")),
            "AST declaration collection should not own behavior helper: {helper}"
        );
        assert!(
            behavior_collection.contains(&format!("fn {helper}")),
            "AST behavior collection should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/mod.rs");
    assert!(
        root.contains("mod declaration_collection_ast_behaviors;"),
        "typechecker root should load focused AST behavior collection"
    );
}

#[test]
fn typechecker_resolver_callable_replay_lives_in_focused_helper() {
    let replay_kinds =
        read("src/typechecker/declaration_collection_resolver_tasks/replay_kinds.rs");
    let callable_replay =
        read("src/typechecker/declaration_collection_resolver_tasks/callables.rs");

    for helper in [
        "collect_resolver_callable_declaration_metadata_tasks",
        "push_resolver_callable_replay_tasks",
        "push_resolver_callable_metadata_task",
    ] {
        assert!(
            !replay_kinds.contains(&format!("fn {helper}")),
            "resolver replay kinds should not own callable helper: {helper}"
        );
        assert!(
            callable_replay.contains(&format!("fn {helper}")),
            "resolver callable replay should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/declaration_collection_resolver_tasks.rs");
    assert!(
        root.contains("mod callables;"),
        "resolver declaration collection should load focused callable replay module"
    );
}

#[test]
fn typechecker_resolver_type_reference_replay_lives_in_focused_helper() {
    let replay_kinds =
        read("src/typechecker/declaration_collection_resolver_tasks/replay_kinds.rs");
    let type_references =
        read("src/typechecker/declaration_collection_resolver_tasks/type_references.rs");

    for helper in [
        "collect_resolver_type_reference_validation_tasks",
        "push_resolver_type_reference_validation_task",
    ] {
        assert!(
            !replay_kinds.contains(&format!("fn {helper}")),
            "resolver replay kinds should not own fallback type-reference helper: {helper}"
        );
        assert!(
            type_references.contains(&format!("fn {helper}")),
            "resolver type-reference replay should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/declaration_collection_resolver_tasks.rs");
    assert!(
        root.contains("mod type_references;"),
        "resolver declaration collection should load focused type-reference replay module"
    );
    assert!(
        replay_kinds.lines().count() < 180,
        "resolver replay kinds should stay focused on metadata replay dispatch"
    );
}

#[test]
fn declaration_callable_tasks_live_in_focused_helper() {
    let root = read("src/typechecker/declaration_tasks.rs");
    let callables = read("src/typechecker/declaration_tasks_callables.rs");

    for helper in [
        "ResolverCallableSignature",
        "ResolverTypeParameterMetadata",
        "ResolverCallableDeclarationMetadataTask",
        "CallableDeclarationTask",
    ] {
        assert!(
            !root.contains(&format!("struct {helper}"))
                && !root.contains(&format!("enum {helper}")),
            "declaration_tasks.rs should not own callable task shape: {helper}"
        );
        assert!(
            callables.contains(&format!("struct {helper}"))
                || callables.contains(&format!("enum {helper}")),
            "callable declaration task shape should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("include!(\"declaration_tasks_callables.rs\");"),
        "declaration tasks should include focused callable task shapes"
    );
    assert!(
        root.lines().count() < 260,
        "declaration_tasks.rs should stay focused on shared declaration task wiring"
    );
}

#[test]
fn declaration_type_reference_tasks_live_in_focused_helper() {
    let root = read("src/typechecker/declaration_tasks.rs");
    let type_refs = read("src/typechecker/declaration_tasks_type_references.rs");

    for helper in [
        "AstTypeReferenceValidationTask",
        "SelfTypeContextValidationTask",
        "ResolverTypeReferenceValidationTask",
    ] {
        assert!(
            !root.contains(&format!("enum {helper}")),
            "declaration_tasks.rs should not own type/self-reference task shape: {helper}"
        );
        assert!(
            type_refs.contains(&format!("enum {helper}")),
            "type/self-reference declaration task shape should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("include!(\"declaration_tasks_type_references.rs\");"),
        "declaration tasks should include focused type-reference task shapes"
    );
    assert!(
        root.lines().count() < 170,
        "declaration_tasks.rs should stay focused on shared declaration task bundles"
    );
}
