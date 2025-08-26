// Compile-time execution framework for Zen
// This module provides an interpreter that executes Zen code during compilation

use crate::ast::{self, AstType, Expression, Statement, Declaration, Program};
use crate::error::{CompileError, Result};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

// Value types that can exist at compile time
#[derive(Debug, Clone)]
pub enum ComptimeValue {
    // Primitive values
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    
    // Compound values
    Array(Vec<ComptimeValue>),
    Struct {
        name: String,
        fields: HashMap<String, ComptimeValue>,
    },
    
    // Type value (for type-level computations)
    Type(AstType),
    
    // Function value (for higher-order functions)
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        closure: Environment,
    },
    
    // Special values
    Void,
    Null,
}

impl ComptimeValue {
    /// Convert a compile-time value to an AST expression
    pub fn to_expression(&self) -> Result<Expression> {
        match self {
            ComptimeValue::I32(v) => Ok(Expression::Integer32(*v)),
            ComptimeValue::I64(v) => Ok(Expression::Integer64(*v)),
            ComptimeValue::F32(v) => Ok(Expression::Float32(*v)),
            ComptimeValue::F64(v) => Ok(Expression::Float64(*v)),
            ComptimeValue::Bool(v) => Ok(Expression::Boolean(*v)),
            ComptimeValue::String(v) => Ok(Expression::String(v.clone())),
            ComptimeValue::Array(values) => {
                let exprs: Result<Vec<_>> = values.iter()
                    .map(|v| v.to_expression())
                    .collect();
                Ok(Expression::ArrayLiteral(exprs?))
            }
            ComptimeValue::Type(t) => {
                // Type values become type annotations
                Err(CompileError::ComptimeError(
                    "Cannot convert type value to runtime expression".to_string()
                ))
            }
            _ => Err(CompileError::ComptimeError(
                format!("Cannot convert {:?} to expression", self)
            ))
        }
    }
    
    /// Get the type of a compile-time value
    pub fn get_type(&self) -> AstType {
        match self {
            ComptimeValue::I8(_) => AstType::I8,
            ComptimeValue::I16(_) => AstType::I16,
            ComptimeValue::I32(_) => AstType::I32,
            ComptimeValue::I64(_) => AstType::I64,
            ComptimeValue::U8(_) => AstType::U8,
            ComptimeValue::U16(_) => AstType::U16,
            ComptimeValue::U32(_) => AstType::U32,
            ComptimeValue::U64(_) => AstType::U64,
            ComptimeValue::F32(_) => AstType::F32,
            ComptimeValue::F64(_) => AstType::F64,
            ComptimeValue::Bool(_) => AstType::Bool,
            ComptimeValue::String(_) => AstType::String,
            ComptimeValue::Array(v) => {
                if v.is_empty() {
                    AstType::Array(Box::new(AstType::Void))
                } else {
                    AstType::Array(Box::new(v[0].get_type()))
                }
            }
            ComptimeValue::Struct { name, .. } => AstType::Struct {
                name: name.clone(),
                fields: vec![], // TODO: Track field types
            },
            ComptimeValue::Type(_) => {
                // Meta-type
                AstType::Generic {
                    name: "Type".to_string(),
                    type_args: vec![],
                }
            }
            ComptimeValue::Void => AstType::Void,
            ComptimeValue::Null => AstType::Pointer(Box::new(AstType::Void)),
            ComptimeValue::Function { .. } => {
                // TODO: Function type
                AstType::Generic {
                    name: "Function".to_string(),
                    type_args: vec![],
                }
            }
        }
    }
}

// Environment for compile-time execution
#[derive(Debug, Clone)]
pub struct Environment {
    variables: Rc<RefCell<HashMap<String, ComptimeValue>>>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            variables: Rc::new(RefCell::new(HashMap::new())),
            parent: None,
        }
    }
    
    pub fn with_parent(parent: Environment) -> Self {
        Environment {
            variables: Rc::new(RefCell::new(HashMap::new())),
            parent: Some(Box::new(parent)),
        }
    }
    
    pub fn define(&self, name: String, value: ComptimeValue) {
        self.variables.borrow_mut().insert(name, value);
    }
    
    pub fn get(&self, name: &str) -> Option<ComptimeValue> {
        self.variables.borrow().get(name).cloned()
            .or_else(|| self.parent.as_ref()?.get(name))
    }
    
    pub fn set(&self, name: &str, value: ComptimeValue) -> Result<()> {
        if self.variables.borrow().contains_key(name) {
            self.variables.borrow_mut().insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.set(name, value)
        } else {
            Err(CompileError::ComptimeError(
                format!("Undefined variable: {}", name)
            ))
        }
    }
}

// Compile-time interpreter
pub struct ComptimeInterpreter {
    env: Environment,
    // Track generated code
    generated_declarations: Vec<Declaration>,
    // Track imports and modules
    modules: HashMap<String, ComptimeValue>,
}

