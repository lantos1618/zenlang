pub mod types;
pub mod inference;
pub mod validation;
pub mod behaviors;

use crate::ast::{Program, Declaration, Statement, Expression, AstType, Function};
use crate::error::{CompileError, Result};
use crate::stdlib::StdNamespace;
use std::collections::HashMap;
use behaviors::BehaviorResolver;

pub struct TypeChecker {
    // Symbol table for tracking variable types
    scopes: Vec<HashMap<String, AstType>>,
    // Function signatures
    functions: HashMap<String, FunctionSignature>,
    // Struct definitions
    structs: HashMap<String, StructInfo>,
    // Enum definitions
    enums: HashMap<String, EnumInfo>,
    // Behavior/trait resolver
    behavior_resolver: BehaviorResolver,
    // Standard library namespace
    std_namespace: StdNamespace,
}

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub is_external: bool,
}

#[derive(Clone, Debug)]
pub struct StructInfo {
    pub fields: Vec<(String, AstType)>,
}

#[derive(Clone, Debug)]
pub struct EnumInfo {
    pub variants: Vec<(String, Option<AstType>)>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            behavior_resolver: BehaviorResolver::new(),
            std_namespace: StdNamespace::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        // First pass: collect all type definitions and function signatures
        for declaration in &program.declarations {
            self.collect_declaration_types(declaration)?;
        }

        // Second pass: type check function bodies
        for declaration in &program.declarations {
            self.check_declaration(declaration)?;
        }

