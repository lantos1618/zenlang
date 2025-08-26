pub mod resolver;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::ast::{Program, Declaration};
use crate::parser::Parser;
use crate::error::CompileError;

/// Module system for Zen language
pub struct ModuleSystem {
    /// Map from module paths to their resolved AST
    modules: HashMap<String, Program>,
    /// Search paths for modules
    search_paths: Vec<PathBuf>,
    /// Current working directory
    cwd: PathBuf,
}

impl ModuleSystem {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut search_paths = vec![
            cwd.clone(),
            cwd.join("lib"),
            cwd.join("modules"),
        ];
        
        // Add standard library path if it exists
        if let Ok(zen_home) = std::env::var("ZEN_HOME") {
            let zen_path = PathBuf::from(zen_home);
            search_paths.push(zen_path.join("std"));
            search_paths.push(zen_path.join("lib"));
        }
        
        ModuleSystem {
            modules: HashMap::new(),
            search_paths,
            cwd,
        }
    }
    
    /// Add a search path for modules
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
    
    /// Resolve and load a module
    pub fn load_module(&mut self, module_path: &str) -> Result<&Program, CompileError> {
        // Check if already loaded
        if self.modules.contains_key(module_path) {
            return Ok(&self.modules[module_path]);
        }
        
        // Try to find the module file
        let file_path = self.resolve_module_path(module_path)?;
        
        // Read and parse the module
        let source = std::fs::read_to_string(&file_path)
            .map_err(|e| CompileError::FileNotFound(file_path.display().to_string(), Some(e.to_string())))?;
        
        let lexer = crate::lexer::Lexer::new(&source);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program()
            .map_err(|e| CompileError::ParseError(format!("Failed to parse module {}: {:?}", module_path, e), None))?;
        
        // Process imports in the loaded module
        let mut processed_program = program.clone();
        for decl in &program.declarations {
            if let Declaration::ModuleImport { alias: _, module_path: import_path } = decl {
                // Recursively load imported modules
                self.load_module(import_path)?;
            }
        }
        
        // Store the loaded module
        self.modules.insert(module_path.to_string(), processed_program.clone());
        Ok(&self.modules[module_path])
    }
    
    /// Resolve a module path to a file path
    fn resolve_module_path(&self, module_path: &str) -> Result<PathBuf, CompileError> {
        // Convert module path (e.g., "std.io") to file path (e.g., "std/io.zen")
        let relative_path = module_path.replace('.', "/") + ".zen";
        
        // Try each search path
        for search_path in &self.search_paths {
            let full_path = search_path.join(&relative_path);
            if full_path.exists() {
                return Ok(full_path);
            }
            
            // Also try as a directory with mod.zen
            let mod_path = search_path.join(&module_path.replace('.', "/")).join("mod.zen");
            if mod_path.exists() {
                return Ok(mod_path);
            }
        }
        
        Err(CompileError::FileNotFound(
            format!("Module '{}' not found in search paths", module_path),
            None
        ))
    }
    
    /// Get all loaded modules
    pub fn get_modules(&self) -> &HashMap<String, Program> {
        &self.modules
    }
    
    /// Merge all loaded modules into a single program
    pub fn merge_programs(&self, main_program: Program) -> Program {
        let mut merged = main_program;
        
        // Add all declarations from imported modules
        for (_path, module) in &self.modules {
            for decl in &module.declarations {
                // Skip duplicate imports
                if !matches!(decl, Declaration::ModuleImport { .. }) {
                    merged.declarations.push(decl.clone());
                }
            }
        }
        
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_system_creation() {
        let ms = ModuleSystem::new();
        assert!(ms.search_paths.len() >= 3);
        assert!(ms.modules.is_empty());
    }
    
    #[test] 
    fn test_add_search_path() {
        let mut ms = ModuleSystem::new();
        let initial_len = ms.search_paths.len();
        ms.add_search_path(PathBuf::from("/custom/path"));
        assert_eq!(ms.search_paths.len(), initial_len + 1);
    }
}