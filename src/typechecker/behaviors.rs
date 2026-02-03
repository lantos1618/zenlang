use crate::ast::{
    AstType, BehaviorDefinition, TraitDefinition, TraitImplementation, TraitRequirement,
};
use crate::error::{CompileError, Result};
use std::collections::HashMap;

/// Tracks behaviors, implementations, and provides trait resolution
pub struct BehaviorResolver {
    /// All defined behaviors/traits
    behaviors: HashMap<String, BehaviorInfo>,
    /// Maps (type_name, behavior_name) -> implementation
    implementations: HashMap<(String, String), ImplInfo>,
    /// Maps type_name -> inherent methods (impl blocks without behavior)
    pub inherent_methods: HashMap<String, Vec<MethodInfo>>,
}

#[derive(Clone, Debug)]
pub struct BehaviorInfo {
    pub name: String,
    pub type_params: Vec<String>,
    pub methods: Vec<BehaviorMethodInfo>,
}

#[derive(Clone, Debug)]
pub struct BehaviorMethodInfo {
    pub name: String,
    pub param_types: Vec<AstType>,
    pub return_type: AstType,
    pub has_self: bool,
}

#[derive(Clone, Debug)]
pub struct ImplInfo {
    pub type_name: String,
    pub trait_name: String,
    pub type_params: Vec<String>,
    pub methods: HashMap<String, MethodInfo>,
}

#[derive(Clone, Debug)]
pub struct MethodInfo {
    pub name: String,
    pub param_types: Vec<AstType>,
    pub return_type: AstType,
}

impl Default for BehaviorResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorResolver {
    pub fn new() -> Self {
        // No longer pre-registering Allocator trait here.
        // Traits are now properly loaded from stdlib via stdlib_loading.rs
        Self {
            behaviors: HashMap::new(),
            implementations: HashMap::new(),
            inherent_methods: HashMap::new(),
        }
    }

    /// Register a behavior definition
    pub fn register_behavior(&mut self, behavior: &BehaviorDefinition) -> Result<()> {
        if self.behaviors.contains_key(&behavior.name) {
            return Err(CompileError::TypeError(
                format!("Behavior '{}' already defined", behavior.name),
                None,
            ));
        }

        let methods = behavior
            .methods
            .iter()
            .map(|m| {
                let has_self = m.params.first().map(|p| p.name == "self").unwrap_or(false);

                BehaviorMethodInfo {
                    name: m.name.clone(),
                    param_types: m.params.iter().map(|p| p.type_.clone()).collect(),
                    return_type: m.return_type.clone(),
                    has_self,
                }
            })
            .collect();

        let info = BehaviorInfo {
            name: behavior.name.clone(),
            type_params: behavior
                .type_params
                .iter()
                .map(|tp| tp.name.clone())
                .collect(),
            methods,
        };

        self.behaviors.insert(behavior.name.clone(), info);
        Ok(())
    }

    /// Register a trait definition (using same storage as behaviors)
    pub fn register_trait(&mut self, trait_def: &TraitDefinition) -> Result<()> {
        let methods: Vec<BehaviorMethodInfo> = trait_def
            .methods
            .iter()
            .map(|m| {
                let has_self = m.params.first().map(|p| p.name == "self").unwrap_or(false);
                BehaviorMethodInfo {
                    name: m.name.clone(),
                    param_types: m.params.iter().map(|p| p.type_.clone()).collect(),
                    return_type: m.return_type.clone(),
                    has_self,
                }
            })
            .collect();

        if let Some(existing) = self.behaviors.get(&trait_def.name) {
            let methods_match = existing.methods.len() == methods.len()
                && existing
                    .methods
                    .iter()
                    .zip(methods.iter())
                    .all(|(a, b)| a.name == b.name);
            if methods_match {
                return Ok(());
            }
            return Err(CompileError::TypeError(
                format!(
                    "Trait '{}' already defined with different signature",
                    trait_def.name
                ),
                trait_def.span.clone(),
            ));
        }

        let info = BehaviorInfo {
            name: trait_def.name.clone(),
            type_params: trait_def
                .type_params
                .iter()
                .map(|tp| tp.name.clone())
                .collect(),
            methods,
        };

        self.behaviors.insert(trait_def.name.clone(), info);
        Ok(())
    }

