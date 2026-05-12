use std::collections::HashMap;

use crate::ast::{
    AstType, Declaration, Expression, Param, Pattern, Program, Statement, StringPart, TypeParam,
};
use crate::error::{Diagnostic, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    Value,
    Type,
    Module,
    Import,
    Local,
    Behavior,
    Variant,
}

impl Namespace {
    pub(crate) fn diagnostic_name(self) -> &'static str {
        match self {
            Namespace::Value => "value",
            Namespace::Type => "type",
            Namespace::Module => "module",
            Namespace::Import => "import",
            Namespace::Local => "local",
            Namespace::Behavior => "behavior",
            Namespace::Variant => "variant",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: SymbolId,
    pub namespace: Namespace,
    pub name: String,
    pub is_public: bool,
    pub import_source: Option<String>,
    pub scope_id: u32,
    pub definition_span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    by_name: HashMap<(Namespace, String), SymbolId>,
    by_scoped_name: HashMap<(Namespace, String, u32), SymbolId>,
    next_scope_id: u32,
}

#[derive(Debug, Clone)]
struct ScopeStack {
    current_scope_id: u32,
    visible_names: HashMap<String, bool>,
}

impl ScopeStack {
    fn new(current_scope_id: u32) -> Self {
        Self {
            current_scope_id,
            visible_names: HashMap::new(),
        }
    }

    fn with_parent(current_scope_id: u32, parent: &ScopeStack) -> Self {
        Self {
            current_scope_id,
            visible_names: parent.visible_names.clone(),
        }
    }

    fn contains(&self, name: &str) -> bool {
        self.visible_names.contains_key(name)
    }

    fn is_mutable(&self, name: &str) -> bool {
        self.visible_names.get(name).copied().unwrap_or(false)
    }

    fn insert(&mut self, name: String, mutable: bool) {
        self.visible_names.insert(name, mutable);
    }
}

impl SymbolTable {
    pub fn lookup(&self, namespace: Namespace, name: &str) -> Option<&Symbol> {
        let id = self.by_name.get(&(namespace, name.to_string()))?;
        self.symbols.get(id.0 as usize)
    }

    pub fn lookup_scoped(&self, namespace: Namespace, name: &str) -> Option<&Symbol> {
        self.symbols
            .iter()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    fn define(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_public: bool,
        import_source: Option<String>,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        self.define_in_scope(
            namespace,
            name,
            is_public,
            import_source,
            0,
            definition_span,
        )
    }

    fn define_in_scope(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_public: bool,
        import_source: Option<String>,
        scope_id: u32,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        let scoped_key = (namespace, name.to_string(), scope_id);
        if self.by_scoped_name.contains_key(&scoped_key) {
            return Err(Box::new(Diagnostic::error(
                "E0200",
                format!(
                    "duplicate {} symbol '{}'",
                    namespace.diagnostic_name(),
                    name
                ),
                definition_span,
            )));
        }

        let id = SymbolId(self.symbols.len() as u32);
        if namespace != Namespace::Local {
            self.by_name.insert((namespace, name.to_string()), id);
        }
        self.symbols.push(Symbol {
            id,
            namespace,
            name: name.to_string(),
            is_public,
            import_source,
            scope_id,
            definition_span,
        });
        self.by_scoped_name.insert(scoped_key, id);
        Ok(id)
    }

    fn define_local(
        &mut self,
        name: &str,
        scope_id: u32,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        self.define_in_scope(
            Namespace::Local,
            name,
            false,
            None,
            scope_id,
            definition_span,
        )
    }

    fn new_scope(&mut self) -> u32 {
        self.next_scope_id += 1;
        self.next_scope_id
    }
}

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

