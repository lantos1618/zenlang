use super::*;

#[test]
fn value_signature_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
add = (left: i32, right: i32) i32 { left + right }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Value, "add")
        .expect("value symbol");
    let entries = ValueSignatureAbsenceValidation {
        parameter_count_code: "PARAM_COUNT",
        parameter_name_code: "PARAM_NAMES",
        parameter_type_name_code: "PARAM_TYPES",
        parameter_type_code: "TYPED_PARAM_TYPES",
        return_type_code: "RETURN_TYPE",
        typed_return_type_code: "TYPED_RETURN_TYPE",
    }
    .entries(symbol);

    assert!(entries.iter().all(|entry| entry.present));
    assert_eq!(
        entries.map(|entry| entry.message("value", "add")),
        [
            "resolver value symbol 'add' has parameter count metadata, expected none",
            "resolver value symbol 'add' has parameter names metadata, expected none",
            "resolver value symbol 'add' has parameter types metadata, expected none",
            "resolver value symbol 'add' has typed parameter types metadata, expected none",
            "resolver value symbol 'add' has return type metadata, expected none",
            "resolver value symbol 'add' has typed return type metadata, expected none",
        ]
    );
}

#[test]
fn value_signature_absence_validation_uses_module_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0265");
    assert_eq!(validation.parameter_name_code, "E0267");
    assert_eq!(validation.parameter_type_name_code, "E0268");
    assert_eq!(validation.parameter_type_code, "E0371");
    assert_eq!(validation.return_type_code, "E0266");
    assert_eq!(validation.typed_return_type_code, "E0372");
}

#[test]
fn value_signature_absence_validation_uses_import_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0281");
    assert_eq!(validation.parameter_name_code, "E0283");
    assert_eq!(validation.parameter_type_name_code, "E0284");
    assert_eq!(validation.parameter_type_code, "E0362");
    assert_eq!(validation.return_type_code, "E0282");
    assert_eq!(validation.typed_return_type_code, "E0363");
}

#[test]
fn value_signature_absence_validation_uses_local_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0249");
    assert_eq!(validation.parameter_name_code, "E0251");
    assert_eq!(validation.parameter_type_name_code, "E0252");
    assert_eq!(validation.parameter_type_code, "E0380");
    assert_eq!(validation.return_type_code, "E0250");
    assert_eq!(validation.typed_return_type_code, "E0381");
}

#[test]
fn value_signature_absence_validation_uses_type_like_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0310");
    assert_eq!(validation.parameter_name_code, "E0312");
    assert_eq!(validation.parameter_type_name_code, "E0313");
    assert_eq!(validation.parameter_type_code, "E0360");
    assert_eq!(validation.return_type_code, "E0311");
    assert_eq!(validation.typed_return_type_code, "E0361");
}

#[test]
fn value_signature_absence_validation_uses_variant_resolver_codes() {
    let validation = ValueSignatureAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.parameter_count_code, "E0330");
    assert_eq!(validation.parameter_name_code, "E0332");
    assert_eq!(validation.parameter_type_name_code, "E0333");
    assert_eq!(validation.parameter_type_code, "E0389");
    assert_eq!(validation.return_type_code, "E0331");
    assert_eq!(validation.typed_return_type_code, "E0390");
}

#[test]
fn type_parameter_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}

identity<T: Json> = (value: T) T { value }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Value, "identity")
        .expect("value symbol");
    let entries = TypeParameterAbsenceValidation {
        count_code: "COUNT",
        name_code: "NAMES",
        bound_code: "BOUNDS",
        bound_ref_code: "BOUND_REFS",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "COUNT", "type parameter count"),
            AbsentMetadataEntry::new(true, "NAMES", "type parameter names"),
            AbsentMetadataEntry::new(true, "BOUNDS", "type parameter bounds"),
            AbsentMetadataEntry::new(true, "BOUND_REFS", "typed type parameter bound refs"),
        ]
    );
}

#[test]
fn type_parameter_absence_validation_uses_module_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.count_code, "E0269");
    assert_eq!(validation.name_code, "E0348");
    assert_eq!(validation.bound_code, "E0270");
    assert_eq!(validation.bound_ref_code, "E0373");
}

#[test]
fn type_parameter_absence_validation_uses_import_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.count_code, "E0285");
    assert_eq!(validation.name_code, "E0349");
    assert_eq!(validation.bound_code, "E0286");
    assert_eq!(validation.bound_ref_code, "E0364");
}

#[test]
fn type_parameter_absence_validation_uses_local_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.count_code, "E0253");
    assert_eq!(validation.name_code, "E0350");
    assert_eq!(validation.bound_code, "E0254");
    assert_eq!(validation.bound_ref_code, "E0382");
}

