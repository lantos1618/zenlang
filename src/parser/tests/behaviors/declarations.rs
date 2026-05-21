use super::super::*;

#[test]
fn parse_behavior_declaration() {
    let prog = parse_ok(
        "Serializable: behavior {\n    to_json: (Self) String\n    eq: (left: Self, right: Self) bool\n}",
    );
    match &prog.declarations[0] {
        Declaration::Behavior { name, methods, .. } => {
            assert_eq!(name, "Serializable");
            assert_eq!(methods.len(), 2);
            assert_eq!(methods[0].name, "to_json");
            assert_eq!(methods[0].params.len(), 1);
            assert!(matches!(methods[0].params[0].ty, AstType::SelfType));
            assert_eq!(methods[1].params[0].name, "left");
            assert_eq!(methods[1].params[1].name, "right");
        }
        other => panic!("expected Behavior, got {:?}", other),
    }
}

#[test]
fn parse_public_behavior_declaration() {
    let prog = parse_ok("pub Json<T>: behavior {\n    encode: (Self) T\n}");
    match &prog.declarations[0] {
        Declaration::Behavior {
            name,
            type_params,
            public,
            ..
        } => {
            assert_eq!(name, "Json");
            assert_eq!(type_params.len(), 1);
            assert!(*public);
        }
        other => panic!("expected Behavior, got {:?}", other),
    }
}