    fn define_declaration(
        &self,
        table: &mut SymbolTable,
        decl: &Declaration,
    ) -> Result<(), Box<Diagnostic>> {
        match decl {
            Declaration::Function {
                name, public, span, ..
            } => {
                table.define(Namespace::Value, name, *public, None, *span)?;
            }
            Declaration::Method {
                type_name,
                method_name,
                public,
                span,
                ..
            } => {
                table.define(
                    Namespace::Value,
                    &format!("{type_name}.{method_name}"),
                    *public,
                    None,
                    *span,
                )?;
            }
            Declaration::Struct {
                name, public, span, ..
            } => {
                table.define(Namespace::Type, name, *public, None, *span)?;
            }
            Declaration::Enum {
                name,
                variants,
                public,
                span,
                ..
            } => {
                table.define(Namespace::Type, name, *public, None, *span)?;
                for variant in variants {
                    table.define(
                        Namespace::Variant,
                        &variant.name,
                        *public,
                        None,
                        variant.span,
                    )?;
                }
            }
            Declaration::Behavior { name, span, .. } => {
                table.define(Namespace::Behavior, name, false, None, *span)?;
            }
            Declaration::Import {
                names,
                module_path,
                span,
                ..
            } => {
                let source = module_path.join(".");
                table.define(Namespace::Module, &source, false, None, *span)?;
                for name in names {
                    table.define(Namespace::Import, name, false, Some(source.clone()), *span)?;
                }
            }
            Declaration::ImplBlock { .. }
            | Declaration::TopLevelExpr { .. }
            | Declaration::Error { .. } => {}
        }
        Ok(())
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
                self.validate_type_param_constraints(table, type_params, diagnostics);
                self.validate_params(table, type_params, params, diagnostics);
                if let Some(return_type) = return_type {
                    self.validate_type_ref(table, type_params, return_type, *span, diagnostics);
                }
                let scope_id = table.new_scope();
                let mut locals = self.param_locals(table, params, scope_id, diagnostics);
                self.validate_expr_refs(table, type_params, body, &mut locals, diagnostics);
            }
            Declaration::Method {
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, diagnostics);
                self.validate_params(table, type_params, params, diagnostics);
                if let Some(return_type) = return_type {
                    self.validate_type_ref(table, type_params, return_type, *span, diagnostics);
                }
                let scope_id = table.new_scope();
                let mut locals = self.param_locals(table, params, scope_id, diagnostics);
                self.validate_expr_refs(table, type_params, body, &mut locals, diagnostics);
            }
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, diagnostics);
                for field in fields {
                    self.validate_type_ref(table, type_params, &field.ty, field.span, diagnostics);
                    if let Some(default) = &field.default {
                        let scope_id = table.new_scope();
                        let mut locals = ScopeStack::new(scope_id);
                        self.validate_expr_refs(
                            table,
                            type_params,
                            default,
                            &mut locals,
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
                self.validate_type_param_constraints(table, type_params, diagnostics);
                for variant in variants {
                    if let Some(payload) = &variant.payload {
                        self.validate_type_ref(
                            table,
                            type_params,
                            payload,
                            variant.span,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, diagnostics);
                for method in methods {
                    self.validate_params(table, type_params, &method.params, diagnostics);
                    if let Some(return_type) = &method.return_type {
                        self.validate_type_ref(
                            table,
                            type_params,
                            return_type,
                            method.span,
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
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::ImplBlock { methods, .. } => {
                for method in methods {
                    self.validate_declaration_types(table, method, diagnostics);
                }
            }
            Declaration::Import { .. } | Declaration::Error { .. } => {}
            Declaration::TopLevelExpr { expr, .. } => {
                let scope_id = table.new_scope();
                self.validate_expr_refs(
                    table,
                    &[],
                    expr,
                    &mut ScopeStack::new(scope_id),
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
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for param in params {
            self.validate_type_ref(table, type_params, &param.ty, param.span, diagnostics);
        }
    }

    fn validate_type_param_constraints(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for type_param in type_params {
            if let Some(constraint) = &type_param.constraint {
                if table.lookup(Namespace::Behavior, constraint).is_none() {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{constraint}'"),
                        type_param.span,
                    ));
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
                    self.validate_type_ref(table, type_params, type_arg, span, diagnostics);
                }
            }
            AstType::Array { elem, .. }
            | AstType::Slice(elem)
            | AstType::Ptr(elem)
            | AstType::MutPtr(elem)
            | AstType::RawPtr(elem) => {
                self.validate_type_ref(table, type_params, elem, span, diagnostics);
            }
            AstType::Function { params, ret } => {
                for param in params {
                    self.validate_type_ref(table, type_params, param, span, diagnostics);
                }
                self.validate_type_ref(table, type_params, ret, span, diagnostics);
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
            | AstType::SelfType
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

    fn validate_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &mut ScopeStack,
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
                    self.validate_type_ref(table, type_params, type_arg, *span, diagnostics);
                }
                if module.is_none() && !self.is_known_value_name(table, locals, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0203",
                        format!("unknown value symbol '{name}'"),
                        *span,
                    ));
                }
                for arg in args {
                    self.validate_expr_refs(table, type_params, arg, locals, diagnostics);
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
                self.validate_expr_refs(table, type_params, receiver, locals, diagnostics);
                for type_arg in type_args {
                    self.validate_type_ref(table, type_params, type_arg, *span, diagnostics);
                }
                for arg in args {
                    self.validate_expr_refs(table, type_params, arg, locals, diagnostics);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.validate_expr_refs(table, type_params, left, locals, diagnostics);
                self.validate_expr_refs(table, type_params, right, locals, diagnostics);
            }
            Expression::UnaryOp { operand, .. } => {
                self.validate_expr_refs(table, type_params, operand, locals, diagnostics);
            }
            Expression::MemberAccess { object, .. } => {
                self.validate_expr_refs(table, type_params, object, locals, diagnostics);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.validate_expr_refs(table, type_params, object, locals, diagnostics);
                self.validate_expr_refs(table, type_params, index, locals, diagnostics);
            }
            Expression::StructLiteral {
                type_args,
                fields,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_type_ref(table, type_params, type_arg, *span, diagnostics);
                }
                for (_, value) in fields {
                    self.validate_expr_refs(table, type_params, value, locals, diagnostics);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_type_ref(table, type_params, type_arg, *span, diagnostics);
                }
                if let Some(payload) = payload {
                    self.validate_expr_refs(table, type_params, payload, locals, diagnostics);
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.validate_expr_refs(table, type_params, element, locals, diagnostics);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_expr_refs(table, type_params, scrutinee, locals, diagnostics);
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
                self.validate_expr_refs(table, type_params, condition, locals, diagnostics);
                let body_scope_id = table.new_scope();
                let mut body_locals = ScopeStack::with_parent(body_scope_id, locals);
                self.validate_expr_refs(table, type_params, body, &mut body_locals, diagnostics);
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
                        diagnostics,
                    );
                }
            }
            Expression::Loop { body, .. } => {
                let body_scope_id = table.new_scope();
                let mut body_locals = ScopeStack::with_parent(body_scope_id, locals);
                self.validate_expr_refs(table, type_params, body, &mut body_locals, diagnostics);
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
                        diagnostics,
                    );
                }
                if let Some(expr) = expr {
                    self.validate_expr_refs(
                        table,
                        type_params,
                        expr,
                        &mut block_locals,
                        diagnostics,
                    );
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.validate_expr_refs(table, type_params, value, locals, diagnostics);
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
                    self.validate_type_ref(table, type_params, &param.ty, param.span, diagnostics);
                    self.define_local_symbol(
                        table,
                        &param.name,
                        false,
                        param.span,
                        &mut closure_locals,
                        diagnostics,
                    );
                }
                if let Some(return_type) = return_type {
                    self.validate_type_ref(table, type_params, return_type, *span, diagnostics);
                }
                self.validate_expr_refs(table, type_params, body, &mut closure_locals, diagnostics);
            }
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_expr_refs(table, type_params, expr, locals, diagnostics);
                self.validate_type_ref(table, type_params, target_type, *span, diagnostics);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let StringPart::Expr(expr) = part {
                        self.validate_expr_refs(table, type_params, expr, locals, diagnostics);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.validate_expr_refs(table, type_params, start, locals, diagnostics);
                self.validate_expr_refs(table, type_params, end, locals, diagnostics);
            }
            Expression::Defer { expr, .. } => {
                self.validate_expr_refs(table, type_params, expr, locals, diagnostics);
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn validate_statement_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        statement: &Statement,
        locals: &mut ScopeStack,
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
                    self.validate_type_ref(table, type_params, ty, statement.span(), diagnostics);
                }
                self.validate_expr_refs(table, type_params, value, locals, diagnostics);
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
                self.validate_expr_refs(table, type_params, target, locals, diagnostics);
                self.validate_expr_refs(table, type_params, value, locals, diagnostics);
            }
            Statement::Expression { expr, .. } => {
                self.validate_expr_refs(table, type_params, expr, locals, diagnostics);
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
                false,
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
        match table.define_local(name, locals.current_scope_id, span) {
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
