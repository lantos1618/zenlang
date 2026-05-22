use crate::ast::AstType;
use crate::typechecker::concrete_self_ast_type_for_target;

pub(super) fn generic_impl_ast_types_compatible(
    expected: &AstType,
    actual: &AstType,
    type_name: &str,
    target_type_args: &[AstType],
) -> bool {
    let expected = concrete_self_ast_type_for_target(expected, type_name, target_type_args);
    let actual = concrete_self_ast_type_for_target(actual, type_name, target_type_args);
    expected == actual
}

pub(super) fn generic_impl_type_display(
    ty: &AstType,
    type_name: &str,
    type_args: &[AstType],
) -> String {
    concrete_self_ast_type_for_target(ty, type_name, type_args).display_name()
}
