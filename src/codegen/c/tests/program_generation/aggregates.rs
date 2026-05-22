use super::*;

#[test]
fn generates_struct() {
    let backend = CBackend;
    let program = TypedProgram {
        functions: vec![],
        types: vec![TypedTypeDef {
            name: "Point".into(),
            kind: TypeDefKind::Struct {
                fields: vec![("x".into(), Type::F64), ("y".into(), Type::F64)],
            },
            methods: vec![],
            span: crate::error::Span::dummy(),
        }],
        globals: vec![],
        entry_point: None,
    };
    let output = backend.generate(&program).unwrap();
    assert!(output.contains("typedef struct Point Point;"));
    assert!(output.contains("double x;"));
    assert!(output.contains("double y;"));
}

#[test]
fn generates_enum() {
    let backend = CBackend;
    let program = TypedProgram {
        functions: vec![],
        types: vec![TypedTypeDef {
            name: "Color".into(),
            kind: TypeDefKind::Enum {
                variants: vec![
                    TypedVariant {
                        name: "Red".into(),
                        tag: 0,
                        payload: None,
                    },
                    TypedVariant {
                        name: "Green".into(),
                        tag: 1,
                        payload: None,
                    },
                    TypedVariant {
                        name: "Blue".into(),
                        tag: 2,
                        payload: None,
                    },
                ],
            },
            methods: vec![],
            span: crate::error::Span::dummy(),
        }],
        globals: vec![],
        entry_point: None,
    };
    let output = backend.generate(&program).unwrap();
    assert!(output.contains("Color_Red = 0"));
    assert!(output.contains("Color_Green = 1"));
    assert!(output.contains("Color_Blue = 2"));
    assert!(output.contains("enum Color_Tag tag;"));
}

#[test]
fn generates_enum_with_payload() {
    let backend = CBackend;
    let program = TypedProgram {
        functions: vec![],
        types: vec![TypedTypeDef {
            name: "Shape".into(),
            kind: TypeDefKind::Enum {
                variants: vec![
                    TypedVariant {
                        name: "Circle".into(),
                        tag: 0,
                        payload: Some(vec![("radius".into(), Type::I32)]),
                    },
                    TypedVariant {
                        name: "Square".into(),
                        tag: 1,
                        payload: Some(vec![("side".into(), Type::I32)]),
                    },
                ],
            },
            methods: vec![],
            span: dummy(),
        }],
        globals: vec![],
        entry_point: None,
    };
    let output = backend.generate(&program).unwrap();
    assert!(output.contains("Shape_Circle = 0"));
    assert!(output.contains("Shape_Square = 1"));
    assert!(output.contains("int32_t circle;"));
    assert!(output.contains("int32_t square;"));
}
