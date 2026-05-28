use super::*;

mod type_refs;

impl TypeChecker {
    pub(in crate::typechecker) fn validate_generic_expr_type_references(
        &mut self,
        expr: &Expression,
        scoped_type_params: &HashSet<String>,
    ) {
        expr.walk_type_refs(&mut |ast_type, span| {
            self.validate_generic_type_ref_bounds(ast_type, scoped_type_params, span);
        });
    }

    pub(super) fn validate_generic_type_ref_bounds(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        ast_type.any(&mut |ty| {
            match ty {
                AstType::Named(name) => {
                    self.validate_type_ref_bounds(name, &[], scoped_type_params, span);
                }
                AstType::Generic { name, type_args } => {
                    self.validate_type_ref_bounds(name, type_args, scoped_type_params, span);
                }
                _ => {}
            }
            false
        });
    }
}
