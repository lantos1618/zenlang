use super::*;

#[test]
fn behavior_parent_refs_from_metadata_restores_keys_and_type_args() {
    let tc = TypeChecker::new();
    let metadata = vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::Named("T".to_string())],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ];

    let refs = tc.behavior_parent_refs_from_metadata(&metadata);

    assert_eq!(refs[0].behavior, "Json");
    assert_eq!(refs[0].type_args, vec![AstType::Named("T".to_string())]);
    assert_eq!(refs[0].key, "Json_T");
    assert_eq!(refs[1].behavior, "Debug");
    assert!(refs[1].type_args.is_empty());
    assert_eq!(refs[1].key, "Debug");
}

#[test]
fn behavior_impl_refs_from_metadata_restores_type_and_behavior_keys() {
    let tc = TypeChecker::new();
    let metadata = vec![
        BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::Str],
        },
        BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: vec![],
        },
    ];

    assert_eq!(
        tc.behavior_impl_refs_from_metadata("Point", &metadata),
        vec![
            ("Point".to_string(), "Json_StaticString".to_string()),
            ("Point".to_string(), "Debug".to_string()),
        ]
    );
}
