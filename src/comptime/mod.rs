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

// Manual PartialEq: compare structurally, Functions/ASTNodes compare by discriminant only
impl PartialEq for ComptimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ComptimeValue::I8(a), ComptimeValue::I8(b)) => a == b,
            (ComptimeValue::I16(a), ComptimeValue::I16(b)) => a == b,
            (ComptimeValue::I32(a), ComptimeValue::I32(b)) => a == b,
            (ComptimeValue::I64(a), ComptimeValue::I64(b)) => a == b,
            (ComptimeValue::U8(a), ComptimeValue::U8(b)) => a == b,
            (ComptimeValue::U16(a), ComptimeValue::U16(b)) => a == b,
            (ComptimeValue::U32(a), ComptimeValue::U32(b)) => a == b,
            (ComptimeValue::U64(a), ComptimeValue::U64(b)) => a == b,
            (ComptimeValue::F32(a), ComptimeValue::F32(b)) => a.to_bits() == b.to_bits(),
            (ComptimeValue::F64(a), ComptimeValue::F64(b)) => a.to_bits() == b.to_bits(),
            (ComptimeValue::Bool(a), ComptimeValue::Bool(b)) => a == b,
            (ComptimeValue::String(a), ComptimeValue::String(b)) => a == b,
            (ComptimeValue::Array(a), ComptimeValue::Array(b)) => a == b,
            (
                ComptimeValue::Struct {
                    name: n1,
                    fields: f1,
                },
                ComptimeValue::Struct {
                    name: n2,
                    fields: f2,
                },
            ) => n1 == n2 && f1 == f2,
            (ComptimeValue::Type(a), ComptimeValue::Type(b)) => a == b,
            (ComptimeValue::Void, ComptimeValue::Void) => true,
            (ComptimeValue::Null, ComptimeValue::Null) => true,
            _ => false,
        }
    }
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

                    // @std.meta (compile-time AST introspection)
                    fields.insert(
                        "meta".to_string(),
                        ComptimeValue::Struct {
                            name: "meta".to_string(),
                            fields: HashMap::new(), // intrinsics dispatched by name
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

            Statement::DestructuringImport { names, source, .. } => {
                // Handle { meta } = @std style imports in comptime
                let source_val = self.evaluate_expression(source, None)?;
                if let ComptimeValue::Struct { fields, .. } = source_val {
                    for name in names {
                        if let Some(val) = fields.get(name) {
                            self.env.define(name.clone(), val.clone());
                        } else {
                            return Err(CompileError::ComptimeError(
                                format!("Module has no member '{}'", name),
                                None,
                            ));
                        }
                    }
                }
                Ok(None)
            }

            Statement::Block { statements, .. } => {
                let mut result = None;
                for s in statements {
                    result = self.execute_statement(s)?;
                }
                Ok(result)
            }

            Statement::Loop { kind, body, .. } => self.execute_loop(kind, body),

            Statement::Break { .. } => {
                Err(CompileError::ComptimeError("__break__".to_string(), None))
            }

            Statement::Continue { .. } => Err(CompileError::ComptimeError(
                "__continue__".to_string(),
                None,
            )),

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

            Expression::MethodCall {
                object,
                method,
                args,
                ..
            } => {
                let obj_val = self.evaluate_expression(object, span.clone())?;
                self.evaluate_method_call(obj_val, method, args, span.clone())
            }

            Expression::StdReference => {
                // Return the @std module
                self.modules.get("@std").cloned().ok_or_else(|| {
                    CompileError::ComptimeError("@std module not available".to_string(), span)
                })
            }

            Expression::Comptime(inner) => {
                // Nested comptime expression
                self.evaluate_expression(inner, span.clone())
            }

            // Block expression: { stmt; stmt; expr }
            Expression::Block(statements) => {
                let child_env = Environment::with_parent(self.env.clone());
                let saved_env = std::mem::replace(&mut self.env, child_env);
                let mut result = ComptimeValue::Void;
                for stmt in statements {
                    match self.execute_statement(stmt) {
                        Ok(Some(val)) => {
                            result = val;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            // Propagate break/continue signals through blocks
                            self.env = saved_env;
                            return Err(e);
                        }
                    }
                }
                self.env = saved_env;
                Ok(result)
            }

            // Pattern matching: scrutinee ? | pattern { body } | pattern2 { body2 }
            Expression::QuestionMatch { scrutinee, arms } => {
                let scrutinee_val = self.evaluate_expression(scrutinee, span.clone())?;
                self.evaluate_question_match(scrutinee_val, arms, span.clone())
            }

            // Array indexing: arr[i]
            Expression::ArrayIndex { array, index } => {
                let arr_val = self.evaluate_expression(array, span.clone())?;
                let idx_val = self.evaluate_expression(index, span.clone())?;
                match (&arr_val, &idx_val) {
                    (ComptimeValue::Array(items), ComptimeValue::I32(i)) => {
                        let idx = *i as usize;
                        if idx < items.len() {
                            Ok(items[idx].clone())
                        } else {
                            Err(CompileError::ComptimeError(
                                format!("Index {} out of bounds (len: {})", idx, items.len()),
                                span,
                            ))
                        }
                    }
                    (ComptimeValue::Array(items), ComptimeValue::I64(i)) => {
                        let idx = *i as usize;
                        if idx < items.len() {
                            Ok(items[idx].clone())
                        } else {
                            Err(CompileError::ComptimeError(
                                format!("Index {} out of bounds (len: {})", idx, items.len()),
                                span,
                            ))
                        }
                    }
                    _ => Err(CompileError::ComptimeError(
                        format!(
                            "Cannot index {:?} with {:?}",
                            arr_val.get_type(),
                            idx_val.get_type()
                        ),
                        span,
                    )),
                }
            }

            // String interpolation: "Hello ${name}!"
            Expression::StringInterpolation { parts } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        ast::StringPart::Literal(s) => result.push_str(s),
                        ast::StringPart::Interpolation(e) => {
                            let val = self.evaluate_expression(e, span.clone())?;
                            match val {
                                ComptimeValue::String(s) => result.push_str(&s),
                                ComptimeValue::I32(n) => result.push_str(&n.to_string()),
                                ComptimeValue::I64(n) => result.push_str(&n.to_string()),
                                ComptimeValue::F32(n) => result.push_str(&n.to_string()),
                                ComptimeValue::F64(n) => result.push_str(&n.to_string()),
                                ComptimeValue::Bool(b) => result.push_str(&b.to_string()),
                                ComptimeValue::Null => result.push_str("null"),
                                other => result.push_str(&format!("{:?}", other)),
                            }
                        }
                    }
                }
                Ok(ComptimeValue::String(result))
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
                BinaryOperator::Add => Ok(ComptimeValue::String(format!("{}{}", l, r))),
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

    /// Execute a loop statement (loop condition { body })
    fn execute_loop(
        &mut self,
        kind: &ast::LoopKind,
        body: &[Statement],
    ) -> Result<Option<ComptimeValue>> {
        const MAX_ITERATIONS: usize = 100_000;
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err(CompileError::ComptimeError(
                    format!(
                        "Compile-time loop exceeded {} iterations (infinite loop?)",
                        MAX_ITERATIONS
                    ),
                    None,
                ));
            }

            // Check loop condition
            if let ast::LoopKind::Condition(cond) = kind {
                let cond_val = self.evaluate_expression(cond, None)?;
                match cond_val {
                    ComptimeValue::Bool(false) => return Ok(None),
                    ComptimeValue::Bool(true) => {}
                    _ => {
                        return Err(CompileError::ComptimeError(
                            "Loop condition must evaluate to a boolean".to_string(),
                            None,
                        ))
                    }
                }
            }

            // Execute body
            let mut should_break = false;
            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(Some(val)) => {
                        // Only Return statements should exit the loop
                        if matches!(stmt, Statement::Return { .. }) {
                            return Ok(Some(val));
                        }
                        // Otherwise, expression result is discarded in loop body
                    }
                    Ok(None) => {}
                    Err(e) => {
                        // Check for break/continue signals
                        if let CompileError::ComptimeError(msg, _) = &e {
                            if msg == "__break__" {
                                should_break = true;
                                break;
                            }
                            if msg == "__continue__" {
                                break;
                            }
                        }
                        return Err(e);
                    }
                }
            }

            if should_break {
                return Ok(None);
            }
        }
    }

    /// Evaluate pattern matching (QuestionMatch): scrutinee ? | pattern { body }
    fn evaluate_question_match(
        &mut self,
        scrutinee: ComptimeValue,
        arms: &[ast::MatchArm],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        for arm in arms {
            if let Some(value) = self.match_pattern(&scrutinee, &arm.pattern)? {
                // Bind pattern variables
                let child_env = Environment::with_parent(self.env.clone());
                for (name, val) in &value {
                    child_env.define(name.clone(), val.clone());
                }

                // Check guard if present
                if let Some(guard) = &arm.guard {
                    let saved_env = std::mem::replace(&mut self.env, child_env);
                    let guard_val = self.evaluate_expression(guard, span.clone())?;
                    let env_after = std::mem::replace(&mut self.env, saved_env);
                    match guard_val {
                        ComptimeValue::Bool(true) => {
                            // Guard passed, evaluate body
                            let saved_env = std::mem::replace(&mut self.env, env_after);
                            let result = self.evaluate_expression(&arm.body, span.clone())?;
                            self.env = saved_env;
                            return Ok(result);
                        }
                        ComptimeValue::Bool(false) => continue,
                        _ => {
                            return Err(CompileError::ComptimeError(
                                "Guard condition must be boolean".to_string(),
                                span,
                            ))
                        }
                    }
                }

                // No guard, evaluate body
                let saved_env = std::mem::replace(&mut self.env, child_env);
                let result = self.evaluate_expression(&arm.body, span.clone())?;
                self.env = saved_env;
                return Ok(result);
            }
        }

        // No arm matched
        Err(CompileError::ComptimeError(
            "Non-exhaustive pattern match: no arm matched".to_string(),
            span,
        ))
    }

    /// Try to match a comptime value against a pattern.
    /// Returns Some(bindings) if matched, None if not.
    fn match_pattern(
        &mut self,
        value: &ComptimeValue,
        pattern: &Pattern,
    ) -> Result<Option<Vec<(String, ComptimeValue)>>> {
        match pattern {
            Pattern::Wildcard => Ok(Some(vec![])),

            Pattern::Identifier(name) => {
                // Binding pattern: always matches, binds the value
                Ok(Some(vec![(name.clone(), value.clone())]))
            }

            Pattern::Literal(expr) => {
                // Evaluate the pattern literal and compare
                let pat_val = self.evaluate_expression(expr, None)?;
                if value == &pat_val {
                    Ok(Some(vec![]))
                } else {
                    Ok(None)
                }
            }

            Pattern::Type { type_name, binding } => {
                // Match on type name (used for string-based variant matching)
                let matches = matches!(
                    (type_name.as_str(), value),
                    ("true", ComptimeValue::Bool(true))
                        | ("false", ComptimeValue::Bool(false))
                        | ("i32", ComptimeValue::I32(_))
                        | ("i64", ComptimeValue::I64(_))
                        | ("f32", ComptimeValue::F32(_))
                        | ("f64", ComptimeValue::F64(_))
                        | ("String", ComptimeValue::String(_))
                        | ("bool", ComptimeValue::Bool(_))
                );

                if matches {
                    let mut bindings = vec![];
                    if let Some(bind_name) = binding {
                        bindings.push((bind_name.clone(), value.clone()));
                    }
                    Ok(Some(bindings))
                } else {
                    Ok(None)
                }
            }

            Pattern::EnumLiteral { variant, payload } => {
                // Match enum-style: .Some(val), .None
                // Also used for string-match dispatch like | "Function" { ... }
                match value {
                    ComptimeValue::String(s) if s == variant => {
                        // String matches variant name
                        Ok(Some(vec![]))
                    }
                    _ => {
                        // Try struct-based enum matching
                        if let ComptimeValue::Struct { name, fields } = value {
                            if name == variant
                                || fields
                                    .get("variant")
                                    .map(|v| {
                                        if let ComptimeValue::String(s) = v {
                                            s == variant
                                        } else {
                                            false
                                        }
                                    })
                                    .unwrap_or(false)
                            {
                                let mut bindings = vec![];
                                if let Some(payload_pat) = payload {
                                    if let Some(inner) = fields.get("payload") {
                                        if let Some(b) = self.match_pattern(inner, payload_pat)? {
                                            bindings.extend(b);
                                        } else {
                                            return Ok(None);
                                        }
                                    }
                                }
                                return Ok(Some(bindings));
                            }
                        }
                        Ok(None)
                    }
                }
            }

            Pattern::Or(patterns) => {
                for pat in patterns {
                    if let Some(bindings) = self.match_pattern(value, pat)? {
                        return Ok(Some(bindings));
                    }
                }
                Ok(None)
            }

            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let start_val = self.evaluate_expression(start, None)?;
                let end_val = self.evaluate_expression(end, None)?;
                match (value, &start_val, &end_val) {
                    (ComptimeValue::I32(v), ComptimeValue::I32(s), ComptimeValue::I32(e)) => {
                        let in_range = if *inclusive {
                            v >= s && v <= e
                        } else {
                            v >= s && v < e
                        };
                        Ok(if in_range {
                            Some(vec![])
                        } else {
                            None
                        })
                    }
                    _ => Ok(None),
                }
            }

            Pattern::Guard { pattern, condition } => {
                if let Some(bindings) = self.match_pattern(value, pattern)? {
                    // Temporarily bind pattern variables for guard evaluation
                    let child_env = Environment::with_parent(self.env.clone());
                    for (name, val) in &bindings {
                        child_env.define(name.clone(), val.clone());
                    }
                    let saved_env = std::mem::replace(&mut self.env, child_env);
                    let guard_result = self.evaluate_expression(condition, None)?;
                    self.env = saved_env;

                    match guard_result {
                        ComptimeValue::Bool(true) => Ok(Some(bindings)),
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }

            _ => {
                // Unsupported pattern type
                Err(CompileError::ComptimeError(
                    format!("Pattern type not yet supported in comptime: {:?}", pattern),
                    None,
                ))
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
        match &object {
            ComptimeValue::Struct { fields, .. } => fields.get(member).cloned().ok_or_else(|| {
                CompileError::ComptimeError(
                    format!("Struct has no field: {}", member),
                    span.clone(),
                )
            }),
            ComptimeValue::ASTNode(node) => {
                // Field access on AST nodes: node.name, node.left, etc.
                let flds = meta::fields(node)?;
                for f in &flds {
                    if let ComptimeValue::Struct { fields: ff, .. } = f {
                        if let Some(ComptimeValue::String(name)) = ff.get("name") {
                            if name == member {
                                return ff.get("value").cloned().ok_or_else(|| {
                                    CompileError::ComptimeError(
                                        format!("AST field '{}' has no value", member),
                                        span.clone(),
                                    )
                                });
                            }
                        }
                    }
                }
                Err(CompileError::ComptimeError(
                    format!(
                        "AST node '{}' has no field '{}'",
                        meta::variant_name(node),
                        member
                    ),
                    span,
                ))
            }
            _ => Err(CompileError::ComptimeError(
                format!("Cannot access member {} on non-struct value", member),
                span,
            )),
        }
    }

    /// Evaluate method calls (UFC-style: object.method(args))
    fn evaluate_method_call(
        &mut self,
        object: ComptimeValue,
        method: &str,
        args: &[Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        // Check if the object is the meta module
        if let ComptimeValue::Struct { name, .. } = &object {
            if name == "meta" {
                return self.evaluate_meta_intrinsic(method, args, span);
            }
        }

        // Check for ASTNode method calls
        if let ComptimeValue::ASTNode(ref node) = object {
            return self.evaluate_ast_node_method(node, method, args, span);
        }

        // Array methods
        if let ComptimeValue::Array(ref items) = object {
            return self.evaluate_array_method(items, method, args, span);
        }

        // String methods
        if let ComptimeValue::String(ref s) = object {
            return self.evaluate_string_method(s, method, args, span);
        }

        Err(CompileError::ComptimeError(
            format!("Cannot call method '{}' on {:?}", method, object.get_type()),
            span,
        ))
    }

    /// Evaluate meta module intrinsic function calls
    fn evaluate_meta_intrinsic(
        &mut self,
        method: &str,
        args: &[Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        match method {
            "type_info" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "meta.type_info() expects exactly 1 argument".to_string(),
                        span,
                    ));
                }
                let val = self.evaluate_expression(&args[0], span.clone())?;
                match val {
                    ComptimeValue::ASTNode(ref node) => meta::type_info(node),
                    _ => Err(CompileError::ComptimeError(
                        "meta.type_info() expects an ASTNode argument".to_string(),
                        span,
                    )),
                }
            }

            "fields" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "meta.fields() expects exactly 1 argument".to_string(),
                        span,
                    ));
                }
                let val = self.evaluate_expression(&args[0], span.clone())?;
                match val {
                    ComptimeValue::ASTNode(ref node) => {
                        Ok(ComptimeValue::Array(meta::fields(node)?))
                    }
                    _ => Err(CompileError::ComptimeError(
                        "meta.fields() expects an ASTNode argument".to_string(),
                        span,
                    )),
                }
            }

            "variant_name" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "meta.variant_name() expects exactly 1 argument".to_string(),
                        span,
                    ));
                }
                let val = self.evaluate_expression(&args[0], span.clone())?;
                match val {
                    ComptimeValue::ASTNode(ref node) => {
                        Ok(ComptimeValue::String(meta::variant_name(node)))
                    }
                    _ => Err(CompileError::ComptimeError(
                        "meta.variant_name() expects an ASTNode argument".to_string(),
                        span,
                    )),
                }
            }

            "children" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "meta.children() expects exactly 1 argument".to_string(),
                        span,
                    ));
                }
                let val = self.evaluate_expression(&args[0], span.clone())?;
                match val {
                    ComptimeValue::ASTNode(ref node) => {
                        Ok(ComptimeValue::Array(meta::children(node)?))
                    }
                    _ => Err(CompileError::ComptimeError(
                        "meta.children() expects an ASTNode argument".to_string(),
                        span,
                    )),
                }
            }

            "parse" => {
                // meta.parse("2 + 3") -> ASTNode(Expression)
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "meta.parse() expects exactly 1 argument".to_string(),
                        span,
                    ));
                }
                let val = self.evaluate_expression(&args[0], span.clone())?;
                match val {
                    ComptimeValue::String(source) => {
                        // Parse the string as a Zen expression
                        let lexer = crate::lexer::Lexer::new(&source);
                        let mut parser = crate::parser::Parser::new(lexer);
                        let program = parser.parse_program().map_err(|e| {
                            CompileError::ComptimeError(
                                format!("meta.parse() failed: {}", e),
                                span.clone(),
                            )
                        })?;
                        Ok(ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Program(
                            program,
                        ))))
                    }
                    _ => Err(CompileError::ComptimeError(
                        "meta.parse() expects a string argument".to_string(),
                        span,
                    )),
                }
            }

            _ => Err(CompileError::ComptimeError(
                format!("Unknown meta intrinsic: meta.{}()", method),
                span,
            )),
        }
    }

    /// Evaluate methods on ASTNode values
    fn evaluate_ast_node_method(
        &mut self,
        node: &ASTNodeValue,
        method: &str,
        args: &[Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        match method {
            // Core introspection
            "type_info" => meta::type_info(node),
            "fields" => Ok(ComptimeValue::Array(meta::fields(node)?)),
            "variant_name" => Ok(ComptimeValue::String(meta::variant_name(node))),
            "children" => Ok(ComptimeValue::Array(meta::children(node)?)),

            // Navigation helpers: Program-level
            "functions" => {
                if let ASTNodeValue::Program(prog) = node {
                    Ok(ComptimeValue::Array(
                        prog.declarations
                            .iter()
                            .filter(|d| matches!(d, Declaration::Function(_)))
                            .map(|d| {
                                ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Declaration(
                                    d.clone(),
                                )))
                            })
                            .collect(),
                    ))
                } else {
                    Err(CompileError::ComptimeError(
                        "functions() only works on Program nodes".to_string(),
                        span,
                    ))
                }
            }

            "structs" => {
                if let ASTNodeValue::Program(prog) = node {
                    Ok(ComptimeValue::Array(
                        prog.declarations
                            .iter()
                            .filter(|d| matches!(d, Declaration::Struct(_)))
                            .map(|d| {
                                ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Declaration(
                                    d.clone(),
                                )))
                            })
                            .collect(),
                    ))
                } else {
                    Err(CompileError::ComptimeError(
                        "structs() only works on Program nodes".to_string(),
                        span,
                    ))
                }
            }

            "enums" => {
                if let ASTNodeValue::Program(prog) = node {
                    Ok(ComptimeValue::Array(
                        prog.declarations
                            .iter()
                            .filter(|d| matches!(d, Declaration::Enum(_)))
                            .map(|d| {
                                ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Declaration(
                                    d.clone(),
                                )))
                            })
                            .collect(),
                    ))
                } else {
                    Err(CompileError::ComptimeError(
                        "enums() only works on Program nodes".to_string(),
                        span,
                    ))
                }
            }

            "find_function" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "find_function() expects 1 argument (name)".to_string(),
                        span,
                    ));
                }
                let name_val = self.evaluate_expression(&args[0], span.clone())?;
                let target_name = match name_val {
                    ComptimeValue::String(s) => s,
                    _ => {
                        return Err(CompileError::ComptimeError(
                            "find_function() expects a string argument".to_string(),
                            span,
                        ))
                    }
                };

                if let ASTNodeValue::Program(prog) = node {
                    for d in &prog.declarations {
                        if let Declaration::Function(f) = d {
                            if f.name == target_name {
                                return Ok(ComptimeValue::ASTNode(Rc::new(
                                    ASTNodeValue::Declaration(d.clone()),
                                )));
                            }
                        }
                    }
                    Ok(ComptimeValue::Null)
                } else {
                    Err(CompileError::ComptimeError(
                        "find_function() only works on Program nodes".to_string(),
                        span,
                    ))
                }
            }

            "find_struct" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "find_struct() expects 1 argument (name)".to_string(),
                        span,
                    ));
                }
                let name_val = self.evaluate_expression(&args[0], span.clone())?;
                let target_name = match name_val {
                    ComptimeValue::String(s) => s,
                    _ => {
                        return Err(CompileError::ComptimeError(
                            "find_struct() expects a string argument".to_string(),
                            span,
                        ))
                    }
                };

                if let ASTNodeValue::Program(prog) = node {
                    for d in &prog.declarations {
                        if let Declaration::Struct(s) = d {
                            if s.name == target_name {
                                return Ok(ComptimeValue::ASTNode(Rc::new(
                                    ASTNodeValue::Declaration(d.clone()),
                                )));
                            }
                        }
                    }
                    Ok(ComptimeValue::Null)
                } else {
                    Err(CompileError::ComptimeError(
                        "find_struct() only works on Program nodes".to_string(),
                        span,
                    ))
                }
            }

            // General find_by_variant: node.find_by_variant("BinaryOp")
            "find_by_variant" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "find_by_variant() expects 1 argument (variant name)".to_string(),
                        span,
                    ));
                }
                let name_val = self.evaluate_expression(&args[0], span.clone())?;
                let target = match name_val {
                    ComptimeValue::String(s) => s,
                    _ => {
                        return Err(CompileError::ComptimeError(
                            "find_by_variant() expects a string argument".to_string(),
                            span,
                        ))
                    }
                };

                let children = meta::children(node)?;
                let mut results = Vec::new();
                Self::collect_by_variant(&children, &target, &mut results);
                Ok(ComptimeValue::Array(results))
            }

            // is_* type checks
            "is_expression" => Ok(ComptimeValue::Bool(matches!(
                node,
                ASTNodeValue::Expression(_)
            ))),
            "is_statement" => Ok(ComptimeValue::Bool(matches!(
                node,
                ASTNodeValue::Statement(_)
            ))),
            "is_declaration" => Ok(ComptimeValue::Bool(matches!(
                node,
                ASTNodeValue::Declaration(_)
            ))),
            "is_type" => Ok(ComptimeValue::Bool(matches!(node, ASTNodeValue::Type(_)))),
            "is_pattern" => Ok(ComptimeValue::Bool(matches!(
                node,
                ASTNodeValue::Pattern(_)
            ))),

            _ => Err(CompileError::ComptimeError(
                format!("ASTNode has no method '{}'", method),
                span,
            )),
        }
    }

    /// Recursively collect all AST nodes matching a variant name
    fn collect_by_variant(
        values: &[ComptimeValue],
        target: &str,
        results: &mut Vec<ComptimeValue>,
    ) {
        for val in values {
            if let ComptimeValue::ASTNode(node) = val {
                if meta::variant_name(node) == target {
                    results.push(val.clone());
                }
                // Recurse into children
                if let Ok(children) = meta::children(node) {
                    Self::collect_by_variant(&children, target, results);
                }
            }
        }
    }

    /// Evaluate array methods
    fn evaluate_array_method(
        &mut self,
        items: &[ComptimeValue],
        method: &str,
        args: &[Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        match method {
            "len" => Ok(ComptimeValue::I64(items.len() as i64)),

            "first" => {
                if items.is_empty() {
                    Ok(ComptimeValue::Null)
                } else {
                    Ok(items[0].clone())
                }
            }

            "last" => {
                if items.is_empty() {
                    Ok(ComptimeValue::Null)
                } else {
                    Ok(items[items.len() - 1].clone())
                }
            }

            "is_empty" => Ok(ComptimeValue::Bool(items.is_empty())),

            "filter_by_variant" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "filter_by_variant() expects 1 argument".to_string(),
                        span,
                    ));
                }
                let name_val = self.evaluate_expression(&args[0], span.clone())?;
                let target = match name_val {
                    ComptimeValue::String(s) => s,
                    _ => {
                        return Err(CompileError::ComptimeError(
                            "filter_by_variant() expects a string argument".to_string(),
                            span,
                        ))
                    }
                };
                Ok(ComptimeValue::Array(
                    items
                        .iter()
                        .filter(|item| {
                            if let ComptimeValue::ASTNode(node) = item {
                                meta::variant_name(node) == target
                            } else {
                                false
                            }
                        })
                        .cloned()
                        .collect(),
                ))
            }

            "at" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "at() expects 1 argument (index)".to_string(),
                        span,
                    ));
                }
                let idx_val = self.evaluate_expression(&args[0], span.clone())?;
                let idx = match idx_val {
                    ComptimeValue::I32(i) => i as usize,
                    ComptimeValue::I64(i) => i as usize,
                    _ => {
                        return Err(CompileError::ComptimeError(
                            "at() expects an integer index".to_string(),
                            span,
                        ))
                    }
                };
                if idx < items.len() {
                    Ok(items[idx].clone())
                } else {
                    Err(CompileError::ComptimeError(
                        format!("Index {} out of bounds (len: {})", idx, items.len()),
                        span,
                    ))
                }
            }

            _ => Err(CompileError::ComptimeError(
                format!("Array has no method '{}'", method),
                span,
            )),
        }
    }

    /// Evaluate string methods
    fn evaluate_string_method(
        &mut self,
        s: &str,
        method: &str,
        args: &[Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        match method {
            "len" => Ok(ComptimeValue::I64(s.len() as i64)),
            "append" => {
                if args.len() != 1 {
                    return Err(CompileError::ComptimeError(
                        "String.append() expects 1 argument".to_string(),
                        span,
                    ));
                }
                let val = self.evaluate_expression(&args[0], span.clone())?;
                match val {
                    ComptimeValue::String(other) => {
                        Ok(ComptimeValue::String(format!("{}{}", s, other)))
                    }
                    _ => Err(CompileError::ComptimeError(
                        "String.append() expects a string argument".to_string(),
                        span,
                    )),
                }
            }
            _ => Err(CompileError::ComptimeError(
                format!("String has no method '{}'", method),
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

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn parse_and_get_program(source: &str) -> ast::Program {
        let lexer = crate::lexer::Lexer::new(source);
        let mut parser = crate::parser::Parser::new(lexer);
        parser.parse_program().unwrap()
    }

    #[test]
    fn test_meta_parse_and_variant_name() {
        let mut interp = ComptimeInterpreter::new();

        // meta.parse("x = 42") returns a Program ASTNode
        let source_expr = Expression::String("x = 42".to_string());
        let meta_val = interp
            .evaluate_member_access(interp.modules.get("@std").unwrap().clone(), "meta", None)
            .unwrap();

        // Call meta.parse()
        let result = interp
            .evaluate_method_call(meta_val.clone(), "parse", &[source_expr], None)
            .unwrap();

        // Should be an ASTNode
        assert!(matches!(result, ComptimeValue::ASTNode(_)));

        // Test variant_name directly on the node
        if let ComptimeValue::ASTNode(ref node) = result {
            assert_eq!(meta::variant_name(node), "Program");
        }
    }

    #[test]
    fn test_meta_type_info_on_parsed_code() {
        let program = parse_and_get_program("add = (a: i32, b: i32) i32 { return a + b }");
        let node = ASTNodeValue::Program(program);
        let info = meta::type_info(&node).unwrap();

        if let ComptimeValue::Struct { fields, .. } = &info {
            assert_eq!(
                fields.get("kind").unwrap().clone(),
                ComptimeValue::String("Program".to_string())
            );
            assert_eq!(
                fields.get("variant").unwrap().clone(),
                ComptimeValue::String("Program".to_string())
            );
        } else {
            panic!("Expected TypeInfo struct");
        }
    }

    #[test]
    fn test_meta_walk_function_declaration() {
        let program = parse_and_get_program("add = (a: i32, b: i32) i32 { return a + b }");

        // Get the first declaration
        assert!(!program.declarations.is_empty());
        let func_node = ASTNodeValue::Declaration(program.declarations[0].clone());

        // variant_name should be "Function"
        assert_eq!(meta::variant_name(&func_node), "Function");

        // fields should include name, args, return_type, body
        let flds = meta::fields(&func_node).unwrap();
        let field_names: Vec<String> = flds
            .iter()
            .filter_map(|f| {
                if let ComptimeValue::Struct { fields, .. } = f {
                    if let Some(ComptimeValue::String(n)) = fields.get("name") {
                        return Some(n.clone());
                    }
                }
                None
            })
            .collect();

        assert!(field_names.contains(&"name".to_string()));
        assert!(field_names.contains(&"args".to_string()));
        assert!(field_names.contains(&"return_type".to_string()));
        assert!(field_names.contains(&"body".to_string()));
    }

    #[test]
    fn test_meta_walk_binary_expression() {
        let program = parse_and_get_program("main = () i32 { return 2 + 3 }");

        // Navigate: Program -> first decl (Function) -> body -> first stmt (Return) -> expr (BinaryOp)
        let func_decl = &program.declarations[0];
        if let Declaration::Function(f) = func_decl {
            assert_eq!(f.name, "main");
            let return_stmt = &f.body[0];
            if let Statement::Return { expr, .. } = return_stmt {
                let expr_node = ASTNodeValue::Expression(expr.clone());
                assert_eq!(meta::variant_name(&expr_node), "BinaryOp");

                let flds = meta::fields(&expr_node).unwrap();
                // Should have left, op, right
                assert_eq!(flds.len(), 3);

                // Check the operator is "+"
                if let ComptimeValue::Struct { fields, .. } = &flds[1] {
                    if let Some(ComptimeValue::String(op)) = fields.get("value") {
                        assert_eq!(op, "+");
                    }
                }
            }
        }
    }

    #[test]
    fn test_meta_parse_intrinsic_via_interpreter() {
        let mut interp = ComptimeInterpreter::new();

        // Simulate: { meta } = @std
        let std_val = interp.modules.get("@std").unwrap().clone();
        if let ComptimeValue::Struct { fields, .. } = std_val {
            if let Some(meta_val) = fields.get("meta") {
                interp.env.define("meta".to_string(), meta_val.clone());
            }
        }

        // Now call meta.parse("x = 42")
        let meta_obj = interp.env.get("meta").unwrap();
        let result = interp
            .evaluate_method_call(
                meta_obj,
                "parse",
                &[Expression::String("x = 42".to_string())],
                None,
            )
            .unwrap();

        assert!(matches!(result, ComptimeValue::ASTNode(_)));

        // Store the parsed AST and call variant_name on it
        interp.env.define("ast".to_string(), result);
        let ast_val = interp.env.get("ast").unwrap();
        let vname_result = interp
            .evaluate_method_call(ast_val, "variant_name", &[], None)
            .unwrap();

        assert_eq!(vname_result, ComptimeValue::String("Program".to_string()));
    }

    #[test]
    fn test_meta_field_access_on_ast_node() {
        let mut interp = ComptimeInterpreter::new();

        // Parse a function and get its AST
        let program = parse_and_get_program("greet = (name: StringLiteral) void {}");
        let func_node = ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Declaration(
            program.declarations[0].clone(),
        )));

        interp.env.define("func".to_string(), func_node);

        // Access func.name via member access
        let func_val = interp.env.get("func").unwrap();
        let name = interp
            .evaluate_member_access(func_val, "name", None)
            .unwrap();

        assert_eq!(name, ComptimeValue::String("greet".to_string()));
    }

    #[test]
    fn test_meta_field_access_on_binary_op() {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Integer32(10)),
            op: ast::BinaryOperator::Multiply,
            right: Box::new(Expression::Integer32(5)),
        };
        let node = ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Expression(expr)));

        let mut interp = ComptimeInterpreter::new();
        interp.env.define("expr".to_string(), node);

        // Access expr.op
        let expr_val = interp.env.get("expr").unwrap();
        let op = interp
            .evaluate_member_access(expr_val.clone(), "op", None)
            .unwrap();
        assert_eq!(op, ComptimeValue::String("*".to_string()));

        // Access expr.left (should be another ASTNode)
        let left = interp
            .evaluate_member_access(expr_val, "left", None)
            .unwrap();
        assert!(matches!(left, ComptimeValue::ASTNode(_)));

        // Access value from the left node
        let left_val = interp.evaluate_member_access(left, "value", None).unwrap();
        assert_eq!(left_val, ComptimeValue::I32(10));
    }

    #[test]
    fn test_destructuring_import_std_meta() {
        let mut interp = ComptimeInterpreter::new();

        // Execute: { meta } = @std
        let import_stmt = Statement::DestructuringImport {
            names: vec!["meta".to_string()],
            source: Expression::StdReference,
            span: None,
        };

        interp.execute_statement(&import_stmt).unwrap();

        // meta should now be in scope
        let meta_val = interp.env.get("meta");
        assert!(meta_val.is_some());
        if let Some(ComptimeValue::Struct { name, .. }) = meta_val {
            assert_eq!(name, "meta");
        }
    }

    #[test]
    fn test_array_len_method() {
        let mut interp = ComptimeInterpreter::new();
        let arr = ComptimeValue::Array(vec![
            ComptimeValue::I32(1),
            ComptimeValue::I32(2),
            ComptimeValue::I32(3),
        ]);
        interp.env.define("arr".to_string(), arr);

        let arr_val = interp.env.get("arr").unwrap();
        let len = interp
            .evaluate_method_call(arr_val, "len", &[], None)
            .unwrap();
        assert_eq!(len, ComptimeValue::I64(3));
    }

    #[test]
    fn test_string_append_method() {
        let mut interp = ComptimeInterpreter::new();
        let s = ComptimeValue::String("hello ".to_string());

        let result = interp
            .evaluate_method_call(
                s,
                "append",
                &[Expression::String("world".to_string())],
                None,
            )
            .unwrap();
        assert_eq!(result, ComptimeValue::String("hello world".to_string()));
    }

    #[test]
    fn test_full_meta_pipeline_parse_walk_introspect() {
        // This test simulates the full meta pipeline:
        // 1. Parse source code into AST
        // 2. Walk the AST using meta intrinsics
        // 3. Extract information from each node

        let mut interp = ComptimeInterpreter::new();

        // Import meta
        let import_stmt = Statement::DestructuringImport {
            names: vec!["meta".to_string()],
            source: Expression::StdReference,
            span: None,
        };
        interp.execute_statement(&import_stmt).unwrap();

        // Parse source code
        let meta_val = interp.env.get("meta").unwrap();
        let ast_node = interp
            .evaluate_method_call(
                meta_val.clone(),
                "parse",
                &[Expression::String(
                    "add = (a: i32, b: i32) i32 { return a + b }".to_string(),
                )],
                None,
            )
            .unwrap();

        // Get type_info
        interp.env.define("program".to_string(), ast_node.clone());
        let type_info = interp
            .evaluate_method_call(
                meta_val.clone(),
                "type_info",
                &[Expression::Identifier("program".to_string())],
                None,
            )
            .unwrap();

        // type_info should be a TypeInfo struct
        if let ComptimeValue::Struct { name, fields } = &type_info {
            assert_eq!(name, "TypeInfo");
            assert_eq!(
                fields.get("kind").unwrap().clone(),
                ComptimeValue::String("Program".to_string())
            );
        } else {
            panic!("Expected TypeInfo struct");
        }

        // Get fields of the program
        let program_fields = interp
            .evaluate_method_call(
                meta_val.clone(),
                "fields",
                &[Expression::Identifier("program".to_string())],
                None,
            )
            .unwrap();

        if let ComptimeValue::Array(flds) = &program_fields {
            assert_eq!(flds.len(), 2); // declarations + statements
        } else {
            panic!("Expected array of fields");
        }

        // Navigate into declarations via field access on the ASTNode
        let decls = interp
            .evaluate_member_access(ast_node, "declarations", None)
            .unwrap();

        if let ComptimeValue::Array(items) = &decls {
            assert_eq!(items.len(), 1); // One function declaration

            // Get the function name
            let func = &items[0];
            let func_name = interp
                .evaluate_member_access(func.clone(), "name", None)
                .unwrap();
            assert_eq!(func_name, ComptimeValue::String("add".to_string()));

            // Get variant name of the function declaration
            let func_variant = interp
                .evaluate_method_call(func.clone(), "variant_name", &[], None)
                .unwrap();
            assert_eq!(func_variant, ComptimeValue::String("Function".to_string()));
        } else {
            panic!("Expected array of declarations");
        }
    }

    // === New tests for QuestionMatch, Loop, ArrayIndex, StringConcat ===

    #[test]
    fn test_question_match_literal_patterns() {
        let mut interp = ComptimeInterpreter::new();

        // x = 42; x ? | 42 { "matched" } | _ { "no match" }
        let scrutinee = Expression::Integer32(42);
        let arms = vec![
            ast::MatchArm {
                pattern: Pattern::Literal(Expression::Integer32(42)),
                guard: None,
                body: Expression::String("matched".to_string()),
            },
            ast::MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Expression::String("no match".to_string()),
            },
        ];

        let expr = Expression::QuestionMatch {
            scrutinee: Box::new(scrutinee),
            arms,
        };

        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("matched".to_string()));
    }

    #[test]
    fn test_question_match_wildcard_fallthrough() {
        let mut interp = ComptimeInterpreter::new();

        // 99 ? | 42 { "forty-two" } | _ { "other" }
        let arms = vec![
            ast::MatchArm {
                pattern: Pattern::Literal(Expression::Integer32(42)),
                guard: None,
                body: Expression::String("forty-two".to_string()),
            },
            ast::MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Expression::String("other".to_string()),
            },
        ];

        let expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::Integer32(99)),
            arms,
        };

        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("other".to_string()));
    }

    #[test]
    fn test_question_match_string_patterns() {
        let mut interp = ComptimeInterpreter::new();

        // "Function" ? | "Function" { "is func" } | "Struct" { "is struct" } | _ { "unknown" }
        let arms = vec![
            ast::MatchArm {
                pattern: Pattern::Literal(Expression::String("Function".to_string())),
                guard: None,
                body: Expression::String("is func".to_string()),
            },
            ast::MatchArm {
                pattern: Pattern::Literal(Expression::String("Struct".to_string())),
                guard: None,
                body: Expression::String("is struct".to_string()),
            },
            ast::MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Expression::String("unknown".to_string()),
            },
        ];

        let expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::String("Function".to_string())),
            arms,
        };

        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("is func".to_string()));
    }

    #[test]
    fn test_question_match_boolean_patterns() {
        let mut interp = ComptimeInterpreter::new();

        // true ? | true { "yes" } | false { "no" }
        let arms = vec![
            ast::MatchArm {
                pattern: Pattern::Literal(Expression::Boolean(true)),
                guard: None,
                body: Expression::String("yes".to_string()),
            },
            ast::MatchArm {
                pattern: Pattern::Literal(Expression::Boolean(false)),
                guard: None,
                body: Expression::String("no".to_string()),
            },
        ];

        let expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::Boolean(true)),
            arms,
        };

        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("yes".to_string()));
    }

    #[test]
    fn test_question_match_with_binding() {
        let mut interp = ComptimeInterpreter::new();

        // 42 ? | n { n }  -- binding pattern captures value
        let arms = vec![ast::MatchArm {
            pattern: Pattern::Identifier("n".to_string()),
            guard: None,
            body: Expression::Identifier("n".to_string()),
        }];

        let expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::Integer32(42)),
            arms,
        };

        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::I32(42));
    }

    #[test]
    fn test_loop_with_condition() {
        let mut interp = ComptimeInterpreter::new();

        // i ::= 0; loop i < 5 { i = i + 1 }; i should be 5
        interp.env.define("i".to_string(), ComptimeValue::I32(0));

        let loop_stmt = Statement::Loop {
            kind: ast::LoopKind::Condition(Expression::BinaryOp {
                left: Box::new(Expression::Identifier("i".to_string())),
                op: ast::BinaryOperator::LessThan,
                right: Box::new(Expression::Integer32(5)),
            }),
            label: None,
            body: vec![Statement::VariableAssignment {
                name: "i".to_string(),
                value: Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("i".to_string())),
                    op: ast::BinaryOperator::Add,
                    right: Box::new(Expression::Integer32(1)),
                },
                span: None,
            }],
            span: None,
        };

        interp.execute_statement(&loop_stmt).unwrap();
        assert_eq!(interp.env.get("i").unwrap(), ComptimeValue::I32(5));
    }

    #[test]
    fn test_loop_with_break() {
        let mut interp = ComptimeInterpreter::new();

        // i ::= 0; loop { i = i + 1; i == 3 ? | true { break } }
        interp.env.define("i".to_string(), ComptimeValue::I32(0));

        let loop_stmt = Statement::Loop {
            kind: ast::LoopKind::Infinite,
            label: None,
            body: vec![
                Statement::VariableAssignment {
                    name: "i".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("i".to_string())),
                        op: ast::BinaryOperator::Add,
                        right: Box::new(Expression::Integer32(1)),
                    },
                    span: None,
                },
                // if i == 3, break
                Statement::Expression {
                    expr: Expression::QuestionMatch {
                        scrutinee: Box::new(Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("i".to_string())),
                            op: ast::BinaryOperator::Equals,
                            right: Box::new(Expression::Integer32(3)),
                        }),
                        arms: vec![
                            ast::MatchArm {
                                pattern: Pattern::Literal(Expression::Boolean(true)),
                                guard: None,
                                // Block that contains a break - but we need to use the statement form
                                // For now, use a direct break since QuestionMatch body is an expression
                                body: Expression::Block(vec![Statement::Break {
                                    label: None,
                                    span: None,
                                }]),
                            },
                            ast::MatchArm {
                                pattern: Pattern::Wildcard,
                                guard: None,
                                body: Expression::Integer32(0),
                            },
                        ],
                    },
                    span: None,
                },
            ],
            span: None,
        };

        interp.execute_statement(&loop_stmt).unwrap();
        assert_eq!(interp.env.get("i").unwrap(), ComptimeValue::I32(3));
    }

    #[test]
    fn test_array_index_expression() {
        let mut interp = ComptimeInterpreter::new();

        // arr = [10, 20, 30]; arr[1] should be 20
        interp.env.define(
            "arr".to_string(),
            ComptimeValue::Array(vec![
                ComptimeValue::I32(10),
                ComptimeValue::I32(20),
                ComptimeValue::I32(30),
            ]),
        );

        let expr = Expression::ArrayIndex {
            array: Box::new(Expression::Identifier("arr".to_string())),
            index: Box::new(Expression::Integer32(1)),
        };

        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::I32(20));
    }

    #[test]
    fn test_array_index_out_of_bounds() {
        let mut interp = ComptimeInterpreter::new();

        interp.env.define(
            "arr".to_string(),
            ComptimeValue::Array(vec![ComptimeValue::I32(1)]),
        );

        let expr = Expression::ArrayIndex {
            array: Box::new(Expression::Identifier("arr".to_string())),
            index: Box::new(Expression::Integer32(5)),
        };

        let result = interp.evaluate_expression(&expr, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_concatenation() {
        let mut interp = ComptimeInterpreter::new();

        // "hello " + "world" should be "hello world"
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::String("hello ".to_string())),
            op: ast::BinaryOperator::Add,
            right: Box::new(Expression::String("world".to_string())),
        };

        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("hello world".to_string()));
    }

    #[test]
    fn test_string_interpolation() {
        let mut interp = ComptimeInterpreter::new();

        interp
            .env
            .define("name".to_string(), ComptimeValue::String("Zen".to_string()));
        interp.env.define("ver".to_string(), ComptimeValue::I32(7));

        // "${name} v${ver}"
        let expr = Expression::StringInterpolation {
            parts: vec![
                ast::StringPart::Interpolation(Expression::Identifier("name".to_string())),
                ast::StringPart::Literal(" v".to_string()),
                ast::StringPart::Interpolation(Expression::Identifier("ver".to_string())),
            ],
        };

        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("Zen v7".to_string()));
    }

    #[test]
    fn test_string_building_with_loop() {
        let mut interp = ComptimeInterpreter::new();

        // Build "0,1,2,3,4," via loop
        // result ::= ""; i ::= 0; loop i < 5 { result = result + i_str + ","; i = i + 1 }
        interp
            .env
            .define("result".to_string(), ComptimeValue::String("".to_string()));
        interp.env.define("i".to_string(), ComptimeValue::I32(0));

        // We'll use string interpolation + concat in a loop
        let loop_stmt = Statement::Loop {
            kind: ast::LoopKind::Condition(Expression::BinaryOp {
                left: Box::new(Expression::Identifier("i".to_string())),
                op: ast::BinaryOperator::LessThan,
                right: Box::new(Expression::Integer32(5)),
            }),
            label: None,
            body: vec![
                // result = result + "${i},"
                Statement::VariableAssignment {
                    name: "result".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("result".to_string())),
                        op: ast::BinaryOperator::Add,
                        right: Box::new(Expression::StringInterpolation {
                            parts: vec![
                                ast::StringPart::Interpolation(Expression::Identifier(
                                    "i".to_string(),
                                )),
                                ast::StringPart::Literal(",".to_string()),
                            ],
                        }),
                    },
                    span: None,
                },
                // i = i + 1
                Statement::VariableAssignment {
                    name: "i".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("i".to_string())),
                        op: ast::BinaryOperator::Add,
                        right: Box::new(Expression::Integer32(1)),
                    },
                    span: None,
                },
            ],
            span: None,
        };

        interp.execute_statement(&loop_stmt).unwrap();
        assert_eq!(
            interp.env.get("result").unwrap(),
            ComptimeValue::String("0,1,2,3,4,".to_string())
        );
    }

    #[test]
    fn test_meta_ast_walk_with_pattern_match() {
        // Full integration: parse code, walk AST, pattern match on variant names
        let mut interp = ComptimeInterpreter::new();

        // Import meta
        let import_stmt = Statement::DestructuringImport {
            names: vec!["meta".to_string()],
            source: Expression::StdReference,
            span: None,
        };
        interp.execute_statement(&import_stmt).unwrap();

        // Parse source
        let meta_val = interp.env.get("meta").unwrap();
        let ast_node = interp
            .evaluate_method_call(
                meta_val,
                "parse",
                &[Expression::String(
                    "add = (a: i32, b: i32) i32 { return a + b }".to_string(),
                )],
                None,
            )
            .unwrap();

        // Get the first function via helper
        interp.env.define("program".to_string(), ast_node);
        let prog = interp.env.get("program").unwrap();
        let func = interp
            .evaluate_method_call(
                prog,
                "find_function",
                &[Expression::String("add".to_string())],
                None,
            )
            .unwrap();

        // Get variant name and pattern match on it
        let vname = interp
            .evaluate_method_call(func.clone(), "variant_name", &[], None)
            .unwrap();

        // Pattern match: vname ? | "Function" { "found function!" } | _ { "other" }
        let match_expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::Identifier("vname".to_string())),
            arms: vec![
                ast::MatchArm {
                    pattern: Pattern::Literal(Expression::String("Function".to_string())),
                    guard: None,
                    body: Expression::String("found function!".to_string()),
                },
                ast::MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expression::String("other".to_string()),
                },
            ],
        };

        interp.env.define("vname".to_string(), vname);
        let result = interp.evaluate_expression(&match_expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("found function!".to_string()));
    }

    #[test]
    fn test_block_expression() {
        let mut interp = ComptimeInterpreter::new();

        // { x = 10; x + 5 } should evaluate to 15
        let block = Expression::Block(vec![
            Statement::VariableDeclaration {
                name: "x".to_string(),
                type_: None,
                initializer: Some(Expression::Integer32(10)),
                is_mutable: false,
                declaration_type: ast::VariableDeclarationType::InferredImmutable,
                span: None,
            },
            Statement::Expression {
                expr: Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    op: ast::BinaryOperator::Add,
                    right: Box::new(Expression::Integer32(5)),
                },
                span: None,
            },
        ]);

        let result = interp.evaluate_expression(&block, None).unwrap();
        assert_eq!(result, ComptimeValue::I32(15));
    }

    #[test]
    fn test_helper_functions_find() {
        let mut interp = ComptimeInterpreter::new();

        // Parse a program with multiple declarations
        let import_stmt = Statement::DestructuringImport {
            names: vec!["meta".to_string()],
            source: Expression::StdReference,
            span: None,
        };
        interp.execute_statement(&import_stmt).unwrap();

        let meta_val = interp.env.get("meta").unwrap();
        let ast_node = interp
            .evaluate_method_call(
                meta_val,
                "parse",
                &[Expression::String(
                    "add = (a: i32, b: i32) i32 { return a + b }\nsub = (a: i32, b: i32) i32 { return a - b }"
                        .to_string(),
                )],
                None,
            )
            .unwrap();

        // Test functions() - should return 2 functions
        let funcs = interp
            .evaluate_method_call(ast_node.clone(), "functions", &[], None)
            .unwrap();
        if let ComptimeValue::Array(items) = &funcs {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected array from functions()");
        }

        // Test find_function("sub") - should find it
        let sub_fn = interp
            .evaluate_method_call(
                ast_node.clone(),
                "find_function",
                &[Expression::String("sub".to_string())],
                None,
            )
            .unwrap();
        assert!(!matches!(sub_fn, ComptimeValue::Null));

        // Test find_function("nonexistent") - should return Null
        let none_fn = interp
            .evaluate_method_call(
                ast_node,
                "find_function",
                &[Expression::String("nonexistent".to_string())],
                None,
            )
            .unwrap();
        assert!(matches!(none_fn, ComptimeValue::Null));
    }
}
