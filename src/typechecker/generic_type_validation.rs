use super::*;

impl TypeChecker {
    #[cfg(test)]
    pub(super) fn collect_ast_type_reference_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<AstTypeReferenceValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_type_reference_validation_task(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_ast_type_reference_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstTypeReferenceValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Struct {
                type_params,
                fields,
            }),
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Enum {
                type_params,
                variants,
            }),
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Function {
                type_params,
                params,
                return_type,
                body,
            }),
            Declaration::Method {
                type_params,
                params,
                return_type,
                body,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Method {
                type_params,
                params,
                return_type,
                body,
            }),
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Behavior {
                type_params,
                methods,
            }),
            Declaration::ImplBlock { methods, .. } => {
                tasks.push(AstTypeReferenceValidationTask::ImplBlock { methods });
            }
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(AstTypeReferenceValidationTask::TopLevelExpr { expr });
            }
            _ => {}
        }
    }

    pub(super) fn validate_ast_type_reference_tasks(
        &mut self,
        tasks: &[AstTypeReferenceValidationTask<'_>],
    ) {
        for task in tasks {
            match task {
                AstTypeReferenceValidationTask::Struct {
                    type_params,
                    fields,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for field in *fields {
                        self.validate_generic_type_ref_bounds(&field.ty, &scoped, field.span);
                        if let Some(default) = &field.default {
                            self.validate_generic_expr_type_references(default, &scoped);
                        }
                    }
                }
                AstTypeReferenceValidationTask::Enum {
                    type_params,
                    variants,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for variant in *variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_generic_type_ref_bounds(payload, &scoped, variant.span);
                        }
                    }
                }
                AstTypeReferenceValidationTask::Function {
                    type_params,
                    params,
                    return_type,
                    body,
                } => {
                    self.validate_ast_callable_type_references(
                        type_params,
                        params,
                        return_type,
                        body,
                        Span::dummy(),
                    );
                }
                AstTypeReferenceValidationTask::Method {
                    type_params,
                    params,
                    return_type,
                    body,
                } => {
                    self.validate_ast_callable_type_references(
                        type_params,
                        params,
                        return_type,
                        body,
                        Span::dummy(),
                    );
                }
                AstTypeReferenceValidationTask::Behavior {
                    type_params,
                    methods,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for method in *methods {
                        for param in &method.params {
                            self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
                        }
                        if let Some(return_type) = &method.return_type {
                            self.validate_generic_type_ref_bounds(
                                return_type,
                                &scoped,
                                method.span,
                            );
                        }
                        if let Some(default_body) = &method.default_body {
                            self.validate_generic_expr_type_references(default_body, &scoped);
                        }
                    }
                }
                AstTypeReferenceValidationTask::ImplBlock { methods } => {
                    for method in *methods {
                        if let Declaration::Function {
                            type_params,
                            params,
                            return_type,
                            body,
                            ..
                        } = method
                        {
                            self.validate_ast_callable_type_references(
                                type_params,
                                params,
                                return_type,
                                body,
                                method.span(),
                            );
                        }
                    }
                }
                AstTypeReferenceValidationTask::TopLevelExpr { expr } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
            }
        }
    }

    fn validate_ast_callable_type_references(
        &mut self,
        type_params: &[ast::TypeParam],
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        return_span: Span,
    ) {
        let scoped = type_param_name_set(type_params);
        for param in params {
            self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
        }
        if let Some(return_type) = return_type {
            self.validate_generic_type_ref_bounds(return_type, &scoped, return_span);
        }
        self.validate_generic_expr_type_references(body, &scoped);
    }

    pub(super) fn validate_resolver_type_reference_tasks(
        &mut self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        for task in &tasks.type_references {
            match task {
                ResolverTypeReferenceValidationTask::Struct { name, fields, span } => {
                    self.validate_resolver_struct_type_references(symbols, name, fields, *span);
                }
                ResolverTypeReferenceValidationTask::Enum { name, span } => {
                    self.validate_resolver_enum_type_references(symbols, name, *span);
                }
                ResolverTypeReferenceValidationTask::Function { name, body, span } => {
                    self.validate_resolver_function_type_references(symbols, name, body, *span);
                }
                ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span,
                } => {
                    let ast_key = Self::method_key(type_name, method_name);
                    self.validate_resolver_method_type_references(
                        symbols, &ast_key, type_name, body, *span,
                    );
                }
                ResolverTypeReferenceValidationTask::Behavior {
                    name,
                    methods,
                    span,
                } => {
                    self.validate_resolver_behavior_type_references(symbols, name, methods, *span);
                }
                ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods } => {
                    self.validate_resolver_impl_method_type_references(symbols, type_name, methods);
                }
                ResolverTypeReferenceValidationTask::TopLevelExpr { expr } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
            }
        }
    }

    fn validate_resolver_enum_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Type, name, span);
        if let Some(scoped) = self.collected_type_type_param_scope(&restored_name) {
            self.validate_collected_enum_type_references(&restored_name, &scoped, span);
        }
    }

    fn validate_resolver_struct_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        fields: &[StructField],
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Type, name, span);
        if let Some(scoped) = self.collected_type_type_param_scope(&restored_name) {
            self.validate_collected_struct_type_references(&restored_name, &scoped, span);
            for field in fields {
                if let Some(default) = &field.default {
                    self.validate_generic_expr_type_references(default, &scoped);
                }
            }
        }
    }

    fn validate_resolver_behavior_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        methods: &[BehaviorMethod],
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Behavior, name, span);
        if let Some(scoped) = self.collected_behavior_type_param_scope(&restored_name) {
            self.validate_collected_behavior_type_references(&restored_name, &scoped, span);
            for method in methods {
                if let Some(default_body) = &method.default_body {
                    self.validate_generic_expr_type_references(default_body, &scoped);
                }
            }
        }
    }

    fn validate_resolver_impl_method_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        methods: &[Declaration],
    ) {
        for method in methods {
            if let Declaration::Function { name, body, .. } = method {
                let ast_key = Self::method_key(type_name, name);
                self.validate_resolver_method_type_references(
                    symbols,
                    &ast_key,
                    type_name,
                    body,
                    method.span(),
                );
            }
        }
    }

    fn validate_resolver_method_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        body: &Expression,
        span: Span,
    ) {
        let restored_key = Self::validation_method_key(symbols, ast_key, type_name, span);
        self.validate_resolver_callable_type_references(&restored_key, body, span);
    }

    fn validate_resolver_function_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        body: &Expression,
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Value, name, span);
        self.validate_resolver_callable_type_references(&restored_name, body, span);
    }

    fn validate_resolver_callable_type_references(
        &mut self,
        restored_key: &str,
        body: &Expression,
        span: Span,
    ) {
        if let Some(scoped) = self.collected_value_type_param_scope(restored_key) {
            self.validate_collected_value_type_references(restored_key, &scoped, span);
            self.validate_generic_expr_type_references(body, &scoped);
        }
    }

    pub(super) fn validation_symbol_name(
        symbols: Option<&SymbolTable>,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) -> String {
        symbols
            .map(|symbols| Self::resolver_symbol_name_for(symbols, namespace, name, span))
            .unwrap_or_else(|| name.to_string())
    }

    pub(super) fn validation_method_key(
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        span: Span,
    ) -> String {
        symbols
            .map(|symbols| {
                Self::resolver_method_signature_name_for(symbols, ast_key, type_name, span)
            })
            .unwrap_or_else(|| ast_key.to_string())
    }

    fn collected_value_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.functions
            .get(name)
            .or_else(|| self.methods.get(name))
            .map(|info| info.type_params.iter().cloned().collect())
    }

    fn collected_type_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.structs
            .get(name)
            .map(|info| info.type_params.iter().cloned().collect())
            .or_else(|| {
                self.enums
                    .get(name)
                    .map(|info| info.type_params.iter().cloned().collect())
            })
    }

    fn collected_behavior_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.behaviors
            .get(name)
            .map(|info| info.type_params.iter().cloned().collect())
    }

    fn validate_collected_struct_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.structs.get(name).cloned() else {
            return;
        };
        for (_, ty) in &info.fields {
            self.validate_generic_type_ref_bounds(ty, scoped, span);
        }
    }

    fn validate_collected_enum_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.enums.get(name).cloned() else {
            return;
        };
        for (_, payload) in &info.variants {
            if let Some(payload) = payload {
                self.validate_generic_type_ref_bounds(payload, scoped, span);
            }
        }
    }

    fn validate_collected_behavior_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.behaviors.get(name).cloned() else {
            return;
        };
        for method in &info.methods {
            for param in &method.params {
                self.validate_generic_type_ref_bounds(&param.ty, scoped, span);
            }
            if let Some(return_type) = &method.return_type {
                self.validate_generic_type_ref_bounds(return_type, scoped, span);
            }
        }
    }

    fn validate_collected_value_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let info = self
            .functions
            .get(name)
            .or_else(|| self.methods.get(name))
            .cloned();
        let Some(info) = info else {
            return;
        };

        for (_, ty) in &info.params {
            self.validate_generic_type_ref_bounds(ty, scoped, span);
        }
        self.validate_generic_type_ref_bounds(&info.return_type, scoped, span);
    }

    fn validate_generic_type_ref_bounds(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        self.validate_generic_type_ref_bounds_with_unknowns(
            ast_type,
            scoped_type_params,
            span,
            true,
        );
    }

    pub(super) fn validate_generic_type_arg_refs_allow_unknowns(
        &mut self,
        type_args: &[AstType],
        span: Span,
    ) {
        let scoped_type_params = HashSet::new();
        self.validate_generic_type_arg_refs_with_unknowns(
            type_args,
            &scoped_type_params,
            span,
            false,
        );
    }

    fn validate_generic_type_arg_refs(
        &mut self,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        self.validate_generic_type_arg_refs_with_unknowns(
            type_args,
            scoped_type_params,
            span,
            true,
        );
    }

    fn validate_generic_type_arg_refs_with_unknowns(
        &mut self,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        for type_arg in type_args {
            self.validate_generic_type_ref_bounds_with_unknowns(
                type_arg,
                scoped_type_params,
                span,
                reject_unknown,
            );
        }
    }

    fn validate_generic_type_ref_bounds_with_unknowns(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        match ast_type {
            AstType::Named(name) => {
                if scoped_type_params.contains(name) {
                    return;
                }

                if !self.is_known_named_type(name) {
                    if reject_unknown {
                        self.diagnostics.push(Diagnostic::error(
                            "E0201",
                            format!("unknown type symbol '{name}'"),
                            span,
                        ));
                    }
                    return;
                }

                let generic = self
                    .structs
                    .get(name)
                    .map(|info| ("struct", info.type_params.len()))
                    .or_else(|| {
                        self.enums
                            .get(name)
                            .map(|info| ("enum", info.type_params.len()))
                    });
                if let Some((kind, type_param_count)) = generic {
                    if type_param_count > 0 {
                        self.diagnostics.push(Diagnostic::error(
                            "E5001",
                            format!(
                                "generic {} `{}` expects {} type arguments, found 0",
                                kind, name, type_param_count
                            ),
                            span,
                        ));
                    }
                }
            }
            AstType::Generic { name, type_args } => {
                self.validate_generic_type_arg_refs_with_unknowns(
                    type_args,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );

                if scoped_type_params.contains(name) {
                    return;
                }

                let (kind, type_params, type_param_bounds) =
                    if let Some(info) = self.structs.get(name) {
                        (
                            "struct",
                            info.type_params.clone(),
                            info.type_param_bounds.clone(),
                        )
                    } else if let Some(info) = self.enums.get(name) {
                        (
                            "enum",
                            info.type_params.clone(),
                            info.type_param_bounds.clone(),
                        )
                    } else {
                        if reject_unknown && !self.imports.contains_key(name) {
                            self.diagnostics.push(Diagnostic::error(
                                "E0201",
                                format!("unknown type symbol '{name}'"),
                                span,
                            ));
                        }
                        return;
                    };

                if type_params.len() != type_args.len() {
                    self.diagnostics.push(Diagnostic::error(
                        "E5001",
                        format!(
                            "generic {} `{}` expects {} type arguments, found {}",
                            kind,
                            name,
                            type_params.len(),
                            type_args.len()
                        ),
                        span,
                    ));
                    return;
                }

                let substitutions: HashMap<String, Type> = type_params
                    .iter()
                    .zip(type_args.iter())
                    .filter_map(|(param, arg)| {
                        if ast_type_references_type_param(arg, scoped_type_params) {
                            None
                        } else {
                            Some((param.clone(), self.resolve_type(arg)))
                        }
                    })
                    .collect();
                self.check_generic_bounds(&type_param_bounds, &substitutions, span);
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => {
                self.validate_generic_type_ref_bounds_with_unknowns(
                    inner,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            AstType::Array { elem, .. } => {
                self.validate_generic_type_ref_bounds_with_unknowns(
                    elem,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            AstType::Function { params, ret } => {
                self.validate_generic_type_arg_refs_with_unknowns(
                    params,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
                self.validate_generic_type_ref_bounds_with_unknowns(
                    ret,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            _ => {}
        }
    }

    fn is_known_named_type(&self, name: &str) -> bool {
        self.structs.contains_key(name)
            || self.enums.contains_key(name)
            || self.imports.contains_key(name)
    }

    fn validate_generic_expr_type_references(
        &mut self,
        expr: &Expression,
        scoped_type_params: &HashSet<String>,
    ) {
        match expr {
            Expression::FunctionCall {
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
                for arg in args {
                    self.validate_generic_expr_type_references(arg, scoped_type_params);
                }
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_generic_expr_type_references(receiver, scoped_type_params);
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
                for arg in args {
                    self.validate_generic_expr_type_references(arg, scoped_type_params);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.validate_generic_expr_type_references(left, scoped_type_params);
                self.validate_generic_expr_type_references(right, scoped_type_params);
            }
            Expression::UnaryOp { operand, .. } => {
                self.validate_generic_expr_type_references(operand, scoped_type_params);
            }
            Expression::MemberAccess { object, .. } => {
                self.validate_generic_expr_type_references(object, scoped_type_params);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.validate_generic_expr_type_references(object, scoped_type_params);
                self.validate_generic_expr_type_references(index, scoped_type_params);
            }
            Expression::StructLiteral {
                type_args,
                fields,
                span,
                ..
            } => {
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
                for (_, value) in fields {
                    self.validate_generic_expr_type_references(value, scoped_type_params);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload,
                span,
                ..
            } => {
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
                if let Some(payload) = payload {
                    self.validate_generic_expr_type_references(payload, scoped_type_params);
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.validate_generic_expr_type_references(element, scoped_type_params);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_generic_expr_type_references(scrutinee, scoped_type_params);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.validate_generic_expr_type_references(guard, scoped_type_params);
                    }
                    self.validate_generic_expr_type_references(&arm.body, scoped_type_params);
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.validate_generic_expr_type_references(condition, scoped_type_params);
                self.validate_generic_expr_type_references(body, scoped_type_params);
            }
            Expression::Loop { body, .. } => {
                self.validate_generic_expr_type_references(body, scoped_type_params);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.validate_generic_expr_type_references(condition, scoped_type_params);
                self.validate_generic_expr_type_references(then_body, scoped_type_params);
                if let Some(else_body) = else_body {
                    self.validate_generic_expr_type_references(else_body, scoped_type_params);
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    self.validate_generic_statement_type_references(statement, scoped_type_params);
                }
                if let Some(expr) = expr {
                    self.validate_generic_expr_type_references(expr, scoped_type_params);
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.validate_generic_expr_type_references(value, scoped_type_params);
                }
            }
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => {
                for param in params {
                    self.validate_generic_type_ref_bounds(
                        &param.ty,
                        scoped_type_params,
                        param.span,
                    );
                }
                if let Some(return_type) = return_type {
                    self.validate_generic_type_ref_bounds(return_type, scoped_type_params, *span);
                }
                self.validate_generic_expr_type_references(body, scoped_type_params);
            }
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_generic_expr_type_references(expr, scoped_type_params);
                self.validate_generic_type_ref_bounds(target_type, scoped_type_params, *span);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let ast::StringPart::Expr(expr) = part {
                        self.validate_generic_expr_type_references(expr, scoped_type_params);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.validate_generic_expr_type_references(start, scoped_type_params);
                self.validate_generic_expr_type_references(end, scoped_type_params);
            }
            Expression::Defer { expr, .. } => {
                self.validate_generic_expr_type_references(expr, scoped_type_params);
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::LoopControl { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn validate_generic_statement_type_references(
        &mut self,
        statement: &ast::Statement,
        scoped_type_params: &HashSet<String>,
    ) {
        match statement {
            ast::Statement::VarDecl {
                ty, value, span, ..
            } => {
                if let Some(ty) = ty {
                    self.validate_generic_type_ref_bounds(ty, scoped_type_params, *span);
                }
                self.validate_generic_expr_type_references(value, scoped_type_params);
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.validate_generic_expr_type_references(target, scoped_type_params);
                self.validate_generic_expr_type_references(value, scoped_type_params);
            }
            ast::Statement::Expression { expr, .. } => {
                self.validate_generic_expr_type_references(expr, scoped_type_params);
            }
            ast::Statement::Block { stmts, .. } => {
                for statement in stmts {
                    self.validate_generic_statement_type_references(statement, scoped_type_params);
                }
            }
        }
    }
}
