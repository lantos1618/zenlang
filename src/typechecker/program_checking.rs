use super::*;

impl TypeChecker {
    pub fn check_program(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        self.collect_declarations(&program.declarations);
        self.validate_collected_declaration_semantics(&program.declarations);
        // Collect module-level bindings into a persistent global scope before any
        // function body is checked, so bodies can reference them.
        self.enter_global_scope();
        let globals = self.register_global_bindings(&program.declarations);
        let mut functions = Vec::new();
        let mut extern_functions = Vec::new();
        let mut extern_types = Vec::new();
        let mut types = Vec::new();
        let mut entry_point = None;

        for decl in &program.declarations {
            if let Declaration::ExternType { name, .. } = decl {
                extern_types.push(name.clone());
                continue;
            }
            // `extern` C functions have no Zen body to check or emit; record a
            // prototype for codegen and skip the normal callable path.
            if let Declaration::Function {
                external: true,
                name,
                params,
                return_type,
                ..
            } = decl
            {
                extern_functions.push(TypedExternFunction {
                    name: name.clone(),
                    params: params
                        .iter()
                        .map(|param| TypedParam {
                            name: param.name.clone(),
                            ty: self.resolve_type(&param.ty),
                            span: param.span,
                        })
                        .collect(),
                    return_type: return_type
                        .as_ref()
                        .map(|ty| self.resolve_type(ty))
                        .unwrap_or(Type::Void),
                });
                continue;
            }

            if let Some(callable) = decl.as_callable() {
                if callable.type_params.is_empty() {
                    let (checked_name, self_type) = match decl {
                        Declaration::Method { type_name, .. } => (
                            method_signature_key(type_name, callable.name),
                            Some(self.resolve_type(&AstType::Named(type_name.clone()))),
                        ),
                        _ => {
                            if callable.name == "main" {
                                entry_point = Some(callable.name.to_string());
                            }
                            (callable.name.to_string(), None)
                        }
                    };
                    self.push_checked_function_decl(&mut functions, decl, &checked_name, self_type);

                    // Surface + typing for `@async`/`@await` ship in milestone 1,
                    // but state-machine lowering does not yet. Type-check the body
                    // (so `@await` rules fire) and then reject the program with a
                    // clear, stable-coded error rather than emit broken C. The
                    // half-built async function is dropped so codegen never sees a
                    // `Future`/`Await` node. (ASYNC_PLAN.md milestone 1.)
                    if callable.is_async {
                        functions.retain(|f| f.name != checked_name);
                        self.diagnostics.push(Diagnostic::error_code(
                            E3082,
                            format!(
                                "`@async` function `{}` cannot be compiled yet: \
                                 async/await lowering is not implemented (milestone 1 \
                                 in progress)",
                                callable.name
                            ),
                            callable.span,
                        ));
                    }
                }
                continue;
            }

            match decl {
                Declaration::Struct {
                    name,
                    type_params,
                    fields,
                    span,
                    ..
                } if type_params.is_empty() => {
                    let fields = fields
                        .iter()
                        .map(|field| (field.name.clone(), self.resolve_type(&field.ty)))
                        .collect();
                    types.push(TypedTypeDef {
                        name: name.to_string(),
                        kind: TypeDefKind::Struct { fields },
                        span: *span,
                    });
                }
                Declaration::Enum {
                    name,
                    type_params,
                    variants,
                    span,
                    ..
                } if type_params.is_empty() => {
                    let variants = variants
                        .iter()
                        .enumerate()
                        .map(|(index, variant)| TypedVariant {
                            name: variant.name.clone(),
                            tag: index as u32,
                            payload: variant.payload.as_ref().map(|ty| self.resolve_type(ty)),
                        })
                        .collect();
                    types.push(TypedTypeDef {
                        name: name.to_string(),
                        kind: TypeDefKind::Enum { variants },
                        span: *span,
                    });
                }
                // Top-level bindings were already handled by register_global_bindings.
                Declaration::TopLevelExpr { .. } => {}
                Declaration::ImplBlock {
                    type_name,
                    type_args,
                    behavior,
                    behavior_type_args,
                    methods,
                    ..
                } => {
                    let self_type =
                        self.resolve_type(&concrete_self_target_type(type_name, type_args));
                    for method in methods {
                        if let Some(method_decl) = method.as_callable() {
                            let full_name = behavior_impl_method_signature_key_with_target_args(
                                type_name,
                                method_decl.name,
                                behavior.as_deref(),
                                behavior_type_args,
                                type_args,
                            );
                            self.push_checked_function_decl(
                                &mut functions,
                                method,
                                &full_name,
                                Some(self_type.clone()),
                            );
                        }
                    }

                    if let Some(behavior) = behavior {
                        for default in self.behavior_default_methods_for_impl(
                            type_name,
                            type_args,
                            behavior,
                            behavior_type_args,
                            methods,
                        ) {
                            if let Some(default_decl) = default.as_callable() {
                                let full_name = method_signature_key(type_name, default_decl.name);
                                self.push_checked_function_decl(
                                    &mut functions,
                                    &default,
                                    &full_name,
                                    Some(self_type.clone()),
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        self.fail_if_errors()?;

        functions.append(&mut self.specialized_functions);
        types.append(&mut self.specialized_types);

        Ok(TypedProgram {
            functions,
            types,
            globals,
            extern_functions,
            extern_types,
            entry_point,
        })
    }

    /// Check each top-level `name = value` / `name := value` binding, register
    /// its name in the global scope (so function bodies can reference it), and
    /// collect it as a `TypedGlobal` for codegen.
    fn register_global_bindings(&mut self, decls: &[Declaration]) -> Vec<TypedGlobal> {
        let mut globals = Vec::new();
        for decl in decls {
            let Declaration::TopLevelExpr { expr, span } = decl else {
                continue;
            };
            match self.check_expr(expr) {
                Ok(typed_expr) => {
                    if let TypedExprKind::Block(block) = &typed_expr.kind {
                        if block.expr.is_none() && block.statements.len() == 1 {
                            if let TypedStatementKind::VarDecl {
                                name,
                                ty,
                                value,
                                mutable,
                            } = &block.statements[0].kind
                            {
                                self.define_var_with_mutability(name, ty.clone(), *mutable);
                                globals.push(TypedGlobal {
                                    name: name.clone(),
                                    ty: ty.clone(),
                                    value: value.clone(),
                                    mutable: *mutable,
                                    span: *span,
                                });
                                continue;
                            }
                        }
                    }
                    globals.push(TypedGlobal {
                        name: "__top_level__".into(),
                        ty: typed_expr.ty.clone(),
                        value: typed_expr,
                        mutable: false,
                        span: *span,
                    });
                }
                Err(d) => self.diagnostics.push(d),
            }
        }
        globals
    }

    fn push_checked_function_decl(
        &mut self,
        functions: &mut Vec<TypedFunction>,
        decl: &Declaration,
        checked_name: &str,
        self_type: Option<Type>,
    ) {
        let Some(callable) = decl.as_callable() else {
            return;
        };
        if !callable.type_params.is_empty() {
            return;
        }
        let saved_self_type = self_type.map(|self_type| self.current_self_type.replace(self_type));
        match self.check_function(
            checked_name,
            callable.params,
            callable.return_type,
            callable.body,
            callable.is_async,
            &callable.span,
        ) {
            Ok(function) => functions.push(function),
            Err(diagnostic) => self.diagnostics.push(diagnostic),
        }
        if let Some(saved_self_type) = saved_self_type {
            self.current_self_type = saved_self_type;
        }
    }
}
