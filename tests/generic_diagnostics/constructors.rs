use super::*;

#[test]
fn generic_struct_type_arg_arity_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box<i32, StaticString> { value: 1 }
    box.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 2")),
        "expected generic struct arity diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("field `value` for struct `Box`")),
        "malformed generic struct constructor should not also report field mismatch, got {errors:?}"
    );
}

#[test]
fn generic_struct_constructor_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box { value: 1 }
    box.value
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic struct `Box` expects 1 type arguments, found 0")),
        "expected unspecialized generic struct constructor diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("field `value` for struct `Box`")),
        "malformed generic struct constructor should not also report field mismatch, got {errors:?}"
    );
}

#[test]
fn nongeneric_struct_constructor_type_args_are_error() {
    let errors = typecheck_errors(
        r#"
Point: {
    x: i32
}

main = () i32 {
    point = Point<i32> { x: 1 }
    point.x
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic struct `Point` does not accept type arguments")),
        "expected non-generic struct constructor type-argument diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("field `x` for struct `Point`")),
        "malformed non-generic struct constructor should not also report field mismatch, got {errors:?}"
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
    value = Option<i32, StaticString>.Some(1)
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 2")),
        "expected generic enum arity diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("payload for enum variant")),
        "malformed generic enum constructor should not also report payload mismatch, got {errors:?}"
    );
}

#[test]
fn generic_enum_constructor_without_type_args_is_error() {
    let errors = typecheck_errors(
        r#"
Option<T>:
    None,
    Some(T)

main = () i32 {
    value = Option.Some(1)
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("generic enum `Option` expects 1 type arguments, found 0")),
        "expected unspecialized generic enum constructor diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("payload for enum variant")),
        "malformed generic enum constructor should not also report payload mismatch, got {errors:?}"
    );
}

#[test]
fn nongeneric_enum_constructor_type_args_are_error() {
    let errors = typecheck_errors(
        r#"
Status:
    Ready,
    Done(i32)

main = () i32 {
    value = Status<i32>.Done(1)
    0
}
"#,
    );

    assert!(
        errors.iter().any(|d| d
            .message
            .contains("non-generic enum `Status` does not accept type arguments")),
        "expected non-generic enum constructor type-argument diagnostic, got {errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|d| !d.message.contains("payload for enum variant")),
        "malformed non-generic enum constructor should not also report payload mismatch, got {errors:?}"
    );
}
