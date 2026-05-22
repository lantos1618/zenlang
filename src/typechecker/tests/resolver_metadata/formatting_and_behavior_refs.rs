use super::*;

#[test]
fn resolver_metadata_display_formats_known_and_missing_values() {
    assert_eq!(resolver_metadata_display(Some("Point")), "Point");
    assert_eq!(resolver_metadata_display(None), "unknown");
    assert_eq!(
        resolver_ast_type_metadata_display(Some(&AstType::I32)),
        "i32"
    );
    assert_eq!(resolver_ast_type_metadata_display(None), "unknown");
    assert_eq!(
        optional_ast_type_display(Some(&AstType::Bool), "none"),
        "bool"
    );
    assert_eq!(optional_ast_type_display(None, "none"), "none");
}

#[test]
fn resolver_string_list_display_formats_known_and_missing_lists() {
    let names = vec!["T".to_string(), "U".to_string()];
    assert_eq!(join_resolver_strings(&names), "T, U");
    assert_eq!(
        join_resolver_display_values(&[AstType::I32, AstType::Bool], AstType::display_name),
        "i32, bool"
    );
    assert_eq!(format_resolver_string_list(Some(&names)), "(T, U)");
    assert_eq!(format_resolver_string_list(None), "unknown");
}

#[test]
fn resolver_display_list_formats_mapped_known_and_missing_items() {
    let types = vec![AstType::I32, AstType::Bool];
    assert_eq!(format_ast_type_list(Some(&types)), "(i32, bool)");
    assert_eq!(format_ast_type_list(None), "unknown");

    let bounds = vec![("T".to_string(), "Display".to_string())];
    assert_eq!(format_type_parameter_bounds(Some(&bounds)), "(T: Display)");
    assert_eq!(format_type_parameter_bounds(None), "unknown");

    let bound_refs = vec![TypeParameterBoundRefMetadata {
        type_parameter: "T".to_string(),
        behavior: "Display".to_string(),
        type_args: vec![AstType::I32],
    }];
    assert_eq!(
        format_type_parameter_bound_refs(Some(&bound_refs)),
        "(T: Display<i32>)"
    );
    assert_eq!(format_type_parameter_bound_refs(None), "unknown");
}

#[test]
fn resolver_nonempty_joined_list_formats_present_empty_and_missing_items() {
    let names = vec!["Json".to_string(), "Debug".to_string()];
    assert_eq!(format_behavior_ref_names(Some(&names)), "Json, Debug");
    assert_eq!(format_behavior_ref_names(Some(&[])), "none");
    assert_eq!(format_behavior_ref_names(None), "none");

    let refs = vec![BehaviorRefMetadata {
        name: "Json".to_string(),
        type_args: vec![AstType::I32],
    }];
    assert_eq!(format_behavior_refs(Some(&refs)), "Json<i32>");
    assert_eq!(format_behavior_refs(Some(&[])), "none");
    assert_eq!(format_behavior_refs(None), "none");
}

#[test]
fn resolver_behavior_ref_helpers_share_pop_and_peek_selection() {
    let refs = VecDeque::from(vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ]);
    let mut refs_by_type = HashMap::from([("Point".to_string(), refs.clone())]);

    assert_eq!(
        TypeChecker::peek_resolver_behavior_ref(true, &refs_by_type, "Point", "Debug", &[])
            .map(|reference| reference.name.as_str()),
        Some("Debug")
    );
    assert_eq!(
        TypeChecker::pop_resolver_behavior_ref(true, &mut refs_by_type, "Point", "Debug", &[])
            .map(|reference| reference.name),
        Some("Debug".to_string())
    );

    let mut refs_by_type = HashMap::from([("Point".to_string(), refs)]);
    assert_eq!(
        TypeChecker::peek_resolver_behavior_ref(true, &refs_by_type, "Point", "Missing", &[])
            .map(|reference| reference.name.as_str()),
        Some("Json")
    );
    assert_eq!(
        TypeChecker::pop_resolver_behavior_ref(true, &mut refs_by_type, "Point", "Missing", &[])
            .map(|reference| reference.name),
        Some("Json".to_string())
    );
    assert!(
        TypeChecker::peek_resolver_behavior_ref(false, &refs_by_type, "Point", "Debug", &[])
            .is_none()
    );
    assert!(TypeChecker::pop_resolver_behavior_ref(
        false,
        &mut refs_by_type,
        "Point",
        "Debug",
        &[]
    )
    .is_none());
}

#[test]
fn resolver_behavior_ref_for_selects_impl_and_required_queues_by_role() {
    let mut tc = TypeChecker::new();
    tc.resolver_backed_collection = true;
    tc.resolver_behavior_impl_refs.insert(
        "Point".to_string(),
        VecDeque::from([BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }]),
    );
    tc.resolver_behavior_required_refs.insert(
        "Point".to_string(),
        VecDeque::from([BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        }]),
    );

    assert_eq!(
        tc.resolver_behavior_ref_for(BehaviorRefRole::Impl, "Point", "Json", &[AstType::I32])
            .map(|reference| reference.name),
        Some("Json".to_string())
    );
    assert_eq!(
        tc.resolver_behavior_ref_for(BehaviorRefRole::Required, "Point", "Debug", &[])
            .map(|reference| reference.name),
        Some("Debug".to_string())
    );
    assert!(tc
        .resolver_behavior_ref_for(BehaviorRefRole::Parent, "Point", "Json", &[])
        .is_none());

    tc.resolver_behavior_impl_refs.insert(
        "Point".to_string(),
        VecDeque::from([BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![],
        }]),
    );
    tc.resolver_backed_collection = false;
    assert!(tc
        .resolver_behavior_ref_for(BehaviorRefRole::Impl, "Point", "Json", &[])
        .is_none());
}
