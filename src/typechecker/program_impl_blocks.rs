use super::*;

pub(super) fn impl_self_ast_type(type_name: &str, type_args: &[AstType]) -> AstType {
    if type_args.is_empty() {
        AstType::Named(type_name.to_string())
    } else {
        AstType::Generic {
            name: type_name.to_string(),
            type_args: type_args.to_vec(),
        }
    }
}
