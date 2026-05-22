use super::*;

#[test]
fn resolver_method_key_formats_type_qualified_method_name() {
    assert_eq!(resolver_method_key("Point", "get"), "Point.get");
}

#[test]
fn resolver_behavior_impl_method_key_includes_generic_behavior_specialization() {
    assert_eq!(
        resolver_behavior_impl_method_key("Point", "encode", "Json", &[AstType::Str]),
        "Point.encode__Json_StaticString"
    );
    assert_eq!(
        resolver_behavior_impl_method_key(
            "Point",
            "encode",
            "Json",
            &[AstType::Named("Point".to_string())]
        ),
        "Point.encode__Json_Point"
    );
}

#[test]
fn resolver_behavior_impl_method_key_keeps_swapped_target_args_distinct() {
    let target_args = &[AstType::Named("T".into()), AstType::Named("U".into())];
    assert_eq!(
        resolver_behavior_impl_method_key_with_target_args(
            "Pair",
            "rel",
            "Rel",
            target_args,
            target_args
        ),
        "Pair.rel__Rel"
    );
    assert_eq!(
        resolver_behavior_impl_method_key_with_target_args(
            "Pair",
            "rel",
            "Rel",
            &[AstType::Named("U".into()), AstType::Named("T".into())],
            target_args
        ),
        "Pair.rel__Rel_U_T"
    );
}
