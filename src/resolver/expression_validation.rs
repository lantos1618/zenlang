use std::collections::HashSet;

use crate::ast::{Expression, StringPart, TypeParam};
use crate::error::Diagnostic;

use super::symbol_table::ScopeStack;
use super::{Namespace, Resolver, SymbolTable};

impl Resolver {
    pub(super) fn validate_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match expr {
            Expression::FunctionCall {
                name,
                module,
                type_args,
                args,
                span,
            } => {
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        *span,
                        allow_self_type,
                        diagnostics,
                    );
                }
                if module.is_none() && !self.is_known_value_name(table, locals, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0203",
                        format!("unknown value symbol '{name}'"),
                        *span,
                    ));
                }
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
            Expression::Identifier { name, span } => {
                if !self.is_known_value_name(table, locals, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0203",
                        format!("unknown value symbol '{name}'"),
                        *span,
                    ));
                }
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    receiver,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        *span,
                        allow_self_type,
                        diagnostics,
                    );
                }
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
            Expression::BinaryOp { left, right, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    left,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_expr_refs(
                    table,
                    type_params,
                    right,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::UnaryOp { operand, .. }
            | Expression::MemberAccess {
                object: operand, ..
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    operand,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::IndexAccess { object, index, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    object,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_expr_refs(
                    table,
                    type_params,
                    index,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::StructLiteral {
                name,
                type_args,
                fields,
                span,
            } => {
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
                                    *span,
                                ));
                            }
                            if !expected_fields.contains(field_name.as_str()) {
                                diagnostics.push(Diagnostic::error(
                                    "E0209",
                                    format!("unknown field `{field_name}` for struct `{name}`"),
                                    *span,
                                ));
                            }
                        }

                        for expected_field in expected_fields {
                            if !provided_fields.contains(expected_field) {
                                diagnostics.push(Diagnostic::error(
                                    "E0210",
                                    format!("missing field `{expected_field}` for struct `{name}`"),
                                    *span,
                                ));
                            }
                        }
                    }
                } else if !self.is_known_type_name(table, type_params, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{name}'"),
                        *span,
                    ));
                }
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        *span,
                        allow_self_type,
                        diagnostics,
                    );
                }
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
            Expression::EnumVariant {
                enum_name,
                type_args,
                variant,
                payload,
                span,
            } => {
                if table.lookup(Namespace::Type, enum_name).is_some() {
                    if let Some(variant_symbol) = table.lookup_variant(enum_name, variant) {
                        match (
                            variant_symbol.variant_payload_count.unwrap_or(0),
                            payload.is_some(),
                        ) {
                            (1, false) => diagnostics.push(Diagnostic::error(
                                "E0206",
                                format!("enum variant `{enum_name}.{variant}` requires a payload"),
                                *span,
                            )),
                            (0, true) => diagnostics.push(Diagnostic::error(
                                "E0207",
                                format!(
                                    "enum variant `{enum_name}.{variant}` does not accept a payload"
                                ),
                                *span,
                            )),
                            _ => {}
                        }
                    } else {
                        diagnostics.push(Diagnostic::error(
                            "E0205",
                            format!("enum `{enum_name}` has no variant `{variant}`"),
                            *span,
                        ));
                    }
                } else if !self.is_known_type_name(table, type_params, enum_name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{enum_name}'"),
                        *span,
                    ));
                }
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        *span,
                        allow_self_type,
                        diagnostics,
                    );
                }
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
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.validate_expr_refs(
                        table,
                        type_params,
                        element,
                        locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    scrutinee,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                for arm in arms {
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
            }
            Expression::WhileLoop {
                condition, body, ..
            }
            | Expression::If {
                condition,
                then_body: body,
                ..
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    condition,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                let body_scope_id = table.new_scope();
                let mut body_locals = ScopeStack::with_parent(body_scope_id, locals);
                self.validate_expr_refs(
                    table,
                    type_params,
                    body,
                    &mut body_locals,
                    allow_self_type,
                    diagnostics,
                );
                if let Expression::If {
                    else_body: Some(else_body),
                    ..
                } = expr
                {
                    let else_scope_id = table.new_scope();
                    let mut else_locals = ScopeStack::with_parent(else_scope_id, locals);
                    self.validate_expr_refs(
                        table,
                        type_params,
                        else_body,
                        &mut else_locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::Loop { body, .. } => {
                let body_scope_id = table.new_scope();
                let mut body_locals = ScopeStack::with_parent(body_scope_id, locals);
                self.validate_expr_refs(
                    table,
                    type_params,
                    body,
                    &mut body_locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Block {
                statements, expr, ..
            } => {
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
            Expression::Return { value, .. } => {
                if let Some(value) = value {
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
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => {
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
                        *span,
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
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    expr,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_type_ref(
                    table,
                    type_params,
                    target_type,
                    *span,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let StringPart::Expr(expr) = part {
                        self.validate_expr_refs(
                            table,
                            type_params,
                            expr,
                            locals,
                            allow_self_type,
                            diagnostics,
                        );
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    start,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_expr_refs(
                    table,
                    type_params,
                    end,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Defer { expr, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    expr,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::LoopControl { .. }
            | Expression::Error { .. } => {}
        }
    }
}
