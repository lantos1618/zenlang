use super::super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_cast_target_annotation_type_arg_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "cast_target_annotation_type_arg_arity.zen",
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = cast(box, Box<i32, StaticString>)
    0
}
"#,
        "cast target annotation type-argument arity",
        "cast target annotation type-argument arity diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_cast_target_annotation_type_arg_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_cast_target_annotation_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "cast_target_annotation_missing_args.zen",
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = cast(box, Box)
    0
}
"#,
        "cast target annotation missing generic arguments",
        "cast target annotation missing-arguments diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_cast_target_annotation_missing_args.golden.json",
    );
}
