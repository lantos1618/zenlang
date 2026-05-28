use std::collections::HashSet;

use zen::error::{
    code_actions_for_lsp, dap_launch_failure, diagnostics_for_ai, diagnostics_for_lsp,
    CompileError, CompilerDiagnosticCode, Diagnostic, DiagnosticCategory, DiagnosticPhase,
    FileTable, Span,
};

#[test]
fn diagnostic_code_registry_has_unique_valid_numbers() {
    let mut seen = HashSet::new();
    let compiler_codes = compiler_diagnostic_codes();
    assert!(!compiler_codes.is_empty());
    for (code, value) in compiler_codes {
        let suffix = code.strip_prefix('E').unwrap_or("");
        let valid_code = suffix.len() == 4 && suffix.bytes().all(|byte| byte.is_ascii_digit());
        assert!(valid_code, "invalid diagnostic code {code}");
        assert_eq!(code, format!("E{value:04}"));
        assert!(seen.insert(code.to_string()));
    }
}

#[test]
fn diagnostics_keep_metadata_and_protocol_views() {
    let coded = Diagnostic::error_code(
        CompilerDiagnosticCode::E0232,
        "missing module graph entry",
        Span::new(0, 5, 10),
    );
    assert_eq!(coded.code(), "E0232");
    assert_eq!(coded.slug(), "resolution_e0232");
    assert_eq!(coded.phase(), DiagnosticPhase::Resolver);
    assert_eq!(coded.category(), DiagnosticCategory::Resolution);

    let converted: Diagnostic =
        CompileError::Syntax("unexpected token".into(), Some(Span::new(1, 5, 10))).into();
    assert_eq!(converted.code(), "E2000");
    assert_eq!(converted.span, Some(Span::new(1, 5, 10)));
    assert_eq!(
        format!("{}", CompileError::Internal("oops".into())),
        "internal error: oops"
    );

    let mut files = FileTable::default();
    let file_id = files.add_file("/tmp/main.zen".into(), "main = () i32 {\n  false\n}\n");
    let diagnostics = vec![Diagnostic::error_code(
        CompilerDiagnosticCode::E3030,
        "return type mismatch: expected `i32`, found `bool`",
        Span::new(file_id, 18, 23),
    )
    .with_related(Span::new(file_id, 0, 4), "function starts here")
    .with_fact("expected", "i32")
    .with_fix(
        "replace_bool_with_i32",
        "Replace bool with integer",
        Span::new(file_id, 18, 23),
        "0",
    )];

    let ai = diagnostics_for_ai(&diagnostics, &files);
    assert_eq!(ai[0].facts[0].key, "expected");
    assert_eq!(ai[0].related[0].message, "function starts here");

    let lsp = diagnostics_for_lsp(&diagnostics, &files);
    assert_eq!(lsp[0].uri, "file:///tmp/main.zen");
    assert_eq!(lsp[0].diagnostic.code, "E3030");
    assert_eq!(lsp[0].diagnostic.data.slug, "type_e3030");
    assert_eq!(lsp[0].diagnostic.related_information.len(), 1);

    let actions = code_actions_for_lsp(&diagnostics, &files);
    assert_eq!(actions[0].data.fix_kind, "replace_bool_with_i32");
    assert_eq!(actions[0].edits[0].new_text, "0");

    let dap = dap_launch_failure(&diagnostics, &files);
    assert_eq!(dap.format, "zen.dap.diagnostics.v1");
    assert!(dap.body.output.contains("error[E3030]"));
}

fn compiler_diagnostic_codes() -> Vec<(&'static str, u16)> {
    include_str!("../src/error/compiler_diagnostic_code.rs")
        .split_once("pub enum CompilerDiagnosticCode")
        .expect("expected CompilerDiagnosticCode enum declaration")
        .1
        .lines()
        .skip(1)
        .take_while(|line| line.trim() != "}")
        .flat_map(|line| line.trim().split(',').map(str::trim))
        .filter(|name| !name.is_empty())
        .map(|entry| {
            let (name, value) = entry
                .split_once('=')
                .expect("expected explicit diagnostic code value");
            (
                name.trim(),
                value.trim().parse().expect("diagnostic code value"),
            )
        })
        .collect()
}