#[test]
fn type_parameter_absence_validation_uses_variant_resolver_codes() {
    let validation = TypeParameterAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.count_code, "E0334");
    assert_eq!(validation.name_code, "E0351");
    assert_eq!(validation.bound_code, "E0335");
    assert_eq!(validation.bound_ref_code, "E0391");
}

#[test]
fn field_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");
    let entries = FieldAbsenceValidation {
        count_code: "COUNT",
        type_name_code: "FIELD_TYPES",
        typed_code: "TYPED_FIELDS",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "COUNT", "field count"),
            AbsentMetadataEntry::new(true, "FIELD_TYPES", "field types"),
            AbsentMetadataEntry::new(true, "TYPED_FIELDS", "typed field types"),
        ]
    );
}

#[test]
fn field_absence_validation_uses_module_resolver_codes() {
    let validation = FieldAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.count_code, "E0271");
    assert_eq!(validation.type_name_code, "E0272");
    assert_eq!(validation.typed_code, "E0374");
}

#[test]
fn field_absence_validation_uses_import_resolver_codes() {
    let validation = FieldAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.count_code, "E0287");
    assert_eq!(validation.type_name_code, "E0288");
    assert_eq!(validation.typed_code, "E0365");
}

#[test]
fn field_absence_validation_uses_local_resolver_codes() {
    let validation = FieldAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.count_code, "E0255");
    assert_eq!(validation.type_name_code, "E0256");
    assert_eq!(validation.typed_code, "E0383");
}

#[test]
fn field_absence_validation_uses_type_like_resolver_codes() {
    let validation = FieldAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.count_code, "E0319");
    assert_eq!(validation.type_name_code, "E0320");
    assert_eq!(validation.typed_code, "E0398");
}

#[test]
fn field_absence_validation_uses_variant_resolver_codes() {
    let validation = FieldAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.count_code, "E0336");
    assert_eq!(validation.type_name_code, "E0337");
    assert_eq!(validation.typed_code, "E0392");
}

#[test]
fn field_absence_validation_uses_behavior_resolver_codes() {
    let validation = FieldAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.count_code, "E0321");
    assert_eq!(validation.type_name_code, "E0322");
    assert_eq!(validation.typed_code, "E0399");
}

#[test]
fn field_absence_validation_uses_value_resolver_codes() {
    let validation = FieldAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.count_code, "E0298");
    assert_eq!(validation.type_name_code, "E0299");
    assert_eq!(validation.typed_code, "E0403");
}

#[test]
fn variant_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Option<T>: Some(T), None
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup_variant("Option", "Some")
        .expect("variant symbol");
    let entries = VariantAbsenceValidation {
        names_code: "NAMES",
        owner_code: "OWNER",
        payload_count_code: "PAYLOAD_COUNT",
        payload_type_name_code: "PAYLOAD_TYPE",
        payload_type_code: "TYPED_PAYLOAD",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(false, "NAMES", "variant names"),
            AbsentMetadataEntry::new(true, "OWNER", "variant owner"),
            AbsentMetadataEntry::new(true, "PAYLOAD_COUNT", "variant payload count"),
            AbsentMetadataEntry::new(true, "PAYLOAD_TYPE", "variant payload type"),
            AbsentMetadataEntry::new(true, "TYPED_PAYLOAD", "typed variant payload type"),
        ]
    );
}

#[test]
fn variant_absence_validation_uses_module_resolver_codes() {
    let validation = VariantAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.names_code, "E0273");
    assert_eq!(validation.owner_code, "E0274");
    assert_eq!(validation.payload_count_code, "E0275");
    assert_eq!(validation.payload_type_name_code, "E0276");
    assert_eq!(validation.payload_type_code, "E0375");
}

#[test]
fn variant_absence_validation_uses_import_resolver_codes() {
    let validation = VariantAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.names_code, "E0289");
    assert_eq!(validation.owner_code, "E0290");
    assert_eq!(validation.payload_count_code, "E0291");
    assert_eq!(validation.payload_type_name_code, "E0292");
    assert_eq!(validation.payload_type_code, "E0366");
}

#[test]
fn variant_absence_validation_uses_local_resolver_codes() {
    let validation = VariantAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.names_code, "E0257");
    assert_eq!(validation.owner_code, "E0258");
    assert_eq!(validation.payload_count_code, "E0259");
    assert_eq!(validation.payload_type_name_code, "E0260");
    assert_eq!(validation.payload_type_code, "E0384");
}

