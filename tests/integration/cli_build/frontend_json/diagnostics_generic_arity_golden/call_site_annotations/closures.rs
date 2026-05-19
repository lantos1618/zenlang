use super::super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_closure_param_annotation_type_arg_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "closure_param_annotation_type_arg_arity.zen",
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    f = (box: Box<i32, StaticString>) i32 {
        0
    }
    0
}
"#,
        "closure parameter annotation type-argument arity",
        "closure parameter annotation type-argument arity diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_closure_param_annotation_type_arg_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_closure_return_annotation_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "closure_return_annotation_missing_args.zen",
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    f = () Box {
        Box<i32> { value: 1 }
    }
    0
}
"#,
        "closure return annotation missing generic arguments",
        "closure return annotation missing-arguments diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_closure_return_annotation_missing_args.golden.json",
    );
}
