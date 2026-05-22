use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_behavior_method_types() {
    let program = parse_program(
        r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_behavior_method_types_for_test(
        Namespace::Behavior,
        "Mapper",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "map".to_string(),
            parameter_names: vec!["__arg0".to_string(), "__arg1".to_string()],
            parameter_types: vec![AstType::SelfType, AstType::I32],
            return_type: AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            },
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed behavior method metadata mismatch should fail");

    let expected = "resolver behavior symbol 'Mapper' has typed methods '(map(__arg0: Self, __arg1: i32) (i32) i32)', expected '(map(__arg0: Self, __arg1: (i32) i32) (i32) i32)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver typed behavior method diagnostic, got {err:?}"
    );
}
