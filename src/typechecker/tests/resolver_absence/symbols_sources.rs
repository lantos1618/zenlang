use super::*;

#[test]
fn resolver_symbol_presence_validation_formats_messages() {
    let extra = ResolverSymbolPresenceValidation {
        code: "EXTRA",
        presence: ResolverSymbolPresence::Extra,
    };
    let missing = ResolverSymbolPresenceValidation {
        code: "MISSING",
        presence: ResolverSymbolPresence::Missing,
    };

    assert_eq!(extra.code, "EXTRA");
    assert_eq!(
        extra.message("value", "main"),
        "resolver symbol table has extra value symbol 'main'"
    );
    assert_eq!(missing.code, "MISSING");
    assert_eq!(
        missing.message("local", "value"),
        "resolver symbol table missing local symbol 'value'"
    );
}

#[test]
fn resolver_symbol_presence_validation_uses_resolver_codes() {
    let missing = ResolverSymbolPresenceValidation::missing_resolver_code();
    let missing_local = ResolverSymbolPresenceValidation::missing_local_resolver_code();
    let extra_declaration = ResolverSymbolPresenceValidation::extra_declaration_resolver_code();
    let extra_local = ResolverSymbolPresenceValidation::extra_local_resolver_code();

    assert_eq!(missing.code, "E0210");
    assert!(matches!(missing.presence, ResolverSymbolPresence::Missing));
    assert_eq!(missing_local.code, "E0228");
    assert!(matches!(
        missing_local.presence,
        ResolverSymbolPresence::Missing
    ));
    assert_eq!(extra_declaration.code, "E0243");
    assert!(matches!(
        extra_declaration.presence,
        ResolverSymbolPresence::Extra
    ));
    assert_eq!(extra_local.code, "E0244");
    assert!(matches!(
        extra_local.presence,
        ResolverSymbolPresence::Extra
    ));
}

#[test]
fn resolver_symbol_presence_validation_pushes_diagnostic() {
    let mut tc = TypeChecker::new();

    tc.validate_resolver_symbol_presence(
        "value",
        "main",
        ResolverSymbolPresenceValidation {
            code: "EXTRA",
            presence: ResolverSymbolPresence::Extra,
        },
        Span::dummy(),
    );

    assert_eq!(tc.diagnostics.len(), 1);
    assert_eq!(tc.diagnostics[0].code, "EXTRA");
    assert_eq!(
        tc.diagnostics[0].message,
        "resolver symbol table has extra value symbol 'main'"
    );
}

#[test]
fn source_absence_validation_builds_source_validation() {
    let validation = SourceAbsenceValidation { code: "SOURCE" }.source_validation();

    assert_eq!(validation.code, "SOURCE");
    assert_eq!(validation.actual_missing, "none");
    assert_eq!(validation.expected_missing, "none");
    assert!(!validation.quote_expected);
}

#[test]
fn source_absence_validation_uses_type_like_resolver_code() {
    let validation = SourceAbsenceValidation::type_like_resolver_code();

    assert_eq!(validation.code, "E0309");
}

#[test]
fn source_absence_validation_uses_variant_resolver_code() {
    let validation = SourceAbsenceValidation::variant_resolver_code();

    assert_eq!(validation.code, "E0329");
}

#[test]
fn source_absence_validation_uses_value_resolver_code() {
    let validation = SourceAbsenceValidation::value_resolver_code();

    assert_eq!(validation.code, "E0297");
}

#[test]
fn source_validation_formats_message() {
    let quoted = SourceValidation {
        code: "SOURCE",
        actual_missing: "unknown",
        expected_missing: "none",
        quote_expected: true,
    };
    let unquoted = SourceValidation {
        code: "SOURCE",
        actual_missing: "none",
        expected_missing: "none",
        quote_expected: false,
    };

    assert_eq!(
        quoted.message("import", "io", Some("other"), Some("std")),
        "resolver import symbol 'io' has source 'other', expected 'std'"
    );
    assert_eq!(
        unquoted.message("value", "main", Some("std"), None),
        "resolver value symbol 'main' has source 'std', expected none"
    );
}

#[test]
fn source_validation_uses_resolver_codes() {
    let module = SourceValidation::module_resolver_code();
    let stripped_import = SourceValidation::stripped_import_resolver_code();
    let import = SourceValidation::import_resolver_code();
    let local = SourceValidation::local_resolver_code();

    assert_eq!(module.code, "E0230");
    assert_eq!(module.actual_missing, "none");
    assert_eq!(module.expected_missing, "none");
    assert!(!module.quote_expected);
    assert_eq!(stripped_import.code, "E0246");
    assert_eq!(stripped_import.actual_missing, "unknown");
    assert_eq!(stripped_import.expected_missing, "a module source");
    assert!(!stripped_import.quote_expected);
    assert_eq!(import.code, "E0227");
    assert_eq!(import.actual_missing, "unknown");
    assert_eq!(import.expected_missing, "none");
    assert!(import.quote_expected);
    assert_eq!(local.code, "E0248");
    assert_eq!(local.actual_missing, "none");
    assert_eq!(local.expected_missing, "none");
    assert!(!local.quote_expected);
}

#[test]
fn absent_metadata_entry_formats_message() {
    let entry = AbsentMetadataEntry {
        present: true,
        code: "ABSENT",
        label: "parameter count",
    };

    assert_eq!(entry.code, "ABSENT");
    assert_eq!(
        entry.message("value", "main"),
        "resolver value symbol 'main' has parameter count metadata, expected none"
    );
}

#[test]
fn resolver_named_list_display_formats_known_and_missing_items() {
    let fields = vec![("value".to_string(), "i32".to_string())];
    assert_eq!(
        format_resolver_named_list(Some(&fields), |ty: &String| ty.clone()),
        "(value: i32)"
    );
    assert_eq!(
        format_resolver_named_list::<String>(None, |ty: &String| ty.clone()),
        "unknown"
    );
}
