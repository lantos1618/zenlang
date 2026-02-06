// Compile-time execution framework for Zen
// This module provides an interpreter that executes Zen code during compilation

use crate::ast::{self, AstType, Declaration, Expression, Pattern, Statement};
use crate::error::{CompileError, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub mod meta;

// AST node wrapper for compile-time introspection.
// This enables Zen programs to walk and inspect the AST via meta.type_info().
#[derive(Debug, Clone)]
pub enum ASTNodeValue {
    Expression(Expression),
    Statement(Statement),
    Declaration(Declaration),
    Type(AstType),
    Pattern(Pattern),
    Program(ast::Program),
}

// Value types that can exist at compile time
#[derive(Debug, Clone)]
#[allow(dead_code)]
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

    // AST node (for meta-programming / AST walking from Zen code)
    ASTNode(Rc<ASTNodeValue>),

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
                let exprs: Result<Vec<_>> = values.iter().map(|v| v.to_expression()).collect();
                Ok(Expression::ArrayLiteral(exprs?))
            }
            ComptimeValue::Type(_t) => {
                // Type values become type annotations
                Err(CompileError::ComptimeError(
                    "Cannot convert type value to runtime expression".to_string(),
                    None,
                ))
            }
            ComptimeValue::ASTNode(node) => match node.as_ref() {
                ASTNodeValue::Expression(e) => Ok(e.clone()),
                other => Err(CompileError::ComptimeError(
                    format!(
                        "Cannot convert {:?} AST node to runtime expression",
                        std::mem::discriminant(other)
                    ),
                    None,
                )),
            },
            _ => Err(CompileError::ComptimeError(
                format!("Cannot convert {:?} to expression", self),
                None,
            )),
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
            ComptimeValue::String(_) => crate::ast::resolve_string_struct_type(),
            ComptimeValue::Array(v) => {
                if v.is_empty() {
                    AstType::Slice(Box::new(AstType::Void))
                } else {
                    AstType::Slice(Box::new(v[0].get_type()))
                }
            }
            ComptimeValue::Struct { name, .. } => AstType::Struct {
                name: name.clone(),
                fields: vec![], // Field types not tracked at comptime
            },
            ComptimeValue::Type(_) => {
                // Meta-type
                AstType::Generic {
                    name: "Type".to_string(),
                    type_args: vec![],
                }
            }
            ComptimeValue::Void => AstType::Void,
            ComptimeValue::Null => AstType::ptr(AstType::Void),
            ComptimeValue::Function { .. } => {
                // Opaque function type (full signature not tracked)
                AstType::Generic {
                    name: "Function".to_string(),
                    type_args: vec![],
                }
            }
            ComptimeValue::ASTNode(node) => {
                let variant = match node.as_ref() {
                    ASTNodeValue::Expression(_) => "Expression",
                    ASTNodeValue::Statement(_) => "Statement",
                    ASTNodeValue::Declaration(_) => "Declaration",
                    ASTNodeValue::Type(_) => "Type",
                    ASTNodeValue::Pattern(_) => "Pattern",
                    ASTNodeValue::Program(_) => "Program",
                };
                AstType::Generic {
                    name: "ASTNode".to_string(),
                    type_args: vec![AstType::Struct {
                        name: variant.to_string(),
                        fields: vec![],
                    }],
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

impl Default for Environment {
    fn default() -> Self {
        Environment {
            variables: Rc::new(RefCell::new(HashMap::new())),
            parent: None,
        }
    }
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
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
        self.variables
            .borrow()
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.get(name))
    }

    pub fn set(
        &self,
        name: &str,
        value: ComptimeValue,
        span: Option<crate::error::Span>,
    ) -> Result<()> {
        if self.variables.borrow().contains_key(name) {
            self.variables.borrow_mut().insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.set(name, value, span.clone())
        } else {
            Err(CompileError::ComptimeError(
                format!("Undefined variable: {}", name),
                span,
            ))
        }
    }
}

// Compile-time interpreter
#[allow(dead_code)]
pub struct ComptimeInterpreter {
    env: Environment,
    // Track generated code
    generated_declarations: Vec<Declaration>,
    // Track imports and modules
    modules: HashMap<String, ComptimeValue>,
}

