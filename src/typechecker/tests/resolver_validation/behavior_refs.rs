use super::*;

#[test]
fn behavior_ref_validation_maps_role_and_check_diagnostics() {
    let cases = [
        (
            BehaviorRefRole::Parent,
            BehaviorRefCheck::Contains,
            ("behavior", "parents", "parent refs", "E0235", "E0245"),
        ),
        (
            BehaviorRefRole::Parent,
            BehaviorRefCheck::List,
            ("behavior", "parents", "parent refs", "E0240", "E0246"),
        ),
        (
            BehaviorRefRole::Impl,
            BehaviorRefCheck::Contains,
            (
                "type",
                "behavior impls",
                "behavior impl refs",
                "E0236",
                "E0247",
            ),
        ),
        (
            BehaviorRefRole::Impl,
            BehaviorRefCheck::List,
            (
                "type",
                "behavior impls",
                "behavior impl refs",
                "E0238",
                "E0248",
            ),
        ),
        (
            BehaviorRefRole::Required,
            BehaviorRefCheck::Contains,
            (
                "type",
                "behavior requires",
                "behavior requires refs",
                "E0237",
                "E0249",
            ),
        ),
        (
            BehaviorRefRole::Required,
            BehaviorRefCheck::List,
            (
                "type",
                "behavior requires",
                "behavior requires refs",
                "E0239",
                "E0250",
            ),
        ),
    ];

    for (role, check, expected) in cases {
        let validation = BehaviorRefValidation::for_role(role, check);
        assert_eq!(
            (
                validation.symbol_kind,
                validation.name_label,
                validation.ref_label,
                validation.name_code,
                validation.ref_code,
            ),
            expected
        );
    }

    let contains =
        BehaviorRefValidation::for_role(BehaviorRefRole::Impl, BehaviorRefCheck::Contains);
    assert_eq!(
            contains.contains_name_message("Point", "PrettyJson", "Json<StaticString>"),
            "resolver type symbol 'Point' has behavior impls 'PrettyJson', expected to include 'Json<StaticString>'"
        );
    assert_eq!(
            contains.contains_ref_message("Point", "PrettyJson", "Json<StaticString>"),
            "resolver type symbol 'Point' has behavior impl refs 'PrettyJson', expected to include 'Json<StaticString>'"
        );

    let list = BehaviorRefValidation::for_role(BehaviorRefRole::Parent, BehaviorRefCheck::List);
    assert_eq!(
        list.list_name_message("PrettyJson", "Json, Debug", "Json"),
        "resolver behavior symbol 'PrettyJson' has parents 'Json, Debug', expected 'Json'"
    );
    assert_eq!(
        list.list_ref_message("PrettyJson", "Json, Debug", "Json"),
        "resolver behavior symbol 'PrettyJson' has parent refs 'Json, Debug', expected 'Json'"
    );
}

#[test]
fn behavior_ref_validation_separates_role_labels_from_check_codes() {
    let parent = BehaviorRefValidation::role_labels(BehaviorRefRole::Parent);
    let implementation = BehaviorRefValidation::role_labels(BehaviorRefRole::Impl);
    let required = BehaviorRefValidation::role_labels(BehaviorRefRole::Required);
    let parent_contains =
        BehaviorRefValidation::codes_for(BehaviorRefRole::Parent, BehaviorRefCheck::Contains);
    let parent_list =
        BehaviorRefValidation::codes_for(BehaviorRefRole::Parent, BehaviorRefCheck::List);

    assert_eq!(parent, ("behavior", "parents", "parent refs"));
    assert_eq!(
        implementation,
        ("type", "behavior impls", "behavior impl refs")
    );
    assert_eq!(
        required,
        ("type", "behavior requires", "behavior requires refs")
    );
    assert_eq!(parent_contains, ("E0235", "E0245"));
    assert_eq!(parent_list, ("E0240", "E0246"));
}

#[test]
fn behavior_ref_actual_selects_role_metadata() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)

Point.implements(PrettyJson) {
    encode = (value: Point) StaticString { "point" }
    pretty = (value: Point) StaticString { "pretty" }
}

Point.requires(Json<StaticString>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let behavior = symbols
        .lookup(Namespace::Behavior, "PrettyJson")
        .expect("behavior symbol");
    let ty = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");

    let parent = BehaviorRefActual::for_role(behavior, BehaviorRefRole::Parent);
    assert_eq!(
        format_behavior_ref_names(parent.names),
        "Json<StaticString>"
    );
    assert_eq!(format_behavior_refs(parent.refs), "Json<StaticString>");

    let implementation = BehaviorRefActual::for_role(ty, BehaviorRefRole::Impl);
    assert_eq!(
        format_behavior_ref_names(implementation.names),
        "PrettyJson"
    );
    assert_eq!(format_behavior_refs(implementation.refs), "PrettyJson");

    let required = BehaviorRefActual::for_role(ty, BehaviorRefRole::Required);
    assert_eq!(
        format_behavior_ref_names(required.names),
        "Json<StaticString>"
    );
    assert_eq!(format_behavior_refs(required.refs), "Json<StaticString>");
}

#[test]
fn behavior_ref_actual_exposes_role_metadata_selection() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)

Point.implements(PrettyJson) {
    encode = (value: Point) StaticString { "point" }
    pretty = (value: Point) StaticString { "pretty" }
}

Point.requires(Json<StaticString>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let behavior = symbols
        .lookup(Namespace::Behavior, "PrettyJson")
        .expect("behavior symbol");
    let ty = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");

    let (parent_names, parent_refs) =
        BehaviorRefActual::metadata_for_role(behavior, BehaviorRefRole::Parent);
    let (impl_names, impl_refs) = BehaviorRefActual::metadata_for_role(ty, BehaviorRefRole::Impl);
    let (required_names, required_refs) =
        BehaviorRefActual::metadata_for_role(ty, BehaviorRefRole::Required);

    assert_eq!(
        format_behavior_ref_names(parent_names),
        "Json<StaticString>"
    );
    assert_eq!(format_behavior_refs(parent_refs), "Json<StaticString>");
    assert_eq!(format_behavior_ref_names(impl_names), "PrettyJson");
    assert_eq!(format_behavior_refs(impl_refs), "PrettyJson");
    assert_eq!(
        format_behavior_ref_names(required_names),
        "Json<StaticString>"
    );
    assert_eq!(format_behavior_refs(required_refs), "Json<StaticString>");
}

#[test]
fn behavior_ref_actual_matches_expected_edges() {
    let names = vec!["Json<i32>".to_string()];
    let refs = vec![BehaviorRefMetadata {
        name: "Json".to_string(),
        type_args: vec![AstType::I32],
    }];
    let actual = BehaviorRefActual {
        names: Some(&names),
        refs: Some(&refs),
    };
    let expected = expected_behavior_edge("Json", &[AstType::I32]);
    let mismatch = expected_behavior_edge("Debug", &[]);
    let expected_list = ExpectedBehaviorEdgeMetadata::from_edges(std::slice::from_ref(&expected));

    assert!(actual.contains_display(&expected.display));
    assert!(actual.contains_metadata(&expected.metadata));
    assert!(!actual.contains_display(&mismatch.display));
    assert!(!actual.contains_metadata(&mismatch.metadata));
    assert!(actual.names_match(&expected_list.names));
    assert!(actual.refs_match(&expected_list.refs));
}