impl ComptimeInterpreter {
    pub fn new() -> Self {
        let mut interpreter = ComptimeInterpreter {
            env: Environment::new(),
            generated_declarations: Vec::new(),
            modules: HashMap::new(),
        };
        
        // Initialize built-in compile-time functions
        interpreter.init_builtins();
        interpreter
    }
    
    fn init_builtins(&mut self) {
        // @std namespace
        self.modules.insert("@std".to_string(), ComptimeValue::Struct {
            name: "@std".to_string(),
            fields: {
                let mut fields = HashMap::new();
                
                // @std.core
                fields.insert("core".to_string(), ComptimeValue::Struct {
                    name: "core".to_string(),
                    fields: HashMap::new(),
                });
                
                // @std.build
                fields.insert("build".to_string(), ComptimeValue::Struct {
                    name: "build".to_string(),
                    fields: {
                        let mut build_fields = HashMap::new();
                        // build.import function
                        build_fields.insert("import".to_string(), ComptimeValue::Function {
                            name: "import".to_string(),
                            params: vec!["module_name".to_string()],
                            body: vec![],
                            closure: Environment::new(),
                        });
                        build_fields
                    },
                });
                
                fields
            },
        });
    }
    
    /// Execute a compile-time block
    pub fn execute_comptime_block(&mut self, statements: &[Statement]) -> Result<()> {
        for stmt in statements {
            self.execute_statement(stmt)?;
        }
        Ok(())
    }
    
    /// Execute a single statement
    pub fn execute_statement(&mut self, stmt: &Statement) -> Result<Option<ComptimeValue>> {
        match stmt {
            Statement::VariableDeclaration { name, initializer, .. } => {
                if let Some(init) = initializer {
                    let value = self.evaluate_expression(init)?;
                    self.env.define(name.clone(), value);
                }
                Ok(None)
            }
            
            Statement::Assignment { target, value } => {
                let val = self.evaluate_expression(value)?;
                self.env.set(target, val)?;
                Ok(None)
            }
            
            Statement::Expression(expr) => {
                let value = self.evaluate_expression(expr)?;
                Ok(Some(value))
            }
            
            Statement::Return(expr) => {
                let value = self.evaluate_expression(expr)?;
                Ok(Some(value))
            }
            
            Statement::ComptimeBlock(stmts) => {
                // Nested comptime block
                self.execute_comptime_block(stmts)?;
                Ok(None)
            }
            
            _ => Err(CompileError::ComptimeError(
                format!("Statement type not supported in comptime: {:?}", stmt)
            ))
        }
    }
    
