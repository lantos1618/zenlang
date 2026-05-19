use super::super::assert_diagnostics_golden;

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
