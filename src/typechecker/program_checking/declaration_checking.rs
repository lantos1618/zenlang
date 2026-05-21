use super::super::program_globals::push_typed_global;
use super::*;

impl TypeChecker {
    pub(super) fn check_program_declaration_after_collection(
        &mut self,
        decl: &Declaration,
        functions: &mut Vec<TypedFunction>,
        types: &mut Vec<TypedTypeDef>,
        globals: &mut Vec<TypedGlobal>,
        entry_point: &mut Option<String>,
    ) {
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
                if type_params.is_empty() {
                    if name == "main" {
                        *entry_point = Some(name.clone());
                    }
                    match self.check_function(name, params, return_type, body, span) {
                        Ok(func) => functions.push(func),
                        Err(d) => self.diagnostics.push(d),
                    }
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
                if type_params.is_empty() {
                    let full_name = Self::method_key(type_name, method_name);
                    self.current_self_type =
                        Some(self.resolve_type(&AstType::Named(type_name.clone())));
                    match self.check_function(&full_name, params, return_type, body, span) {
                        Ok(func) => functions.push(func),
                        Err(d) => self.diagnostics.push(d),
                    }
                    self.current_self_type = None;
                }
            }
            Declaration::Struct {
                name,
                type_params,
                fields,
                span,
                ..
            } => {
                if type_params.is_empty() {
                    self.push_non_generic_struct_type(types, name, fields, span);
                }
            }
            Declaration::Enum {
                name,
                type_params,
                variants,
                span,
                ..
            } => {
                if type_params.is_empty() {
                    self.push_non_generic_enum_type(types, name, variants, span);
                }
            }
            Declaration::TopLevelExpr { expr, span } => match self.check_expr(expr) {
                Ok(typed_expr) => push_typed_global(globals, typed_expr, *span),
                Err(d) => self.diagnostics.push(d),
            },
            Declaration::ImplBlock {
                type_name,
                behavior,
                behavior_type_args,
                methods,
                ..
            } => {
                self.check_impl_block_after_collection(
                    functions,
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                );
            }
            Declaration::Import { .. } | Declaration::Behavior { .. } => {}
            _ => {}
        }
    }

    fn check_impl_block_after_collection(
        &mut self,
        functions: &mut Vec<TypedFunction>,
        type_name: &str,
        behavior: &Option<String>,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) {
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
                self.current_self_type = Some(self.resolve_type(&AstType::Named(type_name.into())));
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
                self.current_self_type = Some(self.resolve_type(&AstType::Named(type_name.into())));
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
}
