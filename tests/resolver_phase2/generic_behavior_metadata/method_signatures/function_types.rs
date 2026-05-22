use super::*;

#[test]
fn resolver_records_behavior_function_type_method_signatures() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Mapper")
            .expect("behavior symbol")
            .behavior_method_signatures
            .as_deref(),
        Some(
            &[(
                "map".to_string(),
                vec!["Self".to_string(), "(i32) i32".to_string()],
                "(i32) i32".to_string()
            )][..]
        )
    );
    assert_eq!(
        table
            .lookup(Namespace::Behavior, "Mapper")
            .expect("behavior symbol")
            .behavior_method_types
            .as_deref(),
        Some(
            &[zen::resolver::BehaviorMethodTypeMetadata {
                name: "map".to_string(),
                parameter_names: vec!["__arg0".to_string(), "__arg1".to_string()],
                parameter_types: vec![
                    zen::ast::AstType::SelfType,
                    zen::ast::AstType::Function {
                        params: vec![zen::ast::AstType::I32],
                        ret: Box::new(zen::ast::AstType::I32),
                    },
                ],
                return_type: zen::ast::AstType::Function {
                    params: vec![zen::ast::AstType::I32],
                    ret: Box::new(zen::ast::AstType::I32),
                },
            }][..]
        )
    );
}
