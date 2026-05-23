use super::*;

#[test]
fn parse_string_interpolation() {
    parse_single_decl(r#"f = () void { io.println("hello ${name}!") }"#);
}

#[test]
fn parse_var_decl_mutable() {
    match parse_single_decl("f = () void {\n    i ::= 0\n}") {
        Declaration::Function { body, .. } => {
            if let Expression::Block { statements, .. } = &body {
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
    parse_single_decl(
        r#"f = () void {
    p = Person {
        name: "Alice",
        age: 30
    }
}"#,
    );
}

#[test]
fn parse_ufc_chain() {
    parse_single_decl("f = () void {\n    result = 5.double().add_ten()\n}");
}

#[test]
fn parse_cast_expr() {
    parse_single_decl("f = (a: f64) i32 {\n    cast(a, i32)\n}");
}

#[test]
fn parse_range_expr() {
    let Declaration::Function { body, .. } = parse_single_decl("f = () i32 {\n    1..=3\n}") else {
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
