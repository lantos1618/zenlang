use super::*;

#[test]
fn resolver_struct_enum_enum_metadata_tests_stay_split_by_responsibility() {
    let root = read("src/typechecker/tests/resolver_struct_enum_metadata/enum_metadata.rs");
    let payloads =
        read("src/typechecker/tests/resolver_struct_enum_metadata/enum_metadata/payloads.rs");
    let generic_payloads = read(
        "src/typechecker/tests/resolver_struct_enum_metadata/enum_metadata/generic_payloads.rs",
    );
    let variant_shape =
        read("src/typechecker/tests/resolver_struct_enum_metadata/enum_metadata/variant_shape.rs");

    assert!(
        root.lines().count() < 80,
        "enum_metadata.rs should only route focused resolver enum metadata tests"
    );
    for module in [
        "mod generic_payloads;",
        "mod payloads;",
        "mod variant_shape;",
    ] {
        assert!(
            root.contains(module),
            "enum_metadata.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("check_program_with_symbols_validates_resolver_enum_variant_payload_counts"),
        "enum payload metadata tests should live in payloads.rs"
    );
    assert!(
        payloads
            .contains("check_program_with_symbols_validates_resolver_enum_typed_payload_metadata"),
        "payloads.rs should cover typed enum payload metadata"
    );
    assert!(
        generic_payloads
            .contains("check_program_with_symbols_validates_resolver_generic_enum_payload_types"),
        "generic_payloads.rs should cover generic enum payload metadata"
    );
    assert!(
        variant_shape
            .contains("check_program_with_symbols_validates_resolver_enum_variant_owner_names"),
        "variant_shape.rs should cover enum variant ownership metadata"
    );
}

#[test]
fn resolver_struct_enum_metadata_root_stays_split_by_aggregate_kind() {
    let root = read("src/typechecker/tests/resolver_struct_enum_metadata.rs");
    let struct_fields =
        read("src/typechecker/tests/resolver_struct_enum_metadata/struct_fields.rs");
    let absent_kind = read("src/typechecker/tests/resolver_struct_enum_metadata/absent_kind.rs");

    assert!(
        root.lines().count() < 80,
        "resolver_struct_enum_metadata.rs should route focused struct/enum metadata test modules"
    );
    for module in [
        "mod absent_kind;",
        "mod enum_metadata;",
        "mod struct_fields;",
        "mod variant_absence;",
    ] {
        assert!(
            root.contains(module),
            "resolver_struct_enum_metadata.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn check_program_with_symbols_validates_resolver_struct_field_counts"),
        "struct field metadata tests should live in struct_fields.rs"
    );
    assert!(
        struct_fields
            .contains("fn check_program_with_symbols_validates_resolver_struct_field_counts")
            && struct_fields.contains(
                "fn check_program_with_symbols_validates_resolver_generic_struct_field_types"
            ),
        "struct_fields.rs should cover concrete and generic struct field metadata"
    );
    assert!(
        absent_kind.contains(
            "fn check_program_with_symbols_validates_resolver_struct_and_enum_absent_kind_metadata"
        ),
        "absent_kind.rs should cover impossible struct/enum metadata combinations"
    );
}
