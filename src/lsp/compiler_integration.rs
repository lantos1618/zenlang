//! Compiler Integration for Zen LSP
//!
//! This module exposes LSP-friendly queries backed by the compiler: parsing stdlib,
//! indexing function signatures, and answering type/method lookups. Method completion
//! performance is optimized via a receiver-type index to avoid linear scans.

use crate::ast::primitives;
use crate::ast::{AstType, Declaration, Program};
use crate::error::{CompileError, Result};
use crate::lexer::Lexer;
use crate::lsp::type_inference::get_base_type_name;
use crate::lsp::utils::format_type;
use crate::name_utils;
use crate::parser::Parser;
use crate::typechecker::TypeChecker;
use std::collections::HashMap;

// ============================================================================
// COMPILER INTEGRATION STRUCT
// ============================================================================

pub struct CompilerIntegration {
    stdlib_programs: HashMap<String, Program>,
    stdlib_functions: HashMap<String, FunctionSignature>,
    method_index: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub receiver_type: Option<String>, // For UFC methods: the receiver type
}

impl Default for CompilerIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerIntegration {
    pub fn new() -> Self {
        Self {
            stdlib_programs: HashMap::new(),
            stdlib_functions: HashMap::new(),
            method_index: HashMap::new(),
        }
    }

    /// Load and index a stdlib file
    pub fn load_stdlib_file(&mut self, path: &str, content: &str) -> Result<()> {
        let lexer = Lexer::new(content);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program()?;

        // Index functions from this stdlib file
        for decl in &program.declarations {
            if let Declaration::Function(func) = decl {
                // Check if this is a UFC method (first param matches receiver type)
                let receiver_type = if !func.args.is_empty() {
                    // Check if first parameter type matches the function name pattern
                    // This is a heuristic - in real stdlib, methods are defined with receiver as first param
                    let first_param_type = format_type(&func.args[0].1);
                    Some(first_param_type)
                } else {
                    None
                };

                let sig = FunctionSignature {
                    name: func.name.clone(),
                    params: func.args.clone(),
                    return_type: func.return_type.clone(),
                    receiver_type: receiver_type.clone(),
                };

                // Store with full name and also with receiver type prefix for method lookup
                self.stdlib_functions.insert(func.name.clone(), sig.clone());
                if let Some(recv) = &receiver_type {
                    let method_key = format!("{}::{}", recv, func.name);
                    self.stdlib_functions.insert(method_key.clone(), sig);

                    // Populate index for exact receiver
                    self.method_index
                        .entry(recv.clone())
                        .or_default()
                        .push(method_key.clone());

                    // Also index by base receiver type (before generics), e.g., "Vec<T>" -> "Vec"
                    let base = get_base_type_name(recv);
                    if base != *recv {
                        self.method_index.entry(base).or_default().push(method_key);
                    }
                }
            }
        }

        self.stdlib_programs.insert(path.to_string(), program);
        Ok(())
    }

    /// Get all methods available for a given receiver type
    pub fn get_methods_for_type(&self, receiver_type: &str) -> Vec<&FunctionSignature> {
        // Fast path: exact receiver index
        let mut out: Vec<&FunctionSignature> = Vec::new();

        if let Some(keys) = self.method_index.get(receiver_type) {
            for k in keys {
                if let Some(sig) = self.stdlib_functions.get(k) {
                    out.push(sig);
                }
            }
        }

        // Fallback: base type of generics (e.g., "Vec<T>" -> "Vec")
        if out.is_empty() {
            let base = get_base_type_name(receiver_type);
            if let Some(keys) = self.method_index.get(&base) {
                for k in keys {
                    if let Some(sig) = self.stdlib_functions.get(k) {
                        out.push(sig);
                    }
                }
            }
        }

        // As a last resort, maintain previous heuristic compatibility
        if out.is_empty() {
            for (key, sig) in &self.stdlib_functions {
                if let Some(recv) = &sig.receiver_type {
                    if recv == receiver_type
                        || recv.starts_with(&format!("{}<", receiver_type))
                        || key.starts_with(&format!("{}::", receiver_type))
                    {
                        out.push(sig);
                    }
                }
            }
        }

        out
    }

    pub fn get_method_return_type(
        &self,
        receiver_type: &str,
        method_name: &str,
    ) -> Option<AstType> {
        use crate::stdlib_types::stdlib_types;

        if let Some(return_type) = stdlib_types().get_method_return_type(receiver_type, method_name)
        {
            return Some(return_type.clone());
        }

        let method_key = name_utils::method_key(receiver_type, method_name);
        if let Some(sig) = self.stdlib_functions.get(&method_key) {
            return Some(sig.return_type.clone());
        }

        if let Some(sig) = self.stdlib_functions.get(method_name) {
            if let Some(recv) = &sig.receiver_type {
                if recv == receiver_type || recv.starts_with(&format!("{}<", receiver_type)) {
                    return Some(sig.return_type.clone());
                }
            }
        }

        None
    }