    /// Replace Self type with concrete type in AST type
    #[allow(clippy::only_used_in_recursion)]
    fn replace_self_type(&self, ast_type: &AstType, concrete_type: &str) -> AstType {
        match ast_type {
            AstType::Generic { name, type_args: _ } if name == "Self" => {
                // Replace Self with the concrete type - use Struct with just name
                AstType::Struct {
                    name: concrete_type.to_string(),
                    fields: vec![], // Empty fields - this is just a type reference
                }
            }
            t if t.is_immutable_ptr() => {
                if let Some(inner) = t.ptr_inner() {
                    AstType::ptr(self.replace_self_type(inner, concrete_type))
                } else {
                    ast_type.clone()
                }
            }
            t if t.is_mutable_ptr() => {
                if let Some(inner) = t.ptr_inner() {
                    AstType::mut_ptr(self.replace_self_type(inner, concrete_type))
                } else {
                    ast_type.clone()
                }
            }
            t if t.is_raw_ptr() => {
                if let Some(inner) = t.ptr_inner() {
                    AstType::raw_ptr(self.replace_self_type(inner, concrete_type))
                } else {
                    ast_type.clone()
                }
            }
            // Option and Result are now Generic types - handled in Generic match above
            AstType::Slice(element) => {
                AstType::Slice(Box::new(self.replace_self_type(element, concrete_type)))
            }
            AstType::FixedArray { element_type, size } => AstType::FixedArray {
                element_type: Box::new(self.replace_self_type(element_type, concrete_type)),
                size: *size,
            },
            AstType::Function { args, return_type } => AstType::Function {
                args: args
                    .iter()
                    .map(|p| self.replace_self_type(p, concrete_type))
                    .collect(),
                return_type: Box::new(self.replace_self_type(return_type, concrete_type)),
            },
            // For other types, return as is
            _ => ast_type.clone(),
        }
    }

    /// Register an implementation block
    pub fn register_trait_implementation(
        &mut self,
        trait_impl: &TraitImplementation,
    ) -> Result<()> {
        // Register a trait implementation for a type
        let key = (trait_impl.type_name.clone(), trait_impl.trait_name.clone());

        if self.implementations.contains_key(&key) {
            return Err(CompileError::TypeError(
                format!(
                    "Trait '{}' already implemented for type '{}'",
                    trait_impl.trait_name, trait_impl.type_name
                ),
                None,
            ));
        }

        // Verify that the trait exists
        if !self.behaviors.contains_key(&trait_impl.trait_name) {
            return Err(CompileError::TypeError(
                format!("Unknown trait: {}", trait_impl.trait_name),
                None,
            ));
        }

        let mut methods = HashMap::new();
        for method in &trait_impl.methods {
            // Replace Self types in parameters and return type with the concrete type
            let param_types: Vec<AstType> = method
                .args
                .iter()
                .map(|(_, t)| self.replace_self_type(t, &trait_impl.type_name))
                .collect();
            let return_type = self.replace_self_type(&method.return_type, &trait_impl.type_name);

            let method_info = MethodInfo {
                name: method.name.clone(),
                param_types,
                return_type,
            };
            methods.insert(method.name.clone(), method_info);
        }

        let impl_info = ImplInfo {
            type_name: trait_impl.type_name.clone(),
            trait_name: trait_impl.trait_name.clone(),
            type_params: trait_impl
                .type_params
                .iter()
                .map(|p| p.name.clone())
                .collect(),
            methods,
        };

        self.implementations.insert(key, impl_info);
        Ok(())
    }

    pub fn register_trait_requirement(&mut self, trait_req: &TraitRequirement) -> Result<()> {
        // Register that a type requires a trait
        // This is mainly for enum variants that must all implement a trait
        // For now, we just verify the trait exists
        if !self.behaviors.contains_key(&trait_req.trait_name) {
            return Err(CompileError::TypeError(
                format!("Unknown trait: {}", trait_req.trait_name),
                None,
            ));
        }
        // Requirements stored in trait; verification happens during enum variant checking
        Ok(())
    }

