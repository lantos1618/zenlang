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
        impl_name_code: "IMPL_NAMES".into(),
        impl_ref_code: "IMPL_REFS".into(),
        required_name_code: "REQUIRED_NAMES".into(),
        required_ref_code: "REQUIRED_REFS".into(),
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "IMPL_NAMES".into(), "behavior impls"),
            AbsentMetadataEntry::new(true, "IMPL_REFS".into(), "typed behavior impls"),
            AbsentMetadataEntry::new(true, "REQUIRED_NAMES".into(), "behavior requires"),
            AbsentMetadataEntry::new(true, "REQUIRED_REFS".into(), "typed behavior requires"),
        ]
    );
}

#[test]
fn behavior_association_absence_validation_uses_module_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.impl_name_code, "E7279");
    assert_eq!(validation.impl_ref_code, "E7378");
    assert_eq!(validation.required_name_code, "E7280");
    assert_eq!(validation.required_ref_code, "E7379");
}

#[test]
fn behavior_association_absence_validation_uses_import_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.impl_name_code, "E7295");
    assert_eq!(validation.impl_ref_code, "E7369");
    assert_eq!(validation.required_name_code, "E7296");
    assert_eq!(validation.required_ref_code, "E7370");
}

#[test]
fn behavior_association_absence_validation_uses_local_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.impl_name_code, "E7263");
    assert_eq!(validation.impl_ref_code, "E7387");
    assert_eq!(validation.required_name_code, "E7264");
    assert_eq!(validation.required_ref_code, "E7388");
}

#[test]
fn behavior_association_absence_validation_uses_variant_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.impl_name_code, "E7341");
    assert_eq!(validation.impl_ref_code, "E7395");
    assert_eq!(validation.required_name_code, "E7342");
    assert_eq!(validation.required_ref_code, "E7396");
}

#[test]
fn behavior_association_absence_validation_uses_behavior_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::behavior_resolver_codes();

    assert_eq!(validation.impl_name_code, "E7327");
    assert_eq!(validation.impl_ref_code, "E7401");
    assert_eq!(validation.required_name_code, "E7328");
    assert_eq!(validation.required_ref_code, "E7402");
}

#[test]
fn behavior_association_absence_validation_uses_value_resolver_codes() {
    let validation = BehaviorAssociationAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.impl_name_code, "E7306");
    assert_eq!(validation.impl_ref_code, "E7407");
    assert_eq!(validation.required_name_code, "E7307");
    assert_eq!(validation.required_ref_code, "E7408");
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
        method_signature_code: "METHODS".into(),
        method_type_code: "TYPED_METHODS".into(),
        parent_name_code: "PARENTS".into(),
        parent_ref_code: "TYPED_PARENTS".into(),
    }
    .entries(symbol);

    assert_eq!(
        entries,
        [
            AbsentMetadataEntry::new(true, "METHODS".into(), "behavior methods"),
            AbsentMetadataEntry::new(true, "TYPED_METHODS".into(), "typed behavior methods"),
            AbsentMetadataEntry::new(true, "PARENTS".into(), "behavior parents"),
            AbsentMetadataEntry::new(true, "TYPED_PARENTS".into(), "typed behavior parents"),
        ]
    );
}

#[test]
fn behavior_declaration_absence_validation_uses_module_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::module_resolver_codes();

    assert_eq!(validation.method_signature_code, "E7277");
    assert_eq!(validation.method_type_code, "E7376");
    assert_eq!(validation.parent_name_code, "E7278");
    assert_eq!(validation.parent_ref_code, "E7377");
}

#[test]
fn behavior_declaration_absence_validation_uses_import_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::import_resolver_codes();

    assert_eq!(validation.method_signature_code, "E7293");
    assert_eq!(validation.method_type_code, "E7367");
    assert_eq!(validation.parent_name_code, "E7294");
    assert_eq!(validation.parent_ref_code, "E7368");
}

#[test]
fn behavior_declaration_absence_validation_uses_local_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::local_resolver_codes();

    assert_eq!(validation.method_signature_code, "E7261");
    assert_eq!(validation.method_type_code, "E7385");
    assert_eq!(validation.parent_name_code, "E7262");
    assert_eq!(validation.parent_ref_code, "E7386");
}

#[test]
fn behavior_declaration_absence_validation_uses_variant_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::variant_resolver_codes();

    assert_eq!(validation.method_signature_code, "E7339");
    assert_eq!(validation.method_type_code, "E7393");
    assert_eq!(validation.parent_name_code, "E7340");
    assert_eq!(validation.parent_ref_code, "E7394");
}

#[test]
fn behavior_declaration_absence_validation_uses_value_resolver_codes() {
    let validation = BehaviorDeclarationAbsenceValidation::value_resolver_codes();

    assert_eq!(validation.method_signature_code, "E7304");
    assert_eq!(validation.method_type_code, "E7405");
    assert_eq!(validation.parent_name_code, "E7305");
    assert_eq!(validation.parent_ref_code, "E7406");
}
