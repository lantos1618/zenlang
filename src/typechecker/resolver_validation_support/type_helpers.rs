/// Scope for variable types.
#[derive(Debug, Clone)]
struct Scope {
    vars: HashMap<String, VarInfo>,
}

#[derive(Debug, Clone)]
pub(crate) struct VarInfo {
    pub ty: Type,
    pub mutable: bool,
}

impl Scope {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

fn method_signature_key(type_name: &str, method_name: &str) -> String {
    format!("{type_name}.{method_name}")
}

fn behavior_impl_method_signature_key(
    type_name: &str,
    method_name: &str,
    behavior: Option<&str>,
    behavior_type_args: &[AstType],
) -> String {
    let key = method_signature_key(type_name, method_name);
    let Some(behavior) = behavior else {
        return key;
    };
    if behavior_type_args.is_empty() {
        return key;
    }

    format!(
        "{}__{}",
        key,
        behavior_ref_symbol_suffix(behavior, behavior_type_args)
    )
}

fn behavior_ref_symbol_suffix(behavior: &str, type_args: &[AstType]) -> String {
    let mut parts = vec![symbol_key_part(behavior)];
    parts.extend(
        type_args
            .iter()
            .map(AstType::display_name)
            .map(|name| symbol_key_part(&name)),
    );
    parts.join("_")
}

fn symbol_key_part(name: &str) -> String {
    name.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn method_signature_key_parts(name: &str) -> Option<(&str, &str)> {
    name.split_once('.')
}

fn method_signature_receiver_name(name: &str) -> Option<&str> {
    method_signature_key_parts(name).map(|(receiver, _)| receiver)
}

fn method_signature_method_name_for_receiver<'a>(name: &'a str, receiver: &str) -> Option<&'a str> {
    method_signature_key_parts(name)
        .and_then(|(actual_receiver, method)| (actual_receiver == receiver).then_some(method))
}

fn is_method_signature_key(name: &str) -> bool {
    method_signature_key_parts(name).is_some()
}

fn type_param_bound_display(type_param: &ast::TypeParam) -> Option<String> {
    type_param.constraint.as_ref().map(|constraint| {
        if type_param.constraint_type_args.is_empty() {
            constraint.clone()
        } else {
            format!(
                "{}<{}>",
                constraint,
                type_param
                    .constraint_type_args
                    .iter()
                    .map(AstType::display_name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    })
}

fn type_param_name_set(type_params: &[ast::TypeParam]) -> HashSet<String> {
    type_param_names(type_params).into_iter().collect()
}

fn ast_type_references_type_param(
    ast_type: &AstType,
    scoped_type_params: &HashSet<String>,
) -> bool {
    match ast_type {
        AstType::Named(name) => scoped_type_params.contains(name),
        AstType::Generic { type_args, .. } => type_args
            .iter()
            .any(|arg| ast_type_references_type_param(arg, scoped_type_params)),
        AstType::Ptr(inner)
        | AstType::MutPtr(inner)
        | AstType::RawPtr(inner)
        | AstType::Slice(inner)
        | AstType::Array { elem: inner, .. } => {
            ast_type_references_type_param(inner, scoped_type_params)
        }
        AstType::Function { params, ret } => {
            params
                .iter()
                .any(|param| ast_type_references_type_param(param, scoped_type_params))
                || ast_type_references_type_param(ret, scoped_type_params)
        }
        _ => false,
    }
}

fn collect_ast_type_names(ast_type: &AstType, names: &mut HashSet<String>) {
    match ast_type {
        AstType::Named(name) => {
            names.insert(name.clone());
        }
        AstType::Generic { name, type_args } => {
            names.insert(name.clone());
            for type_arg in type_args {
                collect_ast_type_names(type_arg, names);
            }
        }
        AstType::Ptr(inner)
        | AstType::MutPtr(inner)
        | AstType::RawPtr(inner)
        | AstType::Slice(inner)
        | AstType::Array { elem: inner, .. } => collect_ast_type_names(inner, names),
        AstType::Function { params, ret } => {
            for param in params {
                collect_ast_type_names(param, names);
            }
            collect_ast_type_names(ret, names);
        }
        _ => {}
    }
}

fn concrete_self_ast_type_for_target(
    ast_type: &AstType,
    self_type_name: &str,
    self_type_args: &[AstType],
) -> AstType {
    match ast_type {
        AstType::SelfType => concrete_self_target_type(self_type_name, self_type_args),
        AstType::Ptr(inner) => {
            AstType::Ptr(Box::new(concrete_self_ast_type_for_target(
                inner,
                self_type_name,
                self_type_args,
            )))
        }
        AstType::MutPtr(inner) => AstType::MutPtr(Box::new(concrete_self_ast_type_for_target(
            inner,
            self_type_name,
            self_type_args,
        ))),
        AstType::RawPtr(inner) => AstType::RawPtr(Box::new(concrete_self_ast_type_for_target(
            inner,
            self_type_name,
            self_type_args,
        ))),
        AstType::Slice(inner) => AstType::Slice(Box::new(concrete_self_ast_type_for_target(
            inner,
            self_type_name,
            self_type_args,
        ))),
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(concrete_self_ast_type_for_target(
                elem,
                self_type_name,
                self_type_args,
            )),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| concrete_self_ast_type_for_target(param, self_type_name, self_type_args))
                .collect(),
            ret: Box::new(concrete_self_ast_type_for_target(
                ret,
                self_type_name,
                self_type_args,
            )),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| concrete_self_ast_type_for_target(arg, self_type_name, self_type_args))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}

fn concrete_self_target_type(self_type_name: &str, self_type_args: &[AstType]) -> AstType {
    if self_type_args.is_empty() {
        AstType::Named(self_type_name.to_string())
    } else {
        AstType::Generic {
            name: self_type_name.to_string(),
            type_args: self_type_args.to_vec(),
        }
    }
}
