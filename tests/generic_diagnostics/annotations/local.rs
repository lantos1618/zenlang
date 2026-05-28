use super::*;

#[test]
fn generic_local_annotation_type_arg_arities_are_errors() {
    for (preamble, use_site, kind, name, found, context) in [
        (
            BOX,
            r#"
main = () i32 {
    box: Box<i32, StaticString> = Box<i32> { value: 1 }
    box.value
}
"#,
            "struct",
            "Box",
            2,
            "local annotation arity",
        ),
        (
            BOX,
            r#"
main = () i32 {
    box: Box = Box<i32> { value: 1 }
    0
}
"#,
            "struct",
            "Box",
            0,
            "local annotation missing args",
        ),
        (
            OPTION,
            r#"
main = () i32 {
    value: Option<i32, StaticString> = Option<i32>.Some(1)
    0
}
"#,
            "enum",
            "Option",
            2,
            "local annotation arity",
        ),
        (
            OPTION,
            r#"
main = () i32 {
    value: Option = Option<i32>.Some(1)
    0
}
"#,
            "enum",
            "Option",
            0,
            "local annotation missing args",
        ),
    ] {
        let errors = typecheck_errors(&format!("{preamble}\n{use_site}"));
        assert_generic_arity_diagnostic(&errors, kind, name, 1, found, context);
    }
}
