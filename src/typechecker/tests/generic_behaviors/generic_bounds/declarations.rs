use super::*;

#[test]
fn behavior_generic_bound_accepts_later_behavior_declaration() {
    let program = parse_program(
        r#"
Serializable<T: Json>: behavior {
    encode: (Self) StaticString
}

Json: behavior {
    to_json: (Self) StaticString
}

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    tc.check_program(&program)
        .expect("behavior generic bounds should be independent of declaration order");
}

#[test]
fn behavior_generic_bound_unknown_behavior_reports_once() {
    let program = parse_program(
        r#"
Serializable<T: Missing>: behavior {
    encode: (Self) StaticString
}

main = () i32 {
    0
}
"#,
    );

    let mut tc = TypeChecker::new();
    let errors = tc
        .check_program(&program)
        .expect_err("unknown behavior generic bound should fail");
    let count = errors
        .iter()
        .filter(|d| {
            d.message.contains(
                "generic bound `Missing` on type parameter `T` references undefined behavior",
            )
        })
        .count();
    assert_eq!(
        count, 1,
        "expected one behavior generic bound diagnostic, got {errors:?}"
    );
}
