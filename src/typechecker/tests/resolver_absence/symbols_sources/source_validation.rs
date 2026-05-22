use super::*;

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
