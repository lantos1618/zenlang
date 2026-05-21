use super::*;

mod type_args;

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
        let resolved =
            self.resolve_enum_variant_type_args(enum_name, type_args, enum_info.as_ref(), span);
        if self.enums.contains_key(enum_name) && resolved.type_args_valid {
            match resolved.variant_defs.get(variant) {
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
                type_name: resolved.type_name,
                variant: variant.to_string(),
                payload: typed_payload,
            },
            ty: resolved.ty,
            span,
        })
    }
}
