use super::*;

#[test]
fn parse_project_example() {
    let src = std::fs::read_to_string("examples/project/main.zen");
    if let Ok(src) = src {
        let result = parse_str(&src);
        match result {
            Ok(prog) => {
                assert!(
                    !prog.declarations.is_empty(),
                    "project example should have declarations, got {}",
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
fn parse_project_build_zen_example() {
    let src = std::fs::read_to_string("examples/project/build.zen");
    if let Ok(src) = src {
        let result = parse_str(&src);
        match result {
            Ok(prog) => {
                assert!(
                    !prog.declarations.is_empty(),
                    "project build.zen should have declarations, got {}",
                    prog.declarations.len()
                );
            }
            Err(errs) => {
                for e in &errs {
                    eprintln!("{:?}", e);
                }
                panic!("project build.zen parse failed with {} errors", errs.len());
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

    // Triple-nested: Map<StaticString, Vec<Ptr<f64>>>
    let prog = parse_ok("baz = (x: Map<StaticString, Vec<Ptr<f64>>>) void { }");
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
