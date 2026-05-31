use crate::ast::AstType;

pub(in crate::typechecker) fn substitute_ast_type<F>(ast_type: &AstType, substitute: &F) -> AstType
where
    F: Fn(&AstType) -> Option<AstType> + ?Sized,
{
    if let Some(replacement) = substitute(ast_type) {
        return replacement;
    }
    let substitute_inner = |inner| Box::new(substitute_ast_type(inner, substitute));
    match ast_type {
        AstType::Ptr(inner) => AstType::Ptr(substitute_inner(inner)),
        AstType::MutPtr(inner) => AstType::MutPtr(substitute_inner(inner)),
        AstType::RawPtr(inner) => AstType::RawPtr(substitute_inner(inner)),
        AstType::Future(inner) => AstType::Future(substitute_inner(inner)),
        AstType::Slice(inner) => AstType::Slice(substitute_inner(inner)),
        AstType::Array { elem, size } => AstType::Array {
            elem: substitute_inner(elem),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| substitute_ast_type(param, substitute))
                .collect(),
            ret: substitute_inner(ret),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| substitute_ast_type(arg, substitute))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}

pub(in crate::typechecker) fn substitute_ast_type_names<F>(
    ast_type: &AstType,
    substitute_name: &F,
) -> AstType
where
    F: Fn(&str) -> Option<AstType> + ?Sized,
{
    substitute_ast_type(ast_type, &|ast_type| match ast_type {
        AstType::Named(name) => substitute_name(name),
        _ => None,
    })
}
