use super::*;

#[test]
fn parse_simple_function() {
    let prog = parse_ok("add = (a: i32, b: i32) i32 {\n    a + b\n}");
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
fn parse_generic_function_call_type_args() {
    let prog = parse_ok("f = () i32 { identity<i32>(1) }");
    match &prog.declarations[0] {
        Declaration::Function { body, .. } => match body {
            Expression::Block {
                expr: Some(expr), ..
            } => match expr.as_ref() {
                Expression::FunctionCall {
                    name, type_args, ..
                } => {
                    assert_eq!(name, "identity");
                    assert_eq!(type_args, &vec![AstType::I32]);
                }
                other => panic!("expected generic function call, got {:?}", other),
            },
            other => panic!("expected block body, got {:?}", other),
        },
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_generic_method_call_type_args() {
    let prog = parse_ok("f = (box: Box<i32>) i32 { box.get<i32>() }");
    match &prog.declarations[0] {
        Declaration::Function { body, .. } => match body {
            Expression::Block {
                expr: Some(expr), ..
            } => match expr.as_ref() {
                Expression::MethodCall {
                    method, type_args, ..
                } => {
                    assert_eq!(method, "get");
                    assert_eq!(type_args, &vec![AstType::I32]);
                }
                other => panic!("expected generic method call, got {:?}", other),
            },
            other => panic!("expected block body, got {:?}", other),
        },
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
    let prog = parse_ok("Point.distance = (self: Ptr<Point>, other: Ptr<Point>) f64 {\n    0.0\n}");
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
