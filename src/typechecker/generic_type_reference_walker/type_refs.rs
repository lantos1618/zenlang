use super::*;

impl TypeChecker {
    pub(super) fn validate_generic_type_ref_bounds_with_unknowns(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        match ast_type {
            AstType::Named(name) => {
                self.validate_named_type_ref_bounds(name, scoped_type_params, span, reject_unknown);
            }
            AstType::Generic { name, type_args } => {
                self.validate_parameterized_type_ref_bounds(
                    name,
                    type_args,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => {
                self.validate_generic_type_ref_bounds_with_unknowns(
                    inner,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            AstType::Array { elem, .. } => {
                self.validate_generic_type_ref_bounds_with_unknowns(
                    elem,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            AstType::Function { params, ret } => {
                self.validate_generic_type_arg_refs_with_unknowns(
                    params,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
                self.validate_generic_type_ref_bounds_with_unknowns(
                    ret,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            _ => {}
        }
    }
}