impl Default for ComptimeInterpreter {
    fn default() -> Self {
        let mut interpreter = ComptimeInterpreter {
            env: Environment::new(),
            generated_declarations: Vec::new(),
            modules: HashMap::new(),
        };

        // Initialize built-in compile-time functions
        interpreter.init_builtins();
        interpreter
    }
}

impl ComptimeInterpreter {
    pub fn new() -> Self {
        Self::default()
    }

    // Helper methods for testing
    #[allow(dead_code)]
    pub fn set_variable(&mut self, name: String, value: ComptimeValue) {
        self.env.variables.borrow_mut().insert(name, value);
    }

    #[allow(dead_code)]
    pub fn get_variable(&self, name: &str) -> Option<ComptimeValue> {
        self.env.variables.borrow().get(name).cloned()
    }

    fn init_builtins(&mut self) {
        // @std namespace
        self.modules.insert(
            "@std".to_string(),
            ComptimeValue::Struct {
                name: "@std".to_string(),
                fields: {
                    let mut fields = HashMap::new();

                    // @std.core
                    fields.insert(
                        "core".to_string(),
                        ComptimeValue::Struct {
                            name: "core".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.io
                    fields.insert(
                        "io".to_string(),
                        ComptimeValue::Struct {
                            name: "io".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.vec
                    fields.insert(
                        "vec".to_string(),
                        ComptimeValue::Struct {
                            name: "vec".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.hashmap
                    fields.insert(
                        "hashmap".to_string(),
                        ComptimeValue::Struct {
                            name: "hashmap".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.string
                    fields.insert(
                        "string".to_string(),
                        ComptimeValue::Struct {
                            name: "string".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.math
                    fields.insert(
                        "math".to_string(),
                        ComptimeValue::Struct {
                            name: "math".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.lexer
                    fields.insert(
                        "lexer".to_string(),
                        ComptimeValue::Struct {
                            name: "lexer".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.parser
                    fields.insert(
                        "parser".to_string(),
                        ComptimeValue::Struct {
                            name: "parser".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.ast
                    fields.insert(
                        "ast".to_string(),
                        ComptimeValue::Struct {
                            name: "ast".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.type_checker
                    fields.insert(
                        "type_checker".to_string(),
                        ComptimeValue::Struct {
                            name: "type_checker".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.codegen
                    fields.insert(
                        "codegen".to_string(),
                        ComptimeValue::Struct {
                            name: "codegen".to_string(),
                            fields: HashMap::new(),
                        },
                    );

                    // @std.build
                    fields.insert(
                        "build".to_string(),
                        ComptimeValue::Struct {
                            name: "build".to_string(),
                            fields: {
                                let mut build_fields = HashMap::new();
                                // build.import function
                                build_fields.insert(
                                    "import".to_string(),
                                    ComptimeValue::Function {
                                        name: "import".to_string(),
                                        params: vec!["module_name".to_string()],
                                        body: vec![],
                                        closure: Environment::new(),
                                    },
                                );
                                build_fields
                            },
                        },
                    );

                    fields
                },
            },
        );
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
            Statement::VariableDeclaration {
                name,
                initializer,
                span,
                ..
            } => {
                if let Some(init) = initializer {
                    let value = self.evaluate_expression(init, span.clone())?;
                    self.env.define(name.clone(), value);
                }
                Ok(None)
            }

            Statement::VariableAssignment { name, value, span } => {
                let val = self.evaluate_expression(value, span.clone())?;
                self.env.set(name, val, span.clone())?;
                Ok(None)
            }

            Statement::Expression { expr, span } => {
                let value = self.evaluate_expression(expr, span.clone())?;
                Ok(Some(value))
            }

            Statement::Return { expr, span } => {
                let value = self.evaluate_expression(expr, span.clone())?;
                Ok(Some(value))
            }

            Statement::ComptimeBlock {
                statements: stmts, ..
            } => {
                // Nested comptime block
                self.execute_comptime_block(stmts)?;
                Ok(None)
            }

            _ => Err(CompileError::ComptimeError(
                format!("Statement type not supported in comptime: {:?}", stmt),
                None,
            )),
        }
    }

    /// Evaluate an expression to a compile-time value
    pub fn evaluate_expression(
        &mut self,
        expr: &Expression,
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
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

                self.env.get(name).ok_or_else(|| {
                    CompileError::ComptimeError(
                        format!("Undefined identifier: {}", name),
                        span.clone(),
                    )
                })
            }

            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression(left, span.clone())?;
                let right_val = self.evaluate_expression(right, span.clone())?;
                self.evaluate_binary_op(left_val, op, right_val, span.clone())
            }

            Expression::FunctionCall { name, args, .. } => {
                self.evaluate_function_call(name, args, span.clone())
            }

            Expression::ArrayLiteral(elements) => {
                let values: Result<Vec<_>> = elements
                    .iter()
                    .map(|e| self.evaluate_expression(e, span.clone()))
                    .collect();
                Ok(ComptimeValue::Array(values?))
            }

            Expression::MemberAccess { object, member } => {
                let obj_val = self.evaluate_expression(object, span.clone())?;
                self.evaluate_member_access(obj_val, member, span.clone())
            }

            Expression::Comptime(inner) => {
                // Nested comptime expression
                self.evaluate_expression(inner, span.clone())
            }

            Expression::Range {
                start,
                end,
                inclusive,
            } => {
                let start_val = self.evaluate_expression(start, span.clone())?;
                let end_val = self.evaluate_expression(end, span.clone())?;

                match (start_val, end_val) {
                    (ComptimeValue::I32(start_i), ComptimeValue::I32(end_i)) => {
                        let end_val = if *inclusive {
                            end_i.checked_add(1).ok_or_else(|| {
                                CompileError::ComptimeError(
                                    "Inclusive range end overflows i32".to_string(),
                                    span.clone(),
                                )
                            })?
                        } else {
                            end_i
                        };

                        // Prevent memory exhaustion from huge ranges
                        const MAX_COMPTIME_RANGE: i32 = 100_000;
                        let range_size = end_val.saturating_sub(start_i);
                        if range_size > MAX_COMPTIME_RANGE {
                            return Err(CompileError::ComptimeError(
                                format!(
                                    "Compile-time range too large: {} elements (max {})",
                                    range_size, MAX_COMPTIME_RANGE
                                ),
                                span.clone(),
                            ));
                        }

                        let mut values = Vec::with_capacity(range_size.max(0) as usize);
                        for i in start_i..end_val {
                            values.push(ComptimeValue::I32(i));
                        }

                        Ok(ComptimeValue::Array(values))
                    }
                    _ => Err(CompileError::ComptimeError(
                        "Range expressions only support integer bounds".to_string(),
                        span.clone(),
                    )),
                }
            }

            _ => Err(CompileError::ComptimeError(
                format!("Expression type not supported in comptime: {:?}", expr),
                span.clone(),
            )),
        }
    }

    /// Evaluate binary operations
    fn evaluate_binary_op(
        &self,
        left: ComptimeValue,
        op: &ast::BinaryOperator,
        right: ComptimeValue,
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        use ast::BinaryOperator;

        match (left, right) {
            (ComptimeValue::I32(l), ComptimeValue::I32(r)) => match op {
                BinaryOperator::Add => Ok(ComptimeValue::I32(l + r)),
                BinaryOperator::Subtract => Ok(ComptimeValue::I32(l - r)),
                BinaryOperator::Multiply => Ok(ComptimeValue::I32(l * r)),
                BinaryOperator::Divide => {
                    if r == 0 {
                        Err(CompileError::ComptimeError(
                            "Division by zero".to_string(),
                            span,
                        ))
                    } else {
                        Ok(ComptimeValue::I32(l / r))
                    }
                }
                BinaryOperator::Equals => Ok(ComptimeValue::Bool(l == r)),
                BinaryOperator::NotEquals => Ok(ComptimeValue::Bool(l != r)),
                BinaryOperator::LessThan => Ok(ComptimeValue::Bool(l < r)),
                BinaryOperator::LessThanEquals => Ok(ComptimeValue::Bool(l <= r)),
                BinaryOperator::GreaterThan => Ok(ComptimeValue::Bool(l > r)),
                BinaryOperator::GreaterThanEquals => Ok(ComptimeValue::Bool(l >= r)),
                _ => Err(CompileError::ComptimeError(
                    format!("Unsupported operation {:?} for I32", op),
                    span,
                )),
            },

            (ComptimeValue::Bool(l), ComptimeValue::Bool(r)) => match op {
                BinaryOperator::And => Ok(ComptimeValue::Bool(l && r)),
                BinaryOperator::Or => Ok(ComptimeValue::Bool(l || r)),
                BinaryOperator::Equals => Ok(ComptimeValue::Bool(l == r)),
                BinaryOperator::NotEquals => Ok(ComptimeValue::Bool(l != r)),
                _ => Err(CompileError::ComptimeError(
                    format!("Unsupported operation {:?} for Bool", op),
                    span,
                )),
            },

            (ComptimeValue::String(l), ComptimeValue::String(r)) => match op {
                BinaryOperator::Equals => Ok(ComptimeValue::Bool(l == r)),
                BinaryOperator::NotEquals => Ok(ComptimeValue::Bool(l != r)),
                _ => Err(CompileError::ComptimeError(
                    format!("Unsupported operation {:?} for String", op),
                    span,
                )),
            },

            _ => Err(CompileError::ComptimeError(
                "Type mismatch in binary operation".to_string(),
                span,
            )),
        }
    }

    /// Evaluate function calls
    fn evaluate_function_call(
        &mut self,
        name: &str,
        args: &[Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        // Check for built-in compile-time functions
        match name {
            "sizeof" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "sizeof expects exactly one argument".to_string(),
                        span,
                    ));
                }
                // Evaluate expression to determine its type
                let val = self.evaluate_expression(&args[0], span.clone())?;
                let arg_type = val.get_type();
                // Return size in bytes for the type
                let size = match &arg_type {
                    AstType::I8 | AstType::U8 | AstType::Bool => 1,
                    AstType::I16 | AstType::U16 => 2,
                    AstType::I32 | AstType::U32 | AstType::F32 => 4,
                    AstType::I64 | AstType::U64 | AstType::F64 | AstType::Usize => 8,
                    // Pointers and references are 8 bytes on 64-bit systems
                    t if t.is_ptr_type() => 8,
                    AstType::Ref(_) => 8,
                    // Default to pointer size for unknown types
                    _ => 8,
                };
                Ok(ComptimeValue::I64(size))
            }

            "typeof" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "typeof expects exactly one argument".to_string(),
                        span,
                    ));
                }
                let val = self.evaluate_expression(&args[0], span.clone())?;
                Ok(ComptimeValue::Type(val.get_type()))
            }

            "comptime_assert" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "comptime_assert expects exactly one argument".to_string(),
                        span.clone(),
                    ));
                }
                let val = self.evaluate_expression(&args[0], span.clone())?;
                match val {
                    ComptimeValue::Bool(true) => Ok(ComptimeValue::Void),
                    ComptimeValue::Bool(false) => Err(CompileError::ComptimeError(
                        "Compile-time assertion failed".to_string(),
                        span.clone(),
                    )),
                    _ => Err(CompileError::ComptimeError(
                        "comptime_assert expects a boolean".to_string(),
                        span,
                    )),
                }
            }

            _ => {
                // Look up user-defined function
                if let Some(ComptimeValue::Function {
                    params,
                    body,
                    closure,
                    ..
                }) = self.env.get(name)
                {
                    // Create new environment for function execution
                    let func_env = Environment::with_parent(closure);

                    // Bind arguments
                    if args.len() != params.len() {
                        return Err(CompileError::ComptimeError(
                            format!(
                                "Function {} expects {} arguments, got {}",
                                name,
                                params.len(),
                                args.len()
                            ),
                            span.clone(),
                        ));
                    }

                    for (param, arg) in params.iter().zip(args) {
                        let val = self.evaluate_expression(arg, span.clone())?;
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
                        format!("Unknown function: {}", name),
                        span,
                    ))
                }
            }
        }
    }

    /// Evaluate member access
    fn evaluate_member_access(
        &mut self,
        object: ComptimeValue,
        member: &str,
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        match object {
            ComptimeValue::Struct { fields, .. } => fields.get(member).cloned().ok_or_else(|| {
                CompileError::ComptimeError(
                    format!("Struct has no field: {}", member),
                    span.clone(),
                )
            }),
            _ => Err(CompileError::ComptimeError(
                format!("Cannot access member {} on non-struct value", member),
                span,
            )),
        }
    }

    /// Get any declarations generated during compile-time execution
    #[allow(dead_code)]
    pub fn get_generated_declarations(&self) -> Vec<Declaration> {
        self.generated_declarations.clone()
    }

    /// Generate code from compile-time values
    #[allow(dead_code)]
    pub fn generate_code(&mut self, value: ComptimeValue) -> Result<Expression> {
        value.to_expression()
    }
}
