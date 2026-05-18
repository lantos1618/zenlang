use super::*;

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
    let prog = parse_ok("foo = () void { }\nbar = () i32 { 1 }");
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
fn parse_return_keyword_is_removed() {
    let err = parse_str("main = () void {\n    return\n}")
        .expect_err("return keyword should no longer parse as language syntax");
    assert!(
        err.iter()
            .any(|err| err.to_string().contains("return keyword has been removed")),
        "expected removed return keyword diagnostic, got {err:?}"
    );
}

#[test]
fn parse_infix_as_cast_syntax_is_removed() {
    let err = parse_str("main = (x: i32) i64 {\n    x as i64\n}")
        .expect_err("infix as-cast syntax should no longer parse");
    assert!(
        err.iter().any(|err| err
            .to_string()
            .contains("`as` cast syntax has been removed")),
        "expected removed infix as-cast syntax diagnostic, got {err:?}"
    );
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
