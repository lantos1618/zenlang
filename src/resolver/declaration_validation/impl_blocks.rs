use crate::ast::{behavior_ref_display, named_type_arg_params, AstType, Declaration};
use crate::error::{CompilerDiagnosticCode::*, Diagnostic, Span};

use super::super::{BehaviorRefMetadata, Resolver, SymbolTable};

impl Resolver {
    pub(super) fn validate_impl_block_declaration(
        &self,
        table: &mut SymbolTable,
        decl: &Declaration,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Declaration::ImplBlock {
            type_name,
            type_args,
            behavior,
            behavior_type_args,
            methods,
            span,
            ..
        } = decl
        else {
            return;
        };
        let scoped_type_params = named_type_arg_params(type_args);

        if !self.is_known_type_name(table, &scoped_type_params, type_name) {
            self.push_unknown_type_symbol(diagnostics, type_name, *span);
        }
        if let Some(behavior) = behavior.as_deref() {
            self.validate_behavior_impl_declaration(
                table,
                type_name,
                behavior,
                behavior_type_args,
                *span,
                diagnostics,
            );
        }
        for type_arg in behavior_type_args {
            self.validate_type_ref(
                table,
                &scoped_type_params,
                type_arg,
                *span,
                false,
                diagnostics,
            );
        }
        for method in methods {
            self.validate_impl_block_method(table, method, diagnostics);
        }
    }

    fn validate_behavior_impl_declaration(
        &self,
        table: &mut SymbolTable,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let behavior_known = self.is_known_behavior_name(table, behavior);
        if !behavior_known {
            self.push_unknown_behavior_symbol(diagnostics, behavior, span);
        }
        if self.is_known_type_name(table, &[], type_name) && behavior_known {
            let behavior_display = behavior_ref_display(behavior, behavior_type_args);
            if !table.record_behavior_impl(
                type_name,
                BehaviorRefMetadata::new(behavior, behavior_type_args),
            ) {
                diagnostics.push(Diagnostic::error_code(
                    E0217,
                    format!(
                        "duplicate behavior implementation `{behavior_display}` for `{type_name}`"
                    ),
                    span,
                ));
            }
        }
    }

    fn validate_impl_block_method(
        &self,
        table: &mut SymbolTable,
        method: &Declaration,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(method) = method.as_callable() else {
            return;
        };
        self.validate_callable_declaration_types(table, method, true, diagnostics);
    }
}
