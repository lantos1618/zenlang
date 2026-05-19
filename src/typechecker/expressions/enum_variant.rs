use super::*;

impl TypeChecker {
    pub(super) fn check_enum_variant_expr(
        &mut self,
        enum_name: &str,
        type_args: &[AstType],
        variant: &str,
        payload: &Option<Box<Expression>>,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_payload = match payload {
            Some(p) => Some(Box::new(self.check_expr(p)?)),
            None => None,
        };
        let enum_info = self.enums.get(enum_name).cloned();
        let type_arg_count = enum_info.as_ref().map(|info| info.type_params.len());
        let type_args_valid = type_arg_count.is_none_or(|expected| expected == type_args.len());
        if !type_args.is_empty() && type_arg_count == Some(0) {
            self.diagnostics.push(Diagnostic::error(
                "E5002",
                format!(
                    "non-generic enum `{}` does not accept type arguments",
                    enum_name
                ),
                span,
            ));
        } else if let Some(expected) = type_arg_count.filter(|expected| {
            !type_args.is_empty() && *expected > 0 && *expected != type_args.len()
        }) {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic enum `{}` expects {} type arguments, found {}",
                    enum_name,
                    expected,
                    type_args.len()
                ),
                span,
            ));
        }

        let (type_name, ty, variant_defs) = if type_args.is_empty() {
            let ty = self.resolve_type(&AstType::Named(enum_name.to_string()));
            if let Some(expected) = type_arg_count.filter(|expected| *expected > 0) {
                self.diagnostics.push(Diagnostic::error(
                    "E5001",
                    format!(
                        "generic enum `{}` expects {} type arguments, found 0",
                        enum_name, expected
                    ),
                    span,
                ));
            }
            let variant_defs = enum_info
                .as_ref()
                .filter(|_| type_args_valid)
                .map(|info| {
                    info.variants
                        .iter()
                        .map(|(variant_name, payload)| {
                            (
                                variant_name.clone(),
                                payload.as_ref().map(|ty| self.resolve_type(ty)),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            (enum_name.to_string(), ty, variant_defs)
        } else {
            let type_name = self.mangle_generic_type_name(enum_name, type_args);
            let ty = if type_args_valid {
                self.resolve_type(&AstType::Generic {
                    name: enum_name.to_string(),
                    type_args: type_args.to_vec(),
                })
            } else {
                Type::Unknown
            };
            let variant_defs = if type_args_valid {
                self.specialize_generic_enum(enum_name, type_args, span)
            } else {
                std::collections::HashMap::new()
            };
            (type_name, ty, variant_defs)
        };
        if self.enums.contains_key(enum_name) && type_args_valid {
            match variant_defs.get(variant) {
                Some(expected_payload) => match (expected_payload, &typed_payload) {
                    (Some(expected_ast), Some(actual)) => {
                        let expected = expected_ast.clone();
                        if expected != Type::Unknown
                            && actual.ty != Type::Unknown
                            && !self.types_compatible(&expected, &actual.ty)
                        {
                            self.diagnostics.push(Diagnostic::error(
                                "E3062",
                                format!(
                                    "payload for enum variant `{}.{}` expects `{}`, found `{}`",
                                    enum_name,
                                    variant,
                                    expected.display_name(),
                                    actual.ty.display_name()
                                ),
                                actual.span,
                            ));
                        }
                    }
                    (Some(_), None) => {
                        self.diagnostics.push(Diagnostic::error(
                            "E3061",
                            format!(
                                "enum variant `{}.{}` requires a payload",
                                enum_name, variant
                            ),
                            span,
                        ));
                    }
                    (None, Some(actual)) => {
                        self.diagnostics.push(Diagnostic::error(
                            "E3063",
                            format!(
                                "enum variant `{}.{}` does not accept a payload",
                                enum_name, variant
                            ),
                            actual.span,
                        ));
                    }
                    (None, None) => {}
                },
                None => {
                    self.diagnostics.push(Diagnostic::error(
                        "E3060",
                        format!("enum `{}` has no variant `{}`", enum_name, variant),
                        span,
                    ));
                }
            }
        } else if !self.enums.contains_key(enum_name) {
            self.diagnostics.push(Diagnostic::error(
                "E3064",
                format!("undefined enum `{}`", enum_name),
                span,
            ));
        }
        Ok(TypedExpression {
            kind: TypedExprKind::EnumVariant {
                type_name,
                variant: variant.to_string(),
                payload: typed_payload,
            },
            ty,
            span,
        })
    }
}