    /// Parse a type string using the parser
    pub fn parse_type_string(&self, type_str: &str) -> Result<AstType> {
        crate::parser::parse_type_from_string(type_str)
    }

    /// Infer the type of an expression using TypeChecker with full program context
    pub fn infer_expression_type(
        &mut self,
        program: &Program,
        expr: &crate::ast::Expression,
    ) -> Result<AstType> {
        // Set up TypeChecker with program context
        // Create a fresh type checker and populate it with program declarations
        let mut type_checker = TypeChecker::new();

        // Run type checker to populate symbol tables (ignoring errors for now)
        // This sets up the context needed for inference - it indexes functions, structs, enums, etc.
        let _ = type_checker.check_program(program);

        // Now use TypeChecker's real inference
        type_checker.infer_expression_type(expr)
    }

    /// Infer type from expression string - parses then uses TypeChecker
    pub fn infer_expression_type_from_string(
        &mut self,
        program: &Program,
        expr_str: &str,
    ) -> Result<AstType> {
        // Try to parse the expression
        let lexer = Lexer::new(expr_str);
        let mut parser = Parser::new(lexer);

        match parser.parse_expression() {
            Ok(expr) => {
                // Use real type inference
                self.infer_expression_type(program, &expr)
            }
            Err(_) => {
                // If parsing fails, fall back to simple heuristics for common cases
                let s = expr_str.trim();

                // Simple literals - use centralized check
                if primitives::is_boolean_literal(s) {
                    return Ok(AstType::Bool);
                }
                if (s.starts_with('"') && s.ends_with('"'))
                    || (s.starts_with('\'') && s.ends_with('\''))
                {
                    return Ok(AstType::StaticString);
                }
                if s.parse::<i64>().is_ok() {
                    return Ok(AstType::I32);
                }
                if s.contains('.') && s.parse::<f64>().is_ok() {
                    return Ok(AstType::F64);
                }

                // Method call lookup
                if let Some(dot_pos) = s.find('.') {
                    if let Some(paren_pos) = s[dot_pos + 1..].find('(') {
                        let method_name = &s[dot_pos + 1..dot_pos + 1 + paren_pos].trim();
                        let receiver = &s[..dot_pos].trim();

                        // Try to infer receiver type from program context
                        let recv_ty_name = if receiver.starts_with('"') {
                            Some("String".to_string())
                        } else if receiver.parse::<i64>().is_ok() {
                            Some("i32".to_string())
                        } else if receiver.parse::<f64>().is_ok() {
                            Some("f64".to_string())
                        } else {
                            // Try to find variable type in program
                            self.find_variable_type_in_program(program, receiver)
                        };

                        if let Some(recv_name) = recv_ty_name {
                            if let Some(ret) = self.get_method_return_type(&recv_name, method_name)
                            {
                                return Ok(ret);
                            }
                        }
                    }
                }

                // Function call lookup
                if let Some(paren_pos) = s.find('(') {
                    let func_name = s[..paren_pos].trim();

                    if let Some(sig) = self.stdlib_functions.get(func_name) {
                        return Ok(sig.return_type.clone());
                    }

                    for decl in &program.declarations {
                        if let Declaration::Function(func) = decl {
                            if func.name == func_name {
                                return Ok(func.return_type.clone());
                            }
                        }
                    }
                }

                // No heuristic matched — return error instead of silent Void
                Err(CompileError::TypeError(
                    format!("Could not infer type for expression: {}", s),
                    None,
                ))
            }
        }
    }

    /// Find variable type in program AST
    fn find_variable_type_in_program(&self, program: &Program, var_name: &str) -> Option<String> {
        use crate::ast::Statement;

        for decl in &program.declarations {
            if let Declaration::Function(func) = decl {
                for stmt in &func.body {
                    if let Statement::VariableDeclaration { name, type_, .. } = stmt {
                        if name == var_name {
                            return type_.as_ref().map(format_type);
                        }
                    }
                }
            }
        }
        None
    }

    /// Get function signature by name
    pub fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature> {
        self.stdlib_functions.get(name)
    }

    /// Get all function signatures
    pub fn get_all_functions(&self) -> &HashMap<String, FunctionSignature> {
        &self.stdlib_functions
    }

    /// Check if a type has a method
    pub fn has_method(&self, receiver_type: &str, method_name: &str) -> bool {
        self.get_method_return_type(receiver_type, method_name)
            .is_some()
    }
}
