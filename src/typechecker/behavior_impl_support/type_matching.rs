use super::*;

impl TypeChecker {
    pub(in crate::typechecker) fn impl_ast_types_compatible(
        &self,
        expected: &AstType,
        actual: &AstType,
        self_type_name: &str,
    ) -> bool {
        let expected = concrete_self_ast_type_for_target(expected, self_type_name, &[]);
        let actual = concrete_self_ast_type_for_target(actual, self_type_name, &[]);
        expected == actual
    }

    pub(in crate::typechecker) fn impl_type_display(
        &self,
        ty: &AstType,
        self_type_name: &str,
    ) -> String {
        concrete_self_ast_type_for_target(ty, self_type_name, &[]).display_name()
    }
}
