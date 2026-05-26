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

#[test]
fn production_diagnostics_use_typed_codes_outside_error_module() {
    let output = std::process::Command::new("git")
        .args(["ls-files", "src/**/*.rs"])
        .current_dir(repo_root())
        .output()
        .expect("list tracked Rust source files");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut offenders = Vec::new();
    for path in String::from_utf8(output.stdout)
        .expect("git ls-files output is utf-8")
        .lines()
        .filter(|path| production_diagnostic_source(path))
    {
        if !repo_root().join(path).exists() {
            continue;
        }
        let source = read(path);
        if source.as_bytes().windows(7).any(|window| {
            window[0] == b'"'
                && window[1] == b'E'
                && window[2..6].iter().all(|byte| byte.is_ascii_digit())
                && window[6] == b'"'
        }) {
            offenders.push(path.to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "production diagnostics should use DiagnosticCode/CompilerDiagnosticCode, not raw diagnostic strings: {}",
        offenders.join(", ")
    );
}

fn production_diagnostic_source(path: &str) -> bool {
    path.starts_with("src/")
        && !path.starts_with("src/error/")
        && !path.contains("/tests/")
        && !path.ends_with("/tests.rs")
        && path != "src/typechecker/tests.rs"
}
