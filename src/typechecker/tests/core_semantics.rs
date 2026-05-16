use super::*;

#[test]
fn binary_op_types() {
    let tc = TypeChecker::new();
    assert_eq!(
        tc.check_binary_op(BinaryOp::Add, &Type::I32, &Type::I32, &Span::dummy())
            .unwrap(),
        Type::I32
    );
    assert_eq!(
        tc.check_binary_op(BinaryOp::Eq, &Type::I32, &Type::I32, &Span::dummy())
            .unwrap(),
        Type::Bool
    );
    assert_eq!(
        tc.check_binary_op(BinaryOp::And, &Type::Bool, &Type::Bool, &Span::dummy())
            .unwrap(),
        Type::Bool
    );
}

#[test]
fn binary_op_type_mismatch() {
    let tc = TypeChecker::new();
    // Arithmetic on non-numeric type
    assert!(tc
        .check_binary_op(BinaryOp::Add, &Type::I32, &Type::Str, &Span::dummy())
        .is_err());
    assert!(tc
        .check_binary_op(BinaryOp::Add, &Type::Bool, &Type::I32, &Span::dummy())
        .is_err());
    // Logical op on non-bool
    assert!(tc
        .check_binary_op(BinaryOp::And, &Type::I32, &Type::Bool, &Span::dummy())
        .is_err());
    // Unknown is permissive (error recovery)
    assert!(tc
        .check_binary_op(BinaryOp::Add, &Type::Unknown, &Type::Str, &Span::dummy())
        .is_ok());
}

#[test]
fn binary_op_mixed_numeric_width_requires_cast() {
    let tc = TypeChecker::new();
    let err = tc
        .check_binary_op(BinaryOp::Add, &Type::I32, &Type::I64, &Span::dummy())
        .expect_err("mixed integer arithmetic should fail");
    assert!(
        err.message
            .contains("arithmetic operands must have the same type"),
        "expected mixed numeric diagnostic, got {err:?}"
    );

    let err = tc
        .check_binary_op(BinaryOp::Mul, &Type::F32, &Type::F64, &Span::dummy())
        .expect_err("mixed float arithmetic should fail");
    assert!(
        err.message
            .contains("arithmetic operands must have the same type"),
        "expected mixed numeric diagnostic, got {err:?}"
    );
}

#[test]
fn unknown_function_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "main".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: Some(AstType::Void),
            body: Expression::Block {
                statements: vec![ast::Statement::Expression {
                    expr: Expression::FunctionCall {
                        name: "nonexistent".into(),
                        module: None,
                        type_args: Vec::new(),
                        args: Vec::new(),
                        span: Span::dummy(),
                    },
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
    let result = tc.check_program(&program);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("undefined function")));
}

#[test]
fn return_type_mismatch_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "foo".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: Some(AstType::I32),
            body: Expression::Block {
                statements: Vec::new(),
                expr: Some(Box::new(Expression::Return {
                    value: Some(Box::new(Expression::StringLiteral {
                        value: "hello".into(),
                        span: Span::dummy(),
                    })),
                    span: Span::dummy(),
                })),
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }],
        file_id: 0,
    };
    let result = tc.check_program(&program);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("return type mismatch")));
}

