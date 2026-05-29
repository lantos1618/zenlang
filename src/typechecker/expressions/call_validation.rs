use super::*;
use crate::typechecker::method_signature_key;

impl TypeChecker {
    pub(super) fn check_call_signature(
        &mut self,
        kind: &str,
        callee: &str,
        params: &[(String, AstType)],
        args: &[TypedExpression],
        span: &Span,
    ) {
        let expected_types = params
            .iter()
            .map(|(_, expected)| self.resolve_type(expected))
            .collect::<Vec<_>>();
        self.check_call_signature_types(kind, callee, &expected_types, args, span);
    }

    pub(super) fn check_call_signature_types(
        &mut self,
        kind: &str,
        callee: &str,
        expected_types: &[Type],
        args: &[TypedExpression],
        span: &Span,
    ) {
        if expected_types.len() != args.len() {
            let expected_count = expected_types.len();
            let actual_count = args.len();
            self.push_error(
                E3021,
                format!(
                    "{kind} `{callee}` expects {expected_count} arguments, found {actual_count}"
                ),
                *span,
            );
            return;
        }

        for (idx, (expected, actual)) in expected_types.iter().zip(args.iter()).enumerate() {
            // An untyped integer/float literal adopts the parameter's numeric
            // type (`char.is_digit(53)` satisfies a `u8` parameter).
            let actual_ty = crate::typechecker::literal_coerced_type(expected, actual);
            if !self.types_compatible(expected, &actual_ty) {
                let position = idx + 1;
                let (expected, actual_display) = type_display_pair(expected, &actual_ty);
                self.push_error(
                    E3022,
                    format!("argument {position} for `{callee}` expects `{expected}`, found `{actual_display}`"),
                    actual.span,
                );
            }
        }
    }

    pub(super) fn field_access_type_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Struct { name, .. } => Some(name.clone()),
            Type::Named(name) if self.structs.contains_key(name) => Some(name.clone()),
            Type::Ptr(inner) | Type::MutPtr(inner) => self.field_access_type_name(inner),
            _ => None,
        }
    }

    pub(super) fn behavior_specialized_method_key(
        &self,
        type_name: &str,
        method: &str,
    ) -> Option<String> {
        let prefix = format!("{}__", method_signature_key(type_name, method));
        let candidates: Vec<_> = self
            .methods
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .collect();

        if candidates.len() == 1 {
            return Some(candidates[0].0.clone());
        }

        let expected = self.current_return_type.as_ref()?;
        let matching: Vec<_> = candidates
            .into_iter()
            .filter(|(_, info)| {
                self.types_compatible(expected, &self.resolve_type(&info.return_type))
            })
            .collect();

        (matching.len() == 1).then(|| matching[0].0.clone())
    }

    pub(super) fn block_definitely_returns(&self, block: &TypedBlock) -> bool {
        block
            .expr
            .as_ref()
            .is_some_and(|expr| self.expr_definitely_returns(expr))
            || block.statements.iter().any(|stmt| match &stmt.kind {
                TypedStatementKind::Expression(expr) => self.expr_definitely_returns(expr),
                TypedStatementKind::VarDecl { .. } => false,
            })
    }

    fn expr_definitely_returns(&self, expr: &TypedExpression) -> bool {
        match &expr.kind {
            TypedExprKind::Block(block) => self.block_definitely_returns(block),
            TypedExprKind::Match { arms, .. } => {
                !arms.is_empty()
                    && arms
                        .iter()
                        .all(|arm| self.block_definitely_returns(&arm.body))
            }
            _ => false,
        }
    }
}
