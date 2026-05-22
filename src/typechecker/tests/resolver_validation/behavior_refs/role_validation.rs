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
