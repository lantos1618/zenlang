use super::*;

#[test]
fn parse_pattern_match() {
    let prog = parse_ok(
        r#"f = (x: bool) i32 {
    x ?
        | true { 1 }
        | false { 0 }
}"#,
    );
    assert_eq!(prog.declarations.len(), 1);
    match &prog.declarations[0] {
        Declaration::Function { name, body, .. } => {
            assert_eq!(name, "f");
            if let Expression::Block {
                statements, expr, ..
            } = body
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
    let prog = parse_ok(
        r#"f = () void {
    loop(() {
        break
    })
}"#,
    );
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_loop_control_param_expr() {
    let prog = parse_ok(
        r#"f = (done: bool) void {
    loop((l) {
        done ?
            | true { l.done() }
            | false { l.next() }
    })
}"#,
    );
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_while_loop() {
    let prog = parse_ok(
        r#"f = () void {
    x ::= 0
    x < 10 ? {
        x = x + 1
    }
}"#,
    );
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_nested_conditionals() {
    let prog = parse_ok(
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
    assert_eq!(prog.declarations.len(), 1);
}