        Ok(())
    }

    fn collect_declaration_types(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Function(func) => {
                let signature = FunctionSignature {
                    params: func.args.clone(),
                    return_type: func.return_type.clone(),
                    is_external: false,
                };
                self.functions.insert(func.name.clone(), signature);
            }
            Declaration::ExternalFunction(ext_func) => {
                // External functions have args as Vec<AstType>, convert to params format
                let params = ext_func.args.iter().enumerate().map(|(i, t)| {
                    (format!("arg{}", i), t.clone())
                }).collect();
                let signature = FunctionSignature {
                    params,
                    return_type: ext_func.return_type.clone(),
                    is_external: true,
                };
                self.functions.insert(ext_func.name.clone(), signature);
            }
            Declaration::Struct(struct_def) => {
                // Convert StructField to (String, AstType)
                let fields = struct_def.fields.iter().map(|f| {
                    (f.name.clone(), f.type_.clone())
                }).collect();
                let info = StructInfo {
                    fields,
                };
                self.structs.insert(struct_def.name.clone(), info);
            }
            Declaration::Enum(enum_def) => {
                // Convert EnumVariant to (String, Option<AstType>)
                let variants = enum_def.variants.iter().map(|v| {
                    (v.name.clone(), v.payload.clone())
                }).collect();
                let info = EnumInfo {
                    variants,
                };
                self.enums.insert(enum_def.name.clone(), info);
            }
            Declaration::Behavior(behavior_def) => {
                self.behavior_resolver.register_behavior(behavior_def)?;
            }
            Declaration::Impl(impl_block) => {
                self.behavior_resolver.register_impl(impl_block)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn check_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Function(func) => {
                self.check_function(func)?;
            }
            Declaration::ComptimeBlock(statements) => {
                self.enter_scope();
                for statement in statements {
                    self.check_statement(statement)?;
                }
                self.exit_scope();
            }
            Declaration::Impl(impl_block) => {
                // Verify that the implementation satisfies the behavior
                self.behavior_resolver.verify_impl(impl_block)?;
                // Type check each method in the impl block
                for method in &impl_block.methods {
                    self.check_function(method)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn check_function(&mut self, function: &Function) -> Result<()> {
        self.enter_scope();

        // Add function parameters to scope
        for (param_name, param_type) in &function.args {
            self.declare_variable(param_name, param_type.clone())?;
        }

        // Check function body
        for statement in &function.body {
            self.check_statement(statement)?;
        }

        self.exit_scope();
        Ok(())
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::VariableDeclaration {
                name,
                type_,
                initializer,
                ..
            } => {
                if let Some(init_expr) = initializer {
                    let inferred_type = self.infer_expression_type(init_expr)?;
                    
                    if let Some(declared_type) = type_ {
                        // Check that the initializer type matches the declared type
                        if !self.types_compatible(declared_type, &inferred_type) {
                            return Err(CompileError::TypeError(
                                format!(
                                    "Type mismatch: variable '{}' declared as {:?} but initialized with {:?}",
                                    name, declared_type, inferred_type
                                ),
                                None
                            ));
                        }
                        self.declare_variable(name, declared_type.clone())?;
                    } else {
                        // Inferred type from initializer
                        self.declare_variable(name, inferred_type)?;
                    }
                } else if let Some(declared_type) = type_ {
                    self.declare_variable(name, declared_type.clone())?;
                } else {
                    return Err(CompileError::TypeError(
                        format!("Cannot infer type for variable '{}' without initializer", name),
                        None
                    ));
                }
            }
            Statement::VariableAssignment { name, value } => {
                let var_type = self.get_variable_type(name)?;
                let value_type = self.infer_expression_type(value)?;
                
                if !self.types_compatible(&var_type, &value_type) {
                    return Err(CompileError::TypeError(
                        format!(
                            "Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
                            value_type, name, var_type
                        ),
                        None
                    ));
                }
            }
            Statement::Return(expr) => {
                let _return_type = self.infer_expression_type(expr)?;
                // TODO: Check against function return type
            }
            Statement::Expression(expr) => {
                self.infer_expression_type(expr)?;
            }
            Statement::Loop { kind, body, .. } => {
                use crate::ast::LoopKind;
                self.enter_scope();
                
                // Handle loop-specific variables
                match kind {
                    LoopKind::Infinite => {
                        // No special handling needed
                    }
                    LoopKind::Condition(expr) => {
                        // Type check the condition
                        let cond_type = self.infer_expression_type(expr)?;
                        // Condition should be boolean or integer (truthy)
                        if !matches!(cond_type, AstType::Bool | AstType::I32 | AstType::I64) {
                            return Err(CompileError::TypeError(
                                format!("Loop condition must be boolean or integer, got {:?}", cond_type),
                                None
                            ));
                        }
                    }
                    LoopKind::Range { variable, start, end, .. } => {
                        // Type check start and end
                        let start_type = self.infer_expression_type(start)?;
                        let end_type = self.infer_expression_type(end)?;
                        
                        // Both should be integers
                        if !matches!(start_type, AstType::I32 | AstType::I64) {
                            return Err(CompileError::TypeError(
                                format!("Loop range start must be integer, got {:?}", start_type),
                                None
                            ));
                        }
                        if !matches!(end_type, AstType::I32 | AstType::I64) {
                            return Err(CompileError::TypeError(
                                format!("Loop range end must be integer, got {:?}", end_type),
                                None
                            ));
                        }
                        
                        // Declare the loop variable in the loop scope
                        self.declare_variable(variable, AstType::I64)?;
                    }
                    LoopKind::Iterator { variable, iterable } => {
                        // For now, assume the iterable is an array
                        let iterable_type = self.infer_expression_type(iterable)?;
                        
                        // Extract element type from array type
                        let element_type = match iterable_type {
                            AstType::Array(element) => *element,
                            AstType::FixedArray { element_type, .. } => *element_type,
                            _ => {
                                // For now, default to i64 if we can't determine the type
                                // TODO: Better handling of iterables
                                AstType::I64
                            }
                        };
                        
                        // Declare the loop variable with the element type
                        self.declare_variable(variable, element_type)?;
                    }
                }
                
                // Check loop body with the variable in scope
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }
            Statement::ComptimeBlock(statements) => {
                self.enter_scope();
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }
            _ => {}
        }
        Ok(())
    }

    fn infer_expression_type(&self, expr: &Expression) -> Result<AstType> {
        match expr {
            Expression::Integer32(_) => Ok(AstType::I32),
            Expression::Integer64(_) => Ok(AstType::I64),
            Expression::Float32(_) => Ok(AstType::F32),
            Expression::Float64(_) => Ok(AstType::F64),
            Expression::Boolean(_) => Ok(AstType::Bool),
            Expression::String(_) => Ok(AstType::String),
            Expression::Identifier(name) => self.get_variable_type(name),
            Expression::BinaryOp { left, op, right } => {
                inference::infer_binary_op_type(self, left, op, right)
            }
            Expression::FunctionCall { name, .. } => {
                if let Some(sig) = self.functions.get(name) {
                    Ok(sig.return_type.clone())
                } else {
                    Err(CompileError::TypeError(format!("Unknown function: {}", name), None))
                }
            }
            Expression::MemberAccess { object, member } => {
                // Check if accessing @std namespace
                if let Expression::Identifier(name) = &**object {
                    if StdNamespace::is_std_reference(name) {
                        // Resolve @std.module access
                        return Ok(AstType::Generic {
                            name: format!("StdModule::{}", member),
                            type_args: vec![],
                        });
                    }
                }
                let object_type = self.infer_expression_type(object)?;
                inference::infer_member_type(&object_type, member, &self.structs)
            }
            Expression::Comptime(inner) => self.infer_expression_type(inner),
            Expression::Range { .. } => Ok(AstType::Range {
                start_type: Box::new(AstType::I32),
                end_type: Box::new(AstType::I32),
                inclusive: false,
            }),
            Expression::StructLiteral { name, .. } => {
                // For struct literals, return the struct type
                // Check if it's a known struct
                if let Some(struct_def) = self.structs.get(name) {
                    Ok(AstType::Struct {
                        name: name.clone(),
                        fields: struct_def.fields.clone(),
                    })
                } else {
                    // It might be a generic struct that will be monomorphized
                    // For now, return a struct type with empty fields
                    Ok(AstType::Struct {
                        name: name.clone(),
                        fields: vec![],
                    })
                }
            }
            Expression::StdModule(module) => {
                // Return a type representing the std module
                Ok(AstType::Generic {
                    name: format!("StdModule::{}", module),
                    type_args: vec![],
                })
            }
            Expression::Module(module) => {
                // Return a type representing a module
                Ok(AstType::Generic {
                    name: format!("Module::{}", module),
                    type_args: vec![],
                })
            }
            Expression::StringInterpolation { .. } => {
                // String interpolation always returns a string (pointer to char)
                Ok(AstType::Pointer(Box::new(AstType::I8)))
            }
            _ => Ok(AstType::Void), // Default for unhandled cases
        }
    }

    fn types_compatible(&self, expected: &AstType, actual: &AstType) -> bool {
        validation::types_compatible(expected, actual)
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_variable(&mut self, name: &str, type_: AstType) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name) {
                return Err(CompileError::TypeError(
                    format!("Variable '{}' already declared in this scope", name),
                    None
                ));
            }
            scope.insert(name.to_string(), type_);
            Ok(())
        } else {
            Err(CompileError::TypeError("No active scope".to_string(), None))
        }
    }

    fn get_variable_type(&self, name: &str) -> Result<AstType> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(type_) = scope.get(name) {
                return Ok(type_.clone());
            }
        }
        Err(CompileError::TypeError(format!("Undefined variable: {}", name), None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_basic_type_checking() {
        let input = "main = () void {
            x := 42
            y : i32 = 100
            z := x + y
        }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut type_checker = TypeChecker::new();
        assert!(type_checker.check_program(&program).is_ok());
    }

    #[test]
    fn test_type_mismatch_error() {
        let input = "main = () void {
            x : i32 = \"hello\"
        }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut type_checker = TypeChecker::new();
        let result = type_checker.check_program(&program);
        assert!(result.is_err());
        if let Err(CompileError::TypeError(msg, _)) = result {
            assert!(msg.contains("Type mismatch"));
        }
    }
}