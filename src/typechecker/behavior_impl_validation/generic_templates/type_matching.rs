use crate::ast::AstType;
use crate::typechecker::TypeChecker;

pub(super) fn generic_impl_ast_types_compatible(
    expected: &AstType,
    actual: &AstType,
    type_name: &str,
    target_type_args: &[AstType],
) -> bool {
    TypeChecker::impl_ast_types_compatible_for_target(expected, actual, type_name, target_type_args)
}

pub(super) fn generic_impl_type_display(
    ty: &AstType,
    type_name: &str,
    type_args: &[AstType],
) -> String {
    TypeChecker::impl_type_display_for_target(ty, type_name, type_args)
}
