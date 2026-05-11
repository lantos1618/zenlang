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

#[test]
fn parse_nested_generics() {
    // Single-level generic
    let prog = parse_ok("foo = (x: Vec<i32>) void { }");
    assert_eq!(prog.declarations.len(), 1);

    // Nested: Vec<Ptr<i32>> — the >> must not be parsed as ShiftRight
    let prog = parse_ok("bar = (x: Vec<Ptr<i32>>) void { }");
    assert_eq!(prog.declarations.len(), 1);

    // Triple-nested: Map<str, Vec<Ptr<f64>>>
    let prog = parse_ok("baz = (x: Map<str, Vec<Ptr<f64>>>) void { }");
    assert_eq!(prog.declarations.len(), 1);

    // Deeply nested: A<B<C<D<i32>>>>
    let prog = parse_ok("deep = (x: A<B<C<D<i32>>>>) void { }");
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_slice_type() {
    let prog = parse_ok("foo = (s: Slice<i32>) void { }");
    assert_eq!(prog.declarations.len(), 1);
    match &prog.declarations[0] {
        Declaration::Function { params, .. } => {
            assert!(
                matches!(&params[0].ty, AstType::Slice(inner) if **inner == AstType::I32),
                "expected Slice<I32>, got {:?}",
                params[0].ty
            );
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_string_type_is_named() {
    // String should parse as AstType::Named("String"), NOT AstType::Str
    let prog = parse_ok("foo = (s: String) void { }");
    match &prog.declarations[0] {
        Declaration::Function { params, .. } => match &params[0].ty {
            AstType::Named(n) => assert_eq!(n, "String"),
            AstType::Str => panic!("String should not parse as Str"),
            other => panic!("expected Named(\"String\"), got {:?}", other),
        },
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_static_string_is_str() {
    // StaticString should still parse as AstType::Str
    let prog = parse_ok("foo = (s: StaticString) void { }");
    match &prog.declarations[0] {
        Declaration::Function { params, .. } => {
            assert!(
                matches!(&params[0].ty, AstType::Str),
                "expected Str, got {:?}",
                params[0].ty
            );
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

// ── Error / negative parser tests ────────────────────────

#[test]
fn parse_error_missing_closing_brace() {
    let result = parse_str("foo = () void {");
    assert!(
        result.is_err(),
        "expected parse error for missing closing brace"
    );
}

#[test]
fn parse_error_unexpected_token() {
    let result = parse_str("foo = () i32 + +");
    assert!(
        result.is_err(),
        "expected parse error for unexpected tokens"
    );
}

#[test]
fn parse_rejects_gated_behavior_declaration_with_clear_error() {
    let result = parse_str("Serializable: behavior { to_json: (Self) String }");
    let errs = result.expect_err("expected gated behavior declaration to be rejected");
    assert!(
        errs.iter()
            .any(|e| e.to_string().contains("gated v1 feature 'behavior'")),
        "expected clear gated behavior diagnostic, got {errs:?}"
    );
}

#[test]
fn parse_rejects_gated_type_association_with_clear_error() {
    let result = parse_str("Point.implements(Json) { }");
    let errs = result.expect_err("expected gated type association syntax to be rejected");
    assert!(
        errs.iter()
            .any(|e| e.to_string().contains("gated v1 feature 'implements'")),
        "expected clear gated implements diagnostic, got {errs:?}"
    );
}

// ── Additional feature tests ─────────────────────────────

#[test]
fn parse_pointer_types() {
    let prog = parse_ok("foo = (p: Ptr<i32>, q: MutPtr<u8>) void { }");
    match &prog.declarations[0] {
        Declaration::Function { params, .. } => {
            assert!(
                matches!(&params[0].ty, AstType::Ptr(inner) if **inner == AstType::I32),
                "expected Ptr<I32>, got {:?}",
                params[0].ty
            );
            assert!(
                matches!(&params[1].ty, AstType::MutPtr(inner) if **inner == AstType::U8),
                "expected MutPtr<U8>, got {:?}",
                params[1].ty
            );
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_multiple_functions() {
    let prog = parse_ok("foo = () void { }\nbar = () i32 { return 1 }");
    assert_eq!(prog.declarations.len(), 2);
}

#[test]
fn parse_mutable_var() {
    let prog = parse_ok("main = () void {\n    x ::= 5\n}");
    match &prog.declarations[0] {
        Declaration::Function { body, .. } => {
            if let Expression::Block { statements, .. } = body {
                match &statements[0] {
                    Statement::VarDecl { name, mutable, .. } => {
                        assert_eq!(name, "x");
                        assert!(*mutable, "expected mutable variable");
                    }
                    other => panic!("expected VarDecl, got {:?}", other),
                }
            } else {
                panic!("expected Block body");
            }
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_immutable_var() {
    let prog = parse_ok("main = () void {\n    x = 5\n}");
    match &prog.declarations[0] {
        Declaration::Function { body, .. } => {
            if let Expression::Block { statements, .. } = body {
                match &statements[0] {
                    Statement::VarDecl { name, mutable, .. } => {
                        assert_eq!(name, "x");
                        assert!(!*mutable, "expected immutable variable");
                    }
                    other => panic!("expected VarDecl, got {:?}", other),
                }
            } else {
                panic!("expected Block body");
            }
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_enum_with_payload() {
    let prog = parse_ok("Shape:\n    Circle(i32),\n    Square(i32),\n    Empty\n");
    match &prog.declarations[0] {
        Declaration::Enum { name, variants, .. } => {
            assert_eq!(name, "Shape");
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name, "Circle");
            assert!(variants[0].payload.is_some());
            assert_eq!(variants[2].name, "Empty");
            assert!(variants[2].payload.is_none());
        }
        other => panic!("expected Enum, got {:?}", other),
    }
}

#[test]
fn parse_struct_with_many_fields() {
    let prog = parse_ok("Rect: {\n    x: i32,\n    y: i32,\n    w: u32,\n    h: u32,\n}\n");
    match &prog.declarations[0] {
        Declaration::Struct { name, fields, .. } => {
            assert_eq!(name, "Rect");
            assert_eq!(fields.len(), 4);
            assert_eq!(fields[0].name, "x");
            assert_eq!(fields[3].name, "h");
        }
        other => panic!("expected Struct, got {:?}", other),
    }
}

#[test]
fn parse_boolean_expressions() {
    let prog = parse_ok("main = () void {\n    x = true\n    y = false\n}");
    match &prog.declarations[0] {
        Declaration::Function { body, .. } => {
            if let Expression::Block { statements, .. } = body {
                assert_eq!(statements.len(), 2);
            }
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_return_without_value() {
    let prog = parse_ok("main = () void {\n    return\n}");
    assert_eq!(prog.declarations.len(), 1);
}

#[test]
fn parse_nested_generic_type() {
    // DynVec<Ptr<i32>> with nested generics
    let prog = parse_ok("foo = (v: DynVec<Ptr<i32>>) void { }");
    match &prog.declarations[0] {
        Declaration::Function { params, .. } => match &params[0].ty {
            AstType::Generic { name, type_args } => {
                assert_eq!(name, "DynVec");
                assert_eq!(type_args.len(), 1);
                assert!(matches!(&type_args[0], AstType::Ptr(inner) if **inner == AstType::I32));
            }
            other => panic!("expected Generic, got {:?}", other),
        },
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_multi_import() {
    let prog = parse_ok("{ foo, bar, baz } = mymod\n");
    match &prog.declarations[0] {
        Declaration::Import {
            names, module_path, ..
        } => {
            assert_eq!(names.len(), 3);
            assert_eq!(module_path, &vec!["mymod".to_string()]);
        }
        other => panic!("expected Import, got {:?}", other),
    }
}
