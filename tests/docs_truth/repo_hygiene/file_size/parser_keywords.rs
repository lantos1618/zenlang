use super::super::*;

#[test]
fn parser_keyword_hygiene_guards_stay_split_by_responsibility() {
    let root = read("tests/docs_truth/repo_hygiene/parser_keywords.rs");
    let expressions = read("tests/docs_truth/repo_hygiene/parser_keywords/expressions.rs");
    let module_roots = read("tests/docs_truth/repo_hygiene/parser_keywords/module_roots.rs");
    let declarations = read("tests/docs_truth/repo_hygiene/parser_keywords/declarations.rs");

    assert!(
        root.lines().count() < 60,
        "parser_keywords.rs should route focused parser keyword hygiene modules"
    );
    for module in ["mod declarations;", "mod expressions;", "mod module_roots;"] {
        assert!(
            root.contains(module),
            "parser_keywords.rs should include focused module `{module}`"
        );
    }

    for expression_guard in [
        "fn parser_prefix_keywords_use_owned_keyword_enum",
        "fn parser_pattern_keywords_use_owned_keyword_enum",
        "fn parser_this_methods_use_owned_method_enum",
    ] {
        assert!(
            !root.contains(expression_guard),
            "expression keyword guard should move out of parser_keywords.rs: {expression_guard}"
        );
        assert!(
            expressions.contains(expression_guard),
            "expressions.rs should keep parser keyword guard: {expression_guard}"
        );
    }

    for module_root_guard in [
        "fn parser_module_roots_use_owned_root_enum",
        "fn parser_module_root_spelling_lives_in_focused_keyword_helper",
    ] {
        assert!(
            !root.contains(module_root_guard),
            "module-root keyword guard should move out of parser_keywords.rs: {module_root_guard}"
        );
        assert!(
            module_roots.contains(module_root_guard),
            "module_roots.rs should keep parser keyword guard: {module_root_guard}"
        );
    }

    for declaration_guard in [
        "fn parser_mutability_keywords_use_owned_keyword_enum",
        "fn parser_behavior_declaration_keyword_uses_owned_keyword_enum",
    ] {
        assert!(
            !root.contains(declaration_guard),
            "declaration keyword guard should move out of parser_keywords.rs: {declaration_guard}"
        );
        assert!(
            declarations.contains(declaration_guard),
            "declarations.rs should keep parser keyword guard: {declaration_guard}"
        );
    }
}
