use super::*;

#[test]
fn generic_function_collection() {
    use crate::ast::Expression;
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Function {
        name: "identity".into(),
        type_params: vec![crate::ast::declarations::TypeParam {
            name: "T".into(),
            constraint: None,
            constraint_type_args: Vec::new(),
            span: Span::dummy(),
        }],
        params: vec![crate::ast::Param {
            name: "x".into(),
            ty: AstType::Named("T".into()),
            mutable: false,
            span: Span::dummy(),
        }],
        return_type: Some(AstType::Named("T".into())),
        body: Expression::Block {
            statements: Vec::new(),
            expr: None,
            span: Span::dummy(),
        },
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    let info = tc.functions.get("identity").unwrap();
    assert_eq!(info.type_params, vec!["T".to_string()]);
}

#[test]
fn generic_method_collection() {
    use crate::ast::Expression;
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Method {
        type_name: "Box".into(),
        method_name: "get".into(),
        type_params: vec![crate::ast::declarations::TypeParam {
            name: "T".into(),
            constraint: None,
            constraint_type_args: Vec::new(),
            span: Span::dummy(),
        }],
        params: vec![crate::ast::Param {
            name: "value".into(),
            ty: AstType::Named("T".into()),
            mutable: false,
            span: Span::dummy(),
        }],
        return_type: Some(AstType::Named("T".into())),
        body: Expression::Block {
            statements: Vec::new(),
            expr: None,
            span: Span::dummy(),
        },
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    let info = tc.methods.get("Box.get").unwrap();
    assert_eq!(info.type_params, vec!["T".to_string()]);
    assert!(tc.generic_methods.contains_key("Box.get"));
}

#[test]
fn type_impl_method_collection() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 {
        self.x
    }
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.collect_declarations(&program.declarations);
    let info = tc.methods.get("Point.get").unwrap();
    assert_eq!(info.params.len(), 1);
    assert_eq!(info.return_type, AstType::I32);
}

#[test]
fn behavior_declaration_collection() {
    let program = parse_program(
        r#"
Serializable: behavior {
    to_json: (Self) String
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.collect_declarations(&program.declarations);
    let info = tc.behaviors.get("Serializable").unwrap();
    assert_eq!(info.name, "Serializable");
    assert_eq!(info.methods.len(), 1);
    assert_eq!(info.methods[0].name, "to_json");
}
