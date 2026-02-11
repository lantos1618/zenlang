use super::*;
use crate::lexer;

fn parse_str(src: &str) -> Result<Program, Vec<CompileError>> {
    let tokens = lexer::tokenize(src, 0).map_err(|e| vec![e])?;
    parse(tokens, 0)
}

fn parse_ok(src: &str) -> Program {
    parse_str(src).unwrap_or_else(|errs| {
        for e in &errs {
            eprintln!("{:?}", e);
        }
        panic!("parse failed with {} errors", errs.len());
    })
}

#[test]
fn parse_simple_function() {
    let prog = parse_ok("add = (a: i32, b: i32) i32 {\n    return a + b\n}");
    assert_eq!(prog.declarations.len(), 1);
    match &prog.declarations[0] {
        Declaration::Function { name, params, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_struct_def() {
    let prog = parse_ok("Point: {\n    x: f64,\n    y: f64\n}");
    assert_eq!(prog.declarations.len(), 1);
    match &prog.declarations[0] {
        Declaration::Struct { name, fields, .. } => {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "x");
            assert_eq!(fields[1].name, "y");
        }
        other => panic!("expected Struct, got {:?}", other),
    }
}

#[test]
fn parse_enum_def() {
    let prog = parse_ok("Color:\n    Red,\n    Green,\n    Blue");
    assert_eq!(prog.declarations.len(), 1);
    match &prog.declarations[0] {
        Declaration::Enum { name, variants, .. } => {
            assert_eq!(name, "Color");
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name, "Red");
        }
        other => panic!("expected Enum, got {:?}", other),
    }
}

#[test]
fn parse_import() {
    let prog = parse_ok("{ io } = std");
    assert_eq!(prog.declarations.len(), 1);
    match &prog.declarations[0] {
        Declaration::Import {
            names, module_path, ..
        } => {
            assert_eq!(names, &["io"]);
            assert_eq!(module_path, &["std"]);
        }
        other => panic!("expected Import, got {:?}", other),
    }
}

#[test]
fn parse_import_multi() {
    let prog = parse_ok("{ Channel, Mutex } = std.sync");
    assert_eq!(prog.declarations.len(), 1);
    match &prog.declarations[0] {
        Declaration::Import {
            names, module_path, ..
        } => {
            assert_eq!(names, &["Channel", "Mutex"]);
            assert_eq!(module_path, &["std", "sync"]);
        }
        other => panic!("expected Import, got {:?}", other),
    }
}

#[test]
fn parse_method() {
    let prog =
        parse_ok("Point.distance = (self: Ptr<Point>, other: Ptr<Point>) f64 {\n    return 0.0\n}");
    assert_eq!(prog.declarations.len(), 1);
    match &prog.declarations[0] {
        Declaration::Method {
            type_name,
            method_name,
            ..
        } => {
            assert_eq!(type_name, "Point");
            assert_eq!(method_name, "distance");
        }
        other => panic!("expected Method, got {:?}", other),
    }
}

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
fn parse_ufc_chain() {
    let prog = parse_ok("f = () void {\n    result = 5.double().add_ten()\n}");
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_cast_expr() {
    let prog = parse_ok("f = (a: f64) i32 {\n    return cast(a, i32)\n}");
    assert_eq!(prog.declarations.len(), 1);
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

#[test]
fn parse_full_demo() {
    let src = std::fs::read_to_string("examples/demo_project/main.zen");
    if let Ok(src) = src {
        let result = parse_str(&src);
        match result {
            Ok(prog) => {
                assert!(
                    prog.declarations.len() >= 10,
                    "demo should have many declarations, got {}",
                    prog.declarations.len()
                );
            }
            Err(errs) => {
                for e in &errs {
                    eprintln!("{:?}", e);
                }
                panic!("demo parse failed with {} errors", errs.len());
            }
        }
    }
}
