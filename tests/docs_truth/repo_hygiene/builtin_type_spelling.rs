use super::*;

#[test]
fn semantic_builtin_type_checks_use_shared_spelling_helper() {
    let helper = read("src/ast/types.rs");
    assert!(
        helper.contains("pub const DYNAMIC_STRING_TYPE_NAME: &str = \"String\""),
        "dynamic String spelling should live in the AST type helper module"
    );
    assert!(
        helper.contains("pub fn is_builtin_type_name"),
        "builtin type-name recognition should be centralized"
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
