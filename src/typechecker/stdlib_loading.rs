use crate::ast::{strip_generic_params, AstType, Declaration, Program};
use crate::error::Result;
use crate::typechecker::{EnumInfo, FunctionSignature, MethodSignature, StructInfo, TypeChecker};
use std::collections::HashMap;

/// Stdlib module loading and type extraction
impl TypeChecker {
    pub fn register_stdlib_module(&mut self, _alias: &str, _module_path: &str) -> Result<()> {
        // Stdlib modules are now loaded via with_stdlib_modules() from ModuleSystem
        // Types are extracted automatically when modules are loaded
        Ok(())
    }

    /// Initialize TypeChecker with already-loaded stdlib modules from ModuleSystem
    /// Extracts type information from the loaded modules
    pub fn with_stdlib_modules(&mut self, modules: &HashMap<String, Program>) {
        for (path, program) in modules {
            if path.starts_with("@std") || path.starts_with("std.") {
                self.extract_types_from_program(program, path);
                self.stdlib_modules.insert(path.clone(), program.clone());
            }
        }
    }

    /// Extract type information from a stdlib program
    pub fn extract_types_from_program(&mut self, program: &Program, module_path: &str) {
        for decl in &program.declarations {
            match decl {
                Declaration::Struct(def) => {
                    self.type_store
                        .borrow_mut()
                        .register_struct(&def.name, StructInfo::from(def));
                }
                Declaration::Function(func) => {
                    if let Some((receiver, method)) = func.name.split_once('.') {
                        // Method: Type.method
                        // Extract base type name (strip generic params: "Vec<T>" -> "Vec")
                        let base_receiver = strip_generic_params(receiver);
                        let sig = MethodSignature {
                            receiver_type: base_receiver.to_string(),
                            method_name: method.to_string(),
                            params: func.args.clone(),
                            return_type: func.return_type.clone(),
                            is_static: func
                                .args
                                .first()
                                .map(|(name, _)| name != "self")
                                .unwrap_or(true),
                        };
                        self.type_store
                            .borrow_mut()
                            .register_method(base_receiver, method, sig);
                    } else {
                        // Standalone function
                        let sig = FunctionSignature {
                            params: func.args.clone(),
                            return_type: func.return_type.clone(),
                            is_external: false,
                        };
                        self.type_store.borrow_mut().register_stdlib_function(
                            module_path,
                            &func.name,
                            sig,
                        );
                    }
                }
                Declaration::Enum(def) => {
                    self.type_store
                        .borrow_mut()
                        .register_enum(&def.name, EnumInfo::from(def));
                }
                Declaration::Trait(trait_def) => {
                    // Register the trait in behavior resolver so it can be used in trait implementations
                    if let Err(e) = self.behavior_resolver.register_trait(trait_def) {
                        eprintln!(
                            "Warning: Failed to register stdlib trait '{}': {:?}",
                            trait_def.name, e
                        );
                    }
                }
                Declaration::TraitImplementation(trait_impl) => {
                    for method in &trait_impl.methods {
                        let sig = MethodSignature {
                            receiver_type: trait_impl.type_name.clone(),
                            method_name: method.name.clone(),
                            params: method.args.clone(),
                            return_type: method.return_type.clone(),
                            is_static: method
                                .args
                                .first()
                                .map(|(n, _)| n != "self")
                                .unwrap_or(true),
                        };
                        self.type_store.borrow_mut().register_method(
                            &trait_impl.type_name,
                            &method.name,
                            sig,
                        );
                    }
                }
                Declaration::TypeAlias(type_alias) => {
                    // Handle struct type aliases
                    if let AstType::Struct { name: _, fields } = &type_alias.target_type {
                        self.type_store
                            .borrow_mut()
                            .register_struct(&type_alias.name, StructInfo::new(fields.clone()));
                    } else if matches!(
                        &type_alias.target_type,
                        AstType::Function { .. } | AstType::FunctionPointer { .. }
                    ) {
                        // Handle function type aliases (e.g., CompletionFn)
                        self.type_store
                            .borrow_mut()
                            .register_type_alias(&type_alias.name, type_alias.target_type.clone());
                    }
                }
                _ => {}
            }
        }
    }

    /// Look up stdlib method return type (replaces stdlib_types().get_method_return_type)
    pub fn get_stdlib_method_type(&self, receiver: &str, method: &str) -> Option<AstType> {
        self.type_store
            .borrow()
            .get_method(receiver, method)
            .map(|sig| sig.return_type.clone())
    }

    /// Look up stdlib function return type (replaces stdlib_types().get_function_return_type)
    ///
    /// Handles module alias resolution: user writes `io.println()` where `io` is a short alias,
    /// but the function is stored under the full module path `@std.io::println`.
    pub fn get_stdlib_function_type(&self, module: &str, func_name: &str) -> Option<AstType> {
        // Fast path: exact key match (e.g., module is already a full path)
        if let Some(sig) = self
            .type_store
            .borrow()
            .get_stdlib_function(module, func_name)
        {
            return Some(sig.return_type.clone());
        }

        // Alias resolution: "io" → "@std.io", "math" → "@std.math", etc.
        let std_module = format!("@std.{}", module);
        if let Some(sig) = self
            .type_store
            .borrow()
            .get_stdlib_function(&std_module, func_name)
        {
            return Some(sig.return_type.clone());
        }

        None
    }

    /// Get stdlib struct definition (replaces stdlib_types().get_struct_definition)
    pub fn get_stdlib_struct(&self, name: &str) -> Option<StructInfo> {
        self.type_store.borrow().get_struct(name).cloned()
    }
}
