use super::assert_diagnostics_golden;

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
fn emit_json_diagnostics_generic_method_type_arg_annotation_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_method_type_arg_annotation_arity.zen",
        r#"
Box<T>: {
    value: T
}

Holder: {
    value: i32
}

Holder.wrap<T> = (self: Holder, value: T) T {
    value
}

main = () i32 {
    holder = Holder { value: 1 }
    box = Box<i32> { value: 1 }
    bad = holder.wrap<Box<i32, StaticString>>(box)
    0
}
"#,
        "generic method type-argument annotation arity",
        "generic method type-argument annotation arity diagnostics should not emit argument followups",
        "tests/fixtures/ir_json/diagnostics_generic_method_type_arg_annotation_arity.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_generic_method_type_arg_annotation_missing_args_schema_matches_golden() {
    assert_diagnostics_golden(
        "generic_method_type_arg_annotation_missing_args.zen",
        r#"
Box<T>: {
    value: T
}

Holder: {
    value: i32
}

Holder.wrap<T> = (self: Holder, value: T) T {
    value
}

main = () i32 {
    holder = Holder { value: 1 }
    box = Box<i32> { value: 1 }
    bad = holder.wrap<Box>(box)
    0
}
"#,
        "generic method type-argument annotation missing arguments",
        "generic method type-argument annotation missing-arguments diagnostics should not emit argument followups",
        "tests/fixtures/ir_json/diagnostics_generic_method_type_arg_annotation_missing_args.golden.json",
    );
}

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

#[test]
fn emit_json_diagnostics_nested_generic_annotation_inner_arity_schema_matches_golden() {
    assert_diagnostics_golden(
        "nested_generic_annotation_inner_arity.zen",
        r#"
Box<T>: {
    value: T
}

Option<T>:
    None,
    Some(T)

read = (box: Box<Option<i32, StaticString>>) i32 {
    0
}
"#,
        "nested generic annotation inner arity",
        "nested generic annotation inner arity diagnostics should be stable",
        "tests/fixtures/ir_json/diagnostics_nested_generic_annotation_inner_arity.golden.json",
    );
}
