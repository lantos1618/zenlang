use super::*;

#[test]
fn types_compatible_basics() {
    let tc = TypeChecker::new();
    // Same types
    assert!(tc.types_compatible(&Type::I32, &Type::I32));
    // Numeric conversions require explicit casts except literal coercion.
    assert!(!tc.types_compatible(&Type::I64, &Type::I32));
    assert!(!tc.types_compatible(&Type::F32, &Type::F64));
    // Unknown is permissive
    assert!(tc.types_compatible(&Type::I32, &Type::Unknown));
    // Named types are nominal and do not match unrelated concrete types.
    assert!(tc.types_compatible(&Type::Named("UserId".into()), &Type::Named("UserId".into())));
    assert!(!tc.types_compatible(
        &Type::Named("UserId".into()),
        &Type::Named("OrderId".into())
    ));
    assert!(!tc.types_compatible(&Type::Str, &Type::Named("StaticString".into())));
    assert!(!tc.types_compatible(&Type::String, &Type::Str));
    assert!(!tc.types_compatible(&Type::Str, &Type::String));
    // Clear mismatch
    assert!(!tc.types_compatible(&Type::I32, &Type::Str));
    assert!(!tc.types_compatible(&Type::Bool, &Type::I32));
}

#[test]
fn static_string_literal_does_not_implicitly_allocate_string() {
    let program = parse_program(
        r#"
takes_string = (value: String) void { }

returns_string = () String {
    "literal"
}

main = () void {
    local: String = "literal"
    takes_string("literal")
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("static string literals should not implicitly satisfy dynamic String");

    for expected in [
        "return type mismatch: expected `String`, found `StaticString`",
        "variable `local` expects `String`, found `StaticString`",
        "argument 1 for `takes_string` expects `String`, found `StaticString`",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
}

#[test]
fn literal_coercion_in_var_decl() {
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "main".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: Some(AstType::Void),
            body: Expression::Block {
                statements: vec![Statement::VarDecl {
                    name: "x".into(),
                    ty: Some(AstType::I64),
                    value: Expression::IntLiteral {
                        value: 42,
                        span: Span::dummy(),
                    },
                    mutable: false,
                    constant: false,
                    span: Span::dummy(),
                }],
                expr: None,
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }],
        file_id: 0,
    };
    let result = tc.check_program(&program).unwrap();
    // The variable should have type I64 (coerced from I32 literal)
    let body = &result.functions[0].body;
    match &body.statements[0].kind {
        TypedStatementKind::VarDecl { ty, .. } => assert_eq!(*ty, Type::I64),
        _ => panic!("expected VarDecl"),
    }
}

#[test]
fn resolve_string_type() {
    let tc = TypeChecker::new();
    // "String" as a named type should resolve to Type::String
    assert_eq!(
        tc.resolve_type(&AstType::Named("String".into())),
        Type::String
    );
}

#[test]
fn resolve_slice_type() {
    let tc = TypeChecker::new();
    assert_eq!(
        tc.resolve_type(&AstType::Slice(Box::new(AstType::I32))),
        Type::Slice(Box::new(Type::I32))
    );
}

#[test]
fn infer_type_args_basic() {
    let tc = TypeChecker::new();
    // Generic function: identity<T>(x: T) -> T
    let type_params = vec!["T".to_string()];
    let params = vec![("x".to_string(), AstType::Named("T".into()))];
    let arg_types = vec![Type::I32];
    let subs = tc.infer_type_args(&type_params, &params, &arg_types);
    assert_eq!(subs.get("T"), Some(&Type::I32));
}

#[test]
fn substitute_type_basic() {
    let tc = TypeChecker::new();
    let mut subs = HashMap::new();
    subs.insert("T".to_string(), Type::I32);
    // T → I32
    assert_eq!(
        tc.substitute_type(&AstType::Named("T".into()), &subs),
        Type::I32
    );
    // Ptr<T> → Ptr<I32>
    assert_eq!(
        tc.substitute_type(&AstType::Ptr(Box::new(AstType::Named("T".into()))), &subs),
        Type::Ptr(Box::new(Type::I32))
    );
    // Non-generic type unchanged
    assert_eq!(tc.substitute_type(&AstType::Bool, &subs), Type::Bool);
}

#[test]
fn substitute_type_covers_all_composite_type_shapes() {
    let tc = TypeChecker::new();
    let mut subs = HashMap::new();
    subs.insert("T".to_string(), Type::I32);

    assert_eq!(
        tc.substitute_type(
            &AstType::RawPtr(Box::new(AstType::Named("T".into()))),
            &subs
        ),
        Type::RawPtr(Box::new(Type::I32))
    );
    assert_eq!(
        tc.substitute_type(
            &AstType::MutPtr(Box::new(AstType::Named("T".into()))),
            &subs
        ),
        Type::MutPtr(Box::new(Type::I32))
    );
    assert_eq!(
        tc.substitute_type(&AstType::Slice(Box::new(AstType::Named("T".into()))), &subs),
        Type::Slice(Box::new(Type::I32))
    );
    assert_eq!(
        tc.substitute_type(
            &AstType::Array {
                elem: Box::new(AstType::Named("T".into())),
                size: Some(3),
            },
            &subs,
        ),
        Type::Array {
            elem: Box::new(Type::I32),
            size: Some(3),
        }
    );
    assert_eq!(
        tc.substitute_type(
            &AstType::Function {
                params: vec![AstType::Named("T".into())],
                ret: Box::new(AstType::Named("T".into())),
            },
            &subs,
        ),
        Type::Function {
            params: vec![Type::I32],
            ret: Box::new(Type::I32),
        }
    );
}

#[test]
fn substitute_type_preserves_function_type_arguments_in_nested_generics() {
    let mut tc = TypeChecker::new();
    tc.structs.insert(
        "Box".to_string(),
        StructInfo {
            specialization_scope: None,
            name: "Box".to_string(),
            fields: vec![("value".to_string(), AstType::Named("T".to_string()))],
            field_defaults: HashMap::new(),
            type_params: vec!["T".to_string()],
            type_param_bounds: HashMap::new(),
        },
    );
    let function_type = Type::Function {
        params: vec![Type::I32],
        ret: Box::new(Type::I32),
    };
    let mut subs = HashMap::new();
    subs.insert("T".to_string(), function_type.clone());

    assert_eq!(
        tc.substitute_type(
            &AstType::Generic {
                name: "Box".to_string(),
                type_args: vec![AstType::Named("T".to_string())],
            },
            &subs,
        ),
        Type::Struct {
            name: "Box_fn_i32_ret_i32".to_string(),
            fields: vec![("value".to_string(), function_type)],
        }
    );
}
