use super::*;

#[test]
fn composite_generic_annotation_arities_are_errors() {
    for (preamble, use_site, kind, name, found, context) in [
        (
            BOX_OPTION,
            r#"
read = (box: Box<Option<i32, StaticString>>) i32 {
    0
}
"#,
            "enum",
            "Option",
            2,
            "nested annotation",
        ),
        (
            BOX_OPTION,
            r#"
main = () i32 {
    value = Box<Option<i32, StaticString>> { value: Option<i32>.Some(1) }
    0
}
"#,
            "enum",
            "Option",
            2,
            "nested instantiation",
        ),
        (
            BOX,
            r#"
call = (f: (Box<i32, StaticString>) i32) i32 {
    0
}
"#,
            "struct",
            "Box",
            2,
            "function parameter",
        ),
        (
            BOX,
            r#"
factory = () () Box {
    0
}
"#,
            "struct",
            "Box",
            0,
            "function return",
        ),
        (
            BOX,
            r#"
read = (ptr: Ptr<Box<i32, StaticString>>) i32 {
    0
}
"#,
            "struct",
            "Box",
            2,
            "pointer inner",
        ),
        (
            BOX,
            r#"
read = (slice: Slice<Box>) i32 {
    0
}
"#,
            "struct",
            "Box",
            0,
            "slice inner",
        ),
        (
            BOX,
            r#"
read = (items: [Box<i32, StaticString>; 1]) i32 {
    0
}
"#,
            "struct",
            "Box",
            2,
            "array inner",
        ),
    ] {
        let errors = typecheck_errors(&format!("{preamble}\n{use_site}"));
        assert_generic_arity_diagnostic(&errors, kind, name, 1, found, context);
    }
}

#[test]
fn empty_array_literal_is_an_error() {
    let errors = typecheck_errors(
        r#"
main = () i32 {
    values = []
    0
}
"#,
    );

    assert_diagnostic_code_and_message(
        &errors,
        "E3055",
        "cannot infer element type for empty array",
        "empty array literal",
    );
}
