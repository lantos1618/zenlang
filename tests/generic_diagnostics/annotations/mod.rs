use super::*;
mod local;

#[test]
fn generic_annotation_type_arg_arities_are_errors() {
    for (preamble, use_site, kind, name, found, context) in [
        (
            BOX,
            r#"
read = (box: Box<i32, StaticString>) i32 {
    0
}
"#,
            "struct",
            "Box",
            2,
            "annotation arity",
        ),
        (
            OPTION,
            r#"
read = (value: Option<i32, StaticString>) i32 {
    0
}
"#,
            "enum",
            "Option",
            2,
            "annotation arity",
        ),
        (
            BOX,
            r#"
read = (box: Box) i32 {
    0
}
"#,
            "struct",
            "Box",
            0,
            "annotation missing args",
        ),
        (
            OPTION,
            r#"
read = (value: Option) i32 {
    0
}
"#,
            "enum",
            "Option",
            0,
            "annotation missing args",
        ),
    ] {
        let errors = typecheck_errors(&format!("{preamble}\n{use_site}"));
        assert_generic_arity_diagnostic(&errors, kind, name, 1, found, context);
    }
}

#[test]
fn nongeneric_annotation_type_args_are_errors() {
    for (preamble, use_site, kind, name) in [
        (
            POINT,
            r#"
read = (point: Point<i32>) i32 {
    0
}
"#,
            "struct",
            "Point",
        ),
        (
            STATUS,
            r#"
read = (value: Status<i32>) i32 {
    0
}
"#,
            "enum",
            "Status",
        ),
    ] {
        let errors = typecheck_errors(&format!("{preamble}\n{use_site}"));
        assert_nongeneric_type_args_diagnostic(&errors, kind, name, "annotation type args");
        assert_no_diagnostic_message(
            &errors,
            &format!("generic {kind} `{name}` expects 0"),
            "annotation",
        );
    }
}
