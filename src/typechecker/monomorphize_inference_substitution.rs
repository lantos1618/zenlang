use std::collections::HashMap;

use super::ast_type_substitution::substitute_ast_type_names;
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
    substitute_ast_type_names(ast_type, &|name| substitutions.get(name).cloned())
}
