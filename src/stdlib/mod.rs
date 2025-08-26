use crate::ast::{Expression, AstType};
use std::collections::HashMap;

pub mod core;
pub mod build;

/// The @std namespace provides built-in compiler intrinsics and standard library access
pub struct StdNamespace {
    modules: HashMap<String, StdModule>,
}

pub enum StdModule {
    Core(core::CoreModule),
    Build(build::BuildModule),
}

impl StdNamespace {
    pub fn new() -> Self {
        let mut modules = HashMap::new();
        
        modules.insert("core".to_string(), StdModule::Core(core::CoreModule::new()));
        modules.insert("build".to_string(), StdModule::Build(build::BuildModule::new()));
        
        StdNamespace { modules }
    }
    
    pub fn get_module(&self, name: &str) -> Option<&StdModule> {
        self.modules.get(name)
    }
    
    /// Check if an identifier refers to @std namespace
    pub fn is_std_reference(name: &str) -> bool {
        name == "@std"
    }
    
    /// Resolve @std.module access
    pub fn resolve_std_access(module_name: &str) -> Option<Expression> {
        match module_name {
            "core" => Some(Expression::StdModule("core".to_string())),
            "build" => Some(Expression::StdModule("build".to_string())),
            _ => None,
        }
    }
}

/// Trait for standard library modules
pub trait StdModuleTrait {
    fn name(&self) -> &str;
    fn get_function(&self, name: &str) -> Option<StdFunction>;
    fn get_type(&self, name: &str) -> Option<AstType>;
}

#[derive(Clone)]
pub struct StdFunction {
    pub name: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub is_builtin: bool,
}