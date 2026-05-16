use super::*;

impl TypeChecker {
    pub(super) fn check_member_access_expr(
        &mut self,
        object: &Expression,
        field: &str,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_obj = self.check_expr(object)?;
        if field == "value"
            && matches!(
                typed_obj.ty,
                Type::Ptr(_) | Type::MutPtr(_) | Type::RawPtr(_)
            )
        {
            let inner_ty = match &typed_obj.ty {
                Type::Ptr(inner) | Type::MutPtr(inner) | Type::RawPtr(inner) => *inner.clone(),
                _ => Type::Unknown,
            };
            return Ok(TypedExpression {
                kind: TypedExprKind::Deref(Box::new(typed_obj)),
                ty: inner_ty,
                span,
            });
        }

        match field {
            "addr" => {
                let ptr_ty = Type::Ptr(Box::new(typed_obj.ty.clone()));
                Ok(TypedExpression {
                    kind: TypedExprKind::Ref(Box::new(typed_obj)),
                    ty: ptr_ty,
                    span,
                })
            }
            "ref" => {
                let ptr_ty = Type::MutPtr(Box::new(typed_obj.ty.clone()));
                Ok(TypedExpression {
                    kind: TypedExprKind::MutRef(Box::new(typed_obj)),
                    ty: ptr_ty,
                    span,
                })
            }
            _ => {
                let field_type = self.lookup_field_type(&typed_obj.ty, field);
                if field_type == Type::Unknown {
                    if let Some(type_name) = self.field_access_type_name(&typed_obj.ty) {
                        self.diagnostics.push(Diagnostic::error(
                            "E3052",
                            format!("type `{}` has no field `{}`", type_name, field),
                            span,
                        ));
                    }
                }
                Ok(TypedExpression {
                    kind: TypedExprKind::FieldAccess {
                        object: Box::new(typed_obj),
                        field: field.to_string(),
                    },
                    ty: field_type,
                    span,
                })
            }
        }
    }

    pub(super) fn check_struct_literal_expr(
        &mut self,
        name: &str,
        type_args: &[AstType],
        fields: &[(String, Expression)],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let struct_info = self.structs.get(name).cloned();
        let (type_name, ty, field_defs) = if type_args.is_empty() {
            let ty = self.resolve_type(&AstType::Named(name.to_string()));
            let field_defs = struct_info
                .as_ref()
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
            let ty = self.resolve_type(&AstType::Generic {
                name: name.to_string(),
                type_args: type_args.to_vec(),
            });
            let field_defs = self.specialize_generic_struct(name, type_args, span);
            (type_name, ty, field_defs)
        };
        let mut typed_fields = Vec::new();
        let mut provided = std::collections::HashSet::new();
        let default_substitutions = if type_args.is_empty() {
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
            } else if struct_info.is_some() {
                self.diagnostics.push(Diagnostic::error(
                    "E3035",
                    format!("unknown field `{}` for struct `{}`", field_name, name),
                    typed.span,
                ));
            }

            typed_fields.push((field_name.clone(), typed));
        }

        if let Some(info) = &struct_info {
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
        let (type_name, ty, variant_defs) = if type_args.is_empty() {
            let ty = self.resolve_type(&AstType::Named(enum_name.to_string()));
            let variant_defs = self
                .enums
                .get(enum_name)
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
            let ty = self.resolve_type(&AstType::Generic {
                name: enum_name.to_string(),
                type_args: type_args.to_vec(),
            });
            let variant_defs = self.specialize_generic_enum(enum_name, type_args, span);
            (type_name, ty, variant_defs)
        };
        if self.enums.contains_key(enum_name) {
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
        } else {
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

    pub(super) fn check_array_literal_expr(
        &mut self,
        elements: &[Expression],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let mut typed_elems = Vec::new();
        let mut elem_type = Type::Unknown;
        for elem in elements {
            let typed = self.check_expr(elem)?;
            if elem_type == Type::Unknown {
                elem_type = typed.ty.clone();
            }
            typed_elems.push(typed);
        }
        if elements.is_empty() {
            self.diagnostics.push(Diagnostic::warning(
                "W3045",
                "cannot infer element type for empty array".to_string(),
                span,
            ));
        }
        Ok(TypedExpression {
            kind: TypedExprKind::ArrayLiteral {
                elements: typed_elems,
            },
            ty: Type::Array {
                elem: Box::new(elem_type),
                size: Some(elements.len()),
            },
            span,
        })
    }

    pub(super) fn check_index_access_expr(
        &mut self,
        object: &Expression,
        index: &Expression,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_obj = self.check_expr(object)?;
        let typed_idx = self.check_expr(index)?;
        let elem_ty = match &typed_obj.ty {
            Type::Array { elem, .. } | Type::Slice(elem) => *elem.clone(),
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E3051",
                    format!(
                        "cannot index into non-array type `{}`",
                        typed_obj.ty.display_name()
                    ),
                    span,
                ));
                Type::Unknown
            }
        };
        Ok(TypedExpression {
            kind: TypedExprKind::IndexAccess {
                object: Box::new(typed_obj),
                index: Box::new(typed_idx),
            },
            ty: elem_ty,
            span,
        })
    }
}
