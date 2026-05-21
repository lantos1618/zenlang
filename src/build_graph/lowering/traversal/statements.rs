use crate::ast::Statement;

use super::{BuildProgramLowering, BuildTargetAddContext};

impl BuildProgramLowering {
    pub(super) fn collect_statement(
        &mut self,
        statement: &Statement,
        add_context: BuildTargetAddContext,
    ) {
        match statement {
            Statement::Expression { expr: value, .. } => {
                self.collect_expr(value, add_context);
            }
            Statement::VarDecl { value, .. } => {
                self.collect_expr(value, BuildTargetAddContext::DynamicExpression);
            }
            Statement::Assignment { target, value, .. } => {
                self.collect_expr(target, BuildTargetAddContext::DynamicExpression);
                self.collect_expr(value, BuildTargetAddContext::DynamicExpression);
            }
            Statement::Block { stmts, .. } => {
                for stmt in stmts {
                    self.collect_statement(stmt, add_context);
                }
            }
        }
    }
}
