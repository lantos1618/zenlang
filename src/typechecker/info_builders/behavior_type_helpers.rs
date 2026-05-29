fn substitute_behavior_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, AstType>,
) -> AstType {
    ast_type_substitution::substitute_ast_type_names(ast_type, &|name| {
        substitutions.get(name).cloned()
    })
}

fn substituted_behavior_method_signature(
    method: &ast::BehaviorMethod,
    substitutions: &HashMap<String, AstType>,
) -> ast::BehaviorMethod {
    let mut method = method.clone();
    for param in &mut method.params {
        param.ty = substitute_behavior_ast_type(&param.ty, substitutions);
    }
    if let Some(return_type) = &mut method.return_type {
        *return_type = substitute_behavior_ast_type(return_type, substitutions);
    }
    method
}
