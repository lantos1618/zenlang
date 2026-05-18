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
fn emit_json_diagnostics_nongeneric_struct_annotation_type_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "nongeneric_struct_annotation_type_args.zen",
        r#"
Point: {
    x: i32
}

read = (point: Point<i32>) i32 {
    point.x
}
"#,
        "non-generic struct annotation type arguments",
        "non-generic struct annotation type-argument diagnostics should not emit dependent-use followups",
        "tests/fixtures/ir_json/diagnostics_nongeneric_struct_annotation_type_args.golden.json",
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
fn emit_json_diagnostics_generic_struct_local_annotation_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_struct_local_annotation_arity.zen",
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box: Box<i32, StaticString> = Box<i32> { value: 1 }
    box.value
}
"#,
        "generic struct local annotation arity",
        "generic struct local annotation arity diagnostics should not emit dependent-use followups",
        "tests/fixtures/ir_json/diagnostics_generic_struct_local_annotation_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_struct_local_annotation_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_struct_local_annotation_missing_args.zen",
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box: Box = Box<i32> { value: 1 }
    0
}
"#,
        "generic struct local annotation missing arguments",
        "generic struct local annotation missing-arguments diagnostics should not emit dependent-use followups",
        "tests/fixtures/ir_json/diagnostics_generic_struct_local_annotation_missing_args.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_enum_local_annotation_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_enum_local_annotation_arity.zen",
        r#"
Option<T>:
    None,
    Some(T)

main = () i32 {
    value: Option<i32, StaticString> = Option<i32>.Some(1)
    0
}
"#,
        "generic enum local annotation arity",
        "generic enum local annotation arity diagnostics should not emit dependent-use followups",
        "tests/fixtures/ir_json/diagnostics_generic_enum_local_annotation_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_enum_local_annotation_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_enum_local_annotation_missing_args.zen",
        r#"
Option<T>:
    None,
    Some(T)

main = () i32 {
    value: Option = Option<i32>.Some(1)
    0
}
"#,
        "generic enum local annotation missing arguments",
        "generic enum local annotation missing-arguments diagnostics should not emit dependent-use followups",
        "tests/fixtures/ir_json/diagnostics_generic_enum_local_annotation_missing_args.golden.json",
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
