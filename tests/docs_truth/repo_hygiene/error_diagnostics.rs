use super::*;

#[test]
fn diagnostic_core_lives_in_focused_helper() {
    let root = read("src/error.rs");
    let diagnostic = read("src/error/diagnostic.rs");

    for item in [
        "pub enum Severity",
        "pub struct Label",
        "pub enum ContextKind",
        "pub struct ContextFrame",
        "pub struct TextEdit",
        "pub struct SuggestedFix",
        "pub struct Diagnostic",
        "impl Diagnostic",
    ] {
        assert!(
            !root.contains(item),
            "error module root should not own diagnostic core item: {item}"
        );
        assert!(
            diagnostic.contains(item),
            "diagnostic core item should live in focused helper: {item}"
        );
    }

    assert!(
        root.contains("mod diagnostic;"),
        "error module should load focused diagnostic core"
    );
}
