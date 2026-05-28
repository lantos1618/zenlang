use super::*;

impl TypeChecker {
    pub(super) fn check_member_access_expr(
        &mut self,
        object: &Expression,
        field: &str,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_obj = self.check_expr(object)?;
        if let ("value", Type::Ptr(inner) | Type::MutPtr(inner) | Type::RawPtr(inner)) =
            (field, &typed_obj.ty)
        {
            let inner_ty = *inner.clone();
            return typed_ok(TypedExprKind::Deref(Box::new(typed_obj)), inner_ty, span);
        }

        match field {
            "addr" => {
                let ptr_ty = Type::Ptr(Box::new(typed_obj.ty.clone()));
                typed_ok(TypedExprKind::Ref(Box::new(typed_obj)), ptr_ty, span)
            }
            "ref" => {
                let ptr_ty = Type::MutPtr(Box::new(typed_obj.ty.clone()));
                typed_ok(TypedExprKind::MutRef(Box::new(typed_obj)), ptr_ty, span)
            }
            _ => {
                let field_type = self.lookup_field_type(&typed_obj.ty, field);
                if field_type == Type::Unknown {
                    if let Some(type_name) = self.field_access_type_name(&typed_obj.ty) {
                        self.push_error(
                            E3052,
                            format!("type `{}` has no field `{}`", type_name, field),
                            span,
                        );
                    }
                }
                typed_ok(
                    TypedExprKind::FieldAccess {
                        object: Box::new(typed_obj),
                        field: field.to_string(),
                    },
                    field_type,
                    span,
                )
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
            self.push_error(E3055, "cannot infer element type for empty array", span);
        }
        typed_ok(
            TypedExprKind::ArrayLiteral {
                elements: typed_elems,
            },
            Type::Array {
                elem: Box::new(elem_type),
                size: Some(elements.len()),
            },
            span,
        )
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
                let ty = typed_obj.ty.display_name();
                self.push_error(
                    E3051,
                    format!("cannot index into non-array type `{ty}`"),
                    span,
                );
                Type::Unknown
            }
        };
        typed_ok(
            TypedExprKind::IndexAccess {
                object: Box::new(typed_obj),
                index: Box::new(typed_idx),
            },
            elem_ty,
            span,
        )
    }
}
