use super::*;

#[test]
fn substitute_type_basic() {
    let tc = TypeChecker::new();
    let mut subs = HashMap::new();
    subs.insert("T".to_string(), Type::I32);
    // T -> I32
    assert_eq!(
        tc.substitute_type(&AstType::Named("T".into()), &subs),
        Type::I32
    );
    // Ptr<T> -> Ptr<I32>
    assert_eq!(
        tc.substitute_type(&AstType::Ptr(Box::new(AstType::Named("T".into()))), &subs),
        Type::Ptr(Box::new(Type::I32))
    );
    // Non-generic type unchanged
    assert_eq!(tc.substitute_type(&AstType::Bool, &subs), Type::Bool);
}

#[test]
fn substitute_type_covers_all_composite_type_shapes() {
    let tc = TypeChecker::new();
    let mut subs = HashMap::new();
    subs.insert("T".to_string(), Type::I32);

    assert_eq!(
        tc.substitute_type(
            &AstType::RawPtr(Box::new(AstType::Named("T".into()))),
            &subs
        ),
        Type::RawPtr(Box::new(Type::I32))
    );
    assert_eq!(
        tc.substitute_type(
            &AstType::MutPtr(Box::new(AstType::Named("T".into()))),
            &subs
        ),
        Type::MutPtr(Box::new(Type::I32))
    );
    assert_eq!(
        tc.substitute_type(&AstType::Slice(Box::new(AstType::Named("T".into()))), &subs),
        Type::Slice(Box::new(Type::I32))
    );
    assert_eq!(
        tc.substitute_type(
            &AstType::Array {
                elem: Box::new(AstType::Named("T".into())),
                size: Some(3),
            },
            &subs,
        ),
        Type::Array {
            elem: Box::new(Type::I32),
            size: Some(3),
        }
    );
    assert_eq!(
        tc.substitute_type(
            &AstType::Function {
                params: vec![AstType::Named("T".into())],
                ret: Box::new(AstType::Named("T".into())),
            },
            &subs,
        ),
        Type::Function {
            params: vec![Type::I32],
            ret: Box::new(Type::I32),
        }
    );
}

#[test]
fn substitute_type_preserves_function_type_arguments_in_nested_generics() {
    let mut tc = TypeChecker::new();
    tc.structs.insert(
        "Box".to_string(),
        StructInfo {
            name: "Box".to_string(),
            fields: vec![("value".to_string(), AstType::Named("T".to_string()))],
            field_defaults: HashMap::new(),
            type_params: vec!["T".to_string()],
            type_param_bounds: HashMap::new(),
        },
    );
    let function_type = Type::Function {
        params: vec![Type::I32],
        ret: Box::new(Type::I32),
    };
    let mut subs = HashMap::new();
    subs.insert("T".to_string(), function_type.clone());

    assert_eq!(
        tc.substitute_type(
            &AstType::Generic {
                name: "Box".to_string(),
                type_args: vec![AstType::Named("T".to_string())],
            },
            &subs,
        ),
        Type::Struct {
            name: "Box_fn_i32_ret_i32".to_string(),
            fields: vec![("value".to_string(), function_type)],
        }
    );
}
