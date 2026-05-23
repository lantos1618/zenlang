use super::*;

#[test]
fn match_arm_return_does_not_force_never_result_type() {
    let program = parse_program(
        r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "early" }
        | false { "late" }
}
"#,
    );

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_program(&program)
        .expect("returning arm should not force match type to never");
    let body = &typed.functions[0].body;
    assert_eq!(body.ty, Type::Str);
}
