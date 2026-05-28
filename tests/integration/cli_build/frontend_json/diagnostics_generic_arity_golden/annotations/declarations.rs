use super::super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_declaration_annotation_schemas_match_golden() {
    for (zen_filename, source, failure_context, count_context) in [
        (
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
        ),
        (
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
        ),
        (
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
        ),
        (
            "nongeneric_enum_annotation_type_args.zen",
            r#"
Direction:
    North,
    South

read = (value: Direction<i32>) i32 {
    0
}
"#,
            "non-generic enum annotation type arguments",
            "non-generic enum annotation type-argument diagnostics should not emit dependent-use followups",
        ),
        (
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
        ),
        (
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
        ),
    ] {
        assert_diagnostics_golden(zen_filename, source, failure_context, count_context);
    }
}
