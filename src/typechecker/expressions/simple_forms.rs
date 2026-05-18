use super::*;
use std::str::FromStr;

#[derive(Clone, Copy)]
enum TypecheckerBoolLiteralKeyword {
    True,
    False,
}

impl TypecheckerBoolLiteralKeyword {
    const ALL: &[TypecheckerBoolLiteralKeyword] = &[Self::True, Self::False];
    const TRUE: &'static str = "true";
    const FALSE: &'static str = "false";

    const fn as_str(self) -> &'static str {
        match self {
            Self::True => Self::TRUE,
            Self::False => Self::FALSE,
        }
    }

    const fn value(self) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
        }
    }
}

impl FromStr for TypecheckerBoolLiteralKeyword {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|keyword| keyword.as_str() == value)
            .ok_or(())
    }
}

impl TypeChecker {
    pub(super) fn check_identifier_expr(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        if let Ok(keyword) = name.parse::<TypecheckerBoolLiteralKeyword>() {
            return Ok(TypedExpression {
                kind: TypedExprKind::BoolLiteral(keyword.value()),
                ty: Type::Bool,
                span,
            });
        }

        let ty = match self.lookup_var(name) {
            Some(t) => t,
            None => {
                if !self.structs.contains_key(name)
                    && !self.enums.contains_key(name)
                    && !self.functions.contains_key(name)
                    && !self.is_import(name)
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E3040",
                        format!("undefined variable `{}`", name),
                        span,
                    ));
                }
                Type::Unknown
            }
        };

        Ok(TypedExpression {
            kind: TypedExprKind::Variable(name.to_owned()),
            ty,
            span,
        })
    }

    pub(super) fn check_block_expr(
        &mut self,
        statements: &[crate::ast::Statement],
        expr: &Option<Box<Expression>>,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        self.push_scope();
        let mut typed_stmts = Vec::new();
        for stmt in statements {
            typed_stmts.push(self.check_statement(stmt)?);
        }
        let typed_expr = match expr {
            Some(e) => Some(Box::new(self.check_expr(e)?)),
            None => None,
        };
        let ty = typed_expr
            .as_ref()
            .map(|e| e.ty.clone())
            .unwrap_or(Type::Void);
        self.pop_scope();

        Ok(TypedExpression {
            kind: TypedExprKind::Block(TypedBlock {
                statements: typed_stmts,
                expr: typed_expr,
                ty: ty.clone(),
                span,
            }),
            ty,
            span,
        })
    }

    pub(super) fn check_cast_expr(
        &mut self,
        expr: &Expression,
        target_type: &AstType,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_expr = self.check_expr(expr)?;
        let from_type = typed_expr.ty.clone();
        let to_type = self.resolve_type(target_type);

        Ok(TypedExpression {
            kind: TypedExprKind::Cast {
                expr: Box::new(typed_expr),
                from_type,
                to_type: to_type.clone(),
            },
            ty: to_type,
            span,
        })
    }

    pub(super) fn check_string_interpolation_expr(
        &mut self,
        parts: &[StringPart],
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let mut typed_parts = Vec::new();
        for part in parts {
            match part {
                StringPart::Literal(s) => {
                    typed_parts.push(TypedStringPart::Literal(s.clone()));
                }
                StringPart::Expr(e) => {
                    let typed = self.check_expr(e)?;
                    typed_parts.push(TypedStringPart::Expr(typed));
                }
            }
        }

        Ok(TypedExpression {
            kind: TypedExprKind::StringInterpolation { parts: typed_parts },
            ty: Type::Str,
            span,
        })
    }

    pub(super) fn check_defer_expr(
        &mut self,
        expr: &Expression,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed_expr = self.check_expr(expr)?;
        self.pending_defers.push(typed_expr);

        Ok(TypedExpression {
            kind: TypedExprKind::Block(TypedBlock {
                statements: Vec::new(),
                expr: None,
                ty: Type::Void,
                span,
            }),
            ty: Type::Void,
            span,
        })
    }

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
            .map(|(k, v)| (k.clone(), v.ty.clone()))
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
