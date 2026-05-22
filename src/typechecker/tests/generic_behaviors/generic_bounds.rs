use super::*;

mod type_args;

#[test]
fn behavior_generic_bound_accepts_later_behavior_declaration() {
    let program = parse_program(
        r#"
Serializable<T: Json>: behavior {
    encode: (Self) StaticString
}

Json: behavior {
    to_json: (Self) StaticString
}

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("behavior generic bounds should be independent of declaration order");
}

#[test]
fn behavior_generic_bound_unknown_behavior_reports_once() {
    let program = parse_program(
        r#"
Serializable<T: Missing>: behavior {
    encode: (Self) StaticString
}

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown behavior generic bound should fail");
    let count = errors
        .iter()
        .filter(|d| {
            d.message.contains(
                "generic bound `Missing` on type parameter `T` references undefined behavior",
            )
        })
        .count();
    assert_eq!(
        count, 1,
        "expected one behavior generic bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_behavior_bound_accepts_type_with_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

Point.implements(Json) {
    to_json = (value: Point) StaticString { "point" }
}

encode<T: Json> = (value: T) StaticString {
    "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("type with behavior impl should satisfy generic bound");
}

#[test]
fn generic_behavior_bound_accepts_inherited_behavior_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    to_json = (value: Point) StaticString { "point" }
    pretty = (value: Point) StaticString { "pretty" }
}

encode<T: Json> = (value: T) StaticString {
    "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    0
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("child behavior impl should satisfy inherited generic bound");
}

#[test]
fn generic_behavior_bound_rejects_type_without_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) StaticString
}

encode<T: Json> = (value: T) StaticString {
    "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("type without behavior impl should not satisfy generic bound");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json`")),
        "expected missing generic bound impl diagnostic, got {errors:?}"
    );
}

#[test]
fn func_info_non_generic_has_empty_type_params() {
    use crate::ast::Expression;
    let mut tc = TypeChecker::new();
    let decls = vec![Declaration::Function {
        name: "add".into(),
        type_params: Vec::new(),
        params: vec![
            crate::ast::Param {
                name: "a".into(),
                ty: AstType::I32,
                mutable: false,
                span: Span::dummy(),
            },
            crate::ast::Param {
                name: "b".into(),
                ty: AstType::I32,
                mutable: false,
                span: Span::dummy(),
            },
        ],
        return_type: Some(AstType::I32),
        body: Expression::Block {
            statements: Vec::new(),
            expr: None,
            span: Span::dummy(),
        },
        public: false,
        span: Span::dummy(),
    }];
    tc.collect_declarations(&decls);
    let info = tc.functions.get("add").unwrap();
    assert!(info.type_params.is_empty());
}
