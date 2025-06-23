use std::collections::HashMap;
use inkwell::{
    types::{BasicTypeEnum, FunctionType, StructType},
    values::{FunctionValue, PointerValue},
};

/// Represents a symbol in the symbol table, which can be a type, variable, or function
#[derive(Debug, Clone)]
pub enum Symbol<'ctx> {
    /// A type symbol
    Type(BasicTypeEnum<'ctx>),
    
    /// A struct type symbol
    StructType(StructType<'ctx>),
    
    /// A function type symbol
    FunctionType(FunctionType<'ctx>),
    
    /// A variable symbol (pointer to the value)
    Variable(PointerValue<'ctx>),
    
    /// A function value
    Function(FunctionValue<'ctx>),
}

/// A scope in the symbol table
pub struct Scope<'ctx> {
    symbols: HashMap<String, Symbol<'ctx>>,
    parent: Option<usize>,
}

/// A symbol table that supports scoping
pub struct SymbolTable<'ctx> {
    scopes: Vec<Scope<'ctx>>,
    current_scope: usize,
}

impl<'ctx> SymbolTable<'ctx> {
    /// Create a new symbol table with a global scope
    pub fn new() -> Self {
        let global_scope = Scope {
            symbols: HashMap::new(),
            parent: None,
        };
        
        SymbolTable {
            scopes: vec![global_scope],
            current_scope: 0,
        }
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        let new_scope = Scope {
            symbols: HashMap::new(),
            parent: Some(self.current_scope),
        };
        
        self.scopes.push(new_scope);
        self.current_scope = self.scopes.len() - 1;
    }
    
    /// Exit the current scope and return to the parent scope
    pub fn exit_scope(&mut self) -> Option<()> {
        if let Some(parent_scope) = self.scopes[self.current_scope].parent {
            self.current_scope = parent_scope;
            Some(())
        } else {
            None // Can't exit the global scope
        }
    }
    
    /// Insert a symbol into the current scope
    pub fn insert<S: Into<String>>(&mut self, name: S, symbol: Symbol<'ctx>) -> Option<Symbol<'ctx>> {
        self.scopes[self.current_scope].symbols.insert(name.into(), symbol)
    }
    
    /// Look up a symbol starting from the current scope and moving up through parent scopes
    pub fn lookup(&self, name: &str) -> Option<&Symbol<'ctx>> {
        let mut current_scope = Some(self.current_scope);
        
        while let Some(scope_idx) = current_scope {
            if let Some(symbol) = self.scopes[scope_idx].symbols.get(name) {
                return Some(symbol);
            }
            
            current_scope = self.scopes[scope_idx].parent;
        }
        
        None
    }
    
    /// Get a mutable reference to a symbol if it exists in the current scope
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Symbol<'ctx>> {
        self.scopes[self.current_scope].symbols.get_mut(name)
    }
    
    /// Check if a symbol exists in the current scope (doesn't check parent scopes)
    pub fn exists_in_current_scope(&self, name: &str) -> bool {
        self.scopes[self.current_scope].symbols.contains_key(name)
    }
    
    /// Get the current scope depth (0 = global scope)
    pub fn depth(&self) -> usize {
        let mut depth = 0;
        let mut current = Some(self.current_scope);
        
        while let Some(scope_idx) = current {
            depth += 1;
            current = self.scopes[scope_idx].parent;
        }
        
        depth - 1 // Subtract 1 because we start counting from 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;
    
    #[test]
    fn test_symbol_table_scoping() {
        let context = Context::create();
        let i32_type = context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        
        let mut symtab = SymbolTable::new();
        
        // Add to global scope
        symtab.insert("i32", Symbol::Type(i32_type.into()));
        
        // Enter function scope
        symtab.enter_scope();
        symtab.insert("x", Symbol::Type(i32_type.into()));
        
        // Can find in current scope
        assert!(matches!(symtab.lookup("x"), Some(Symbol::Type(_))));
        
        // Can find in parent scope
        assert!(matches!(symtab.lookup("i32"), Some(Symbol::Type(_))));
        
        // Exit scope
        symtab.exit_scope();
        
        // Can't find in global scope
        assert!(symtab.lookup("x").is_none());
        
        // Still can find in global scope
        assert!(matches!(symtab.lookup("i32"), Some(Symbol::Type(_))));
    }
}
