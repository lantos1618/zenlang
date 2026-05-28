use super::assert_behavior_association_diagnostics_golden;

#[test]
fn emit_json_diagnostics_generic_requires_schemas_match_golden() {
    for (source, filename, description) in [
        (
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<StaticString>)

main = () i32 {
    0
}
"#,
            "generic_requires_missing_impl.zen",
            "missing generic behavior requires impl",
        ),
        (
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
Point.requires(Json<StaticString>)

main = () i32 {
    0
}
"#,
            "duplicate_generic_requires.zen",
            "duplicate generic behavior requires",
        ),
        (
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<i32, StaticString>)

main = () i32 {
    0
}
"#,
            "generic_requires_arity.zen",
            "generic behavior requires arity",
        ),
        (
            r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) StaticString
}

Point.requires(Json<i32>)

main = () i32 {
    0
}
"#,
            "nongeneric_requires_type_args.zen",
            "non-generic behavior requires type arguments",
        ),
    ] {
        assert_behavior_association_diagnostics_golden(source, filename, description);
    }
}