#[test]
fn variant_absence_validation_uses_type_like_resolver_codes() {
    let validation = VariantAbsenceValidation::type_like_resolver_codes();

    assert_eq!(validation.names_code, "E0315");
    assert_eq!(validation.owner_code, "E0316");
    assert_eq!(validation.payload_count_code, "E0317");
    assert_eq!(validation.payload_type_name_code, "E0318");
    assert_eq!(validation.payload_type_code, "E0397");
}

#[test]
fn variant_absence_validation_uses_behavior_resolver_codes() {
    let validation = VariantAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.names_code, "E0323");
    assert_eq!(validation.owner_code, "E0324");
    assert_eq!(validation.payload_count_code, "E0325");
    assert_eq!(validation.payload_type_name_code, "E0326");
    assert_eq!(validation.payload_type_code, "E0400");
}

#[test]
fn variant_absence_validation_uses_value_resolver_codes() {
    let validation = VariantAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.names_code, "E0300");
    assert_eq!(validation.owner_code, "E0301");
    assert_eq!(validation.payload_count_code, "E0302");
    assert_eq!(validation.payload_type_name_code, "E0303");
    assert_eq!(validation.payload_type_code, "E0404");
}

#[test]
fn mutability_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
main = (mut input: i32) i32 {
    value ::= input
    value
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup_scoped(Namespace::Local, "input")
        .expect("local symbol");
    let entries = MutabilityAbsenceValidation { code: "MUTABLE" }.entries(symbol);

    assert_eq!(
        entries,
        [AbsentMetadataEntry::new(true, "MUTABLE", "mutability")]
    );
}

#[test]
fn mutability_absence_validation_uses_module_resolver_code() {
    let validation = MutabilityAbsenceValidation::module_resolver_code();

    assert_eq!(validation.code, "E0345");
}

#[test]
fn mutability_absence_validation_uses_import_resolver_code() {
    let validation = MutabilityAbsenceValidation::import_resolver_code();

    assert_eq!(validation.code, "E0344");
}

#[test]
fn mutability_absence_validation_uses_type_like_resolver_code() {
    let validation = MutabilityAbsenceValidation::type_like_resolver_code();

    assert_eq!(validation.code, "E0314");
}

#[test]
fn mutability_absence_validation_uses_variant_resolver_code() {
    let validation = MutabilityAbsenceValidation::variant_resolver_code();

    assert_eq!(validation.code, "E0343");
}

#[test]
fn mutability_absence_validation_uses_value_resolver_code() {
    let validation = MutabilityAbsenceValidation::value_resolver_code();

    assert_eq!(validation.code, "E0308");
}

#[test]
fn mutability_validation_formats_actual_and_expected() {
    let validation = MutabilityValidation { code: "MUTABLE" };

    assert_eq!(validation.code, "MUTABLE");
    assert_eq!(
        validation.display(Some(false), true),
        ("immutable", "mutable")
    );
    assert_eq!(validation.display(None, false), ("unknown", "immutable"));
    assert_eq!(
        validation.message("local", "value", Some(false), true),
        "resolver local symbol 'value' has mutability immutable, expected mutable"
    );
}

#[test]
fn mutability_validation_uses_resolver_code() {
    let validation = MutabilityValidation::resolver_code();

    assert_eq!(validation.code, "E0231");
}

#[test]
fn visibility_validation_formats_actual_and_expected() {
    let validation = VisibilityValidation { code: "VISIBLE" };

    assert_eq!(validation.code, "VISIBLE");
    assert_eq!(validation.display(true, false), ("public", "private"));
    assert_eq!(validation.display(false, true), ("private", "public"));
    assert_eq!(
        validation.message("import", "io", true, false),
        "resolver import symbol 'io' has visibility public, expected private"
    );
}

#[test]
fn visibility_validation_uses_local_resolver_code() {
    let validation = VisibilityValidation::local_resolver_code();

    assert_eq!(validation.code, "E0247");
}

#[test]
fn visibility_validation_uses_module_resolver_code() {
    let validation = VisibilityValidation::module_resolver_code();

    assert_eq!(validation.code, "E0229");
}

#[test]
fn visibility_validation_uses_import_resolver_code() {
    let validation = VisibilityValidation::import_resolver_code();

    assert_eq!(validation.code, "E0245");
}

#[test]
fn visibility_validation_uses_type_like_resolver_code() {
    let validation = VisibilityValidation::type_like_resolver_code();

    assert_eq!(validation.code, "E0225");
}

#[test]
fn visibility_validation_uses_variant_resolver_code() {
    let validation = VisibilityValidation::variant_resolver_code();

    assert_eq!(validation.code, "E0226");
}

#[test]
fn visibility_validation_uses_value_resolver_code() {
    let validation = VisibilityValidation::value_resolver_code();

    assert_eq!(validation.code, "E0224");
}

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
