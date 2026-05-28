use super::*;

#[test]
fn box_call_site_annotation_arities_are_errors() {
    for (source, found, context, forbidden) in [
        (
            r#"
identity<T> = (value: T) T {
    value
}

main = () i32 {
    box = Box<i32> { value: 1 }
    bad = identity<Box<i32, StaticString>>(box)
    bad.value
}
"#,
            2,
            "generic function type-argument annotation",
            Some("argument 1"),
        ),
        (
            r#"
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
    bad.value
}
"#,
            2,
            "generic method type-argument annotation",
            Some("argument 2"),
        ),
        (
            r#"
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
    bad.value
}
"#,
            0,
            "generic method type-argument annotation without args",
            Some("argument 2"),
        ),
        (
            r#"
main = () i32 {
    f = (box: Box<i32, StaticString>) i32 {
        0
    }
    0
}
"#,
            2,
            "closure parameter generic annotation",
            None,
        ),
        (
            r#"
main = () i32 {
    f = () Box {
        Box<i32> { value: 1 }
    }
    0
}
"#,
            0,
            "closure return generic annotation",
            None,
        ),
        (
            r#"
main = () i32 {
    box = Box<i32> { value: 1 }
    bad = cast(box, Box<i32, StaticString>)
    bad.value
}
"#,
            2,
            "cast target generic annotation",
            None,
        ),
        (
            r#"
main = () i32 {
    box = Box<i32> { value: 1 }
    bad = cast(box, Box)
    0
}
"#,
            0,
            "cast target generic annotation",
            None,
        ),
    ] {
        let errors = typecheck_errors(&format!("{BOX}\n{source}"));
        assert_generic_arity_diagnostic(&errors, "struct", "Box", 1, found, context);
        if let Some(forbidden) = forbidden {
            assert_no_diagnostic_message(&errors, forbidden, context);
        }
    }
}
