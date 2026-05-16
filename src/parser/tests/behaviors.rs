use super::*;

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

// ── Additional feature tests ─────────────────────────────

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

#[test]
fn parse_behavior_impl_block() {
    let prog = parse_ok("Point.implements(Json) {\n    to_json = (value: Point) str { \"{}\" }\n}");
    match &prog.declarations[0] {
        Declaration::ImplBlock {
            type_name,
            behavior,
            methods,
            ..
        } => {
            assert_eq!(type_name, "Point");
            assert_eq!(behavior.as_deref(), Some("Json"));
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name(), Some("to_json"));
        }
        other => panic!("expected ImplBlock, got {:?}", other),
    }
}

#[test]
fn parse_impl_block() {
    let prog = parse_ok("Point.impl = {\n    get = (self: Point) i32 { self.x }\n}");
    match &prog.declarations[0] {
        Declaration::ImplBlock {
            type_name,
            behavior,
            methods,
            ..
        } => {
            assert_eq!(type_name, "Point");
            assert_eq!(behavior, &None);
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name(), Some("get"));
        }
        other => panic!("expected ImplBlock, got {:?}", other),
    }
}

#[test]
fn parse_behavior_impl_with_generic_behavior_args() {
    let prog = parse_ok("Point.implements(Json<i32>) { }");
    match &prog.declarations[0] {
        Declaration::ImplBlock {
            behavior,
            behavior_type_args,
            ..
        } => {
            assert_eq!(behavior.as_deref(), Some("Json"));
            assert_eq!(behavior_type_args, &vec![AstType::I32]);
        }
        other => panic!("expected ImplBlock, got {:?}", other),
    }
}

#[test]
fn parse_behavior_requires_with_generic_behavior_args() {
    let prog = parse_ok("Point.requires(Json<i32>)");
    match &prog.declarations[0] {
        Declaration::Requires {
            behavior,
            behavior_type_args,
            ..
        } => {
            assert_eq!(behavior, "Json");
            assert_eq!(behavior_type_args, &vec![AstType::I32]);
        }
        other => panic!("expected Requires, got {:?}", other),
    }
}

#[test]
fn parse_behavior_extends_with_generic_parent_args() {
    let prog = parse_ok("PrettyJson.extends(Json<i32>)");
    match &prog.declarations[0] {
        Declaration::BehaviorExtends {
            behavior,
            parent,
            parent_type_args,
            ..
        } => {
            assert_eq!(behavior, "PrettyJson");
            assert_eq!(parent, "Json");
            assert_eq!(parent_type_args, &vec![AstType::I32]);
        }
        other => panic!("expected BehaviorExtends, got {:?}", other),
    }
}

#[test]
fn parse_generic_behavior_function_bound_with_type_args() {
    let prog = parse_ok("encode<T: Json<T>> = (value: T) str { \"\" }");
    match &prog.declarations[0] {
        Declaration::Function { type_params, .. } => {
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0].name, "T");
            assert_eq!(type_params[0].constraint.as_deref(), Some("Json"));
            assert_eq!(
                type_params[0].constraint_type_args,
                vec![AstType::Named("T".into())]
            );
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_generic_behavior_type_bound_with_type_args() {
    let prog = parse_ok("Box<T: Json<T>>: { value: T }");
    match &prog.declarations[0] {
        Declaration::Struct { type_params, .. } => {
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0].name, "T");
            assert_eq!(type_params[0].constraint.as_deref(), Some("Json"));
            assert_eq!(
                type_params[0].constraint_type_args,
                vec![AstType::Named("T".into())]
            );
        }
        other => panic!("expected Struct, got {:?}", other),
    }
}
