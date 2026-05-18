use super::*;

#[test]
fn behavior_association_absence_validation_builds_entries() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) StaticString
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let symbol = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");
    let entries = BehaviorAssociationAbsenceValidation {
        impl_name_code: "IMPL_NAMES",
        impl_ref_code: "IMPL_REFS",
        required_name_code: "REQUIRED_NAMES",
        required_ref_code: "REQUIRED_REFS",
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "IMPL_NAMES", "behavior impls"),
            AbsentMetadataEntry::new(true, "IMPL_REFS", "typed behavior impls"),
            AbsentMetadataEntry::new(true, "REQUIRED_NAMES", "behavior requires"),
            AbsentMetadataEntry::new(true, "REQUIRED_REFS", "typed behavior requires"),
        ]
    );
}

#[test]
fn behavior_association_absence_validation_uses_module_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0279");
    assert_eq!(validation.impl_ref_code, "E0378");
    assert_eq!(validation.required_name_code, "E0280");
    assert_eq!(validation.required_ref_code, "E0379");
}

#[test]
fn behavior_association_absence_validation_uses_import_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0295");
    assert_eq!(validation.impl_ref_code, "E0369");
    assert_eq!(validation.required_name_code, "E0296");
    assert_eq!(validation.required_ref_code, "E0370");
}

#[test]
fn behavior_association_absence_validation_uses_local_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0263");
    assert_eq!(validation.impl_ref_code, "E0387");
    assert_eq!(validation.required_name_code, "E0264");
    assert_eq!(validation.required_ref_code, "E0388");
}

#[test]
fn behavior_association_absence_validation_uses_variant_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0341");
    assert_eq!(validation.impl_ref_code, "E0395");
    assert_eq!(validation.required_name_code, "E0342");
    assert_eq!(validation.required_ref_code, "E0396");
}

#[test]
fn behavior_association_absence_validation_uses_behavior_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0327");
    assert_eq!(validation.impl_ref_code, "E0401");
    assert_eq!(validation.required_name_code, "E0328");
    assert_eq!(validation.required_ref_code, "E0402");
}

#[test]
fn behavior_association_absence_validation_uses_value_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.impl_name_code, "E0306");
    assert_eq!(validation.impl_ref_code, "E0407");
    assert_eq!(validation.required_name_code, "E0307");
    assert_eq!(validation.required_ref_code, "E0408");
}

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
