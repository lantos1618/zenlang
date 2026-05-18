use super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_generic_struct_annotation_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_struct_annotation_arity.zen",
        r#"
Box<T>: {
    value: T
}

read = (box: Box<i32, StaticString>) i32 {
    0
}
"#,
        "generic struct annotation arity",
        "generic struct annotation arity diagnostics should not emit dependent-use followups",
        "tests/fixtures/ir_json/diagnostics_generic_struct_annotation_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_enum_annotation_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_enum_annotation_arity.zen",
        r#"
Option<T>:
    None,
    Some(T)

read = (value: Option<i32, StaticString>) i32 {
    0
}
"#,
        "generic enum annotation arity",
        "generic enum annotation arity diagnostics should not emit dependent-use followups",
        "tests/fixtures/ir_json/diagnostics_generic_enum_annotation_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_struct_annotation_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_struct_annotation_missing_args.zen",
        r#"
Box<T>: {
    value: T
}

read = (box: Box) i32 {
    0
}
"#,
        "generic struct annotation missing args",
        "generic struct annotation missing-args diagnostics should not emit dependent-use followups",
        "tests/fixtures/ir_json/diagnostics_generic_struct_annotation_missing_args.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_enum_annotation_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_enum_annotation_missing_args.zen",
        r#"
Option<T>:
    None,
    Some(T)

read = (value: Option) i32 {
    0
}
"#,
        "generic enum annotation missing args",
        "generic enum annotation missing-args diagnostics should not emit dependent-use followups",
        "tests/fixtures/ir_json/diagnostics_generic_enum_annotation_missing_args.golden.json",
    );
}

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
