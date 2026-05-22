use super::*;

impl TypeChecker {
    pub(super) fn validate_self_type_statement(
        &mut self,
        statement: &ast::Statement,
        allow_self_type: bool,
    ) {
        match statement {
            ast::Statement::VarDecl {
                ty, value, span, ..
            } => {
                if let Some(ty) = ty {
                    self.validate_self_type_ref(ty, *span, allow_self_type);
                }
                self.validate_self_type_expr(value, allow_self_type);
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.validate_self_type_expr(target, allow_self_type);
                self.validate_self_type_expr(value, allow_self_type);
            }
            ast::Statement::Expression { expr, .. } => {
                self.validate_self_type_expr(expr, allow_self_type);
            }
            ast::Statement::Block { stmts, .. } => {
                for statement in stmts {
                    self.validate_self_type_statement(statement, allow_self_type);
                }
            }
        }
    }
}
