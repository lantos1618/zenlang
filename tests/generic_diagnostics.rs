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
