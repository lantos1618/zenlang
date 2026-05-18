use super::*;

mod expressions;

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

    fn validate_generic_type_ref_bounds_with_unknowns(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        match ast_type {
            AstType::Named(name) => {
                if scoped_type_params.contains(name) {
                    return;
                }

                if !self.is_known_named_type(name) {
                    if reject_unknown {
                        self.diagnostics.push(Diagnostic::error(
                            "E0201",
                            format!("unknown type symbol '{name}'"),
                            span,
                        ));
                    }
                    return;
                }

                let generic = self
                    .structs
                    .get(name)
                    .map(|info| ("struct", info.type_params.len()))
                    .or_else(|| {
                        self.enums
                            .get(name)
                            .map(|info| ("enum", info.type_params.len()))
                    });
                if let Some((kind, type_param_count)) = generic {
                    if type_param_count > 0 {
                        self.diagnostics.push(Diagnostic::error(
                            "E5001",
                            format!(
                                "generic {} `{}` expects {} type arguments, found 0",
                                kind, name, type_param_count
                            ),
                            span,
                        ));
                    }
                }
            }
            AstType::Generic { name, type_args } => {
                self.validate_generic_type_arg_refs_with_unknowns(
                    type_args,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );

                if scoped_type_params.contains(name) {
                    return;
                }

                let (kind, type_params, type_param_bounds) =
                    if let Some(info) = self.structs.get(name) {
                        (
                            "struct",
                            info.type_params.clone(),
                            info.type_param_bounds.clone(),
                        )
                    } else if let Some(info) = self.enums.get(name) {
                        (
                            "enum",
                            info.type_params.clone(),
                            info.type_param_bounds.clone(),
                        )
                    } else {
                        if reject_unknown && !self.imports.contains_key(name) {
                            self.diagnostics.push(Diagnostic::error(
                                "E0201",
                                format!("unknown type symbol '{name}'"),
                                span,
                            ));
                        }
                        return;
                    };

                if type_params.is_empty() && !type_args.is_empty() {
                    self.diagnostics.push(Diagnostic::error(
                        "E5002",
                        format!(
                            "non-generic {} `{}` does not accept type arguments",
                            kind, name
                        ),
                        span,
                    ));
                    return;
                }

                if type_params.len() != type_args.len() {
                    self.diagnostics.push(Diagnostic::error(
                        "E5001",
                        format!(
                            "generic {} `{}` expects {} type arguments, found {}",
                            kind,
                            name,
                            type_params.len(),
                            type_args.len()
                        ),
                        span,
                    ));
                    return;
                }

                let substitutions: HashMap<String, Type> = type_params
                    .iter()
                    .zip(type_args.iter())
                    .filter_map(|(param, arg)| {
                        if ast_type_references_type_param(arg, scoped_type_params) {
                            None
                        } else {
                            Some((param.clone(), self.resolve_type(arg)))
                        }
                    })
                    .collect();
                self.check_generic_bounds(&type_param_bounds, &substitutions, span);
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

    fn is_known_named_type(&self, name: &str) -> bool {
        if name == "String" {
            return true;
        }
        self.structs.contains_key(name)
            || self.enums.contains_key(name)
            || self.imports.contains_key(name)
    }
}
