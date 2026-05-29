use super::*;
mod arity_followups;
mod enum_methods;

const BOX_GET_I32: &str = r#"
Box: {
    value: i32
}

Box.get = (self: Box) i32 {
    self.value
}
"#;

const GENERIC_BOX_GET: &str = r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    self.value
}
"#;

#[test]
fn nongeneric_explicit_type_args_are_errors() {
    // The module-function variant (`io.println<i32>`) is covered through the real
    // frontend by the `nongeneric_module_function_type_args` golden, which splices
    // the actual `io` module — no compiler-side stub needed here.
    let source = format!(
        r#"{BOX_GET_I32}
main = () i32 {{
    box = Box {{ value: 1 }}
    box.get<i32>()
}}
"#,
    );
    let errors = typecheck_errors(&source);
    assert_nongeneric_type_args_diagnostic(&errors, "method", "Box.get", "non-generic method type-argument");
}

#[test]
fn generic_method_explicit_type_arg_arity_is_error() {
    let errors = typecheck_errors(&format!(
        r#"{GENERIC_BOX_GET}
main = () i32 {{
    box = Box<i32> {{ value: 1 }}
    box.get<i32, StaticString>()
}}
"#,
    ));

    assert_generic_arity_diagnostic(&errors, "method", "Box.get", 1, 2, "generic method arity");
}

#[test]
fn generic_method_inference_failure_is_error() {
    let errors = typecheck_errors(
        r#"
Box: {
    value: i32
}

Box.make<T> = (self: Box) T {
    self.value
}

main = () i32 {
    box = Box { value: 1 }
    box.make()
}
"#,
    );

    assert_diagnostic_message(
        &errors,
        "cannot infer type argument `T` for generic method `Box.make`",
        "generic method inference",
    );
}

#[test]
fn generic_method_argument_arity_uses_method_diagnostic() {
    let errors = typecheck_errors(&format!(
        r#"{GENERIC_BOX_GET}
main = () i32 {{
    box = Box<i32> {{ value: 1 }}
    box.get(2)
}}
"#,
    ));

    assert_diagnostic_message(
        &errors,
        "method `Box.get` expects 1 arguments, found 2",
        "generic method argument arity",
    );
}
