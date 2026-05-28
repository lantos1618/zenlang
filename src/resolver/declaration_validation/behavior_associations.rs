use crate::ast::{behavior_ref_display, type_params_from_names, AstType};
use crate::error::CompilerDiagnosticCode::*;
use crate::error::{
    Diagnostic, Span, GATED_GENERATED_BEHAVIOR_DERIVE_CONTEXT,
    GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE, GATED_GENERATED_BEHAVIOR_DERIVE_NOTE,
};

use super::super::{BehaviorRefMetadata, Namespace, Resolver, SymbolTable};

impl Resolver {
    pub(super) fn validate_requires_declaration(
        &self,
        table: &mut SymbolTable,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_known_type_name(table, &[], type_name) {
            self.push_unknown_type_symbol(diagnostics, type_name, span);
        }
        if !self.is_known_behavior_name(table, behavior) {
            self.push_unknown_behavior_symbol(diagnostics, behavior, span);
        } else if self.is_known_type_name(table, &[], type_name) {
            let behavior_display = behavior_ref_display(behavior, behavior_type_args);
            if !table.record_behavior_required(
                type_name,
                BehaviorRefMetadata::new(behavior, behavior_type_args),
            ) {
                diagnostics.push(Diagnostic::error_code(
                    E0216,
                    format!("duplicate required behavior `{behavior_display}` for `{type_name}`"),
                    span,
                ));
            }
        }
        for type_arg in behavior_type_args {
            self.validate_type_ref(table, &[], type_arg, span, false, diagnostics);
        }
    }

    pub(super) fn validate_derive_declaration(
        &self,
        table: &mut SymbolTable,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_known_type_name(table, &[], type_name) {
            self.push_unknown_type_symbol(diagnostics, type_name, span);
        }
        if !self.is_known_behavior_name(table, behavior) {
            self.push_unknown_behavior_symbol(diagnostics, behavior, span);
        }
        for type_arg in behavior_type_args {
            self.validate_type_ref(table, &[], type_arg, span, false, diagnostics);
        }
        diagnostics.push(
            Diagnostic::error_code(
                crate::error::CompilerDiagnosticCode::E2000,
                GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE,
                span,
            )
            .with_feature_gate_context(
                GATED_GENERATED_BEHAVIOR_DERIVE_NOTE,
                GATED_GENERATED_BEHAVIOR_DERIVE_CONTEXT,
            ),
        );
    }

    pub(super) fn validate_behavior_extends_declaration(
        &self,
        table: &mut SymbolTable,
        behavior: &str,
        parent: &str,
        parent_type_args: &[AstType],
        span: Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let behavior_known = self.is_known_behavior_name(table, behavior);
        let parent_known = self.is_known_behavior_name(table, parent);
        if !behavior_known {
            self.push_unknown_behavior_symbol(diagnostics, behavior, span);
        }
        if !parent_known {
            self.push_unknown_behavior_symbol(diagnostics, parent, span);
        }
        let behavior_type_params = table
            .lookup(Namespace::Behavior, behavior)
            .and_then(|symbol| symbol.type_parameter_names.as_ref())
            .map(|names| type_params_from_names(names.iter().cloned()))
            .unwrap_or_default();
        for type_arg in parent_type_args {
            self.validate_type_ref(
                table,
                &behavior_type_params,
                type_arg,
                span,
                false,
                diagnostics,
            );
        }
        if behavior_known && parent_known {
            let parent_display = behavior_ref_display(parent, parent_type_args);
            if !table.record_behavior_parent(
                behavior,
                BehaviorRefMetadata::new(parent, parent_type_args),
            ) {
                diagnostics.push(Diagnostic::error_code(
                    E0215,
                    format!("duplicate behavior parent `{parent_display}` for `{behavior}`"),
                    span,
                ));
            }
        }
    }
}
