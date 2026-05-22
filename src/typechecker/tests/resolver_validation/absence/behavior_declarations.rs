use super::*;

#[test]
fn behavior_declaration_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Behavior, "PrettyJson")
        .expect("behavior symbol");
    let entries = BehaviorDeclarationAbsenceValidation {
        method_signature_code: "METHODS",
        method_type_code: "TYPED_METHODS",
        parent_name_code: "PARENTS",
        parent_ref_code: "TYPED_PARENTS",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "METHODS", "behavior methods"),
            AbsentMetadataEntry::new(true, "TYPED_METHODS", "typed behavior methods"),
            AbsentMetadataEntry::new(true, "PARENTS", "behavior parents"),
            AbsentMetadataEntry::new(true, "TYPED_PARENTS", "typed behavior parents"),
        ]
    );
}

#[test]
fn behavior_declaration_absence_validation_uses_module_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.method_signature_code, "E0277");
    assert_eq!(validation.method_type_code, "E0376");
    assert_eq!(validation.parent_name_code, "E0278");
    assert_eq!(validation.parent_ref_code, "E0377");
}

#[test]
fn behavior_declaration_absence_validation_uses_import_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.method_signature_code, "E0293");
    assert_eq!(validation.method_type_code, "E0367");
    assert_eq!(validation.parent_name_code, "E0294");
    assert_eq!(validation.parent_ref_code, "E0368");
}

#[test]
fn behavior_declaration_absence_validation_uses_local_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.method_signature_code, "E0261");
    assert_eq!(validation.method_type_code, "E0385");
    assert_eq!(validation.parent_name_code, "E0262");
    assert_eq!(validation.parent_ref_code, "E0386");
}

#[test]
fn behavior_declaration_absence_validation_uses_variant_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.method_signature_code, "E0339");
    assert_eq!(validation.method_type_code, "E0393");
    assert_eq!(validation.parent_name_code, "E0340");
    assert_eq!(validation.parent_ref_code, "E0394");
}

#[test]
fn behavior_declaration_absence_validation_uses_value_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.method_signature_code, "E0304");
    assert_eq!(validation.method_type_code, "E0405");
    assert_eq!(validation.parent_name_code, "E0305");
    assert_eq!(validation.parent_ref_code, "E0406");
}
