use super::*;

#[test]
fn template_dependency_entries_use_named_fields() {
    let entry = TemplateDependencyEntry::<StructInfo> {
        name: "Point".to_string(),
        previous: None,
    };

    assert_eq!(entry.name, "Point");
    assert!(entry.previous.is_none());
}

#[test]
fn behavior_default_synthesis_skip_requires_resolver_collection_and_missing_impl_ref() {
    let mut tc = TypeChecker::new();
    tc.resolver_missing_behavior_impl_refs
        .insert("Point".to_string());

    assert!(!tc.should_skip_behavior_default_synthesis("Point"));
    tc.resolver_backed_collection = true;
    assert!(tc.should_skip_behavior_default_synthesis("Point"));
    assert!(!tc.should_skip_behavior_default_synthesis("Other"));
}

#[test]
fn resolver_backed_behavior_collection_defers_generic_metadata_to_resolver() {
    let program = parse_program(
        r#"
Json<T: Json<T>>: behavior {
    encode: (Self) T {
        1
    }
}
"#,
    );
    let mut tc = TypeChecker::new();

    tc.with_resolver_backed_collection(|checker| {
        checker.collect_declarations(&program.declarations);
    });

    let behavior = tc.behaviors.get("Json").expect("behavior stub");
    assert!(
        behavior.type_params.is_empty(),
        "resolver-backed behavior collection should not keep AST generic names before resolver metadata"
    );
    assert!(
        behavior.type_param_bounds.is_empty(),
        "resolver-backed behavior collection should not keep AST generic bounds before resolver metadata"
    );
    assert!(
        behavior.methods[0].default_body.is_some(),
        "resolver-backed behavior collection should still keep default bodies for later resolver metadata restoration"
    );
}
