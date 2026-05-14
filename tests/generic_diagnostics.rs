use zen::error::Diagnostic;
use zen::lexer;
use zen::parser;
use zen::resolver::Resolver;
use zen::typechecker::TypeChecker;

fn typecheck_errors(source: &str) -> Vec<Diagnostic> {
    let tokens = lexer::tokenize(source, 0).expect("lex source");
    let program = parser::parse(tokens, 0).expect("parse source");
    let symbols = Resolver::new()
        .resolve_program(&program)
        .expect("resolve source");
    TypeChecker::new()
        .check_program_with_symbols(&program, &symbols)
        .expect_err("typecheck should fail")
}

fn frontend_errors(source: &str) -> Vec<Diagnostic> {
    let tokens = lexer::tokenize(source, 0).expect("lex source");
    let program = parser::parse(tokens, 0).expect("parse source");
    match Resolver::new().resolve_program(&program) {
        Ok(symbols) => TypeChecker::new()
            .check_program_with_symbols(&program, &symbols)
            .expect_err("typecheck should fail"),
        Err(errors) => errors,
    }
}

#[test]
fn nongeneric_method_explicit_type_args_are_error() {
    let errors = typecheck_errors(
        r#"
Box: {
    value: i32
}

Box.get = (self: Box) i32 {
    return self.value
}

main = () i32 {
    box = Box { value: 1 }
    return box.get<i32>()
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic method `Box.get` does not accept type arguments")),
        "expected non-generic method type-argument diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_method_explicit_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    return self.value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    return box.get<i32, str>()
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic method `Box.get` expects 1 type arguments, found 2")),
        "expected generic method arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_method_inference_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Box: {
    value: i32
}

Box.make<T> = (self: Box) T {
    return self.value
}

main = () i32 {
    box = Box { value: 1 }
    return box.make()
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("cannot infer type argument `T` for generic method `Box.make`")),
        "expected generic method inference diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_type_arg_annotation_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

identity<T> = (value: T) T {
    return value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = identity<Box<i32, str>>(box)
    return bad.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected generic function type-argument annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_method_type_arg_annotation_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Holder: {
    value: i32
}

Holder.wrap<T> = (self: Holder, value: T) T {
    return value
}

main = () i32 {
    holder = Holder { value: 1 }
    box = Box<i32> { value: 1 }
    bad = holder.wrap<Box<i32, str>>(box)
    return bad.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected generic method type-argument annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn closure_param_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    f = (box: Box<i32, str>) i32 {
        return 0
    }
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected closure parameter generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn closure_return_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    f = () Box {
        return Box<i32> { value: 1 }
    }
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected closure return generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn cast_target_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = cast(box, Box<i32, str>)
    return bad.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected cast target generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn as_cast_target_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = box as Box
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected as-cast target generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box<i32, str> { value: 1 }
    return box.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected generic struct arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

main = () i32 {
    value = Option<i32, str>.Some(1)
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 2")),
        "expected generic enum arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

read = (box: Box<i32, str>) i32 {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected generic struct annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

read = (value: Option<i32, str>) i32 {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 2")),
        "expected generic enum annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

read = (box: Box) i32 {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected unspecialized generic struct annotation diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

read = (value: Option) i32 {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 0")),
        "expected unspecialized generic enum annotation diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_local_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box: Box<i32, str> = Box<i32> { value: 1 }
    return box.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected generic struct local annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_local_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box: Box = Box<i32> { value: 1 }
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected unspecialized generic struct local annotation diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_local_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

main = () i32 {
    value: Option<i32, str> = Option<i32>.Some(1)
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 2")),
        "expected generic enum local annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_local_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

main = () i32 {
    value: Option = Option<i32>.Some(1)
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 0")),
        "expected unspecialized generic enum local annotation diagnostic, got {errors:?}"
    );
}

#[test]
fn nested_generic_annotation_inner_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Option<T>:
    None,
    Some(T)

read = (box: Box<Option<i32, str>>) i32 {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 2")),
        "expected nested generic annotation inner arity diagnostic, got {errors:?}"
    );
}

#[test]
fn nested_generic_instantiation_inner_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

Option<T>:
    None,
    Some(T)

main = () i32 {
    value = Box<Option<i32, str>> { value: Option<i32>.Some(1) }
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 2")),
        "expected nested generic instantiation inner arity diagnostic, got {errors:?}"
    );
}

#[test]
fn function_type_parameter_annotation_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

call = (f: (Box<i32, str>) i32) i32 {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected function type parameter generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn function_type_return_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

factory = () () Box {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected function type return generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn pointer_type_inner_generic_annotation_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

read = (ptr: Ptr<Box<i32, str>>) i32 {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected pointer inner generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn slice_type_inner_generic_annotation_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

read = (slice: Slice<Box>) i32 {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected slice inner generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn array_type_inner_generic_annotation_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

read = (items: [Box<i32, str>; 1]) i32 {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected array inner generic annotation arity diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

Box<T: Json>: {
    value: T
}

main = () i32 {
    point = Point { x: 1 }
    box = Box<Point> { value: point }
    return box.value.x
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic struct bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

Option<T: Json>:
    None,
    Some(T)

main = () i32 {
    point = Point { x: 1 }
    value = Option<Point>.Some(point)
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic enum bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_annotation_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

Box<T: Json>: {
    value: T
}

read = (box: Box<Point>) i32 {
    return box.value.x
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic struct annotation bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_annotation_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

Option<T: Json>:
    None,
    Some(T)

read = (value: Option<Point>) i32 {
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic enum annotation bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_struct_local_annotation_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

Box<T: Json>: {
    value: T
}

main = () i32 {
    point = Point { x: 1 }
    box: Box<Point> = Box<Point> { value: point }
    return box.value.x
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic struct local annotation bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_enum_local_annotation_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

Option<T: Json>:
    None,
    Some(T)

main = () i32 {
    point = Point { x: 1 }
    value: Option<Point> = Option<Point>.Some(point)
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic enum local annotation bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_function_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

encode<T: Json> = (value: T) str {
    return value.encode()
}

main = () i32 {
    point = Point { x: 1 }
    text = encode(point)
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic function bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_method_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

Holder: {
    value: i32
}

Holder.wrap<T: Json> = (self: Holder, value: T) T {
    return value
}

main = () i32 {
    holder = Holder { value: 1 }
    point = Point { x: 1 }
    bad = holder.wrap(point)
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic method bound diagnostic, got {errors:?}"
    );
}

#[test]
fn generic_ufc_function_behavior_bound_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Json: behavior {
    encode: (Self) str
}

Point: {
    x: i32
}

as_json<T: Json> = (value: T) str {
    return value.encode()
}

main = () i32 {
    point = Point { x: 1 }
    text = point.as_json()
    return 0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("type `Point` does not implement behavior `Json` required by `T`")),
        "expected generic UFC function bound diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_for_unknown_type_is_error() {
    let errors = frontend_errors(
        r#"
Json: behavior {
    to_json: (Self) str
}

Missing.implements(Json) {
    to_json = (value: Missing) str {
        return "missing"
    }
}
"#,
    );

    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown behavior impl target diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_for_unspecialized_generic_type_is_error() {
    let errors = frontend_errors(
        r#"
Box<T>: {
    value: T
}

Json: behavior {
    to_json: (Self) str
}

Box.implements(Json) {
    to_json = (value: Box) str {
        return "box"
    }
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic type `Box` expects 1 type arguments, found 0")),
        "expected generic impl target arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_requires_unspecialized_generic_type_is_error() {
    let errors = frontend_errors(
        r#"
Box<T>: {
    value: T
}

Json: behavior {
    to_json: (Self) str
}

Box.requires(Json)
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic type `Box` expects 1 type arguments, found 0")),
        "expected generic requires target arity diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_extra_method_is_error() {
    let errors = typecheck_errors(
        r#"
Point: {
    x: i32
}

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str {
        return "point"
    }

    extra = (value: Point) str {
        return "extra"
    }
}
"#,
    );

    assert!(
        errors.iter().any(|d| {
            d.message
                .contains("method `extra` is not declared by behavior `Json`")
        }),
        "expected extra behavior impl method diagnostic, got {errors:?}"
    );
}

#[test]
fn behavior_impl_duplicate_method_is_error() {
    let errors = frontend_errors(
        r#"
Point: {
    x: i32
}

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str {
        return "point"
    }

    to_json = (value: Point) str {
        return "point again"
    }
}
"#,
    );

    assert!(
        errors
            .iter()
            .any(|d| { d.message.contains("duplicate value symbol 'Point.to_json'") }),
        "expected duplicate behavior impl method diagnostic, got {errors:?}"
    );
}
