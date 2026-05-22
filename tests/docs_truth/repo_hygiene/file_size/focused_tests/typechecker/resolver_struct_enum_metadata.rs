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
