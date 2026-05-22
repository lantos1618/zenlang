use super::*;

#[test]
fn callable_signature_insert_routes_function_and_method_keys() {
    let mut tc = TypeChecker::new();
    let function = FuncInfo {
        name: "make".to_string(),
        params: vec![],
        return_type: AstType::I32,
        type_params: vec![],
        type_param_bounds: HashMap::new(),
    };
    let method = FuncInfo {
        name: "Point.get".to_string(),
        params: vec![("self".to_string(), AstType::Named("Point".to_string()))],
        return_type: AstType::I32,
        type_params: vec![],
        type_param_bounds: HashMap::new(),
    };

    tc.insert_callable_signature("make", function);
    tc.insert_callable_signature("Point.get", method);

    assert!(tc.functions.contains_key("make"));
    assert!(!tc.methods.contains_key("make"));
    assert!(tc.methods.contains_key("Point.get"));
    assert!(!tc.functions.contains_key("Point.get"));
}
