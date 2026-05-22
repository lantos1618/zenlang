use super::*;

#[test]
fn resolve_string_type() {
    let tc = TypeChecker::new();
    // "String" as a named type should resolve to Type::String
    assert_eq!(
        tc.resolve_type(&AstType::Named("String".into())),
        Type::String
    );
}

#[test]
fn resolve_slice_type() {
    let tc = TypeChecker::new();
    assert_eq!(
        tc.resolve_type(&AstType::Slice(Box::new(AstType::I32))),
        Type::Slice(Box::new(Type::I32))
    );
}

#[test]
fn infer_type_args_basic() {
    let tc = TypeChecker::new();
    // Generic function: identity<T>(x: T) -> T
    let type_params = vec!["T".to_string()];
    let params = vec![("x".to_string(), AstType::Named("T".into()))];
    let arg_types = vec![Type::I32];
    let subs = tc.infer_type_args(&type_params, &params, &arg_types);
    assert_eq!(subs.get("T"), Some(&Type::I32));
}
