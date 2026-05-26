use super::*;

#[test]
fn file_table_add_and_lookup() {
    let mut ft = FileTable::new();
    let id = ft.add_file("test.zen".into(), "hello\nworld\n".into());
    assert_eq!(id, 0);
    assert_eq!(ft.get_source(id), Some("hello\nworld\n"));
    assert_eq!(ft.get_path(id), Some("test.zen"));
}

#[test]
fn file_table_multiple_files() {
    let mut ft = FileTable::new();
    let a = ft.add_file("a.zen".into(), "aaa".into());
    let b = ft.add_file("b.zen".into(), "bbb".into());
    assert_eq!(a, 0);
    assert_eq!(b, 1);
    assert_eq!(ft.get_source(a), Some("aaa"));
    assert_eq!(ft.get_source(b), Some("bbb"));
}

#[test]
fn file_table_invalid_id() {
    let ft = FileTable::new();
    assert_eq!(ft.get_source(99), None);
    assert_eq!(ft.get_path(99), None);
}

#[test]
fn line_col_simple() {
    let mut ft = FileTable::new();
    // "hello\nworld\n"
    //  01234 5 678...
    let id = ft.add_file("t.zen".into(), "hello\nworld\n".into());
    assert_eq!(ft.line_col(id, 0), Some((0, 0))); // 'h'
    assert_eq!(ft.line_col(id, 4), Some((0, 4))); // 'o'
    assert_eq!(ft.line_col(id, 6), Some((1, 0))); // 'w'
    assert_eq!(ft.line_col(id, 10), Some((1, 4))); // 'd'
}

#[test]
fn span_dummy() {
    let s = Span::dummy();
    assert_eq!(s.file_id, 0);
    assert_eq!(s.start, 0);
    assert_eq!(s.end, 0);
    assert!(s.is_empty());
}

#[test]
fn span_merge() {
    let a = Span::new(0, 5, 10);
    let b = Span::new(0, 8, 15);
    let merged = a.merge(b);
    assert_eq!(merged.start, 5);
    assert_eq!(merged.end, 15);
}

#[test]
fn diagnostic_error_constructor() {
    let d = Diagnostic::error("E1001", "unterminated string", Span::new(0, 5, 10));
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.code, "E1001");
    assert_eq!(d.slug, "raw_e1001");
    assert_eq!(d.phase, DiagnosticPhase::Unknown);
    assert_eq!(d.category, DiagnosticCategory::Unknown);
    assert!(d.is_error());
}

#[test]
fn diagnostic_error_code_constructor_uses_descriptor_metadata() {
    let d = Diagnostic::error_code(
        ResolverContractCode::E0233,
        "missing module graph module",
        Span::new(0, 5, 10),
    );
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.code, "E7233");
    assert_eq!(d.slug, "resolver_contract_e7233");
    assert_eq!(d.phase, DiagnosticPhase::ResolverContract);
    assert_eq!(d.category, DiagnosticCategory::ResolverContract);
    assert_eq!(d.docs_path, "docs/DIAGNOSTICS.md#resolver-contract");
}

#[test]
fn diagnostic_warning_constructor() {
    let d = Diagnostic::warning("W3001", "unused variable", Span::new(0, 0, 3));
    assert_eq!(d.severity, Severity::Warning);
    assert!(!d.is_error());
}

#[test]
fn diagnostic_builder_chain() {
    let d = Diagnostic::error("E3001", "type mismatch", Span::new(0, 10, 20))
        .with_label(Span::new(0, 30, 40), "expected i32 here")
        .with_note("try casting with `as i32`");
    assert_eq!(d.labels.len(), 1);
    assert_eq!(d.notes.len(), 1);
    assert_eq!(d.labels[0].message, "expected i32 here");
}

#[test]
fn diagnostic_display() {
    let d = Diagnostic::error("E1001", "bad token", Span::dummy());
    assert_eq!(format!("{d}"), "error[E1001]: bad token");
}

