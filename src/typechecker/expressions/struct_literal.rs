use super::*;

mod type_args;

impl TypeChecker {
    pub(super) fn check_struct_literal_expr(
        &mut self,
        name: &str,
        type_args: &[AstType],
        fields: &[(String, Expression)],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let struct_info = self.structs.get(name).cloned();
        let resolved =
            self.resolve_struct_literal_type_args(name, type_args, struct_info.as_ref(), span);
        let mut typed_fields = Vec::new();
        let mut provided = std::collections::HashSet::new();

        for (field_name, field_expr) in fields {
            let typed = self.check_expr(field_expr)?;

            if !provided.insert(field_name.as_str()) {
                self.diagnostics.push(Diagnostic::error(
                    "E3034",
                    format!("duplicate field `{}` for struct `{}`", field_name, name),
                    typed.span,
                ));
            }

            if let Some(expected) = resolved.field_defs.get(field_name) {
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
            } else if struct_info.is_some() && resolved.constructor_type_args_valid {
                self.diagnostics.push(Diagnostic::error(
                    "E3035",
                    format!("unknown field `{}` for struct `{}`", field_name, name),
                    typed.span,
                ));
            }

            typed_fields.push((field_name.clone(), typed));
        }

        if let Some(info) = struct_info
            .as_ref()
            .filter(|_| resolved.constructor_type_args_valid)
        {
            for (field_name, _) in &info.fields {
                if !provided.contains(field_name.as_str()) {
                    if let Some(default) = info.field_defaults.get(field_name) {
                        if let Some(substitutions) = &resolved.default_substitutions {
                            self.type_substitutions.push(substitutions.clone());
                        }
                        let typed = self.check_expr(default);
                        if resolved.default_substitutions.is_some() {
                            self.type_substitutions.pop();
                        }
                        let typed = typed?;
                        if let Some(expected) = resolved.field_defs.get(field_name) {
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
                type_name: resolved.type_name,
                fields: typed_fields,
            },
            ty: resolved.ty,
            span,
        })
    }
}
