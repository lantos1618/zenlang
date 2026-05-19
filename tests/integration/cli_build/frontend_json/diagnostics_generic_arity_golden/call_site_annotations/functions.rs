use super::super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_generic_function_type_arg_annotation_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_function_type_arg_annotation_arity.zen",
        r#"
Box<T>: {
    value: T
}

identity<T> = (value: T) T {
    value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = identity<Box<i32, StaticString>>(box)
    0
}
"#,
        "generic function type-argument annotation arity",
        "generic function type-argument annotation arity diagnostics should not emit argument followups",
        "tests/fixtures/ir_json/diagnostics_generic_function_type_arg_annotation_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_nongeneric_function_type_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "nongeneric_function_type_args.zen",
        r#"
id = (value: i32) i32 {
    value
}

main = () i32 {
    id<i32>(1)
}
"#,
        "non-generic function type arguments",
        "non-generic function type-argument diagnostics should not emit argument followups",
        "tests/fixtures/ir_json/diagnostics_nongeneric_function_type_args.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_nongeneric_module_function_type_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "nongeneric_module_function_type_args.zen",
        r#"
{ io } = std

main = () i32 {
    io.println<i32>("bad")
    0
}
"#,
        "non-generic module function type arguments",
        "non-generic module function type-argument diagnostics should not emit argument followups",
        "tests/fixtures/ir_json/diagnostics_nongeneric_module_function_type_args.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_nongeneric_builtin_function_type_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "nongeneric_builtin_function_type_args.zen",
        r#"
main = () i32 {
    @builtin.panic<i32>("bad")
    0
}
"#,
        "non-generic builtin function type arguments",
        "non-generic builtin function type-argument diagnostics should not emit argument followups",
        "tests/fixtures/ir_json/diagnostics_nongeneric_builtin_function_type_args.golden.json",
    );
}
