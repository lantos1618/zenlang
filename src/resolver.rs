use std::collections::HashSet;

use crate::ast::{
    AstType, Declaration, Expression, Param, Pattern, Program, Statement, StringPart, TypeParam,
};
use crate::error::{Diagnostic, Span};

#[cfg(test)]
mod symbol_table_test_support;

mod declaration_definition;
mod metadata_helpers;
mod symbol_table;

use metadata_helpers::behavior_ref_display;
use symbol_table::ScopeStack;
pub use symbol_table::{
    BehaviorMethodTypeMetadata, BehaviorRefMetadata, MethodSignatureMetadata, Namespace, Symbol,
    SymbolId, SymbolTable, TypeParameterBoundMetadata, TypeParameterBoundRefMetadata,
};

#[derive(Debug, Default)]
pub struct Resolver;

impl Resolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_program(&self, program: &Program) -> Result<SymbolTable, Vec<Diagnostic>> {
        let mut table = SymbolTable::default();
        let mut diagnostics = Vec::new();

        for decl in &program.declarations {
            if let Err(diagnostic) = self.define_declaration(&mut table, decl) {
                diagnostics.push(*diagnostic);
            }
        }

        for decl in &program.declarations {
            self.validate_declaration_types(&mut table, decl, &mut diagnostics);
        }

