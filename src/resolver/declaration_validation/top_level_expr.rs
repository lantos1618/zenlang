use crate::ast::Expression;
use crate::error::Diagnostic;

use super::super::symbol_table::ScopeStack;
use super::super::{Resolver, SymbolTable};

impl Resolver {
    pub(super) fn validate_top_level_expr_declaration(
        &self,
        table: &mut SymbolTable,
        expr: &Expression,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let scope_id = table.new_scope();
        self.validate_expr_refs(
            table,
            &[],
            expr,
            &mut ScopeStack::new(scope_id),
            false,
            diagnostics,
        );
    }
}
