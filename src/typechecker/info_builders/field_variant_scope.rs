fn callable_signature_from_declaration(
    decl: &Declaration,
) -> Option<(&str, FuncInfo, Option<GenericFunctionTemplate>)> {
    let callable = decl.as_callable()?;

    let type_params = type_param_names(callable.type_params);
    let info = FuncInfo {
        params: callable
            .params
            .iter()
            .map(|param| (param.name.clone(), param.ty.clone()))
            .collect(),
        return_type: callable.return_type.clone().unwrap_or(AstType::Void),
        type_params: type_params.clone(),
        type_param_bounds: type_param_bounds(callable.type_params),
        is_async: callable.is_async,
    };
    let template = (!type_params.is_empty()).then(|| GenericFunctionTemplate {
        type_params,
        params: callable.params.to_vec(),
        return_type: callable.return_type.clone(),
        body: callable.body.clone(),
        span: callable.span,
        dependencies: SourceModuleDependencies::default(),
    });

    Some((callable.name, info, template))
}

fn insert_callable_signature(
    key: String,
    decl: &Declaration,
    callables: &mut HashMap<String, FuncInfo>,
    generic_callables: &mut HashMap<String, GenericFunctionTemplate>,
) {
    insert_callable_signature_scoped(&key, decl, callables, generic_callables, None);
}

fn insert_callable_signature_scoped(
    key: &str,
    decl: &Declaration,
    callables: &mut HashMap<String, FuncInfo>,
    generic_callables: &mut HashMap<String, GenericFunctionTemplate>,
    specialization_scope: Option<&str>,
) {
    let Some((_, info, template)) = callable_signature_from_declaration(decl) else {
        return;
    };
    callables.insert(key.to_string(), info);
    if let Some(mut template) = template {
        if let Some(scope) = specialization_scope {
            template.dependencies.specialization_scope = Some(scope.to_string());
        }
        generic_callables.insert(key.to_string(), template);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BehaviorParentRef {
    behavior: String,
    type_args: Vec<AstType>,
    key: String,
}
