use super::*;

impl TypeChecker {
    pub(in crate::typechecker) fn impl_ast_types_compatible(
        &self,
        expected: &AstType,
        actual: &AstType,
        self_type_name: &str,
    ) -> bool {
        Self::impl_ast_types_compatible_for_target(expected, actual, self_type_name, &[])
    }

    pub(in crate::typechecker) fn impl_ast_types_compatible_for_target(
        expected: &AstType,
        actual: &AstType,
        self_type_name: &str,
        target_type_args: &[AstType],
    ) -> bool {
        let expected =
            concrete_self_ast_type_for_target(expected, self_type_name, target_type_args);
        let actual = concrete_self_ast_type_for_target(actual, self_type_name, target_type_args);
        expected == actual
    }

    pub(in crate::typechecker) fn impl_type_display(
        &self,
        ty: &AstType,
        self_type_name: &str,
    ) -> String {
        Self::impl_type_display_for_target(ty, self_type_name, &[])
    }

    pub(in crate::typechecker) fn impl_type_display_for_target(
        ty: &AstType,
        self_type_name: &str,
        target_type_args: &[AstType],
    ) -> String {
        concrete_self_ast_type_for_target(ty, self_type_name, target_type_args).display_name()
    }
}
