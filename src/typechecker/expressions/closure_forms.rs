use super::*;

impl TypeChecker {
    pub(super) fn check_closure_expr(
        &mut self,
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let outer_vars: std::collections::HashMap<String, Type> = self
            .scopes
            .iter()
            .flat_map(|s| s.vars.iter())
            .map(|(name, var)| (name.clone(), var.ty.clone()))
            .collect();

        self.push_scope();
        let mut param_types = Vec::new();
        let mut param_names = std::collections::HashSet::new();
        for param in params {
            let ty = self.resolve_type(&param.ty);
            self.define_var_with_mutability(&param.name, ty.clone(), param.mutable);
            param_types.push(ty);
            param_names.insert(param.name.clone());
        }
        let typed_body = self.check_expr(body)?;
        self.pop_scope();

        let ret_type = if let Some(return_type) = return_type {
            self.resolve_type(return_type)
        } else {
            typed_body.ty.clone()
        };

        let mut captures = Vec::new();
        let mut seen = std::collections::HashSet::new();
        collect_captures(
            &typed_body,
            &param_names,
            &outer_vars,
            &mut captures,
            &mut seen,
        );

        let fn_name = format!("__closure_{}_{}", span.start, span.end);
        let env_type = if captures.is_empty() {
            String::new()
        } else {
            format!("__env_{}_{}", span.start, span.end)
        };

        let fn_type = Type::Function {
            params: param_types,
            ret: Box::new(ret_type),
        };

        Ok(TypedExpression {
            kind: TypedExprKind::Closure {
                fn_name,
                env_type,
                captures,
            },
            ty: fn_type,
            span,
        })
    }
}
