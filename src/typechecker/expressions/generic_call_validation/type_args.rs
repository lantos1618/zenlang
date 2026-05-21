use super::*;

impl TypeChecker {
    pub(in crate::typechecker) fn explicit_type_arg_substitutions(
        &mut self,
        kind: &str,
        callee: &str,
        type_params: &[String],
        type_args: &[AstType],
        span: Span,
    ) -> (std::collections::HashMap<String, Type>, bool) {
        let arity_valid = Self::explicit_type_args_valid(type_args, type_params);
        let diagnostic_count = self.diagnostics.len();
        let substitutions =
            self.type_param_substitutions(type_params, type_args, kind, callee, span);
        let resolved_without_errors = self.diagnostics.len() == diagnostic_count;
        let annotations_valid = type_args
            .iter()
            .all(|type_arg| self.generic_type_annotation_arities_valid(type_arg));
        (
            substitutions,
            arity_valid && annotations_valid && resolved_without_errors,
        )
    }

    pub(super) fn explicit_type_args_valid(type_args: &[AstType], type_params: &[String]) -> bool {
        type_args.is_empty() || type_args.len() == type_params.len()
    }

    pub(crate) fn generic_type_annotation_arities_valid(&self, ast_type: &AstType) -> bool {
        match ast_type {
            AstType::Named(name) => self
                .structs
                .get(name)
                .map(|info| info.type_params.is_empty())
                .or_else(|| self.enums.get(name).map(|info| info.type_params.is_empty()))
                .unwrap_or(true),
            AstType::Generic { name, type_args } => {
                let own_arity_valid = self
                    .structs
                    .get(name)
                    .map(|info| info.type_params.len())
                    .or_else(|| self.enums.get(name).map(|info| info.type_params.len()))
                    .is_none_or(|expected| expected == type_args.len());
                own_arity_valid
                    && type_args
                        .iter()
                        .all(|type_arg| self.generic_type_annotation_arities_valid(type_arg))
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => self.generic_type_annotation_arities_valid(inner),
            AstType::Array { elem, .. } => self.generic_type_annotation_arities_valid(elem),
            AstType::Function { params, ret } => {
                params
                    .iter()
                    .all(|param| self.generic_type_annotation_arities_valid(param))
                    && self.generic_type_annotation_arities_valid(ret)
            }
            _ => true,
        }
    }
}
