use super::*;
mod imported_type_dependencies;

#[test]
fn imported_generic_function_transitive_dependencies_are_not_directly_visible() {
    let message = compile_error_message_for_modules(
        &[
            (
                "helper.zen",
                r#"
inner<T> = (value: T) T {
    value
}

middle<T> = (value: T) T {
    inner(value)
}
@export({ middle })
"#,
            ),
            (
                "model.zen",
                r#"
{ middle } = helper

outer<T> = (value: T) T {
    middle(value)
}
@export({ outer })
"#,
            ),
        ],
        r#"
{ outer } = model

main = () i32 {
    middle<i32>(89)
}
"#,
        "compile_to_c should reject direct transitive helper calls",
    );
    assert_message_contains_any(
        &message,
        &[
            "unknown value symbol 'middle'",
            "undefined function `middle`",
        ],
        "expected unimported transitive helper diagnostic",
    );
}

#[test]
fn imported_function_signature_type_dependencies_are_not_directly_visible() {
    let message = compile_error_message_for_modules(
        &[(
            "model.zen",
            r#"
Point: {
    x: i32
}

make_point = () Point {
    Point { x: 109 }
}
@export({ Point, make_point })
"#,
        )],
        r#"
{ make_point } = model

main = () i32 {
    point = Point { x: 109 }
    point.x
}
"#,
        "compile_to_c should reject direct signature dependency type use",
    );
    assert_message_contains_any(
        &message,
        &[
            "unknown type symbol 'Point'",
            "unknown type `Point`",
            "unknown struct `Point`",
        ],
        "expected unimported signature dependency type diagnostic",
    );
}
