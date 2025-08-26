use std::collections::{HashMap, HashSet};
use crate::ast::{Program, Declaration, Function, Statement, Expression};

/// Resolves module imports and manages symbol visibility
pub struct ModuleResolver {
    /// Map from module alias to actual module path
    imports: HashMap<String, String>,
    /// Exported symbols from each module
    exports: HashMap<String, HashSet<String>>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        ModuleResolver {
            imports: HashMap::new(),
            exports: HashMap::new(),
        }
    }
    
    /// Register an import alias
    pub fn add_import(&mut self, alias: String, module_path: String) {
        self.imports.insert(alias, module_path);
    }
    
    /// Register exported symbols from a module
    pub fn add_exports(&mut self, module_path: String, symbols: HashSet<String>) {
        self.exports.insert(module_path, symbols);
    }
    
    /// Resolve a qualified name (e.g., "io.println") to its full module path
    pub fn resolve_qualified_name(&self, name: &str) -> Option<(String, String)> {
        if let Some(dot_pos) = name.find('.') {
            let module_alias = &name[..dot_pos];
            let symbol = &name[dot_pos + 1..];
            
            if let Some(module_path) = self.imports.get(module_alias) {
                return Some((module_path.clone(), symbol.to_string()));
            }
        }
        None
    }
    
    /// Check if a symbol is exported from a module
    pub fn is_exported(&self, module_path: &str, symbol: &str) -> bool {
        self.exports
            .get(module_path)
            .map(|symbols| symbols.contains(symbol))
            .unwrap_or(false)
    }
    
    /// Extract exported symbols from a module
    pub fn extract_exports(program: &Program) -> HashSet<String> {
        let mut exports = HashSet::new();
        
        for decl in &program.declarations {
            match decl {
                Declaration::Function(func) if !func.name.starts_with('_') => {
                    // Public functions (not starting with _) are exported
                    exports.insert(func.name.clone());
                }
                Declaration::Struct(struct_def) if !struct_def.name.starts_with('_') => {
                    exports.insert(struct_def.name.clone());
                }
                Declaration::Enum(enum_def) if !enum_def.name.starts_with('_') => {
                    exports.insert(enum_def.name.clone());
                }
                Declaration::TypeAlias(alias) if !alias.name.starts_with('_') => {
                    exports.insert(alias.name.clone());
                }
                _ => {}
            }
        }
        
        exports
    }
    
    /// Rewrite a program to resolve module references
    pub fn resolve_program(&self, program: &mut Program) -> Result<(), String> {
        // Process each declaration
        for decl in &mut program.declarations {
            if let Declaration::Function(func) = decl {
                self.resolve_function(func)?;
            }
        }
        Ok(())
    }
    
    /// Resolve module references in a function
    fn resolve_function(&self, func: &mut Function) -> Result<(), String> {
        for stmt in &mut func.body {
            self.resolve_statement(stmt)?;
        }
        Ok(())
    }
    
    /// Resolve module references in a statement
    fn resolve_statement(&self, stmt: &mut Statement) -> Result<(), String> {
        match stmt {
            Statement::Expression(expr) => self.resolve_expression(expr),
            Statement::VariableDeclaration { initializer, .. } => {
                if let Some(init) = initializer {
                    self.resolve_expression(init)?;
                }
                Ok(())
            }
            Statement::VariableAssignment { value, .. } => {
                self.resolve_expression(value)
            }
            Statement::Return(expr) => self.resolve_expression(expr),
            Statement::Loop { body, .. } => {
                for s in body {
                    self.resolve_statement(s)?;
                }
                Ok(())
            }
            _ => Ok(())
        }
    }
    
    /// Resolve module references in an expression
    fn resolve_expression(&self, expr: &mut Expression) -> Result<(), String> {
        match expr {
            Expression::FunctionCall { name, args } => {
                // Check if this is a qualified module call
                if let Some((module_path, symbol)) = self.resolve_qualified_name(name) {
                    if !self.is_exported(&module_path, &symbol) {
                        return Err(format!("Symbol '{}' is not exported from module '{}'", symbol, module_path));
                    }
                    // Rewrite to fully qualified name for codegen
                    *name = format!("{}_{}", module_path.replace('.', "_"), symbol);
                }
                
                // Resolve arguments
                for arg in args {
                    self.resolve_expression(arg)?;
                }
                Ok(())
            }
            Expression::BinaryOp { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
                Ok(())
            }
            _ => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resolve_qualified_name() {
        let mut resolver = ModuleResolver::new();
        resolver.add_import("io".to_string(), "std.io".to_string());
        
        let result = resolver.resolve_qualified_name("io.println");
        assert_eq!(result, Some(("std.io".to_string(), "println".to_string())));
        
        let result = resolver.resolve_qualified_name("unknown.func");
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_is_exported() {
        let mut resolver = ModuleResolver::new();
        let mut exports = HashSet::new();
        exports.insert("println".to_string());
        exports.insert("readln".to_string());
        resolver.add_exports("std.io".to_string(), exports);
        
        assert!(resolver.is_exported("std.io", "println"));
        assert!(!resolver.is_exported("std.io", "private_func"));
        assert!(!resolver.is_exported("unknown.module", "anything"));
    }
}