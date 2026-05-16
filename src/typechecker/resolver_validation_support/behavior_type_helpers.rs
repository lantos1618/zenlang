fn substitute_behavior_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, AstType>,
) -> AstType {
    match ast_type {
        AstType::Named(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ast_type.clone()),
        AstType::Ptr(inner) => {
            AstType::Ptr(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::MutPtr(inner) => {
            AstType::MutPtr(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::RawPtr(inner) => {
            AstType::RawPtr(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::Slice(inner) => {
            AstType::Slice(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(substitute_behavior_ast_type(elem, substitutions)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| substitute_behavior_ast_type(param, substitutions))
                .collect(),
            ret: Box::new(substitute_behavior_ast_type(ret, substitutions)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| substitute_behavior_ast_type(arg, substitutions))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}

fn substitute_behavior_bound_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, Type>,
) -> AstType {
    match ast_type {
        AstType::Named(name) => substitutions
            .get(name)
            .map(monomorphize::type_to_ast)
            .unwrap_or_else(|| ast_type.clone()),
        AstType::Ptr(inner) => AstType::Ptr(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::MutPtr(inner) => AstType::MutPtr(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::RawPtr(inner) => AstType::RawPtr(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::Slice(inner) => AstType::Slice(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(substitute_behavior_bound_ast_type(elem, substitutions)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| substitute_behavior_bound_ast_type(param, substitutions))
                .collect(),
            ret: Box::new(substitute_behavior_bound_ast_type(ret, substitutions)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: substitute_behavior_bound_type_args(type_args, substitutions),
        },
        _ => ast_type.clone(),
    }
}

fn substitute_behavior_bound_type_args(
    type_args: &[AstType],
    substitutions: &HashMap<String, Type>,
) -> Vec<AstType> {
    type_args
        .iter()
        .map(|arg| substitute_behavior_bound_ast_type(arg, substitutions))
        .collect()
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
    if type_args.is_empty() {
        behavior.to_string()
    } else {
        format!(
            "{}<{}>",
            behavior,
            type_args
                .iter()
                .map(AstType::display_name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
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
