use super::*;

impl TypeChecker {
    pub(super) fn check_struct_literal_expr(
        &mut self,
        name: &str,
        type_args: &[AstType],
        fields: &[(String, Expression)],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let struct_info = self.structs.get(name).cloned();
        let type_arg_count = struct_info.as_ref().map(|info| info.type_params.len());
        let type_args_valid = type_arg_count.is_none_or(|expected| expected == type_args.len());
        if !type_args.is_empty() && type_arg_count == Some(0) {
            self.diagnostics.push(Diagnostic::error(
                "E5002",
                format!(
                    "non-generic struct `{}` does not accept type arguments",
                    name
                ),
                span,
            ));
        } else if let Some(expected) = type_arg_count.filter(|expected| {
            !type_args.is_empty() && *expected > 0 && *expected != type_args.len()
        }) {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic struct `{}` expects {} type arguments, found {}",
                    name,
                    expected,
                    type_args.len()
                ),
                span,
            ));
        }

        let (type_name, ty, field_defs) = if type_args.is_empty() {
            let ty = if let Some(expected) = type_arg_count.filter(|expected| *expected > 0) {
                self.diagnostics.push(Diagnostic::error(
                    "E5001",
                    format!(
                        "generic struct `{}` expects {} type arguments, found 0",
                        name, expected
                    ),
                    span,
                ));
                Type::Unknown
            } else {
                self.resolve_type(&AstType::Named(name.to_string()))
            };
            let field_defs = struct_info
                .as_ref()
                .filter(|_| type_args_valid)
                .map(|info| {
                    info.fields
                        .iter()
                        .map(|(field_name, field_type)| {
                            (field_name.clone(), self.resolve_type(field_type))
                        })
                        .collect()
                })
                .unwrap_or_default();
            (name.to_string(), ty, field_defs)
        } else {
            let type_name = self.mangle_generic_type_name(name, type_args);
            let ty = if type_args_valid {
                self.resolve_type(&AstType::Generic {
                    name: name.to_string(),
                    type_args: type_args.to_vec(),
                })
            } else {
                Type::Unknown
            };
            let field_defs = if type_args_valid {
                self.specialize_generic_struct(name, type_args, span)
            } else {
                std::collections::HashMap::new()
            };
            (type_name, ty, field_defs)
        };
        let mut typed_fields = Vec::new();
        let mut provided = std::collections::HashSet::new();
        let default_substitutions = if type_args.is_empty() || !type_args_valid {
            None
        } else {
            struct_info.as_ref().and_then(|info| {
                (info.type_params.len() == type_args.len()).then(|| {
                    info.type_params
                        .iter()
                        .zip(type_args.iter())
                        .map(|(param, arg)| (param.clone(), self.resolve_type(arg)))
                        .collect::<std::collections::HashMap<_, _>>()
                })
            })
        };

        for (field_name, field_expr) in fields {
            let typed = self.check_expr(field_expr)?;

            if !provided.insert(field_name.as_str()) {
                self.diagnostics.push(Diagnostic::error(
                    "E3034",
                    format!("duplicate field `{}` for struct `{}`", field_name, name),
                    typed.span,
                ));
            }

            if let Some(expected) = field_defs.get(field_name) {
                if *expected != Type::Unknown
                    && typed.ty != Type::Unknown
                    && !self.types_compatible(expected, &typed.ty)
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E3036",
                        format!(
                            "field `{}` for struct `{}` expects `{}`, found `{}`",
                            field_name,
                            name,
                            expected.display_name(),
                            typed.ty.display_name()
                        ),
                        typed.span,
                    ));
                }
            } else if struct_info.is_some() && type_args_valid {
                self.diagnostics.push(Diagnostic::error(
                    "E3035",
                    format!("unknown field `{}` for struct `{}`", field_name, name),
                    typed.span,
                ));
            }

            typed_fields.push((field_name.clone(), typed));
        }

        if let Some(info) = &struct_info.filter(|_| type_args_valid) {
            for (field_name, _) in &info.fields {
                if !provided.contains(field_name.as_str()) {
                    if let Some(default) = info.field_defaults.get(field_name) {
                        if let Some(substitutions) = &default_substitutions {
                            self.type_substitutions.push(substitutions.clone());
                        }
                        let typed = self.check_expr(default);
                        if default_substitutions.is_some() {
                            self.type_substitutions.pop();
                        }
                        let typed = typed?;
                        if let Some(expected) = field_defs.get(field_name) {
                            let actual_ty = if (expected.is_integer()
                                && matches!(typed.kind, TypedExprKind::IntLiteral(_)))
                                || (expected.is_float()
                                    && matches!(typed.kind, TypedExprKind::FloatLiteral(_)))
                            {
                                expected.clone()
                            } else {
                                typed.ty.clone()
                            };
                            if !self.types_compatible(expected, &actual_ty) {
                                self.diagnostics.push(Diagnostic::error(
                                    "E3036",
                                    format!(
                                        "field `{}` for struct `{}` expects `{}`, found `{}`",
                                        field_name,
                                        name,
                                        expected.display_name(),
                                        typed.ty.display_name()
                                    ),
                                    typed.span,
                                ));
                            }
                        }
                        typed_fields.push((field_name.clone(), typed));
                        continue;
                    }
                    self.diagnostics.push(Diagnostic::error(
                        "E3037",
                        format!("missing field `{}` for struct `{}`", field_name, name),
                        span,
                    ));
                }
            }
        }

        Ok(TypedExpression {
            kind: TypedExprKind::StructLiteral {
                type_name,
                fields: typed_fields,
            },
            ty,
            span,
        })
    }

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
