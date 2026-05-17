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
            // Body is a block containing a match expression
            if let Expression::Block {
                statements, expr, ..
            } = body
            {
                // The match should be the final expression or a statement
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
fn parse_string_interpolation() {
    let prog = parse_ok(r#"f = () void { io.println("hello ${name}!") }"#);
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_var_decl_mutable() {
    let prog = parse_ok("f = () void {\n    i ::= 0\n}");
    match &prog.declarations[0] {
        Declaration::Function { body, .. } => {
            if let Expression::Block { statements, .. } = body {
                assert_eq!(statements.len(), 1);
                match &statements[0] {
                    Statement::VarDecl { name, mutable, .. } => {
                        assert_eq!(name, "i");
                        assert!(*mutable);
                    }
                    other => panic!("expected VarDecl, got {:?}", other),
                }
            }
        }
        _ => panic!("expected Function"),
    }
}

#[test]
fn parse_struct_literal() {
    let prog = parse_ok(
        r#"f = () void {
    p = Person {
        name: "Alice",
        age: 30
    }
}"#,
    );
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_enum_variant_expr() {
    let prog = parse_ok("f = () void {\n    s = Status.Active\n}");
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_enum_variant_payload_expr() {
    let prog = parse_ok("f = () void {\n    s = Maybe.Some(42)\n}");
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_shorthand_enum_variant_expr_and_pattern() {
    let prog = parse_ok(
        r#"f = (value: Result<i32, str>) Result<i32, str> {
    value ?
        | .Ok(v) { .Ok(v) }
        | .Err(msg) { .Err(msg) }
}"#,
    );
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_ufc_chain() {
    let prog = parse_ok("f = () void {\n    result = 5.double().add_ten()\n}");
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_cast_expr() {
    let prog = parse_ok("f = (a: f64) i32 {\n    cast(a, i32)\n}");
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_range_expr() {
    let prog = parse_ok("f = () i32 {\n    1..=3\n}");
    let Declaration::Function { body, .. } = &prog.declarations[0] else {
        panic!("expected Function");
    };
    let Expression::Block {
        expr: Some(expr), ..
    } = body
    else {
        panic!("expected block final expression");
    };
    let Expression::Range { inclusive, .. } = expr.as_ref() else {
        panic!("expected range expression, got {expr:?}");
    };
    assert!(*inclusive);
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
