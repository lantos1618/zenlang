use super::assert_diagnostics_golden;

#[test]
fn emit_json_diagnostics_constructor_arity_schemas_match_golden() {
    for (zen_filename, source, failure_context, count_context) in [
        (
            "generic_function_arity.zen",
            r#"
identity<T> = (value: T) T {
    value
}

main = () i32 {
    identity<i32, StaticString>(1)
}
"#,
            "generic function arity",
            "generic function arity diagnostics should not emit inference or argument followups",
        ),
        (
            "generic_struct_constructor_arity.zen",
            r#"
Box<T>: {
    value: T
}

main = () i32 {
    boxed = Box<i32, StaticString> { value: 1 }
    boxed.value
}
"#,
            "generic struct constructor arity",
            "generic struct constructor arity diagnostics should not emit field mismatch followups",
        ),
        (
            "generic_struct_constructor_missing_args.zen",
            r#"
Box<T>: {
    value: T
}

main = () i32 {
    boxed = Box { value: 1 }
    0
}
"#,
            "generic struct constructor missing args",
            "generic struct constructor missing-args diagnostics should not emit field mismatch followups",
        ),
        (
            "nongeneric_struct_constructor_type_args.zen",
            r#"
Point: {
    x: i32
}

main = () i32 {
    point = Point<i32> { x: 1 }
    point.x
}
"#,
            "non-generic struct constructor type arguments",
            "non-generic struct constructor type-argument diagnostics should not emit field mismatch followups",
        ),
        (
            "generic_enum_constructor_arity.zen",
            r#"
Option<T>:
    Some(T),
    None

main = () i32 {
    value = Option<i32, StaticString>.Some(1)
    0
}
"#,
            "generic enum constructor arity",
            "generic enum constructor arity diagnostics should not emit payload mismatch followups",
        ),
        (
            "generic_enum_constructor_missing_args.zen",
            r#"
Option<T>:
    Some(T),
    None

main = () i32 {
    value = Option.Some(1)
    0
}
"#,
            "generic enum constructor missing args",
            "generic enum constructor missing-args diagnostics should not emit payload mismatch followups",
        ),
        (
            "nongeneric_enum_constructor_type_args.zen",
            r#"
Direction:
    North,
    South

main = () i32 {
    value = Direction<i32>.North
    0
}
"#,
            "non-generic enum constructor type arguments",
            "non-generic enum constructor type-argument diagnostics should not emit payload mismatch followups",
        ),
        (
            "nested_generic_instantiation_inner_arity.zen",
            r#"
Box<T>: {
    value: T
}

Option<T>:
    None,
    Some(T)

main = () i32 {
    value = Box<Option<i32, StaticString>> { value: Option<i32>.Some(1) }
    0
}
"#,
            "nested generic instantiation inner arity",
            "nested generic instantiation inner arity diagnostics should be stable",
        ),
    ] {
        assert_diagnostics_golden(zen_filename, source, failure_context, count_context);
    }
}
