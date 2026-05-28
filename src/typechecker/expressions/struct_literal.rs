use super::*;
use crate::typechecker::literal_coerced_type;

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
        let type_args_valid = type_arg_count.is_none_or(|expected| {
            self.validate_type_arg_arity("struct", name, expected, type_args, span)
        });
        let type_arg_annotations_valid = type_args
            .iter()
            .all(|type_arg| self.generic_type_annotation_arities_valid(type_arg));
        let constructor_type_args_valid = type_args_valid && type_arg_annotations_valid;

        let (type_name, ty, field_defs) = if type_args.is_empty() {
            let ty = if type_arg_count.is_some_and(|expected| expected > 0) {
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
            let field_defs = if constructor_type_args_valid {
                self.specialize_generic_struct(name, type_args, span)
            } else {
                std::collections::HashMap::new()
            };
            let requested = self.mangle_generic_type_name(name, type_args);
            let type_name = self.reserved_or_requested_generic_type_name(
                "struct",
                struct_info
                    .as_ref()
                    .and_then(|info| info.specialization_scope.as_deref()),
                requested,
            );
            let ty = if type_args_valid {
                self.resolve_type(&AstType::Generic {
                    name: name.to_string(),
                    type_args: type_args.to_vec(),
                })
            } else {
                Type::Unknown
            };
            (type_name, ty, field_defs)
        };
        let mut typed_fields = Vec::new();
        let mut provided = std::collections::HashSet::new();
        let default_substitutions = if type_args.is_empty() || !constructor_type_args_valid {
            None
        } else {
            struct_info
                .as_ref()
                .map(|info| self.type_arg_substitutions(&info.type_params, type_args))
        };

        for (field_name, field_expr) in fields {
            let typed = self.check_expr(field_expr)?;

            if !provided.insert(field_name.as_str()) {
                self.push_error(
                    E3034,
                    format!("duplicate field `{}` for struct `{}`", field_name, name),
                    typed.span,
                );
            }

            if let Some(expected) = field_defs.get(field_name) {
                if !self.types_compatible(expected, &typed.ty) {
                    self.push_struct_field_type_error(field_name, name, expected, &typed);
                }
            } else if struct_info.is_some() && constructor_type_args_valid {
                self.push_error(
                    E3035,
                    format!("unknown field `{}` for struct `{}`", field_name, name),
                    typed.span,
                );
            }

            typed_fields.push((field_name.clone(), typed));
        }

        if let Some(info) = &struct_info.filter(|_| constructor_type_args_valid) {
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
                            let actual_ty = literal_coerced_type(expected, &typed);
                            if !self.types_compatible(expected, &actual_ty) {
                                self.push_struct_field_type_error(
                                    field_name, name, expected, &typed,
                                );
                            }
                        }
                        typed_fields.push((field_name.clone(), typed));
                        continue;
                    }
                    self.push_error(
                        E3037,
                        format!("missing field `{}` for struct `{}`", field_name, name),
                        span,
                    );
                }
            }
        }

        typed_ok(
            TypedExprKind::StructLiteral {
                type_name,
                fields: typed_fields,
            },
            ty,
            span,
        )
    }

    fn push_struct_field_type_error(
        &mut self,
        field_name: &str,
        struct_name: &str,
        expected: &Type,
        actual: &TypedExpression,
    ) {
        let (expected, actual_display) = type_display_pair(expected, &actual.ty);
        self.push_error(
            E3036,
            format!("field `{field_name}` for struct `{struct_name}` expects `{expected}`, found `{actual_display}`"),
            actual.span,
        );
    }
}
