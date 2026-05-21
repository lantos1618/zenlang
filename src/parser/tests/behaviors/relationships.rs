use super::super::*;

#[test]
fn parse_behavior_requires_assertion() {
    let prog = parse_ok("Point.requires(Json)");
    match &prog.declarations[0] {
        Declaration::Requires {
            type_name,
            behavior,
            ..
        } => {
            assert_eq!(type_name, "Point");
            assert_eq!(behavior, "Json");
        }
        other => panic!("expected Requires, got {:?}", other),
    }
}

#[test]
fn parse_behavior_extends_declaration() {
    let prog = parse_ok("PrettyPrint.extends(Json)");
    match &prog.declarations[0] {
        Declaration::BehaviorExtends {
            behavior, parent, ..
        } => {
            assert_eq!(behavior, "PrettyPrint");
            assert_eq!(parent, "Json");
        }
        other => panic!("expected BehaviorExtends, got {:?}", other),
    }
}

#[test]
fn parse_generated_behavior_derive_association() {
    let prog = parse_ok("Point.derive(Json)");
    match &prog.declarations[0] {
        Declaration::Derive {
            type_name,
            behavior,
            behavior_type_args,
            ..
        } => {
            assert_eq!(type_name, "Point");
            assert_eq!(behavior, "Json");
            assert!(behavior_type_args.is_empty());
        }
        other => panic!("expected Derive, got {:?}", other),
    }
}
