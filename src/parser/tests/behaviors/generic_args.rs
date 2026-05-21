use super::super::*;

#[test]
fn generic_type_association_keywords_are_explicitly_gated() {
    for (keyword, source) in [
        ("implements", "Box<T>.implements(Json<T>) { }"),
        ("requires", "Box<T>.requires(Json<T>)"),
        ("extends", "Box<T>.extends(Json<T>)"),
        ("derive", "Box<T>.derive(Json<T>)"),
    ] {
        let errors = parse_err(source);
        let message = errors
            .first()
            .map(ToString::to_string)
            .unwrap_or_else(|| "missing parser error".to_string());

        assert!(
            message.contains(&format!(
                "generic association target `Type<T>.{keyword}` is gated"
            )),
            "expected explicit generic association gate for {keyword}, got {errors:?}"
        );
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
    let prog = parse_ok("encode<T: Json<T>> = (value: T) StaticString { \"\" }");
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
