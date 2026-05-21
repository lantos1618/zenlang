use super::*;

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
            .contains("payload for enum variant `Maybe.Some` expects `i32`, found `StaticString`")),
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
            .contains("assignment to `x` expects `i32`, found `StaticString`")),
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