#[test]
fn function_call_wrong_arity_is_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "add".into(),
                type_params: Vec::new(),
                params: vec![
                    ast::Param {
                        name: "a".into(),
                        ty: AstType::I32,
                        mutable: false,
                        span: Span::dummy(),
                    },
                    ast::Param {
                        name: "b".into(),
                        ty: AstType::I32,
                        mutable: false,
                        span: Span::dummy(),
                    },
                ],
                return_type: Some(AstType::I32),
                body: Expression::Block {
                    statements: Vec::new(),
                    expr: Some(Box::new(Expression::Return {
                        value: Some(Box::new(Expression::Identifier {
                            name: "a".into(),
                            span: Span::dummy(),
                        })),
                        span: Span::dummy(),
                    })),
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![ast::Statement::Expression {
                        expr: Expression::FunctionCall {
                            name: "add".into(),
                            module: None,
                            type_args: Vec::new(),
                            args: vec![Expression::IntLiteral {
                                value: 1,
                                span: Span::dummy(),
                            }],
                            span: Span::dummy(),
                        },
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
        .expect_err("wrong arity should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("function `add` expects 2 arguments, found 1")),
        "expected arity diagnostic, got {errors:?}"
    );
}

#[test]
fn function_call_argument_type_mismatch_is_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "takes_i32".into(),
                type_params: Vec::new(),
                params: vec![ast::Param {
                    name: "value".into(),
                    ty: AstType::I32,
                    mutable: false,
                    span: Span::dummy(),
                }],
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: Vec::new(),
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![ast::Statement::Expression {
                        expr: Expression::FunctionCall {
                            name: "takes_i32".into(),
                            module: None,
                            type_args: Vec::new(),
                            args: vec![Expression::StringLiteral {
                                value: "bad".into(),
                                span: Span::dummy(),
                            }],
                            span: Span::dummy(),
                        },
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
        .expect_err("argument type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("argument 1 for `takes_i32` expects `i32`, found `str`")),
        "expected argument type diagnostic, got {errors:?}"
    );
}

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

#[test]
fn enum_variant_unknown_variant_is_error() {
    let program = parse_program(
        r#"
Status: Ok, Err

main = () void {
    value = Status.Pending
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown enum variant should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("enum `Status` has no variant `Pending`")),
        "expected unknown variant diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_variant_payload_type_mismatch_is_error() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

main = () void {
    value = Maybe.Some("bad")
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("enum payload type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("payload for enum variant `Maybe.Some` expects `i32`, found `str`")),
        "expected payload type diagnostic, got {errors:?}"
    );
}

#[test]
fn assignment_to_immutable_binding_is_error() {
    let program = parse_program(
        r#"
main = () void {
    x = 1
    x = 2
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("immutable assignment should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("cannot assign to immutable variable `x`")),
        "expected immutable assignment diagnostic, got {errors:?}"
    );
}

#[test]
fn assignment_to_mutable_closure_parameter_is_allowed() {
    let program = parse_program(
        r#"
main = () void {
    mapper = (mut input: i32) i32 {
        input = input + 1
        input
    }
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("mutable closure parameter assignment should pass");
}

#[test]
fn assignment_type_mismatch_is_error() {
    let program = parse_program(
        r#"
main = () void {
    x ::= 1
    x = "bad"
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("assignment type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("assignment to `x` expects `i32`, found `str`")),
        "expected assignment type diagnostic, got {errors:?}"
    );
}

#[test]
fn invalid_field_access_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = () void {
    p = Point { x: 1 }
    y = p.y
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("invalid field access should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("type `Point` has no field `y`")),
        "expected invalid field diagnostic, got {errors:?}"
    );
}

#[test]
fn implicit_integer_width_conversion_is_error() {
    let program = parse_program(
        r#"
take_i64 = (value: i64) void {}

main = () void {
    x: i32 = 1
    take_i64(x)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("implicit integer conversion should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("argument 1 for `take_i64` expects `i64`, found `i32`")),
        "expected integer conversion diagnostic, got {errors:?}"
    );
}

#[test]
fn implicit_float_width_conversion_is_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "take_f32".into(),
            type_params: Vec::new(),
            params: vec![ast::Param {
                name: "value".into(),
                ty: AstType::F32,
                mutable: false,
                span: Span::dummy(),
            }],
            return_type: Some(AstType::Void),
            body: Expression::Block {
                statements: Vec::new(),
                expr: None,
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }],
        file_id: 0,
    };
    tc.collect_declarations(&program.declarations);

    let expected = tc.functions["take_f32"].params[0].1.clone();
    assert!(!tc.types_compatible(&tc.resolve_type(&expected), &Type::F64));
}

#[test]
fn unknown_root_std_module_call_is_error() {
    let program = parse_program(
        r#"
{ io } = std

main = () void {
    io.nope("bad")
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown std module function should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("undefined module function `io.nope`")),
        "expected undefined module function diagnostic, got {errors:?}"
    );
}

#[test]
fn known_root_std_runtime_standins_remain_allowed() {
    let program = parse_program(
        r#"
{ io } = std

main = () void {
    io.print("hello")
    io.println("world")
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("temporary root std io stand-ins should typecheck");
}

#[test]
fn non_void_function_without_return_is_error() {
    let program = parse_program(
        r#"
missing = () i32 {
    x = 1
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-void fallthrough should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("function `missing` must return `i32` on all non-error paths")),
        "expected missing return diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_missing_variant_is_error() {
    let program = parse_program(
        r#"
Color: Red, Green, Blue

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Green { "green" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-exhaustive enum match should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-exhaustive match on `Color`: missing `Blue`")),
        "expected non-exhaustive enum diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_duplicate_variant_is_error() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Red { "again" }
        | Green { "green" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("duplicate enum match arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate match arm for `Color.Red`")),
        "expected duplicate enum arm diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_unknown_variant_is_error() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Blue { "blue" }
        | Green { "green" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown enum match arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("enum `Color` has no variant `Blue`")),
        "expected unknown enum arm diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_payload_shape_is_checked() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

describe = (m: Maybe) StaticString {
    m ?
        | Some { "some" }
        | None(value) { "none" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("enum match payload shape should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("match arm `Maybe.Some` requires a payload")),
        "expected missing payload diagnostic, got {errors:?}"
    );
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("match arm `Maybe.None` does not accept a payload")),
        "expected forbidden payload diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_wildcard_after_all_variants_is_redundant() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Green { "green" }
        | _ { "fallback" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("redundant enum wildcard arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("redundant wildcard match arm")),
        "expected redundant wildcard diagnostic, got {errors:?}"
    );
}

#[test]
fn enum_match_variant_after_wildcard_is_redundant() {
    let program = parse_program(
        r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | _ { "fallback" }
        | Red { "red" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("enum variant after wildcard should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("redundant match arm for `Color.Red`")),
        "expected redundant enum arm diagnostic, got {errors:?}"
    );
}

#[test]
fn bool_match_missing_arm_is_error_for_value_match() {
    let program = parse_program(
        r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "yes" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-exhaustive boolean value match should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-exhaustive bool match: missing `false`")),
        "expected non-exhaustive bool diagnostic, got {errors:?}"
    );
}

#[test]
fn bool_match_duplicate_arm_is_error() {
    let program = parse_program(
        r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "yes" }
        | true { "again" }
        | false { "no" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("duplicate boolean match arm should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("duplicate match arm for `true`")),
        "expected duplicate bool arm diagnostic, got {errors:?}"
    );
}

#[test]
fn match_arm_return_does_not_force_never_result_type() {
    let program = parse_program(
        r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "early" }
        | false { "late" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_program(&program)
        .expect("returning arm should not force match type to never");
    let body = &typed.functions[0].body;
    assert_eq!(body.ty, Type::Str);
}

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
    // Clear mismatch
    assert!(!tc.types_compatible(&Type::I32, &Type::Str));
    assert!(!tc.types_compatible(&Type::Bool, &Type::I32));
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
