use super::*;

#[test]
fn c_type_primitives() {
    let e = CEmitter::new();
    assert_eq!(e.c_type(&Type::I8), "int8_t");
    assert_eq!(e.c_type(&Type::I16), "int16_t");
    assert_eq!(e.c_type(&Type::I32), "int32_t");
    assert_eq!(e.c_type(&Type::I64), "int64_t");
    assert_eq!(e.c_type(&Type::U8), "uint8_t");
    assert_eq!(e.c_type(&Type::U16), "uint16_t");
    assert_eq!(e.c_type(&Type::U32), "uint32_t");
    assert_eq!(e.c_type(&Type::U64), "uint64_t");
    assert_eq!(e.c_type(&Type::Usize), "size_t");
    assert_eq!(e.c_type(&Type::F32), "float");
    assert_eq!(e.c_type(&Type::F64), "double");
    assert_eq!(e.c_type(&Type::Bool), "bool");
    assert_eq!(e.c_type(&Type::Void), "void");
}

#[test]
fn c_type_strings() {
    let e = CEmitter::new();
    assert_eq!(e.c_type(&Type::Str), "zen_str");
    assert_eq!(e.c_type(&Type::String), "zen_string");
}

#[test]
fn c_type_pointers() {
    let e = CEmitter::new();
    assert_eq!(e.c_type(&Type::Ptr(Box::new(Type::I32))), "const int32_t*");
    assert_eq!(e.c_type(&Type::MutPtr(Box::new(Type::I32))), "int32_t*");
    assert_eq!(e.c_type(&Type::RawPtr(Box::new(Type::U8))), "uint8_t*");
    assert_eq!(e.c_type(&Type::Slice(Box::new(Type::F64))), "double*");
}

#[test]
fn c_type_named_and_struct() {
    let e = CEmitter::new();
    assert_eq!(e.c_type(&Type::Named("Widget".into())), "Widget");
    assert_eq!(
        e.c_type(&Type::Struct {
            name: "Point".into(),
            fields: vec![],
        }),
        "Point"
    );
    assert_eq!(
        e.c_type(&Type::Enum {
            name: "Color".into(),
            variants: vec![],
        }),
        "Color"
    );
}

#[test]
fn c_type_function_pointer() {
    let e = CEmitter::new();
    assert_eq!(
        e.c_type(&Type::Function {
            params: vec![Type::I32, Type::I32],
            ret: Box::new(Type::Bool),
        }),
        "bool(*)(int32_t, int32_t)"
    );
}
