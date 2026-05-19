use super::*;

#[test]
fn semantic_builtin_type_checks_use_shared_spelling_helper() {
    let helper_root = read("src/ast/types.rs");
    let helper_names = read("src/ast/types/names.rs");
    let helper_gated = read("src/ast/types/gated.rs");
    let helper = format!("{helper_root}\n{helper_names}\n{helper_gated}");
    assert!(
        helper.contains("pub const DYNAMIC_STRING_TYPE_NAME: &str = \"String\""),
        "dynamic String spelling should live in the AST type helper module"
    );
    for required in [
        "pub const ALLOCATOR_TYPE_NAME: &str = \"Allocator\"",
        "pub const SYNC_EFFECT_TYPE_NAME: &str = \"Sync\"",
        "pub const ASYNC_EFFECT_TYPE_NAME: &str = \"Async\"",
        "pub const ACTOR_TYPE_NAME: &str = \"Actor\"",
        "pub const ACTOR_REF_TYPE_NAME: &str = \"ActorRef\"",
        "pub const MAILBOX_TYPE_NAME: &str = \"Mailbox\"",
        "pub const SUPERVISOR_TYPE_NAME: &str = \"Supervisor\"",
        "pub enum GatedBuiltinType",
        "DynamicString",
        "pub const ALL: &[GatedBuiltinType]",
        "pub fn gate_message(self) -> &'static str",
        "pub fn gated_builtin_type_name",
    ] {
        assert!(
            helper.contains(required),
            "gated builtin type spelling should live in the AST type helper module: {required}"
        );
    }
    assert!(
        helper.contains("GatedBuiltinType::ALL")
            && helper.contains(".iter()")
            && helper.contains(".copied()")
            && helper.contains(".find(|ty| ty.as_str() == name)"),
        "gated builtin type lookup should use the enum-owned static table"
    );
    assert!(
        !helper.contains("format!(\n                    \"`{}`"),
        "gated builtin type diagnostics should be enum-owned static strings, not allocated formatting"
    );
    assert!(
        helper.contains("pub fn is_builtin_type_name"),
        "builtin type-name recognition should be centralized"
    );
    assert!(
        !helper.contains("name == DYNAMIC_STRING_TYPE_NAME"),
        "builtin type-name recognition should route through GatedBuiltinType, not a direct spelling check"
    );
    assert!(
        helper.contains("Some(GatedBuiltinType::DynamicString)"),
        "dynamic String recognition should be expressed through the gated builtin type enum"
    );
    assert!(
        helper_root.contains("mod gated;"),
        "AST type module should include a focused gated builtin helper"
    );
    assert!(
        helper_root.contains("pub use gated::"),
        "AST type module should re-export gated builtin names from the focused helper"
    );
    assert!(
        helper_gated.contains("pub enum GatedBuiltinType"),
        "gated.rs should own gated builtin type classification"
    );
    assert!(
        !helper_names.contains("pub enum GatedBuiltinType"),
        "names.rs should stay focused on stable builtin and generic type names"
    );
    assert!(
        helper_names.lines().count() < 220,
        "names.rs should stay compact after gated builtin types move out"
    );

    for path in [
        "src/resolver/type_validation.rs",
        "src/typechecker/generic_type_reference_walker.rs",
        "src/typechecker/resolve.rs",
    ] {
        let source = read(path);
        assert!(
            !source.contains("name == \"String\""),
            "{path} should not hard-code dynamic String spelling in semantic logic"
        );
    }
}
