use super::*;

mod expressions;
mod statements;
mod type_refs;

impl TypeChecker {
    pub(super) fn validate_generic_type_ref_bounds(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        self.validate_generic_type_ref_bounds_with_unknowns(
            ast_type,
            scoped_type_params,
            span,
            true,
        );
    }

    pub(super) fn validate_generic_type_arg_refs_allow_unknowns(
        &mut self,
        type_args: &[AstType],
        span: Span,
    ) {
        let scoped_type_params = HashSet::new();
        self.validate_generic_type_arg_refs_with_unknowns(
            type_args,
            &scoped_type_params,
            span,
            false,
        );
    }

    fn validate_generic_type_arg_refs(
        &mut self,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        self.validate_generic_type_arg_refs_with_unknowns(
            type_args,
            scoped_type_params,
            span,
            true,
        );
    }

    fn validate_generic_type_arg_refs_with_unknowns(
        &mut self,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        for type_arg in type_args {
            self.validate_generic_type_ref_bounds_with_unknowns(
                type_arg,
                scoped_type_params,
                span,
                reject_unknown,
            );
        }
    }
}
