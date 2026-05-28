use std::collections::HashSet;

use crate::ast::{AstType, Expression};
use crate::error::{CompilerDiagnosticCode::*, Diagnostic, Span};

use super::super::{Namespace, Resolver};
use super::ExprRefContext;

impl Resolver {
    pub(in crate::resolver) fn validate_struct_literal_refs(
        &self,
        name: &str,
        type_args: &[AstType],
        fields: &[(String, Expression)],
        span: Span,
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        if let Some(symbol) = ctx.table.lookup(Namespace::Type, name) {
            if let Some(field_types) = symbol.field_types.as_ref() {
                let expected_fields: HashSet<&str> = field_types
                    .iter()
                    .map(|(field_name, _)| field_name.as_str())
                    .collect();
                let mut provided_fields = HashSet::new();

                for (field_name, _) in fields {
                    if !provided_fields.insert(field_name.as_str()) {
                        ctx.diagnostics.push(Diagnostic::error_code(
                            E0208,
                            format!("duplicate field `{field_name}` for struct `{name}`"),
                            span,
                        ));
                    }
                    if !expected_fields.contains(field_name.as_str()) {
                        ctx.diagnostics.push(Diagnostic::error_code(
                            E0209,
                            format!("unknown field `{field_name}` for struct `{name}`"),
                            span,
                        ));
                    }
                }

                for expected_field in expected_fields {
                    if !provided_fields.contains(expected_field) {
                        ctx.diagnostics.push(Diagnostic::error_code(
                            E0210,
                            format!("missing field `{expected_field}` for struct `{name}`"),
                            span,
                        ));
                    }
                }
            }
        } else if !self.is_known_type_name(ctx.table, ctx.type_params, name) {
            self.push_unknown_type_symbol(ctx.diagnostics, name, span);
        }

        self.validate_type_arg_refs(type_args, span, ctx);
        for (_, value) in fields {
            self.validate_expr_refs_in(value, ctx);
        }
    }

    pub(in crate::resolver) fn validate_enum_variant_refs(
        &self,
        enum_name: &str,
        type_args: &[AstType],
        variant: &str,
        payload: Option<&Expression>,
        span: Span,
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        if ctx.table.lookup(Namespace::Type, enum_name).is_some() {
            if let Some(variant_symbol) = ctx.table.lookup_variant(enum_name, variant) {
                match (
                    variant_symbol.variant_payload_type.is_some(),
                    payload.is_some(),
                ) {
                    (true, false) => ctx.diagnostics.push(Diagnostic::error_code(
                        E0206,
                        format!("enum variant `{enum_name}.{variant}` requires a payload"),
                        span,
                    )),
                    (false, true) => ctx.diagnostics.push(Diagnostic::error_code(
                        E0207,
                        format!("enum variant `{enum_name}.{variant}` does not accept a payload"),
                        span,
                    )),
                    _ => {}
                }
            } else {
                ctx.diagnostics.push(Diagnostic::error_code(
                    E0205,
                    format!("enum `{enum_name}` has no variant `{variant}`"),
                    span,
                ));
            }
        } else if !self.is_known_type_name(ctx.table, ctx.type_params, enum_name) {
            self.push_unknown_type_symbol(ctx.diagnostics, enum_name, span);
        }

        self.validate_type_arg_refs(type_args, span, ctx);
        if let Some(payload) = payload {
            self.validate_expr_refs_in(payload, ctx);
        }
    }
}
