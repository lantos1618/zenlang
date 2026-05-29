#[derive(Debug, Clone)]
pub(crate) struct VarInfo {
    pub ty: Type,
    pub mutable: bool,
}

fn method_signature_receiver_name(name: &str) -> Option<&str> {
    name.split_once('.').map(|(receiver, _)| receiver)
}

fn type_param_name_set(type_params: &[ast::TypeParam]) -> HashSet<String> {
    type_params.iter().map(|param| param.name.clone()).collect()
}

fn ast_type_references_type_param(
    ast_type: &AstType,
    scoped_type_params: &HashSet<String>,
) -> bool {
    ast_type.any(&mut |ty| {
        matches!(ty, AstType::Named(name) if scoped_type_params.contains(name))
    })
}

fn collect_ast_type_names(ast_type: &AstType, names: &mut HashSet<String>) {
    ast_type.any(&mut |ty| {
        match ty {
            AstType::Named(name) | AstType::Generic { name, .. } => {
                names.insert(name.clone());
            }
            _ => {}
        }
        false
    });
}

fn concrete_self_ast_type_for_target(
    ast_type: &AstType,
    self_type_name: &str,
    self_type_args: &[AstType],
) -> AstType {
    let concrete_self = concrete_self_target_type(self_type_name, self_type_args);
    ast_type_substitution::substitute_ast_type(ast_type, &|ty| {
        matches!(ty, AstType::SelfType).then(|| concrete_self.clone())
    })
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
