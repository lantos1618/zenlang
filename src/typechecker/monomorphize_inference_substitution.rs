use std::collections::HashMap;

use crate::ast::AstType;

pub(super) fn ast_type_substitutions(
    params: &[String],
    args: &[AstType],
) -> HashMap<String, AstType> {
    params.iter().cloned().zip(args.iter().cloned()).collect()
}

pub(super) fn substitute_inference_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, AstType>,
) -> AstType {
    match ast_type {
        AstType::Named(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ast_type.clone()),
        AstType::Ptr(inner) => AstType::Ptr(Box::new(substitute_inference_ast_type(
            inner,
            substitutions,
        ))),
        AstType::MutPtr(inner) => AstType::MutPtr(Box::new(substitute_inference_ast_type(
            inner,
            substitutions,
        ))),
        AstType::RawPtr(inner) => AstType::RawPtr(Box::new(substitute_inference_ast_type(
            inner,
            substitutions,
        ))),
        AstType::Slice(inner) => AstType::Slice(Box::new(substitute_inference_ast_type(
            inner,
            substitutions,
        ))),
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(substitute_inference_ast_type(elem, substitutions)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| substitute_inference_ast_type(param, substitutions))
                .collect(),
            ret: Box::new(substitute_inference_ast_type(ret, substitutions)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| substitute_inference_ast_type(arg, substitutions))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}
