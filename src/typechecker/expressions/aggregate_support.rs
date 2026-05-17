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
