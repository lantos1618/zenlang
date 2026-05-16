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

#[test]
fn behavior_impl_with_required_method_passes() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("valid behavior impl should typecheck");
}

#[test]
fn behavior_impl_missing_required_method_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("missing behavior method should fail");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "type `Point` implementation of `Json` is missing required method `to_json`"
        )),
        "expected missing behavior method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_can_omit_default_method() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str { "{}" }
}

Point.implements(Json) {
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("behavior impl may omit a method with a default body");
}

#[test]
fn behavior_impl_duplicate_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("duplicate behavior impl should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("duplicate implementation of behavior `Json` for type `Point`")),
        "expected duplicate behavior impl diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_without_type_args_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { "point" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior impl without type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 0")),
        "expected generic behavior impl arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_with_type_args_passes_requires() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior impl should satisfy matching generic requires");
}

#[test]
fn behavior_impl_generic_behavior_type_arg_bound_failure_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { value }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior type argument bound should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json<Point>` required by `T`")),
        "expected generic behavior type argument bound diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Json<Point>) {
    encode = (value: Point) Point { value }
}

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { value }
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior type argument bound should pass when satisfied");
}

#[test]
fn behavior_requires_generic_behavior_type_arg_arity_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<i32, str>)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior requires arity mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 2")),
        "expected generic behavior requires arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_generic_behavior_substitutes_method_signature() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) i32 { 1 }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior impl return mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("method `encode` for behavior `Json_str` expects return `str`, found `i32`")),
        "expected substituted behavior method return diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_overlapping_inherited_behavior_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}

Point.implements(PrettyJson) {
    to_json = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("overlapping inherited behavior impl should fail");
    assert!(
        errors.iter().any(|d| {
            d.message.contains(
                "overlapping implementations of behaviors `Json` and `PrettyJson` for type `Point`",
            )
        }),
        "expected overlapping behavior impl diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_requires_passes_when_impl_exists() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}

Point.requires(Json)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("requires should pass when behavior impl exists");
}

#[test]
fn behavior_requires_rejects_missing_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.requires(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("requires should fail without behavior impl");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement required behavior `Json`")),
        "expected requires missing impl diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_requires_generic_behavior_without_type_args_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) str
}

Point.requires(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior requires without type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 0")),
        "expected generic behavior requires arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_requires_parent_methods() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("extended behavior should require parent methods");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "type `Point` implementation of `PrettyJson` is missing required method `to_json`"
        )),
        "expected inherited missing method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_impl_satisfies_parent_requires() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    to_json = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}

Point.requires(Json)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("implementation of child behavior should satisfy parent requires");
}

#[test]
fn behavior_extends_generic_parent_requires_substituted_methods() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic parent method should be required with substituted signature");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "type `Point` implementation of `PrettyJson` is missing required method `encode`"
        )),
        "expected inherited generic parent missing method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_generic_parent_satisfies_specialized_requires() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}

Point.requires(Json<str>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("child behavior impl should satisfy specialized generic parent requires");
}

#[test]
fn behavior_extends_generic_parent_accepts_child_type_parameter_arg() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior parent should accept child type parameter args");
}

#[test]
fn behavior_impl_generic_parent_overlap_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.implements(PrettyJson) {
    encode = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("specialized parent and child behavior impls should overlap");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "overlapping implementations of behaviors `Json_str` and `PrettyJson` for type `Point`"
        )),
        "expected specialized behavior impl overlap diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_distinct_generic_specializations_do_not_overlap() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.implements(Json<i32>) {
    encode = (value: Point) i32 { value.x }
}

Point.requires(Json<str>)
Point.requires(Json<i32>)
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("distinct behavior specializations should not overlap");
}

#[test]
fn behavior_extends_cycle_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

Json.extends(PrettyJson)
PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("cyclic behavior inheritance should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("behavior inheritance cycle")),
        "expected behavior inheritance cycle diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_duplicate_parent_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("duplicate behavior inheritance edge should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("duplicate behavior inheritance `PrettyJson.extends(Json)`")
        }),
        "expected duplicate behavior inheritance diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_duplicate_generic_parent_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
PrettyJson.extends(Json<str>)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("duplicate specialized behavior inheritance edge should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("duplicate behavior inheritance `PrettyJson.extends(Json<str>)`")
        }),
        "expected duplicate generic behavior inheritance diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_generic_parent_without_type_args_is_error() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior extends parent without type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic behavior `Json` expects 1 type arguments, found 0")),
        "expected generic behavior extends parent arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_extends_conflicting_method_signature_is_error() {
    let program = parse_program(
        r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    to_json: (Self) i32
}

PrettyJson.extends(Json)
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("conflicting inherited behavior method should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("conflicting behavior method `to_json` inherited by `PrettyJson`")
        }),
        "expected conflicting inherited behavior method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_signature_mismatch_is_error() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: i32) i32 { value }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("behavior impl signature mismatch should fail");
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("parameter 1 for method `to_json`")),
        "expected behavior parameter mismatch diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("expects return `str`, found `i32`")),
        "expected behavior return mismatch diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_explicit_type_arg_arity_is_error() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T {
    value
}

main = () i32 {
    identity<i32, str>(1)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("wrong generic type-argument arity should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic function `identity` expects 1 type arguments, found 2")),
        "expected generic arity diagnostic, got {errors:?}"
    );
}

#[test]
fn nongeneric_function_explicit_type_args_are_error() {
    let program = parse_program(
        r#"
id = (value: i32) i32 {
    value
}

main = () i32 {
    id<i32>(1)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("non-generic function type arguments should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic function `id` does not accept type arguments")),
        "expected non-generic type-argument diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_inference_failure_is_error() {
    let program = parse_program(
        r#"
make_default<T> = () T {
    0
}

main = () i32 {
    make_default()
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("uninferred generic type argument should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("cannot infer type argument `T` for generic function `make_default`")),
        "expected generic inference diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_bound_references_unknown_behavior_is_error() {
    let program = parse_program(
        r#"
show<T: Display> = (value: T) T {
    value
}

main = () i32 {
    show(1)
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown generic behavior bounds should fail");
    assert!(
        errors.iter().any(|d| d.message.contains(
            "generic bound `Display` on type parameter `T` references undefined behavior"
        )),
        "expected generic bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_bound_rejects_unspecialized_generic_behavior() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    to_json: (Self) str
}

encode<T: Json> = (value: T) str {
    "encoded"
}

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("generic behavior bound without type arguments should fail");
    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("generic behavior `Json` expects 1 type arguments, found 0")
        }),
        "expected generic behavior bound arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_behavior_bound_with_type_args_accepts_matching_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<Point>) {
    encode = (value: Point) Point { value }
}

identity<T: Json<T>> = (value: T) T {
    value
}

main = () i32 {
    p = Point { x: 1 }
    same = identity(p)
    same.x
}
"#,
    );

    TypeChecker::new()
        .check_program(&program)
        .expect("generic behavior bound type argument should substitute at call site");
}

#[test]
fn generic_behavior_bound_with_type_args_rejects_mismatched_impl() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

identity<T: Json<T>> = (value: T) T {
    value
}

main = () i32 {
    p = Point { x: 1 }
    same = identity(p)
    same.x
}
"#,
    );

    let errors = TypeChecker::new()
        .check_program(&program)
        .expect_err("generic behavior bound should require matching behavior type args");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json<Point>` required by `T`")),
        "expected generic behavior bound type argument diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_generic_bound_accepts_later_behavior_declaration() {
    let program = parse_program(
        r#"
Serializable<T: Json>: behavior {
    encode: (Self) str
}

Json: behavior {
    to_json: (Self) str
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
    encode: (Self) str
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
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { "point" }
}

encode<T: Json> = (value: T) str {
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
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    to_json = (value: Point) str { "point" }
    pretty = (value: Point) str { "pretty" }
}

encode<T: Json> = (value: T) str {
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
    to_json: (Self) str
}

encode<T: Json> = (value: T) str {
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
