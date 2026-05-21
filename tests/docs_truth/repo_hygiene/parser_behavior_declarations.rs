use super::*;

#[test]
fn parser_behavior_relationships_share_parenthesized_ref_parser() {
    let source = read("src/parser/behavior_declarations.rs");

    assert!(
        source.contains("fn parse_parenthesized_behavior_ref"),
        "behavior relationship parsing should share a helper for `(Behavior<T>)` references"
    );

    for parser in [
        "parse_behavior_impl_block",
        "parse_behavior_requires",
        "parse_behavior_derive",
        "parse_behavior_extends",
    ] {
        let start = source
            .find(&format!("fn {parser}"))
            .unwrap_or_else(|| panic!("missing parser helper: {parser}"));
        let body = &source[start..];
        let next_fn = body[1..]
            .find("\n    fn ")
            .or_else(|| body[1..].find("\n    pub(super) fn "))
            .map(|offset| offset + 1)
            .unwrap_or(body.len());
        let body = &body[..next_fn];
        assert!(
            body.contains("self.parse_parenthesized_behavior_ref()?"),
            "{parser} should use the shared parenthesized behavior reference parser"
        );
    }
}

#[test]
fn parser_behavior_method_signatures_live_in_focused_helper() {
    let behavior_declarations = read("src/parser/behavior_declarations.rs");
    let method_signatures = read("src/parser/behavior_declarations/method_signatures.rs");

    for helper in [
        "type BehaviorMethodSignature",
        "fn parse_behavior_method_signature",
    ] {
        assert!(
            !behavior_declarations.contains(helper),
            "behavior declaration root should not own method signature helper: {helper}"
        );
        assert!(
            method_signatures.contains(helper),
            "behavior method signature parsing should live in focused helper: {helper}"
        );
    }

    assert!(
        behavior_declarations.contains("mod method_signatures;"),
        "behavior declaration root should include focused method signature helper"
    );
    assert!(
        behavior_declarations.lines().count() < 170,
        "behavior_declarations.rs should stay focused on behavior declarations and relationships"
    );
}

#[test]
fn parser_impl_blocks_live_in_focused_helper() {
    let behavior_declarations = read("src/parser/behavior_declarations.rs");
    let impl_blocks = read("src/parser/impl_blocks.rs");
    let parser_module = read("src/parser/mod.rs");

    assert!(
        behavior_declarations.lines().count() < 240,
        "behavior_declarations.rs should stay focused on behavior declarations and relationships"
    );
    assert!(
        !behavior_declarations.contains("fn prepend_impl_type_params"),
        "generic impl-block type parameter merging should live in impl_blocks.rs"
    );
    assert!(
        impl_blocks.contains("pub(super) fn parse_impl_block"),
        "impl_blocks.rs should parse non-behavior impl blocks"
    );
    assert!(
        impl_blocks.contains("pub(super) fn parse_impl_block_with_type_params"),
        "impl_blocks.rs should parse generic non-behavior impl blocks"
    );
    assert!(
        impl_blocks.contains("fn prepend_impl_type_params"),
        "impl_blocks.rs should own impl type parameter merging"
    );
    assert!(
        parser_module.contains("mod impl_blocks;"),
        "parser module should include the focused impl block helper"
    );
}
