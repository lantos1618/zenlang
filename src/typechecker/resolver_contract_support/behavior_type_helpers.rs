fn substitute_behavior_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, AstType>,
) -> AstType {
    ast_type_substitution::substitute_ast_type_names(ast_type, &|name| {
        substitutions.get(name).cloned()
    })
}

fn substitute_behavior_bound_type_args(
    type_args: &[AstType],
    substitutions: &HashMap<String, Type>,
) -> Vec<AstType> {
    ast_type_substitution::substitute_ast_type_args_names(type_args, &|name| {
        substitutions.get(name).map(monomorphize::type_to_ast)
    })
}

fn behavior_bound_display(bound: &BehaviorBound, substitutions: &HashMap<String, Type>) -> String {
    let type_args = substitute_behavior_bound_type_args(&bound.type_args, substitutions);
    if type_args.is_empty() {
        bound.behavior.clone()
    } else {
        format!(
            "{}<{}>",
            bound.behavior,
            type_args
                .iter()
                .map(AstType::display_name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn behavior_ref_display(behavior: &str, type_args: &[AstType]) -> String {
    crate::ast::behavior_ref_display(behavior, type_args)
}

fn behavior_method_signatures_match(
    left: &ast::BehaviorMethod,
    right: &ast::BehaviorMethod,
) -> bool {
    left.return_type == right.return_type
        && left.params.len() == right.params.len()
        && left
            .params
            .iter()
            .zip(&right.params)
            .all(|(left, right)| left.mutable == right.mutable && left.ty == right.ty)
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
