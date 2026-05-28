use super::*;

#[test]
fn imported_type_method_worklist_helpers_are_not_directly_visible() {
    let message = compile_error_message_for_modules(
        &[(
            "model.zen",
            r#"
inner<T> = (value: T) T {
    value
}

pub Box<T>: {
    value: T
}

pub Box.get_inner<T> = (self: Box<T>) T {
    inner(self.value)
}
"#,
        )],
        r#"
{ Box } = model

main = () i32 {
    inner<i32>(1)
}
"#,
        "compile_to_c should reject direct calls to unimported helpers",
    );
    assert_message_contains_any(
        &message,
        &["unknown value symbol 'inner'", "undefined function `inner`"],
        "expected unimported helper diagnostic",
    );
}

#[test]
fn imported_type_method_dependencies_are_not_directly_visible() {
    let message = compile_error_message_for_modules(
        &[(
            "model.zen",
            r#"
pub Box<T>: {
    value: T
}

Box.inner<T> = (self: Box<T>) T {
    self.value
}

pub Box.get_inner<T> = (self: Box<T>) T {
    self.inner<T>()
}
"#,
        )],
        r#"
{ Box } = model

main = () i32 {
    box = Box<i32> { value: 47 }
    box.inner<i32>()
}
"#,
        "compile_to_c should reject direct calls to unimported methods",
    );
    assert_message_contains_any(
        &message,
        &[
            "type `Box_i32` has no method `inner`",
            "type `Box` has no method `inner`",
        ],
        "expected unimported method diagnostic",
    );
}

#[test]
fn imported_type_method_imported_dependencies_are_not_directly_visible() {
    let message = compile_error_message_for_modules(
        &[
            (
                "helper.zen",
                r#"
pub inner<T> = (value: T) T {
    value
}
"#,
            ),
            (
                "model.zen",
                r#"
{ inner } = helper

pub Box<T>: {
    value: T
}

pub Box.get_inner<T> = (self: Box<T>) T {
    inner(self.value)
}
"#,
            ),
        ],
        r#"
{ Box } = model

main = () i32 {
    inner<i32>(59)
}
"#,
        "compile_to_c should reject direct calls to source-module imports",
    );
    assert_message_contains_any(
        &message,
        &["unknown value symbol 'inner'", "undefined function `inner`"],
        "expected unimported helper diagnostic",
    );
}
