use std::collections::HashSet;

use crate::ast::{AstType, Expression, MatchArm, Param, Statement, TypeParam};
use crate::error::{Diagnostic, Span};

use super::symbol_table::ScopeStack;
use super::{Namespace, Resolver, SymbolTable};

pub(super) struct StructLiteralRef<'a> {
    pub name: &'a str,
    pub type_args: &'a [AstType],
    pub fields: &'a [(String, Expression)],
    pub span: Span,
}

pub(super) struct EnumVariantRef<'a> {
    pub enum_name: &'a str,
    pub type_args: &'a [AstType],
    pub variant: &'a str,
    pub payload: Option<&'a Expression>,
    pub span: Span,
}

pub(super) struct BlockRef<'a> {
    pub statements: &'a [Statement],
    pub expr: Option<&'a Expression>,
}

pub(super) struct ClosureRef<'a> {
    pub params: &'a [Param],
    pub return_type: Option<&'a AstType>,
    pub body: &'a Expression,
    pub span: Span,
}

impl Resolver {
    pub(super) fn validate_type_arg_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        type_args: &[AstType],
        span: Span,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for type_arg in type_args {
            self.validate_type_ref(
                table,
                type_params,
                type_arg,
                span,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(super) fn validate_expr_arg_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        args: &[Expression],
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for arg in args {
            self.validate_expr_refs(
                table,
                type_params,
                arg,
                locals,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(super) fn validate_struct_literal_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        literal: StructLiteralRef<'_>,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let StructLiteralRef {
            name,
            type_args,
            fields,
            span,
        } = literal;

        if let Some(symbol) = table.lookup(Namespace::Type, name) {
            if let Some(field_type_names) = symbol.field_type_names.as_ref() {
                let expected_fields: HashSet<&str> = field_type_names
                    .iter()
                    .map(|(field_name, _)| field_name.as_str())
                    .collect();
                let mut provided_fields = HashSet::new();

                for (field_name, _) in fields {
                    if !provided_fields.insert(field_name.as_str()) {
                        diagnostics.push(Diagnostic::error(
                            "E0208",
                            format!("duplicate field `{field_name}` for struct `{name}`"),
                            span,
                        ));
                    }
                    if !expected_fields.contains(field_name.as_str()) {
                        diagnostics.push(Diagnostic::error(
                            "E0209",
                            format!("unknown field `{field_name}` for struct `{name}`"),
                            span,
                        ));
                    }
                }

                for expected_field in expected_fields {
                    if !provided_fields.contains(expected_field) {
                        diagnostics.push(Diagnostic::error(
                            "E0210",
                            format!("missing field `{expected_field}` for struct `{name}`"),
                            span,
                        ));
                    }
                }
            }
        } else if !self.is_known_type_name(table, type_params, name) {
            diagnostics.push(Diagnostic::error(
                "E0201",
                format!("unknown type symbol '{name}'"),
                span,
            ));
        }

        self.validate_type_arg_refs(
            table,
            type_params,
            type_args,
            span,
            allow_self_type,
            diagnostics,
        );
        for (_, value) in fields {
            self.validate_expr_refs(
                table,
                type_params,
                value,
                locals,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(super) fn validate_enum_variant_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        variant_ref: EnumVariantRef<'_>,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let EnumVariantRef {
            enum_name,
            type_args,
            variant,
            payload,
            span,
        } = variant_ref;

        if table.lookup(Namespace::Type, enum_name).is_some() {
            if let Some(variant_symbol) = table.lookup_variant(enum_name, variant) {
                match (
                    variant_symbol.variant_payload_count.unwrap_or(0),
                    payload.is_some(),
                ) {
                    (1, false) => diagnostics.push(Diagnostic::error(
                        "E0206",
                        format!("enum variant `{enum_name}.{variant}` requires a payload"),
                        span,
                    )),
                    (0, true) => diagnostics.push(Diagnostic::error(
                        "E0207",
                        format!("enum variant `{enum_name}.{variant}` does not accept a payload"),
                        span,
                    )),
                    _ => {}
                }
            } else {
                diagnostics.push(Diagnostic::error(
                    "E0205",
                    format!("enum `{enum_name}` has no variant `{variant}`"),
                    span,
                ));
            }
        } else if !self.is_known_type_name(table, type_params, enum_name) {
            diagnostics.push(Diagnostic::error(
                "E0201",
                format!("unknown type symbol '{enum_name}'"),
                span,
            ));
        }

        self.validate_type_arg_refs(
            table,
            type_params,
            type_args,
            span,
            allow_self_type,
            diagnostics,
        );
        if let Some(payload) = payload {
            self.validate_expr_refs(
                table,
                type_params,
                payload,
                locals,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(super) fn validate_match_arm_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        arm: &MatchArm,
        locals: &ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if let Some(guard) = &arm.guard {
            let arm_scope_id = table.new_scope();
            let mut arm_locals = ScopeStack::with_parent(arm_scope_id, locals);
            self.bind_pattern_locals(table, &arm.pattern, &mut arm_locals, diagnostics);
            self.validate_expr_refs(
                table,
                type_params,
                guard,
                &mut arm_locals,
                allow_self_type,
                diagnostics,
            );
        }

        let arm_scope_id = table.new_scope();
        let mut arm_locals = ScopeStack::with_parent(arm_scope_id, locals);
        self.bind_pattern_locals(table, &arm.pattern, &mut arm_locals, diagnostics);
        self.validate_expr_refs(
            table,
            type_params,
            &arm.body,
            &mut arm_locals,
            allow_self_type,
            diagnostics,
        );
    }

    pub(super) fn validate_child_scope_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let scope_id = table.new_scope();
        let mut child_locals = ScopeStack::with_parent(scope_id, locals);
        self.validate_expr_refs(
            table,
            type_params,
            expr,
            &mut child_locals,
            allow_self_type,
            diagnostics,
        );
    }

    pub(super) fn validate_block_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        block: BlockRef<'_>,
        locals: &ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let BlockRef { statements, expr } = block;

        let block_scope_id = table.new_scope();
        let mut block_locals = ScopeStack::with_parent(block_scope_id, locals);
        for statement in statements {
            self.validate_statement_refs(
                table,
                type_params,
                statement,
                &mut block_locals,
                allow_self_type,
                diagnostics,
            );
        }
        if let Some(expr) = expr {
            self.validate_expr_refs(
                table,
                type_params,
                expr,
                &mut block_locals,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(super) fn validate_closure_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        closure: ClosureRef<'_>,
        locals: &ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let ClosureRef {
            params,
            return_type,
            body,
            span,
        } = closure;

        let closure_scope_id = table.new_scope();
        let mut closure_locals = ScopeStack::with_parent(closure_scope_id, locals);
        for param in params {
            self.validate_type_ref(
                table,
                type_params,
                &param.ty,
                param.span,
                allow_self_type,
                diagnostics,
            );
            self.define_local_symbol(
                table,
                &param.name,
                param.mutable,
                param.span,
                &mut closure_locals,
                diagnostics,
            );
        }
        if let Some(return_type) = return_type {
            self.validate_type_ref(
                table,
                type_params,
                return_type,
                span,
                allow_self_type,
                diagnostics,
            );
        }
        self.validate_expr_refs(
            table,
            type_params,
            body,
            &mut closure_locals,
            allow_self_type,
            diagnostics,
        );
    }
}
