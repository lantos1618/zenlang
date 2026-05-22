use super::*;

#[test]
fn generates_function() {
    let backend = CBackend;
    let program = make_simple_program();
    let output = backend.generate(&program).unwrap();
    assert!(output.contains("int32_t add(int32_t a, int32_t b)"));
    assert!(output.contains("return (a + b)"));
}

#[test]
fn runtime_separates_static_and_allocator_backed_strings() {
    let backend = CBackend;
    let output = backend.generate(&make_simple_program()).unwrap();

    assert!(output.contains("typedef struct { const char* ptr; size_t len; } zen_str;"));
    assert!(output.contains("typedef struct zen_allocator zen_allocator;"));
    assert!(output.contains(
        "typedef struct { char* ptr; size_t len; size_t cap; zen_allocator* allocator; } zen_string;"
    ));
    assert!(output.contains("static zen_str zen_str_from_cstr(const char* s)"));
    assert!(!output.contains("zen_str_from_literal"));
    assert!(!output.contains("#define ZEN_STATIC_STR"));
}

#[test]
fn generates_entry_point() {
    let backend = CBackend;
    let program = TypedProgram {
        functions: vec![TypedFunction {
            name: "main".into(),
            params: vec![],
            return_type: Type::I32,
            body: TypedBlock {
                statements: vec![],
                expr: Some(Box::new(TypedExpression {
                    kind: TypedExprKind::IntLiteral(0),
                    ty: Type::I32,
                    span: crate::error::Span::dummy(),
                })),
                ty: Type::I32,
                span: crate::error::Span::dummy(),
            },
            defers: vec![],
            span: crate::error::Span::dummy(),
        }],
        types: vec![],
        globals: vec![],
        entry_point: Some("main".into()),
    };
    let output = backend.generate(&program).unwrap();
    assert!(output.contains("int main(int argc, char** argv)"));
    assert!(output.contains("return zen_main()"));
}

#[test]
fn generates_function_with_defers() {
    let backend = CBackend;
    let program = TypedProgram {
        functions: vec![TypedFunction {
            name: "process".into(),
            params: vec![],
            return_type: Type::I32,
            body: TypedBlock {
                statements: vec![],
                expr: Some(Box::new(texpr(TypedExprKind::IntLiteral(42), Type::I32))),
                ty: Type::I32,
                span: dummy(),
            },
            defers: vec![texpr(
                TypedExprKind::FunctionCall {
                    function: "cleanup".into(),
                    args: vec![],
                },
                Type::Void,
            )],
            span: dummy(),
        }],
        types: vec![],
        globals: vec![],
        entry_point: None,
    };
    let output = backend.generate(&program).unwrap();
    assert!(output.contains("cleanup()"));
    assert!(output.contains("__tmp"));
}
