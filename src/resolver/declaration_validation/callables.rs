use crate::ast::{AstType, Declaration, Expression, Param, TypeParam};
use crate::error::{Diagnostic, Span};

use super::super::metadata_helpers::behavior_ref_display;
use super::super::{BehaviorRefMetadata, Resolver, SymbolTable};

pub(super) struct FunctionDeclarationValidation<'a> {
    pub(super) table: &'a mut SymbolTable,
    pub(super) type_params: &'a [TypeParam],
    pub(super) params: &'a [Param],
    pub(super) return_type: &'a Option<AstType>,
    pub(super) body: &'a Expression,
    pub(super) span: Span,
    pub(super) diagnostics: &'a mut Vec<Diagnostic>,
}

pub(super) struct MethodDeclarationValidation<'a> {
    pub(super) table: &'a mut SymbolTable,
    pub(super) type_name: &'a str,
    pub(super) type_params: &'a [TypeParam],
    pub(super) params: &'a [Param],
    pub(super) return_type: &'a Option<AstType>,
    pub(super) body: &'a Expression,
    pub(super) span: Span,
    pub(super) diagnostics: &'a mut Vec<Diagnostic>,
}

pub(super) struct ImplBlockDeclarationValidation<'a> {
    pub(super) table: &'a mut SymbolTable,
    pub(super) type_name: &'a str,
    pub(super) behavior: &'a Option<String>,
    pub(super) behavior_type_args: &'a [AstType],
    pub(super) methods: &'a [Declaration],
    pub(super) span: Span,
    pub(super) diagnostics: &'a mut Vec<Diagnostic>,
}

struct CallableValidation<'a> {
    table: &'a mut SymbolTable,
    type_params: &'a [TypeParam],
    params: &'a [Param],
    return_type: &'a Option<AstType>,
    body: &'a Expression,
    span: Span,
    self_type_allowed: bool,
    diagnostics: &'a mut Vec<Diagnostic>,
}

struct ImplBehaviorAssociationValidation<'a> {
    table: &'a mut SymbolTable,
    type_name: &'a str,
    behavior: &'a Option<String>,
    behavior_type_args: &'a [AstType],
    type_known: bool,
    span: Span,
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl Resolver {
    pub(super) fn validate_function_declaration(&self, input: FunctionDeclarationValidation<'_>) {
        self.validate_callable_declaration(CallableValidation {
            table: input.table,
            type_params: input.type_params,
            params: input.params,
            return_type: input.return_type,
            body: input.body,
            span: input.span,
            self_type_allowed: false,
            diagnostics: input.diagnostics,
        });
    }

    pub(super) fn validate_method_declaration(&self, input: MethodDeclarationValidation<'_>) {
        if !self.is_known_type_name(input.table, &[], input.type_name) {
            input.diagnostics.push(Diagnostic::error(
                "E0201",
                format!("unknown type symbol '{}'", input.type_name),
                input.span,
            ));
        }
        self.validate_callable_declaration(CallableValidation {
            table: input.table,
            type_params: input.type_params,
            params: input.params,
            return_type: input.return_type,
            body: input.body,
            span: input.span,
            self_type_allowed: true,
            diagnostics: input.diagnostics,
        });
    }

    pub(super) fn validate_impl_block_declaration(
        &self,
        input: ImplBlockDeclarationValidation<'_>,
    ) {
        let table = input.table;
        let diagnostics = input.diagnostics;
        let type_known =
            self.validate_impl_type_name(table, input.type_name, input.span, diagnostics);
        self.validate_impl_behavior_association(ImplBehaviorAssociationValidation {
            table,
            type_name: input.type_name,
            behavior: input.behavior,
            behavior_type_args: input.behavior_type_args,
            type_known,
            span: input.span,
            diagnostics,
        });
        for type_arg in input.behavior_type_args {
            self.validate_type_ref(table, &[], type_arg, input.span, false, diagnostics);
        }
        self.validate_impl_methods(table, input.methods, diagnostics);
    }

    fn validate_impl_type_name(
        &self,
        table: &mut SymbolTable,
        type_name: &str,
        span: Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> bool {
        let type_known = self.is_known_type_name(table, &[], type_name);
        if !type_known {
            diagnostics.push(Diagnostic::error(
                "E0201",
                format!("unknown type symbol '{type_name}'"),
                span,
            ));
        }
        type_known
    }

    fn validate_impl_behavior_association(&self, input: ImplBehaviorAssociationValidation<'_>) {
        let Some(behavior) = input.behavior else {
            return;
        };
        let behavior_known = self.is_known_behavior_name(input.table, behavior);
        if !behavior_known {
            input.diagnostics.push(Diagnostic::error(
                "E0202",
                format!("unknown behavior symbol '{behavior}'"),
                input.span,
            ));
        }
        if input.type_known && behavior_known {
            let behavior_display = behavior_ref_display(behavior, input.behavior_type_args);
            if !input.table.record_behavior_impl(
                input.type_name,
                BehaviorRefMetadata {
                    name: behavior.clone(),
                    type_args: input.behavior_type_args.to_vec(),
                },
            ) {
                input.diagnostics.push(Diagnostic::error(
                    "E0217",
                    format!(
                        "duplicate behavior implementation `{behavior_display}` for `{}`",
                        input.type_name
                    ),
                    input.span,
                ));
            }
        }
    }

    fn validate_impl_methods(
        &self,
        table: &mut SymbolTable,
        methods: &[Declaration],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for method in methods {
            if let Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } = method
            {
                self.validate_callable_declaration(CallableValidation {
                    table: &mut *table,
                    type_params,
                    params,
                    return_type,
                    body,
                    span: *span,
                    self_type_allowed: true,
                    diagnostics: &mut *diagnostics,
                });
            }
        }
    }

    fn validate_callable_declaration(&self, input: CallableValidation<'_>) {
        let table = input.table;
        let diagnostics = input.diagnostics;
        self.validate_type_param_constraints(
            table,
            input.type_params,
            input.self_type_allowed,
            diagnostics,
        );
        self.validate_params(
            table,
            input.type_params,
            input.params,
            input.self_type_allowed,
            diagnostics,
        );
        if let Some(return_type) = input.return_type {
            self.validate_type_ref(
                table,
                input.type_params,
                return_type,
                input.span,
                input.self_type_allowed,
                diagnostics,
            );
        }
        let scope_id = table.new_scope();
        let mut locals = self.param_locals(table, input.params, scope_id, diagnostics);
        self.validate_expr_refs(
            table,
            input.type_params,
            input.body,
            &mut locals,
            input.self_type_allowed,
            diagnostics,
        );
    }
}
