use super::*;

#[test]
fn types_compatible_basics() {
    let tc = TypeChecker::new();
    // Same types
    assert!(tc.types_compatible(&Type::I32, &Type::I32));
    // Numeric conversions require explicit casts except literal coercion.
    assert!(!tc.types_compatible(&Type::I64, &Type::I32));
    assert!(!tc.types_compatible(&Type::F32, &Type::F64));
    // Unknown is permissive
    assert!(tc.types_compatible(&Type::I32, &Type::Unknown));
    // Named types are nominal and do not match unrelated concrete types.
    assert!(tc.types_compatible(&Type::Named("UserId".into()), &Type::Named("UserId".into())));
    assert!(!tc.types_compatible(
        &Type::Named("UserId".into()),
        &Type::Named("OrderId".into())
    ));
    assert!(!tc.types_compatible(&Type::Str, &Type::Named("StaticString".into())));
    assert!(!tc.types_compatible(&Type::String, &Type::Str));
    assert!(!tc.types_compatible(&Type::Str, &Type::String));
    // Clear mismatch
    assert!(!tc.types_compatible(&Type::I32, &Type::Str));
    assert!(!tc.types_compatible(&Type::Bool, &Type::I32));
}

#[test]
fn static_string_literal_does_not_implicitly_allocate_string() {
    let program = parse_program(
        r#"
takes_string = (value: String) void { }

returns_string = () String {
    "literal"
}

main = () void {
    local: String = "literal"
    takes_string("literal")
}
"#,
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program(&program)
        .expect_err("static string literals should not implicitly satisfy dynamic String");

    for expected in [
        "return type mismatch: expected `String`, found `StaticString`",
        "variable `local` expects `String`, found `StaticString`",
        "argument 1 for `takes_string` expects `String`, found `StaticString`",
    ] {
        assert!(
            err.iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected diagnostic `{expected}`, got {err:?}"
        );
    }
}
