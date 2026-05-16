use super::*;

#[test]
fn struct_literal_missing_field_is_error() {
    use crate::ast::declarations::StructField;
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Struct {
                name: "Point".into(),
                type_params: Vec::new(),
                fields: vec![
                    StructField {
                        name: "x".into(),
                        ty: AstType::I32,
                        default: None,
                        mutable: false,
                        span: Span::dummy(),
                    },
                    StructField {
                        name: "y".into(),
                        ty: AstType::I32,
                        default: None,
                        mutable: false,
                        span: Span::dummy(),
                    },
                ],
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name: "p".into(),
                        ty: None,
                        value: Expression::StructLiteral {
                            name: "Point".into(),
                            type_args: Vec::new(),
                            fields: vec![(
                                "x".into(),
                                Expression::IntLiteral {
                                    value: 1,
                                    span: Span::dummy(),
                                },
                            )],
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
            },
        ],
        file_id: 0,
    };

    let errors = tc
        .check_program(&program)
        .expect_err("missing struct field should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("missing field `y` for struct `Point`")),
        "expected missing field diagnostic, got {errors:?}"
    );
}

#[test]
fn struct_literal_uses_default_for_omitted_field() {
    use crate::ast::declarations::StructField;
    use crate::ast::typed::{TypedExprKind, TypedStatementKind};
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Struct {
                name: "Point".into(),
                type_params: Vec::new(),
                fields: vec![
                    StructField {
                        name: "x".into(),
                        ty: AstType::I32,
                        default: None,
                        mutable: false,
                        span: Span::dummy(),
                    },
                    StructField {
                        name: "y".into(),
                        ty: AstType::I32,
                        default: Some(Expression::IntLiteral {
                            value: 2,
                            span: Span::dummy(),
                        }),
                        mutable: false,
                        span: Span::dummy(),
                    },
                ],
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name: "p".into(),
                        ty: None,
                        value: Expression::StructLiteral {
                            name: "Point".into(),
                            type_args: Vec::new(),
                            fields: vec![(
                                "x".into(),
                                Expression::IntLiteral {
                                    value: 1,
                                    span: Span::dummy(),
                                },
                            )],
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
            },
        ],
        file_id: 0,
    };

    let typed = tc
        .check_program(&program)
        .expect("defaulted struct field may be omitted");
    let TypedStatementKind::VarDecl { value, .. } = &typed.functions[0].body.statements[0].kind
    else {
        panic!("expected var decl");
    };
    let TypedExprKind::StructLiteral { fields, .. } = &value.kind else {
        panic!("expected struct literal");
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[1].0, "y");
    assert!(matches!(fields[1].1.kind, TypedExprKind::IntLiteral(2)));
}

#[test]
fn generic_struct_literal_uses_substituted_default_for_omitted_field() {
    use crate::ast::declarations::{StructField, TypeParam};
    use crate::ast::typed::{TypedExprKind, TypedStatementKind};
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Struct {
                name: "Box".into(),
                type_params: vec![TypeParam {
                    name: "T".into(),
                    constraint: None,
                    constraint_type_args: Vec::new(),
                    span: Span::dummy(),
                }],
                fields: vec![StructField {
                    name: "value".into(),
                    ty: AstType::Named("T".into()),
                    default: Some(Expression::Block {
                        statements: vec![Statement::VarDecl {
                            name: "same".into(),
                            ty: Some(AstType::Named("T".into())),
                            value: Expression::StringLiteral {
                                value: "fallback".into(),
                                span: Span::dummy(),
                            },
                            mutable: false,
                            constant: false,
                            span: Span::dummy(),
                        }],
                        expr: Some(Box::new(Expression::Identifier {
                            name: "same".into(),
                            span: Span::dummy(),
                        })),
                        span: Span::dummy(),
                    }),
                    mutable: false,
                    span: Span::dummy(),
                }],
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name: "box".into(),
                        ty: None,
                        value: Expression::StructLiteral {
                            name: "Box".into(),
                            type_args: vec![AstType::Str],
                            fields: Vec::new(),
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
            },
        ],
        file_id: 0,
    };

    let typed = tc
        .check_program(&program)
        .expect("generic defaulted struct field may be omitted");
    let TypedStatementKind::VarDecl { value, .. } = &typed.functions[0].body.statements[0].kind
    else {
        panic!("expected var decl");
    };
    let TypedExprKind::StructLiteral { fields, .. } = &value.kind else {
        panic!("expected struct literal");
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "value");
    assert_eq!(fields[0].1.ty, Type::Str);
}

#[test]
fn struct_literal_field_type_mismatch_is_error() {
    use crate::ast::declarations::StructField;
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Struct {
                name: "Point".into(),
                type_params: Vec::new(),
                fields: vec![StructField {
                    name: "x".into(),
                    ty: AstType::I32,
                    default: None,
                    mutable: false,
                    span: Span::dummy(),
                }],
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name: "p".into(),
                        ty: None,
                        value: Expression::StructLiteral {
                            name: "Point".into(),
                            type_args: Vec::new(),
                            fields: vec![(
                                "x".into(),
                                Expression::StringLiteral {
                                    value: "bad".into(),
                                    span: Span::dummy(),
                                },
                            )],
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
            },
        ],
        file_id: 0,
    };

    let errors = tc
        .check_program(&program)
        .expect_err("struct field type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("field `x` for struct `Point` expects `i32`, found `str`")),
        "expected field type diagnostic, got {errors:?}"
    );
}
