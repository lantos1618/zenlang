use super::*;

#[test]
fn parse_simple_function() {
    match parse_single_decl("add = (a: i32, b: i32) i32 {\n    a + b\n}") {
        Declaration::Function { name, params, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn parse_generic_function_call_type_args() {
    match parse_single_decl("f = () i32 { identity<i32>(1) }") {
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
    match parse_single_decl("f = (box: Box<i32>) i32 { box.get<i32>() }") {
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
    match parse_single_decl("Point: {\n    x: f64,\n    y: f64\n}") {
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
    match parse_single_decl("Color:\n    Red,\n    Green,\n    Blue") {
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
    match parse_single_decl("{ io } = std") {
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
    match parse_single_decl("{ Channel, Mutex } = std.sync") {
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
    match parse_single_decl(
        "Point.distance = (self: Ptr<Point>, other: Ptr<Point>) f64 {\n    0.0\n}",
    ) {
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