    pub fn verify_trait_implementation(&mut self, trait_impl: &TraitImplementation) -> Result<()> {
        // Verify that the implementation satisfies the trait
        if let Some(behavior) = self.behaviors.get(&trait_impl.trait_name) {
            // Check that all required methods are implemented with correct signatures
            for required_method in &behavior.methods {
                let mut found = false;
                for impl_method in &trait_impl.methods {
                    if impl_method.name == required_method.name {
                        // Check that the method signatures match
                        self.check_method_signature(
                            &trait_impl.type_name,
                            &trait_impl.trait_name,
                            required_method,
                            impl_method,
                        )?;
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(CompileError::TypeError(
                        format!(
                            "Missing required method '{}' in implementation of trait '{}' for type '{}'",
                            required_method.name, trait_impl.trait_name, trait_impl.type_name
                        ),
                        None,
                    ));
                }
            }
            Ok(())
        } else {
            Err(CompileError::TypeError(
                format!("Unknown trait: {}", trait_impl.trait_name),
                None,
            ))
        }
    }

    /// Check that an implementation method signature matches the trait method signature
    fn check_method_signature(
        &self,
        type_name: &str,
        trait_name: &str,
        required: &BehaviorMethodInfo,
        impl_method: &crate::ast::Function,
    ) -> Result<()> {
        // Check return type matches (allowing Self -> concrete type substitution)
        if !self.types_compatible(&required.return_type, &impl_method.return_type, type_name) {
            return Err(CompileError::TypeError(
                format!(
                    "Method '{}' in implementation of trait '{}' for type '{}' has wrong return type: expected {:?}, got {:?}",
                    required.name, trait_name, type_name, required.return_type, impl_method.return_type
                ),
                None,
            ));
        }

        // Check parameter count (excluding self from required if has_self)
        let required_param_count = if required.has_self {
            required.param_types.len() - 1
        } else {
            required.param_types.len()
        };

        let impl_param_count = if impl_method
            .args
            .first()
            .map(|(n, _)| n == "self")
            .unwrap_or(false)
        {
            impl_method.args.len() - 1
        } else {
            impl_method.args.len()
        };

        if required_param_count != impl_param_count {
            return Err(CompileError::TypeError(
                format!(
                    "Method '{}' in implementation of trait '{}' for type '{}' has wrong number of parameters: expected {}, got {}",
                    required.name, trait_name, type_name, required_param_count, impl_param_count
                ),
                None,
            ));
        }

        // Check parameter types match
        let required_params: Vec<_> = if required.has_self {
            required.param_types.iter().skip(1).collect()
        } else {
            required.param_types.iter().collect()
        };

        let impl_params: Vec<_> = if impl_method
            .args
            .first()
            .map(|(n, _)| n == "self")
            .unwrap_or(false)
        {
            impl_method.args.iter().skip(1).collect()
        } else {
            impl_method.args.iter().collect()
        };

        for (i, (req_type, (_, impl_type))) in
            required_params.iter().zip(impl_params.iter()).enumerate()
        {
            if !self.types_compatible(req_type, impl_type, type_name) {
                return Err(CompileError::TypeError(
                    format!(
                        "Method '{}' in implementation of trait '{}' for type '{}' has wrong type for parameter {}: expected {:?}, got {:?}",
                        required.name, trait_name, type_name, i + 1, req_type, impl_type
                    ),
                    None,
                ));
            }
        }

        Ok(())
    }

    /// Check if two types are compatible, allowing Self -> concrete type substitution
    fn types_compatible(&self, expected: &AstType, actual: &AstType, self_type: &str) -> bool {
        // Handle Self substitution
        if let AstType::Generic { name, type_args } = expected {
            if name == "Self" && type_args.is_empty() {
                // Self should match the implementing type
                if let AstType::Generic {
                    name: actual_name,
                    type_args: actual_args,
                } = actual
                {
                    return actual_name == self_type && actual_args.is_empty();
                }
                // Also allow direct type name match
                return match actual {
                    AstType::Struct { name, .. } => name == self_type,
                    _ => false,
                };
            }
        }

        // Direct type equality check
        expected == actual
    }

    pub fn verify_trait_requirement(&mut self, trait_req: &TraitRequirement) -> Result<()> {
        // Verify that a trait requirement is valid
        if !self.behaviors.contains_key(&trait_req.trait_name) {
            return Err(CompileError::TypeError(
                format!("Unknown trait: {}", trait_req.trait_name),
                None,
            ));
        }
        Ok(())
    }

    /// Resolve a method call on a type
    pub fn resolve_method(&self, type_name: &str, method_name: &str) -> Option<MethodInfo> {
        // First check inherent methods
        if let Some(methods) = self.inherent_methods.get(type_name) {
            if let Some(method) = methods.iter().find(|m| m.name == method_name) {
                return Some(method.clone());
            }
        }

        // Then check behavior implementations
        for ((impl_type, _), impl_info) in &self.implementations {
            if impl_type == type_name {
                if let Some(method) = impl_info.methods.get(method_name) {
                    return Some(method.clone());
                }
            }
        }

        None
    }

    /// Get all implementations for building TypeContext
    pub fn implementations(&self) -> &HashMap<(String, String), ImplInfo> {
        &self.implementations
    }
}
