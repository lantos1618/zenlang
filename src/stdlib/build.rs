use crate::ast::{AstType, Expression};
use super::{StdModuleTrait, StdFunction};
use std::collections::HashMap;

/// The @std.build module provides build system access and module importing
pub struct BuildModule {
    functions: HashMap<String, StdFunction>,
    types: HashMap<String, AstType>,
}

impl BuildModule {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        let mut types = HashMap::new();
        
        // Build system functions
        functions.insert("import".to_string(), StdFunction {
            name: "import".to_string(),
            params: vec![("module_name".to_string(), AstType::String)],
            return_type: AstType::Generic { name: "Module".to_string(), type_args: vec![] },
            is_builtin: true,
        });
        
        functions.insert("link".to_string(), StdFunction {
            name: "link".to_string(),
            params: vec![("library".to_string(), AstType::String)],
            return_type: AstType::Void,
            is_builtin: true,
        });
        
        functions.insert("export".to_string(), StdFunction {
            name: "export".to_string(),
            params: vec![
                ("name".to_string(), AstType::String),
                ("value".to_string(), AstType::Generic { name: "Any".to_string(), type_args: vec![] }),
            ],
            return_type: AstType::Void,
            is_builtin: true,
        });
        
        functions.insert("target".to_string(), StdFunction {
            name: "target".to_string(),
            params: vec![],
            return_type: AstType::String,
            is_builtin: true,
        });
        
        functions.insert("os".to_string(), StdFunction {
            name: "os".to_string(),
            params: vec![],
            return_type: AstType::String,
            is_builtin: true,
        });
        
        functions.insert("arch".to_string(), StdFunction {
            name: "arch".to_string(),
            params: vec![],
            return_type: AstType::String,
            is_builtin: true,
        });
        
        // Build-related types
        // Build-related types
        types.insert("Module".to_string(), AstType::Generic { name: "Module".to_string(), type_args: vec![] });
        types.insert("Target".to_string(), AstType::String);
        
        BuildModule { functions, types }
    }
    
    /// Import a module by name
    pub fn import_module(name: &str) -> Result<Expression, String> {
        // This would integrate with the module system to load and compile modules
        // For now, return a placeholder
        Ok(Expression::Identifier(format!("module_{}", name)))
    }
}

impl StdModuleTrait for BuildModule {
    fn name(&self) -> &str {
        "build"
    }
    
    fn get_function(&self, name: &str) -> Option<StdFunction> {
        self.functions.get(name).cloned()
    }
    
    fn get_type(&self, name: &str) -> Option<AstType> {
        self.types.get(name).cloned()
    }
}