#[test]
fn compile_error_display() {
    let e = CompileError::Syntax("unexpected token".into(), None);
    assert_eq!(format!("{e}"), "syntax error: unexpected token");

    let e = CompileError::Internal("oops".into());
    assert_eq!(format!("{e}"), "internal error: oops");
}

#[test]
fn compile_error_to_diagnostic() {
    let e = CompileError::Type("mismatch".into(), Some(Span::new(1, 5, 10)));
    let d: Diagnostic = e.into();
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.code, "E3000");
    assert_eq!(d.slug, "type");
    assert_eq!(d.phase, DiagnosticPhase::TypeChecker);
    assert_eq!(d.category, DiagnosticCategory::Type);
    assert_eq!(d.span, Some(Span::new(1, 5, 10)));
}

#[test]
fn diagnostic_code_registry_has_unique_numbers() {
    let mut seen = std::collections::HashSet::new();

    for code in DiagnosticCode::BASE {
        let number = code.number();
        assert!(
            seen.insert(number.clone()),
            "duplicate diagnostic code {number}"
        );
    }

    for code in ResolverContractCode::ALL {
        let diagnostic_code = DiagnosticCode::from(*code);
        let number = diagnostic_code.number();
        assert!(
            seen.insert(number.clone()),
            "duplicate diagnostic code {number}"
        );
    }

    for code in CompilerDiagnosticCode::ALL {
        let diagnostic_code = DiagnosticCode::from(*code);
        let number = diagnostic_code.number();
        assert!(
            seen.insert(number.clone()),
            "duplicate diagnostic code {number}"
        );
    }
}

#[test]
fn diagnostic_protocol_views_keep_agent_lsp_and_dap_metadata() {
    let mut files = FileTable::new();
    let file_id = files.add_file(
        "/tmp/main.zen".into(),
        "main = () i32 {\n  false\n}\n".into(),
    );
    let diagnostic = Diagnostic::error_code(
        DiagnosticCode::Type,
        "return type mismatch: expected `i32`, found `bool`",
        Span::new(file_id, 18, 23),
    )
    .with_related(Span::new(file_id, 0, 4), "function starts here")
    .with_fact("expected", "i32")
    .with_suggested_fix(SuggestedFix::new(
        "replace_bool_with_i32",
        "Replace bool with integer",
        vec![TextEdit::new(Span::new(file_id, 18, 23), "0")],
    ));
    let diagnostics = vec![diagnostic];

    let ai = diagnostics_for_ai(&diagnostics, &files);
    assert_eq!(ai[0].slug, "type");
    assert_eq!(ai[0].phase, "typechecker");
    assert_eq!(ai[0].category, "type");
    assert_eq!(ai[0].facts[0].key, "expected");
    assert_eq!(ai[0].related[0].message, "function starts here");

    let lsp = diagnostics_for_lsp(&diagnostics, &files);
    assert_eq!(lsp[0].uri, "file:///tmp/main.zen");
    assert_eq!(lsp[0].diagnostic.severity, 1);
    assert_eq!(lsp[0].diagnostic.code, "E3000");
    assert_eq!(lsp[0].diagnostic.data.slug, "type");
    assert_eq!(lsp[0].diagnostic.related_information.len(), 1);
    assert_eq!(
        lsp[0].diagnostic.code_description.as_ref().unwrap().href,
        "docs/DIAGNOSTICS.md#type-checking"
    );

    let actions = code_actions_for_lsp(&diagnostics, &files);
    assert_eq!(actions[0].kind, "quickfix");
    assert_eq!(actions[0].data.fix_kind, "replace_bool_with_i32");
    assert_eq!(actions[0].edits[0].new_text, "0");

    let dap = dap_launch_failure(&diagnostics, &files);
    assert_eq!(dap.format, "zen.dap.diagnostics.v1");
    assert!(dap.body.output.contains("error[E3000]"));
    assert_eq!(dap.body.data.diagnostics[0].slug, "type");
}
