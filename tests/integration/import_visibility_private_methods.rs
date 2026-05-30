use super::*;

#[test]
fn imported_private_type_impl_methods_are_not_visible() {
    let message = compile_error_message_for_modules(
        &[(
            "model.zen",
            r#"
Box<T>: {
    value: T
}

Box.impl = {
    get<T> = (self: Box<T>) T {
        self.value
    }
}
@export({ Box })
"#,
        )],
        r#"
{ Box } = model

main = () i32 {
    box = Box<i32> { value: 34 }
    box.get<i32>()
}
"#,
        "compile_to_c should reject private imported impl methods",
    );
    assert_message_contains_any(
        &message,
        &["type `Box_i32` has no method `get`"],
        "expected private imported impl method diagnostic",
    );
}

#[test]
fn imported_private_behavior_impl_methods_are_not_directly_visible() {
    let message = compile_error_message_for_modules(
        &[(
            "model.zen",
            r#"
Hidden: behavior {
    reveal: (Self) StaticString
}

Point: {
    x: i32
}

Point.implements(Hidden) {
    reveal = (value: Point) StaticString {
        "hidden"
    }
}
@export({ Point })
"#,
        )],
        r#"
{ Point } = model

main = () i32 {
    point = Point { x: 34 }
    point.reveal()
}
"#,
        "compile_to_c should reject private imported behavior impl methods",
    );
    assert_message_contains_any(
        &message,
        &["type `Point` has no method `reveal`"],
        "expected private imported behavior impl method diagnostic",
    );
}