        if diagnostics.is_empty() {
            Ok(table)
        } else {
            Err(diagnostics)
        }
    }

    fn validate_declaration_types(
        &self,
        table: &mut SymbolTable,
        decl: &Declaration,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match decl {
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, false, diagnostics);
                self.validate_params(table, type_params, params, false, diagnostics);
                if let Some(return_type) = return_type {
                    self.validate_type_ref(
                        table,
                        type_params,
                        return_type,
                        *span,
                        false,
                        diagnostics,
                    );
                }
                let scope_id = table.new_scope();
                let mut locals = self.param_locals(table, params, scope_id, diagnostics);
                self.validate_expr_refs(table, type_params, body, &mut locals, false, diagnostics);
            }
            Declaration::Method {
                type_name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                if !self.is_known_type_name(table, &[], type_name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{type_name}'"),
                        *span,
                    ));
                }
                self.validate_type_param_constraints(table, type_params, true, diagnostics);
                self.validate_params(table, type_params, params, true, diagnostics);
                if let Some(return_type) = return_type {
                    self.validate_type_ref(
                        table,
                        type_params,
                        return_type,
                        *span,
                        true,
                        diagnostics,
                    );
                }
                let scope_id = table.new_scope();
                let mut locals = self.param_locals(table, params, scope_id, diagnostics);
                self.validate_expr_refs(table, type_params, body, &mut locals, true, diagnostics);
            }
            Declaration::Struct {
                name,
                type_params,
                fields,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, false, diagnostics);
                let mut seen_fields = HashSet::new();
                for field in fields {
                    if !seen_fields.insert(field.name.as_str()) {
                        diagnostics.push(Diagnostic::error(
                            "E0211",
                            format!("duplicate field `{}` for struct `{name}`", field.name),
                            field.span,
                        ));
                    }
                    self.validate_type_ref(
                        table,
                        type_params,
                        &field.ty,
                        field.span,
                        false,
                        diagnostics,
                    );
                    if let Some(default) = &field.default {
                        let scope_id = table.new_scope();
                        let mut locals = ScopeStack::new(scope_id);
                        self.validate_expr_refs(
                            table,
                            type_params,
                            default,
                            &mut locals,
                            false,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, false, diagnostics);
                for variant in variants {
                    if let Some(payload) = &variant.payload {
                        self.validate_type_ref(
                            table,
                            type_params,
                            payload,
                            variant.span,
                            false,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::Behavior {
                name,
                type_params,
                methods,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, true, diagnostics);
                let mut seen_methods = HashSet::new();
                for method in methods {
                    if !seen_methods.insert(method.name.as_str()) {
                        diagnostics.push(Diagnostic::error(
                            "E0212",
                            format!("duplicate behavior method `{}` in `{name}`", method.name),
                            method.span,
                        ));
                    }
                    self.validate_params(table, type_params, &method.params, true, diagnostics);
                    if let Some(return_type) = &method.return_type {
                        self.validate_type_ref(
                            table,
                            type_params,
                            return_type,
                            method.span,
                            true,
                            diagnostics,
                        );
                    }
                    if let Some(default_body) = &method.default_body {
                        let scope_id = table.new_scope();
                        let mut locals =
                            self.param_locals(table, &method.params, scope_id, diagnostics);
                        self.validate_expr_refs(
                            table,
                            type_params,
                            default_body,
                            &mut locals,
                            true,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::ImplBlock {
                type_name,
                behavior,
                behavior_type_args,
                methods,
                span,
                ..
            } => {
                if !self.is_known_type_name(table, &[], type_name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{type_name}'"),
                        *span,
                    ));
                }
                if let Some(behavior) = behavior {
                    let behavior_known = self.is_known_behavior_name(table, behavior);
                    if !behavior_known {
                        diagnostics.push(Diagnostic::error(
                            "E0202",
                            format!("unknown behavior symbol '{behavior}'"),
                            *span,
                        ));
                    }
                    if self.is_known_type_name(table, &[], type_name) && behavior_known {
                        let behavior_display = behavior_ref_display(behavior, behavior_type_args);
                        if !table.record_behavior_impl(
                            type_name,
                            BehaviorRefMetadata {
                                name: behavior.clone(),
                                type_args: behavior_type_args.clone(),
                            },
                        ) {
                            diagnostics.push(Diagnostic::error(
                                "E0217",
                                format!(
                                    "duplicate behavior implementation `{behavior_display}` for `{type_name}`"
                                ),
                                *span,
                            ));
                        }
                    }
                }
                for type_arg in behavior_type_args {
                    self.validate_type_ref(table, &[], type_arg, *span, false, diagnostics);
                }
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
                        self.validate_type_param_constraints(table, type_params, true, diagnostics);
                        self.validate_params(table, type_params, params, true, diagnostics);
                        if let Some(return_type) = return_type {
                            self.validate_type_ref(
                                table,
                                type_params,
                                return_type,
                                *span,
                                true,
                                diagnostics,
                            );
                        }
                        let scope_id = table.new_scope();
                        let mut locals = self.param_locals(table, params, scope_id, diagnostics);
                        self.validate_expr_refs(
                            table,
                            type_params,
                            body,
                            &mut locals,
                            true,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::Import { .. } | Declaration::Error { .. } => {}
            Declaration::Requires {
                type_name,
                behavior,
                behavior_type_args,
                span,
            } => {
                if !self.is_known_type_name(table, &[], type_name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{type_name}'"),
                        *span,
                    ));
                }
                if !self.is_known_behavior_name(table, behavior) {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{behavior}'"),
                        *span,
                    ));
                } else if self.is_known_type_name(table, &[], type_name) {
                    let behavior_display = behavior_ref_display(behavior, behavior_type_args);
                    if !table.record_behavior_required(
                        type_name,
                        BehaviorRefMetadata {
                            name: behavior.clone(),
                            type_args: behavior_type_args.clone(),
                        },
                    ) {
                        diagnostics.push(Diagnostic::error(
                            "E0216",
                            format!(
                                "duplicate required behavior `{behavior_display}` for `{type_name}`"
                            ),
                            *span,
                        ));
                    }
                }
                for type_arg in behavior_type_args {
                    self.validate_type_ref(table, &[], type_arg, *span, false, diagnostics);
                }
            }
            Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } => {
                let behavior_known = self.is_known_behavior_name(table, behavior);
                let parent_known = self.is_known_behavior_name(table, parent);
                if !behavior_known {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{behavior}'"),
                        *span,
                    ));
                }
                if !parent_known {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{parent}'"),
                        *span,
                    ));
                }
                let behavior_type_params = self.behavior_type_params_for_ref(table, behavior);
                for type_arg in parent_type_args {
                    self.validate_type_ref(
                        table,
                        &behavior_type_params,
                        type_arg,
                        *span,
                        false,
                        diagnostics,
                    );
                }
                if behavior_known && parent_known {
                    let parent_display = behavior_ref_display(parent, parent_type_args);
                    if !table.record_behavior_parent(
                        behavior,
                        BehaviorRefMetadata {
                            name: parent.clone(),
                            type_args: parent_type_args.clone(),
                        },
                    ) {
                        diagnostics.push(Diagnostic::error(
                            "E0215",
                            format!(
                                "duplicate behavior parent `{parent_display}` for `{behavior}`"
                            ),
                            *span,
                        ));
                    }
                }
            }
            Declaration::TopLevelExpr { expr, .. } => {
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
    }

    fn validate_params(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        params: &[Param],
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut seen_params = HashSet::new();
        for param in params {
            if !seen_params.insert(param.name.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "E0214",
                    format!("duplicate parameter `{}`", param.name),
                    param.span,
                ));
            }
            self.validate_type_ref(
                table,
                type_params,
                &param.ty,
                param.span,
                allow_self_type,
                diagnostics,
            );
        }
    }

    fn validate_type_param_constraints(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut seen_type_params = HashSet::new();
        for type_param in type_params {
            if !seen_type_params.insert(type_param.name.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "E0213",
                    format!("duplicate type parameter `{}`", type_param.name),
                    type_param.span,
                ));
            }
            if let Some(constraint) = &type_param.constraint {
                if !self.is_known_behavior_name(table, constraint) {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{constraint}'"),
                        type_param.span,
                    ));
                }
                for type_arg in &type_param.constraint_type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        type_param.span,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
        }
    }

    fn validate_type_ref(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        ast_type: &AstType,
        span: Span,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match ast_type {
            AstType::Named(name) => {
                if !self.is_known_type_name(table, type_params, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{name}'"),
                        span,
                    ));
                }
            }
            AstType::Generic { name, type_args } => {
                if !self.is_known_type_name(table, type_params, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{name}'"),
                        span,
                    ));
                }
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
            AstType::Array { elem, .. }
            | AstType::Slice(elem)
            | AstType::Ptr(elem)
            | AstType::MutPtr(elem)
            | AstType::RawPtr(elem) => {
                self.validate_type_ref(
                    table,
                    type_params,
                    elem,
                    span,
                    allow_self_type,
                    diagnostics,
                );
            }
            AstType::Function { params, ret } => {
                for param in params {
                    self.validate_type_ref(
                        table,
                        type_params,
                        param,
                        span,
                        allow_self_type,
                        diagnostics,
                    );
                }
                self.validate_type_ref(table, type_params, ret, span, allow_self_type, diagnostics);
            }
            AstType::SelfType => {
                if !allow_self_type {
                    diagnostics.push(Diagnostic::error(
                        "E0204",
                        "Self type is only valid in method or behavior contexts",
                        span,
                    ));
                }
            }
            AstType::I8
            | AstType::I16
            | AstType::I32
            | AstType::I64
            | AstType::U8
            | AstType::U16
            | AstType::U32
            | AstType::U64
            | AstType::Usize
            | AstType::F32
            | AstType::F64
            | AstType::Bool
            | AstType::Void
            | AstType::Str
            | AstType::String
            | AstType::Inferred => {}
        }
    }

    fn is_known_type_name(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        name: &str,
    ) -> bool {
        table.lookup(Namespace::Type, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
            || type_params.iter().any(|type_param| type_param.name == name)
    }

    fn is_known_behavior_name(&self, table: &SymbolTable, name: &str) -> bool {
        table.lookup(Namespace::Behavior, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
    }

    fn behavior_type_params_for_ref(&self, table: &SymbolTable, behavior: &str) -> Vec<TypeParam> {
        table
            .lookup(Namespace::Behavior, behavior)
            .and_then(|symbol| symbol.type_parameter_names.as_ref())
            .map(|names| {
                names
                    .iter()
                    .map(|name| TypeParam {
                        name: name.clone(),
                        constraint: None,
                        constraint_type_args: Vec::new(),
                        span: Span::dummy(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn validate_expr_refs(
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
            Expression::UnaryOp { operand, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    operand,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::MemberAccess { object, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    object,
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

    fn validate_statement_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        statement: &Statement,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
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
                    self.validate_type_ref(
                        table,
                        type_params,
                        ty,
                        statement.span(),
                        allow_self_type,
                        diagnostics,
                    );
                }
                self.validate_expr_refs(
                    table,
                    type_params,
                    value,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                if *constant || *mutable || !locals.is_mutable(name) {
                    self.define_local_symbol(
                        table,
                        name,
                        *mutable,
                        statement.span(),
                        locals,
                        diagnostics,
                    );
                }
            }
            Statement::Assignment { target, value, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    target,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_expr_refs(
                    table,
                    type_params,
                    value,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Statement::Expression { expr, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    expr,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Statement::Block { stmts, .. } => {
                let block_scope_id = table.new_scope();
                let mut block_locals = ScopeStack::with_parent(block_scope_id, locals);
                for statement in stmts {
                    self.validate_statement_refs(
                        table,
                        type_params,
                        statement,
                        &mut block_locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
        }
    }

    fn is_known_value_name(&self, table: &SymbolTable, locals: &ScopeStack, name: &str) -> bool {
        table.lookup(Namespace::Value, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
            || locals.contains(name)
    }

    fn param_locals(
        &self,
        table: &mut SymbolTable,
        params: &[Param],
        scope_id: u32,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> ScopeStack {
        let mut locals = ScopeStack::new(scope_id);
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

    fn define_local_symbol(
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
            Err(diagnostic) => diagnostics.push(*diagnostic),
        }
    }

    fn bind_pattern_locals(
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
            Pattern::Or { patterns, .. } => {
                for pattern in patterns {
                    self.bind_pattern_locals(table, pattern, locals, diagnostics);
                }
            }
            Pattern::Wildcard { .. }
            | Pattern::Literal { .. }
            | Pattern::Enum { payload: None, .. }
            | Pattern::Range { .. }
            | Pattern::BoolTrue { .. }
            | Pattern::BoolFalse { .. } => {}
        }
    }
}
