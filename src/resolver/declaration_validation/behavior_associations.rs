use crate::ast::AstType;
use crate::error::{Diagnostic, Span, GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE};

use super::super::metadata_helpers::behavior_ref_display;
use super::super::{BehaviorRefMetadata, Resolver, SymbolTable};

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
            diagnostics.push(Diagnostic::error(
                "E0201",
                format!("unknown type symbol '{type_name}'"),
                span,
            ));
        }
        if !self.is_known_behavior_name(table, behavior) {
            diagnostics.push(Diagnostic::error(
                "E0202",
                format!("unknown behavior symbol '{behavior}'"),
                span,
            ));
        } else if self.is_known_type_name(table, &[], type_name) {
            let behavior_display = behavior_ref_display(behavior, behavior_type_args);
            if !table.record_behavior_required(
                type_name,
                BehaviorRefMetadata {
                    name: behavior.to_string(),
                    type_args: behavior_type_args.to_vec(),
                },
            ) {
                diagnostics.push(Diagnostic::error(
                    "E0216",
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
            diagnostics.push(Diagnostic::error(
                "E0201",
                format!("unknown type symbol '{type_name}'"),
                span,
            ));
        }
        if !self.is_known_behavior_name(table, behavior) {
            diagnostics.push(Diagnostic::error(
                "E0202",
                format!("unknown behavior symbol '{behavior}'"),
                span,
            ));
        }
        for type_arg in behavior_type_args {
            self.validate_type_ref(table, &[], type_arg, span, false, diagnostics);
        }
        diagnostics.push(
            Diagnostic::error("E2000", GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE, span)
                .with_generated_behavior_derive_gate_context(),
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
            diagnostics.push(Diagnostic::error(
                "E0202",
                format!("unknown behavior symbol '{behavior}'"),
                span,
            ));
        }
        if !parent_known {
            diagnostics.push(Diagnostic::error(
                "E0202",
                format!("unknown behavior symbol '{parent}'"),
                span,
            ));
        }
        let behavior_type_params = self.behavior_type_params_for_ref(table, behavior);
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
                BehaviorRefMetadata {
                    name: parent.to_string(),
                    type_args: parent_type_args.to_vec(),
                },
            ) {
                diagnostics.push(Diagnostic::error(
                    "E0215",
                    format!("duplicate behavior parent `{parent_display}` for `{behavior}`"),
                    span,
                ));
            }
        }
    }
}
