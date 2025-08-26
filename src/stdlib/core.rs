use crate::ast::AstType;
use super::{StdModuleTrait, StdFunction};
use std::collections::HashMap;

/// The @std.core module provides compiler intrinsics and core functionality
pub struct CoreModule {
    functions: HashMap<String, StdFunction>,
    types: HashMap<String, AstType>,
}

impl CoreModule {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        let mut types = HashMap::new();
        
        // Core intrinsic functions
        functions.insert("size_of".to_string(), StdFunction {
            name: "size_of".to_string(),
            params: vec![("T".to_string(), AstType::Generic { name: "T".to_string(), type_args: vec![] })],
            return_type: AstType::U64,
            is_builtin: true,
        });
        
        functions.insert("align_of".to_string(), StdFunction {
            name: "align_of".to_string(),
            params: vec![("T".to_string(), AstType::Generic { name: "T".to_string(), type_args: vec![] })],
            return_type: AstType::U64,
            is_builtin: true,
        });
        
        functions.insert("type_name".to_string(), StdFunction {
            name: "type_name".to_string(),
            params: vec![("T".to_string(), AstType::Generic { name: "T".to_string(), type_args: vec![] })],
            return_type: AstType::String,
            is_builtin: true,
        });
        
        functions.insert("panic".to_string(), StdFunction {
            name: "panic".to_string(),
            params: vec![("message".to_string(), AstType::String)],
            return_type: AstType::Void,
            is_builtin: true,
        });
        
        functions.insert("assert".to_string(), StdFunction {
            name: "assert".to_string(),
            params: vec![("condition".to_string(), AstType::Bool)],
            return_type: AstType::Void,
            is_builtin: true,
        });
        
        // Core types
        // Core types - for now using placeholder generic types
        types.insert("type".to_string(), AstType::Generic { name: "type".to_string(), type_args: vec![] });
        types.insert("Any".to_string(), AstType::Generic { name: "Any".to_string(), type_args: vec![] });
        
        CoreModule { functions, types }
    }
}

impl StdModuleTrait for CoreModule {
    fn name(&self) -> &str {
        "core"
    }
    
    fn get_function(&self, name: &str) -> Option<StdFunction> {
        self.functions.get(name).cloned()
    }
    
    fn get_type(&self, name: &str) -> Option<AstType> {
        self.types.get(name).cloned()
    }
}