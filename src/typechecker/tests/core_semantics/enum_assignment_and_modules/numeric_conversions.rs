use super::*;

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
