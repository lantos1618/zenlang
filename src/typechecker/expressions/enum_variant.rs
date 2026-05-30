use super::*;
use crate::typechecker::literal_coerced_type;

impl TypeChecker {
    pub(super) fn check_enum_variant_expr(
        &mut self,
        enum_name: &str,
        type_args: &[AstType],
        variant: &str,
        payload: &Option<Box<Expression>>,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let mut typed_payload = match payload {
            Some(p) => Some(Box::new(self.check_expr(p)?)),
            None => None,
        };
        let filled_type_args = self.fill_type_arg_defaults(enum_name, type_args);
        let type_args = filled_type_args.as_slice();
        let enum_info = self.enums.get(enum_name).cloned();
        let type_arg_count = enum_info.as_ref().map(|info| info.type_params.len());
        let type_args_valid = type_arg_count.is_none_or(|expected| {
            self.validate_type_arg_arity("enum", enum_name, expected, type_args, span)
        });

        let (type_name, ty, variant_defs) = if type_args.is_empty() {
            let ty = self.resolve_type(&AstType::Named(enum_name.to_string()));
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
            let variant_defs = if type_args_valid {
                self.specialize_generic_enum(enum_name, type_args, span)
            } else {
                std::collections::HashMap::new()
            };
            let requested = self.mangle_generic_type_name(enum_name, type_args);
            let type_name = self.reserved_or_requested_generic_type_name(
                "enum",
                enum_info
                    .as_ref()
                    .and_then(|info| info.specialization_scope.as_deref()),
                requested,
            );
            let ty = if type_args_valid {
                self.resolve_type(&AstType::Generic {
                    name: enum_name.to_string(),
                    type_args: type_args.to_vec(),
                })
            } else {
                Type::Unknown
            };
            (type_name, ty, variant_defs)
        };
        // An untyped integer/float literal payload adopts the variant's declared
        // numeric type (`Option<i64>.Some(42)` where `42` parses as `i32`).
        if type_args_valid {
            if let (Some(Some(expected)), Some(actual)) =
                (variant_defs.get(variant), typed_payload.as_mut())
            {
                actual.ty = literal_coerced_type(expected, actual);
            }
        }
        if enum_info.is_none() {
            self.push_error(E3064, format!("undefined enum `{enum_name}`"), span);
        } else if type_args_valid {
            match variant_defs.get(variant) {
                Some(expected_payload) => match (expected_payload, &typed_payload) {
                    (Some(expected_ast), Some(actual)) => {
                        let expected = expected_ast.clone();
                        if !self.types_compatible(&expected, &actual.ty) {
                            let (expected, actual_display) =
                                type_display_pair(&expected, &actual.ty);
                            self.push_error(
                                E3062,
                                format!("payload for enum variant `{enum_name}.{variant}` expects `{expected}`, found `{actual_display}`"),
                                actual.span,
                            );
                        }
                    }
                    (Some(_), None) => self.push_error(
                        E3061,
                        format!("enum variant `{enum_name}.{variant}` requires a payload"),
                        span,
                    ),
                    (None, Some(actual)) => self.push_error(
                        E3063,
                        format!("enum variant `{enum_name}.{variant}` does not accept a payload"),
                        actual.span,
                    ),
                    (None, None) => {}
                },
                None => {
                    self.push_error(
                        E3060,
                        format!("enum `{enum_name}` has no variant `{variant}`"),
                        span,
                    );
                }
            }
        }
        typed_ok(
            TypedExprKind::EnumVariant {
                type_name,
                variant: variant.to_string(),
                payload: typed_payload,
            },
            ty,
            span,
        )
    }
}
