use super::super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_function_type_parameter_annotation_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "function_type_parameter_annotation_arity.zen",
        r#"
Box<T>: {
    value: T
}

call = (f: (Box<i32, StaticString>) i32) i32 {
    0
}
"#,
        "function type parameter annotation arity",
        "function type parameter annotation arity diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_function_type_parameter_annotation_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_function_type_return_annotation_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "function_type_return_annotation_missing_args.zen",
        r#"
Box<T>: {
    value: T
}

factory = () () Box {
    0
}
"#,
        "function type return annotation missing generic arguments",
        "function type return annotation missing-arguments diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_function_type_return_annotation_missing_args.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_pointer_inner_generic_annotation_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "pointer_inner_generic_annotation_arity.zen",
        r#"
Box<T>: {
    value: T
}

read = (ptr: Ptr<Box<i32, StaticString>>) i32 {
    0
}
"#,
        "pointer inner generic annotation arity",
        "pointer inner generic annotation arity diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_pointer_inner_generic_annotation_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_slice_inner_generic_annotation_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "slice_inner_generic_annotation_missing_args.zen",
        r#"
Box<T>: {
    value: T
}

read = (slice: Slice<Box>) i32 {
    0
}
"#,
        "slice inner generic annotation missing arguments",
        "slice inner generic annotation missing-arguments diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_slice_inner_generic_annotation_missing_args.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_array_inner_generic_annotation_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "array_inner_generic_annotation_arity.zen",
        r#"
Box<T>: {
    value: T
}

read = (items: [Box<i32, StaticString>; 1]) i32 {
    0
}
"#,
        "array inner generic annotation arity",
        "array inner generic annotation arity diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_array_inner_generic_annotation_arity.golden.json",
    );
}
