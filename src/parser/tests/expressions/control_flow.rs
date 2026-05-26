use super::*;

#[test]
fn parse_pattern_match() {
    match parse_single_decl(
        r#"f = (x: bool) i32 {
    x ?
        | true { 1 }
        | false { 0 }
}"#,
    ) {
        Declaration::Function { name, body, .. } => {
            assert_eq!(name, "f");
            if let Expression::Block {
                statements, expr, ..
            } = &body
            {
                assert!(
                    !statements.is_empty() || expr.is_some(),
                    "block should have content"
                );
            }
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_loop_expr() {
    parse_single_decl(
        r#"f = () void {
    loop(() {
        break
    })
}"#,
    );
}

#[test]
fn parse_loop_control_param_expr() {
    parse_single_decl(
        r#"f = (done: bool) void {
    loop((l) {
        done ?
            | true { l.done() }
            | false { l.next() }
    })
}"#,
    );
}

#[test]
fn parse_while_loop() {
    parse_single_decl(
        r#"f = () void {
    x ::= 0
    x < 10 ? {
        x = x + 1
    }
}"#,
    );
}

#[test]
fn parse_nested_conditionals() {
    parse_single_decl(
        r#"f = (hour: i32) void {
    hour < 12 ?
        | true { greeting = "morning" }
        | false {
            hour < 18 ?
                | true { greeting = "afternoon" }
                | false { greeting = "evening" }
        }
}"#,
    );
}
