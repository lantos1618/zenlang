use super::*;

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
