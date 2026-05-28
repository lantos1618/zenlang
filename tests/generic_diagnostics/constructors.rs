use super::*;

#[test]
fn generic_constructor_type_arg_arities_are_errors() {
    for (preamble, use_site, kind, name, found, context, forbidden) in [
        (
            BOX,
            r#"
main = () i32 {
    box = Box<i32, StaticString> { value: 1 }
    box.value
}
"#,
            "struct",
            "Box",
            2,
            "generic struct arity",
            "field `value` for struct `Box`",
        ),
        (
            BOX,
            r#"
main = () i32 {
    box = Box { value: 1 }
    box.value
}
"#,
            "struct",
            "Box",
            0,
            "generic struct missing args",
            "field `value` for struct `Box`",
        ),
        (
            OPTION,
            r#"
main = () i32 {
    value = Option<i32, StaticString>.Some(1)
    0
}
"#,
            "enum",
            "Option",
            2,
            "generic enum arity",
            "payload for enum variant",
        ),
        (
            OPTION,
            r#"
main = () i32 {
    value = Option.Some(1)
    0
}
"#,
            "enum",
            "Option",
            0,
            "generic enum missing args",
            "payload for enum variant",
        ),
    ] {
        let errors = typecheck_errors(&format!("{preamble}\n{use_site}"));
        assert_generic_arity_diagnostic(&errors, kind, name, 1, found, context);
        assert_no_diagnostic_message(&errors, forbidden, "constructor arity");
    }
}

#[test]
fn nongeneric_constructor_type_args_are_errors() {
    for (preamble, use_site, kind, name, forbidden) in [
        (
            POINT,
            r#"
main = () i32 {
    point = Point<i32> { x: 1 }
    point.x
}
"#,
            "struct",
            "Point",
            "field `x` for struct `Point`",
        ),
        (
            STATUS,
            r#"
main = () i32 {
    value = Status<i32>.Done(1)
    0
}
"#,
            "enum",
            "Status",
            "payload for enum variant",
        ),
    ] {
        let errors = typecheck_errors(&format!("{preamble}\n{use_site}"));
        assert_nongeneric_type_args_diagnostic(&errors, kind, name, "constructor type args");
        assert_no_diagnostic_message(&errors, forbidden, "constructor arity");
    }
}