    /// Evaluate an expression to a compile-time value
    pub fn evaluate_expression(&mut self, expr: &Expression) -> Result<ComptimeValue> {
        match expr {
            Expression::Integer32(v) => Ok(ComptimeValue::I32(*v)),
            Expression::Integer64(v) => Ok(ComptimeValue::I64(*v)),
            Expression::Float32(v) => Ok(ComptimeValue::F32(*v)),
            Expression::Float64(v) => Ok(ComptimeValue::F64(*v)),
            Expression::Boolean(v) => Ok(ComptimeValue::Bool(*v)),
            Expression::String(v) => Ok(ComptimeValue::String(v.clone())),
            
            Expression::Identifier(name) => {
                // Check for module reference
                if name.starts_with("@") {
                    if let Some(module) = self.modules.get(name) {
                        return Ok(module.clone());
                    }
                }
                
                self.env.get(name)
                    .ok_or_else(|| CompileError::ComptimeError(
                        format!("Undefined identifier: {}", name)
                    ))
            }
            
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.evaluate_binary_op(left_val, op, right_val)
            }
            
            Expression::FunctionCall { name, args } => {
                self.evaluate_function_call(name, args)
            }
            
            Expression::ArrayLiteral(elements) => {
                let values: Result<Vec<_>> = elements.iter()
                    .map(|e| self.evaluate_expression(e))
                    .collect();
                Ok(ComptimeValue::Array(values?))
            }
            
            Expression::MemberAccess { object, member } => {
                let obj_val = self.evaluate_expression(object)?;
                self.evaluate_member_access(obj_val, member)
            }
            
            Expression::Comptime(inner) => {
                // Nested comptime expression
                self.evaluate_expression(inner)
            }
            
            _ => Err(CompileError::ComptimeError(
                format!("Expression type not supported in comptime: {:?}", expr)
            ))
        }
    }
    
    /// Evaluate binary operations
    fn evaluate_binary_op(
        &self,
        left: ComptimeValue,
        op: &ast::BinaryOperator,
        right: ComptimeValue,
    ) -> Result<ComptimeValue> {
        use ast::BinaryOperator;
        
        match (left, right) {
            (ComptimeValue::I32(l), ComptimeValue::I32(r)) => {
                match op {
                    BinaryOperator::Add => Ok(ComptimeValue::I32(l + r)),
                    BinaryOperator::Subtract => Ok(ComptimeValue::I32(l - r)),
                    BinaryOperator::Multiply => Ok(ComptimeValue::I32(l * r)),
                    BinaryOperator::Divide => {
                        if r == 0 {
                            Err(CompileError::ComptimeError("Division by zero".to_string()))
                        } else {
                            Ok(ComptimeValue::I32(l / r))
                        }
                    }
                    BinaryOperator::Equal => Ok(ComptimeValue::Bool(l == r)),
                    BinaryOperator::NotEqual => Ok(ComptimeValue::Bool(l != r)),
                    BinaryOperator::LessThan => Ok(ComptimeValue::Bool(l < r)),
                    BinaryOperator::LessThanOrEqual => Ok(ComptimeValue::Bool(l <= r)),
                    BinaryOperator::GreaterThan => Ok(ComptimeValue::Bool(l > r)),
                    BinaryOperator::GreaterThanOrEqual => Ok(ComptimeValue::Bool(l >= r)),
                    _ => Err(CompileError::ComptimeError(
                        format!("Unsupported operation {:?} for I32", op)
                    ))
                }
            }
            
            (ComptimeValue::Bool(l), ComptimeValue::Bool(r)) => {
                match op {
                    BinaryOperator::LogicalAnd => Ok(ComptimeValue::Bool(l && r)),
                    BinaryOperator::LogicalOr => Ok(ComptimeValue::Bool(l || r)),
                    BinaryOperator::Equal => Ok(ComptimeValue::Bool(l == r)),
                    BinaryOperator::NotEqual => Ok(ComptimeValue::Bool(l != r)),
                    _ => Err(CompileError::ComptimeError(
                        format!("Unsupported operation {:?} for Bool", op)
                    ))
                }
            }
            
            (ComptimeValue::String(l), ComptimeValue::String(r)) => {
                match op {
                    BinaryOperator::Equal => Ok(ComptimeValue::Bool(l == r)),
                    BinaryOperator::NotEqual => Ok(ComptimeValue::Bool(l != r)),
                    _ => Err(CompileError::ComptimeError(
                        format!("Unsupported operation {:?} for String", op)
                    ))
                }
            }
            
            _ => Err(CompileError::ComptimeError(
                "Type mismatch in binary operation".to_string()
            ))
        }
    }
    
    /// Evaluate function calls
    fn evaluate_function_call(&mut self, name: &str, args: &[Expression]) -> Result<ComptimeValue> {
        // Check for built-in compile-time functions
        match name {
            "sizeof" => {
                // TODO: Implement sizeof
                Ok(ComptimeValue::I64(8))
            }
            
            "typeof" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "typeof expects exactly one argument".to_string()
                    ));
                }
                let val = self.evaluate_expression(&args[0])?;
                Ok(ComptimeValue::Type(val.get_type()))
            }
            
            "comptime_assert" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "comptime_assert expects exactly one argument".to_string()
                    ));
                }
                let val = self.evaluate_expression(&args[0])?;
                match val {
                    ComptimeValue::Bool(true) => Ok(ComptimeValue::Void),
                    ComptimeValue::Bool(false) => {
                        Err(CompileError::ComptimeError(
                            "Compile-time assertion failed".to_string()
                        ))
                    }
                    _ => Err(CompileError::ComptimeError(
                        "comptime_assert expects a boolean".to_string()
                    ))
                }
            }
            
            _ => {
                // Look up user-defined function
                if let Some(ComptimeValue::Function { params, body, closure, .. }) = self.env.get(name) {
                    // Create new environment for function execution
                    let func_env = Environment::with_parent(closure);
                    
                    // Bind arguments
                    if args.len() != params.len() {
                        return Err(CompileError::ComptimeError(
                            format!("Function {} expects {} arguments, got {}", 
                                    name, params.len(), args.len())
                        ));
                    }
                    
                    for (param, arg) in params.iter().zip(args) {
                        let val = self.evaluate_expression(arg)?;
                        func_env.define(param.clone(), val);
                    }
                    
                    // Execute function body
                    let saved_env = std::mem::replace(&mut self.env, func_env);
                    let mut result = ComptimeValue::Void;
                    
                    for stmt in &body {
                        if let Some(val) = self.execute_statement(stmt)? {
                            result = val;
                            break;
                        }
                    }
                    
                    self.env = saved_env;
                    Ok(result)
                } else {
                    Err(CompileError::ComptimeError(
                        format!("Unknown function: {}", name)
                    ))
                }
            }
        }
    }
    
    /// Evaluate member access
    fn evaluate_member_access(&mut self, object: ComptimeValue, member: &str) -> Result<ComptimeValue> {
        match object {
            ComptimeValue::Struct { fields, .. } => {
                fields.get(member).cloned()
                    .ok_or_else(|| CompileError::ComptimeError(
                        format!("Struct has no field: {}", member)
                    ))
            }
            _ => Err(CompileError::ComptimeError(
                format!("Cannot access member {} on non-struct value", member)
            ))
        }
    }
    
    /// Get any declarations generated during compile-time execution
    pub fn get_generated_declarations(&self) -> Vec<Declaration> {
        self.generated_declarations.clone()
    }
    
    /// Generate code from compile-time values
    pub fn generate_code(&mut self, value: ComptimeValue) -> Result<Expression> {
        value.to_expression()
    }
}