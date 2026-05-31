use crate::ast::{Param, Pattern, Statement};
use crate::error::{Diagnostic, Span};

use super::expression_validation::ExprRefContext;
use super::symbol_table::ScopeStack;
use super::{Namespace, Resolver, SymbolTable};

impl Resolver {
    pub(super) fn validate_statement_refs(
        &self,
        statement: &Statement,
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        match statement {
            Statement::VarDecl {
                name,
                ty,
                value,
                mutable,
                constant,
                ..
            } => {
                if let Some(ty) = ty {
                    self.validate_expr_type_ref(ty, statement.span(), ctx);
                }
                self.validate_expr_refs_in(value, ctx);
                if *constant || *mutable || !ctx.locals.is_mutable(name) {
                    self.define_local_symbol(
                        ctx.table,
                        name,
                        *mutable,
                        statement.span(),
                        ctx.locals,
                        ctx.diagnostics,
                    );
                }
            }
            Statement::Assignment { target, value, .. } => {
                self.validate_expr_refs_in(target, ctx);
                self.validate_expr_refs_in(value, ctx);
            }
            Statement::Expression { expr, .. } => {
                self.validate_expr_refs_in(expr, ctx);
            }
        }
    }

    pub(super) fn is_known_value_name(
        &self,
        table: &SymbolTable,
        locals: &ScopeStack,
        name: &str,
    ) -> bool {
        table.lookup(Namespace::Value, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
            || locals.contains(name)
    }

    pub(super) fn param_locals(
        &self,
        table: &mut SymbolTable,
        params: &[Param],
        diagnostics: &mut Vec<Diagnostic>,
    ) -> ScopeStack {
        let mut locals = ScopeStack::new(table.new_scope());
        for param in params {
            self.define_local_symbol(
                table,
                &param.name,
                param.mutable,
                param.span,
                &mut locals,
                diagnostics,
            );
        }
        locals
    }

    pub(super) fn define_local_symbol(
        &self,
        table: &mut SymbolTable,
        name: &str,
        mutable: bool,
        span: Span,
        locals: &mut ScopeStack,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match table.define_local(name, mutable, locals.current_scope_id, span) {
            Ok(_) => locals.insert(name.to_string(), mutable),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    pub(super) fn bind_pattern_locals(
        &self,
        table: &mut SymbolTable,
        pattern: &Pattern,
        locals: &mut ScopeStack,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match pattern {
            Pattern::Identifier { name, span } => {
                self.define_local_symbol(table, name, false, *span, locals, diagnostics);
            }
            Pattern::Struct { fields, .. } => {
                for (name, nested) in fields {
                    if let Some(nested) = nested {
                        self.bind_pattern_locals(table, nested, locals, diagnostics);
                    } else {
                        self.define_local_symbol(
                            table,
                            name,
                            false,
                            pattern.span(),
                            locals,
                            diagnostics,
                        );
                    }
                }
            }
            Pattern::Enum {
                payload: Some(payload),
                ..
            } => {
                self.bind_pattern_locals(table, payload, locals, diagnostics);
            }
            Pattern::Wildcard { .. }
            | Pattern::Literal { .. }
            | Pattern::Enum { payload: None, .. }
            | Pattern::BoolTrue { .. }
            | Pattern::BoolFalse { .. } => {}
        }
    }
}
