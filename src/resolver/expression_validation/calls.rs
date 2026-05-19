use crate::ast::{AstType, Expression, TypeParam};
use crate::error::{Diagnostic, Span};

use super::super::symbol_table::ScopeStack;
use super::super::{Resolver, SymbolTable};

pub(in crate::resolver) struct FunctionCallRef<'a> {
    pub name: &'a str,
    pub module: Option<&'a str>,
    pub type_args: &'a [AstType],
    pub args: &'a [Expression],
    pub span: Span,
}

pub(in crate::resolver) struct MethodCallRef<'a> {
    pub receiver: &'a Expression,
    pub type_args: &'a [AstType],
    pub args: &'a [Expression],
    pub span: Span,
}

impl Resolver {
    pub(in crate::resolver) fn validate_function_call_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        call: FunctionCallRef<'_>,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let FunctionCallRef {
            name,
            module,
            type_args,
            args,
            span,
        } = call;

        self.validate_type_arg_refs(
            table,
            type_params,
            type_args,
            span,
            allow_self_type,
            diagnostics,
        );
        if module.is_none() && !self.is_known_value_name(table, locals, name) {
            diagnostics.push(Diagnostic::error(
                "E0203",
                format!("unknown value symbol '{name}'"),
                span,
            ));
        }
        self.validate_expr_arg_refs(
            table,
            type_params,
            args,
            locals,
            allow_self_type,
            diagnostics,
        );
    }

    pub(in crate::resolver) fn validate_identifier_expr_refs(
        &self,
        table: &SymbolTable,
        name: &str,
        span: Span,
        locals: &ScopeStack,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_known_value_name(table, locals, name) {
            diagnostics.push(Diagnostic::error(
                "E0203",
                format!("unknown value symbol '{name}'"),
                span,
            ));
        }
    }

    pub(in crate::resolver) fn validate_method_call_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        call: MethodCallRef<'_>,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let MethodCallRef {
            receiver,
            type_args,
            args,
            span,
        } = call;

        self.validate_expr_refs(
            table,
            type_params,
            receiver,
            locals,
            allow_self_type,
            diagnostics,
        );
        self.validate_type_arg_refs(
            table,
            type_params,
            type_args,
            span,
            allow_self_type,
            diagnostics,
        );
        self.validate_expr_arg_refs(
            table,
            type_params,
            args,
            locals,
            allow_self_type,
            diagnostics,
        );
    }
}
