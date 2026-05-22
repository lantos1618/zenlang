use super::program_globals::push_typed_global;
use super::*;

impl TypeChecker {
    pub fn check_program(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        // Phase 1: Collect type definitions and function signatures
        self.collect_declarations(&program.declarations);
        self.validate_collected_declaration_semantics(&program.declarations, None);
        self.check_program_after_collection(program)
    }

    pub(super) fn check_program_after_collection(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        // Phase 2: Check function bodies and produce typed AST
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut globals = Vec::new();
        let mut entry_point = None;

        for decl in &program.declarations {
            match decl {
                Declaration::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
                    if name == "main" {
                        entry_point = Some(name.clone());
                    }
                    match self.check_function(name, params, return_type, body, span) {
                        Ok(func) => functions.push(func),
                        Err(d) => self.diagnostics.push(d),
                    }
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
                    let full_name = Self::method_key(type_name, method_name);
                    // Set Self type for method body
                    self.current_self_type =
                        Some(self.resolve_type(&AstType::Named(type_name.clone())));
                    match self.check_function(&full_name, params, return_type, body, span) {
                        Ok(func) => functions.push(func),
                        Err(d) => self.diagnostics.push(d),
                    }
                    self.current_self_type = None;
                }
                Declaration::Struct {
                    name,
                    type_params,
                    fields,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
                    types.push(self.typed_struct_def(name, fields, *span));
                }
                Declaration::Enum {
                    name,
                    type_params,
                    variants,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
                    types.push(self.typed_enum_def(name, variants, *span));
                }
                Declaration::TopLevelExpr { expr, span } => {
                    // Top-level expressions like main() call
                    match self.check_expr(expr) {
                        Ok(typed_expr) => {
                            push_typed_global(&mut globals, typed_expr, *span);
                        }
                        Err(d) => self.diagnostics.push(d),
                    }
                }
                Declaration::Import { .. } => {
                    // Imports are handled by the module system, not the typechecker
                }
                Declaration::Behavior { .. } => {}
                Declaration::ImplBlock {
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                    ..
                } => {
                    for method in methods {
                        if let Declaration::Function {
                            name,
                            type_params,
                            params,
                            return_type,
                            body,
                            span,
                            ..
                        } = method
                        {
                            if !type_params.is_empty() {
                                continue;
                            }
                            let full_name = Self::behavior_impl_method_key(
                                type_name,
                                name,
                                behavior.as_deref(),
                                behavior_type_args,
                            );
                            self.current_self_type =
                                Some(self.resolve_type(&AstType::Named(type_name.clone())));
                            match self.check_function(&full_name, params, return_type, body, span) {
                                Ok(func) => functions.push(func),
                                Err(d) => self.diagnostics.push(d),
                            }
                            self.current_self_type = None;
                        }
                    }

                    if let Some(behavior) = behavior {
                        for default in self.behavior_default_methods_for_impl(
                            type_name,
                            behavior,
                            behavior_type_args,
                            methods,
                        ) {
                            let full_name = Self::method_key(type_name, &default.name);
                            self.current_self_type =
                                Some(self.resolve_type(&AstType::Named(type_name.clone())));
                            match self.check_function(
                                &full_name,
                                &default.params,
                                &default.return_type,
                                &default.body,
                                &default.span,
                            ) {
                                Ok(func) => functions.push(func),
                                Err(d) => self.diagnostics.push(d),
                            }
                            self.current_self_type = None;
                        }
                    }
                }
                _ => {}
            }
        }

        let errors: Vec<_> = self
            .diagnostics
            .iter()
            .filter(|d| d.is_error())
            .cloned()
            .collect();
        if !errors.is_empty() {
            return Err(errors);
        }

        functions.append(&mut self.specialized_functions);
        types.append(&mut self.specialized_types);

        Ok(TypedProgram {
            functions,
            types,
            globals,
            entry_point,
        })
    }

    pub fn check_program_with_symbols(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        self.validate_resolver_symbols(program, symbols);
        if self.diagnostics.iter().any(|diag| diag.is_error()) {
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        }
        self.collect_resolver_imports(symbols);
        self.collect_declarations_with_symbols(&program.declarations, symbols);
        self.check_program_after_collection(program)
    }

    /// Get all diagnostics (errors + warnings).
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
