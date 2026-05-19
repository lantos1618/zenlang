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
