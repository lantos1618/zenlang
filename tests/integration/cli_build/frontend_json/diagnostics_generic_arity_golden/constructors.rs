use super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_generic_function_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_function_arity.zen",
        r#"
identity<T> = (value: T) T {
    value
}

main = () i32 {
    identity<i32, StaticString>(1)
}
"#,
        "generic function arity",
        "generic function arity diagnostics should not emit inference or argument followups",
        "tests/fixtures/ir_json/diagnostics_generic_function_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_struct_constructor_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_struct_constructor_arity.zen",
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    boxed = Box<i32, StaticString> { value: 1 }
    boxed.value
}
"#,
        "generic struct constructor arity",
        "generic struct constructor arity diagnostics should not emit field mismatch followups",
        "tests/fixtures/ir_json/diagnostics_generic_struct_constructor_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_struct_constructor_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_struct_constructor_missing_args.zen",
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    boxed = Box { value: 1 }
    0
}
"#,
        "generic struct constructor missing args",
        "generic struct constructor missing-args diagnostics should not emit field mismatch followups",
        "tests/fixtures/ir_json/diagnostics_generic_struct_constructor_missing_args.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_enum_constructor_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_enum_constructor_arity.zen",
        r#"
Option<T>:
    Some(T),
    None

main = () i32 {
    value = Option<i32, StaticString>.Some(1)
    0
}
"#,
        "generic enum constructor arity",
        "generic enum constructor arity diagnostics should not emit payload mismatch followups",
        "tests/fixtures/ir_json/diagnostics_generic_enum_constructor_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_enum_constructor_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_enum_constructor_missing_args.zen",
        r#"
Option<T>:
    Some(T),
    None

main = () i32 {
    value = Option.Some(1)
    0
}
"#,
        "generic enum constructor missing args",
        "generic enum constructor missing-args diagnostics should not emit payload mismatch followups",
        "tests/fixtures/ir_json/diagnostics_generic_enum_constructor_missing_args.golden.json",
    );
}
