use crate::ast::AstType;
use crate::stdlib::StdFunction;
use std::collections::HashMap;

/// The IO module provides input/output operations
pub struct IOModule {
    functions: HashMap<String, StdFunction>,
}

impl IOModule {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        
        // Print functions
        functions.insert("print".to_string(), StdFunction {
            name: "print".to_string(),
            params: vec![("message".to_string(), AstType::String)],
            return_type: AstType::Void,
            is_builtin: true,
        });
        
        functions.insert("println".to_string(), StdFunction {
            name: "println".to_string(),
            params: vec![("message".to_string(), AstType::String)],
            return_type: AstType::Void,
            is_builtin: true,
        });
        
        functions.insert("eprint".to_string(), StdFunction {
            name: "eprint".to_string(),
            params: vec![("message".to_string(), AstType::String)],
            return_type: AstType::Void,
            is_builtin: true,
        });
        
        functions.insert("eprintln".to_string(), StdFunction {
            name: "eprintln".to_string(),
            params: vec![("message".to_string(), AstType::String)],
            return_type: AstType::Void,
            is_builtin: true,
        });
        
        // Input functions
        functions.insert("read_line".to_string(), StdFunction {
            name: "read_line".to_string(),
            params: vec![],
            return_type: AstType::Result {
                ok_type: Box::new(AstType::String),
                err_type: Box::new(AstType::String),
            },
            is_builtin: true,
        });
        
        functions.insert("read_input".to_string(), StdFunction {
            name: "read_input".to_string(),
            params: vec![("prompt".to_string(), AstType::String)],
            return_type: AstType::Result {
                ok_type: Box::new(AstType::String),
                err_type: Box::new(AstType::String),
            },
            is_builtin: true,
        });
        
        // File operations
        functions.insert("read_file".to_string(), StdFunction {
            name: "read_file".to_string(),
            params: vec![("path".to_string(), AstType::String)],
            return_type: AstType::Result {
                ok_type: Box::new(AstType::String),
                err_type: Box::new(AstType::String),
            },
            is_builtin: true,
        });
        
        functions.insert("write_file".to_string(), StdFunction {
            name: "write_file".to_string(),
            params: vec![
                ("path".to_string(), AstType::String),
                ("content".to_string(), AstType::String),
            ],
            return_type: AstType::Result {
                ok_type: Box::new(AstType::Void),
                err_type: Box::new(AstType::String),
            },
            is_builtin: true,
        });
        
        functions.insert("append_file".to_string(), StdFunction {
            name: "append_file".to_string(),
            params: vec![
                ("path".to_string(), AstType::String),
                ("content".to_string(), AstType::String),
            ],
            return_type: AstType::Result {
                ok_type: Box::new(AstType::Void),
                err_type: Box::new(AstType::String),
            },
            is_builtin: true,
        });
        
        functions.insert("file_exists".to_string(), StdFunction {
            name: "file_exists".to_string(),
            params: vec![("path".to_string(), AstType::String)],
            return_type: AstType::Bool,
            is_builtin: true,
        });
        
        functions.insert("is_directory".to_string(), StdFunction {
            name: "is_directory".to_string(),
            params: vec![("path".to_string(), AstType::String)],
            return_type: AstType::Bool,
            is_builtin: true,
        });
        
        functions.insert("is_file".to_string(), StdFunction {
            name: "is_file".to_string(),
            params: vec![("path".to_string(), AstType::String)],
            return_type: AstType::Bool,
            is_builtin: true,
        });
        
        functions.insert("create_dir".to_string(), StdFunction {
            name: "create_dir".to_string(),
            params: vec![("path".to_string(), AstType::String)],
            return_type: AstType::Result {
                ok_type: Box::new(AstType::Void),
                err_type: Box::new(AstType::String),
            },
            is_builtin: true,
        });
        
        functions.insert("remove_file".to_string(), StdFunction {
            name: "remove_file".to_string(),
            params: vec![("path".to_string(), AstType::String)],
            return_type: AstType::Result {
                ok_type: Box::new(AstType::Void),
                err_type: Box::new(AstType::String),
            },
            is_builtin: true,
        });
        
        functions.insert("remove_dir".to_string(), StdFunction {
            name: "remove_dir".to_string(),
            params: vec![("path".to_string(), AstType::String)],
            return_type: AstType::Result {
                ok_type: Box::new(AstType::Void),
                err_type: Box::new(AstType::String),
            },
            is_builtin: true,
        });
        
        IOModule { functions }
    }
    
    pub fn get_function(&self, name: &str) -> Option<&StdFunction> {
        self.functions.get(name)
    }
    
    /// Get all available IO functions
    pub fn list_functions(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

impl super::StdModuleTrait for IOModule {
    fn name(&self) -> &str {
        "io"
    }
    
    fn get_function(&self, name: &str) -> Option<StdFunction> {
        self.functions.get(name).cloned()
    }
    
    fn get_type(&self, _name: &str) -> Option<AstType> {
        None // IO module doesn't define types
    }
}