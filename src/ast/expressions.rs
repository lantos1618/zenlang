//! Expression nodes in the AST

use super::fields::{match_arms_fields, AstFields, FieldValue};
use super::patterns::Pattern;
use super::types::AstType;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    #[allow(dead_code)]
    StringConcat,
    And,
    Or,
    // Bitwise operators
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Modulo => write!(f, "%"),
            BinaryOperator::Equals => write!(f, "=="),
            BinaryOperator::NotEquals => write!(f, "!="),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::LessThanEquals => write!(f, "<="),
            BinaryOperator::GreaterThanEquals => write!(f, ">="),
            BinaryOperator::And => write!(f, "and"),
            BinaryOperator::Or => write!(f, "or"),
            BinaryOperator::StringConcat => write!(f, "++"),
            BinaryOperator::BitwiseAnd => write!(f, "&"),
            BinaryOperator::BitwiseOr => write!(f, "|"),
            BinaryOperator::BitwiseXor => write!(f, "^"),
            BinaryOperator::ShiftLeft => write!(f, "<<"),
            BinaryOperator::ShiftRight => write!(f, ">>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    #[allow(dead_code)]
    Integer8(i8),
    #[allow(dead_code)]
    Integer16(i16),
    Integer32(i32),
    Integer64(i64),
    #[allow(dead_code)]
    Unsigned8(u8),
    #[allow(dead_code)]
    Unsigned16(u16),
    #[allow(dead_code)]
    Unsigned32(u32),
    #[allow(dead_code)]
    Unsigned64(u64),
    #[allow(dead_code)]
    Float32(f32),
    Float64(f64),
    Boolean(bool),
    String(String),
    Identifier(String),
    Unit, // The unit value, similar to () in Rust or void in C
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    FunctionCall {
        /// Optional qualifier before the dot (module name, type name, or intrinsic prefix).
        /// e.g., `compiler.sizeof()` → module = Some("compiler"), name = "sizeof"
        /// e.g., `@builtin.cast()` → module = Some("@builtin"), name = "cast"
        /// e.g., `println()` → module = None, name = "println"
        module: Option<String>,
        name: String,
        type_args: Vec<AstType>,
        args: Vec<Expression>,
        span: Option<crate::error::Span>,
    },
    // Pattern matching with ? operator (no match keyword!)
    QuestionMatch {
        scrutinee: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    // Conditional expression for simple boolean patterns (expr ? { block })
    #[allow(dead_code)]
    Conditional {
        scrutinee: Box<Expression>,
        arms: Vec<ConditionalArm>,
    },
    #[allow(dead_code)]
    AddressOf(Box<Expression>),
    #[allow(dead_code)]
    Dereference(Box<Expression>),
    #[allow(dead_code)]
    PointerOffset {
        pointer: Box<Expression>,
        offset: Box<Expression>,
    },
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    #[allow(dead_code)]
    StructField {
        struct_: Box<Expression>,
        field: String,
    },
    // New expressions for enhanced features
    ArrayLiteral(Vec<Expression>),
    ArrayIndex {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    EnumVariant {
        enum_name: String,
        variant: String,
        payload: Option<Box<Expression>>,
    },
    // Enum literal syntax: .Some(value), .None (without enum name)
    EnumLiteral {
        variant: String,
        payload: Option<Box<Expression>>,
    },
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
    // Pointer-specific operations for Zen spec
    #[allow(dead_code)]
    PointerDereference(Box<Expression>), // .val operation
    #[allow(dead_code)]
    PointerAddress(Box<Expression>), // .addr operation
    #[allow(dead_code)]
    CreateReference(Box<Expression>), // .ref() method
    #[allow(dead_code)]
    CreateMutableReference(Box<Expression>), // .mut_ref() method
    #[allow(dead_code)]
    StringLength(Box<Expression>),
    // Option<T> constructors
    Some(Box<Expression>), // Some(value)
    None,                  // None (also accessible as "null")
    // String interpolation: "Hello ${name}!"
    StringInterpolation {
        parts: Vec<StringPart>,
    },
    // For comptime expressions
    Comptime(Box<Expression>),
    // Range expressions
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
    // Pattern matching expressions
    #[allow(dead_code)]
    PatternMatch {
        scrutinee: Box<Expression>,
        arms: Vec<PatternArm>,
    },
    // @std reference
    StdReference,
    // @builtin reference (raw compiler intrinsics)
    BuiltinReference,
    // @this reference (current scope)
    ThisReference,
    // Method call with UFC (Uniform Function Call)
    MethodCall {
        object: Box<Expression>,
        method: String,
        type_args: Vec<AstType>,
        args: Vec<Expression>,
        span: Option<crate::error::Span>,
    },
    // Loop expression (returns value)
    Loop {
        body: Box<Expression>,
    },
    // Collection loop: collection.loop((item) { ... })
    CollectionLoop {
        collection: Box<Expression>,
        param: (String, Option<AstType>), // The loop parameter name and optional type
        index_param: Option<(String, Option<AstType>)>, // Optional index parameter and type
        body: Box<Expression>,
    },
    // Closure expression
    Closure {
        params: Vec<(String, Option<AstType>)>,
        return_type: Option<AstType>,
        body: Box<Expression>,
    },
    // Block expression - evaluates to the last expression or void
    Block(Vec<super::statements::Statement>),
    // Return expression - for early returns in pattern match arms
    Return(Box<Expression>),
    // Error propagation: expr.raise()
    Raise(Box<Expression>),
    // Defer expression: @this.defer(expr)
    #[allow(dead_code)]
    Defer(Box<Expression>),
    // Break expression for loops (can be used in expression contexts like pattern arms)
    Break {
        label: Option<String>,
        value: Option<Box<Expression>>, // Break can optionally return a value
    },
    // Continue expression for loops
    Continue {
        label: Option<String>,
    },
    // Collection constructors
    // Vec<T, size>() - Fixed-size vector constructor
    VecConstructor {
        element_type: AstType,
        size: usize,
        initial_values: Option<Vec<Expression>>, // Optional initial values
    },
    // DynVec<T>(allocator) or DynVec<T1, T2, ...>(allocator) - Dynamic vector constructor
    DynVecConstructor {
        element_types: Vec<AstType>,
        allocator: Box<Expression>,                // Allocator expression
        initial_capacity: Option<Box<Expression>>, // Optional initial capacity
    },
    // Array<T>() - Array constructor
    ArrayConstructor {
        element_type: AstType,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Interpolation(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>, // Optional guard condition
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>, // Optional guard condition using ->
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>, // Optional guard condition
    pub body: Expression,
}

impl Expression {
    /// Returns the variant name of this expression as a static string.
    /// Used by the comptime meta system and anywhere else that needs
    /// to identify expression variants without duplicating match arms.
    pub fn variant_name(&self) -> &'static str {
        match self {
            Expression::Integer8(_) => "Integer8",
            Expression::Integer16(_) => "Integer16",
            Expression::Integer32(_) => "Integer32",
            Expression::Integer64(_) => "Integer64",
            Expression::Unsigned8(_) => "Unsigned8",
            Expression::Unsigned16(_) => "Unsigned16",
            Expression::Unsigned32(_) => "Unsigned32",
            Expression::Unsigned64(_) => "Unsigned64",
            Expression::Float32(_) => "Float32",
            Expression::Float64(_) => "Float64",
            Expression::Boolean(_) => "Boolean",
            Expression::String(_) => "String",
            Expression::Identifier(_) => "Identifier",
            Expression::Unit => "Unit",
            Expression::BinaryOp { .. } => "BinaryOp",
            Expression::FunctionCall { .. } => "FunctionCall",
            Expression::QuestionMatch { .. } => "QuestionMatch",
            Expression::Conditional { .. } => "Conditional",
            Expression::AddressOf(_) => "AddressOf",
            Expression::Dereference(_) => "Dereference",
            Expression::PointerOffset { .. } => "PointerOffset",
            Expression::StructLiteral { .. } => "StructLiteral",
            Expression::StructField { .. } => "StructField",
            Expression::ArrayLiteral(_) => "ArrayLiteral",
            Expression::ArrayIndex { .. } => "ArrayIndex",
            Expression::EnumVariant { .. } => "EnumVariant",
            Expression::EnumLiteral { .. } => "EnumLiteral",
            Expression::MemberAccess { .. } => "MemberAccess",
            Expression::PointerDereference(_) => "PointerDereference",
            Expression::PointerAddress(_) => "PointerAddress",
            Expression::CreateReference(_) => "CreateReference",
            Expression::CreateMutableReference(_) => "CreateMutableReference",
            Expression::StringLength(_) => "StringLength",
            Expression::Some(_) => "Some",
            Expression::None => "None",
            Expression::StringInterpolation { .. } => "StringInterpolation",
            Expression::Comptime(_) => "Comptime",
            Expression::Range { .. } => "Range",
            Expression::PatternMatch { .. } => "PatternMatch",
            Expression::StdReference => "StdReference",
            Expression::BuiltinReference => "BuiltinReference",
            Expression::ThisReference => "ThisReference",
            Expression::MethodCall { .. } => "MethodCall",
            Expression::Loop { .. } => "Loop",
            Expression::CollectionLoop { .. } => "CollectionLoop",
            Expression::Closure { .. } => "Closure",
            Expression::Block(_) => "Block",
            Expression::Return(_) => "Return",
            Expression::Raise(_) => "Raise",
            Expression::Defer(_) => "Defer",
            Expression::Break { .. } => "Break",
            Expression::Continue { .. } => "Continue",
            Expression::VecConstructor { .. } => "VecConstructor",
            Expression::DynVecConstructor { .. } => "DynVecConstructor",
            Expression::ArrayConstructor { .. } => "ArrayConstructor",
        }
    }
}

impl AstFields for Expression {
    fn ast_fields(&self) -> Vec<(&'static str, FieldValue)> {
        match self {
            Expression::Integer8(v) => vec![("value", FieldValue::I8(*v))],
            Expression::Integer16(v) => vec![("value", FieldValue::I16(*v))],
            Expression::Integer32(v) => vec![("value", FieldValue::I32(*v))],
            Expression::Integer64(v) => vec![("value", FieldValue::I64(*v))],
            Expression::Unsigned8(v) => vec![("value", FieldValue::U8(*v))],
            Expression::Unsigned16(v) => vec![("value", FieldValue::U16(*v))],
            Expression::Unsigned32(v) => vec![("value", FieldValue::U32(*v))],
            Expression::Unsigned64(v) => vec![("value", FieldValue::U64(*v))],
            Expression::Float32(v) => vec![("value", FieldValue::F32(*v))],
            Expression::Float64(v) => vec![("value", FieldValue::F64(*v))],
            Expression::Boolean(v) => vec![("value", FieldValue::Bool(*v))],
            Expression::String(v) => vec![("value", FieldValue::String(v.clone()))],
            Expression::Identifier(name) => {
                vec![("name", FieldValue::String(name.clone()))]
            }
            Expression::Unit
            | Expression::None
            | Expression::StdReference
            | Expression::BuiltinReference
            | Expression::ThisReference => vec![],

            Expression::BinaryOp { left, op, right } => vec![
                ("left", FieldValue::boxed_expr(left)),
                ("op", FieldValue::String(op.to_string())),
                ("right", FieldValue::boxed_expr(right)),
            ],

            Expression::FunctionCall {
                module,
                name,
                type_args,
                args,
                ..
            } => {
                let qualified_name = match module {
                    Some(m) => format!("{}.{}", m, name),
                    None => name.clone(),
                };
                vec![
                    ("name", FieldValue::String(qualified_name)),
                    ("type_args", FieldValue::type_array(type_args)),
                    ("args", FieldValue::expr_array(args)),
                ]
            }

            Expression::MethodCall {
                object,
                method,
                type_args,
                args,
                ..
            } => vec![
                ("object", FieldValue::boxed_expr(object)),
                ("method", FieldValue::String(method.clone())),
                ("type_args", FieldValue::type_array(type_args)),
                ("args", FieldValue::expr_array(args)),
            ],

            Expression::QuestionMatch { scrutinee, arms } => {
                match_arms_fields("MatchArm", scrutinee, arms)
            }
            Expression::Conditional { scrutinee, arms } => {
                match_arms_fields("ConditionalArm", scrutinee, arms)
            }
            Expression::PatternMatch { scrutinee, arms } => {
                match_arms_fields("PatternArm", scrutinee, arms)
            }

            Expression::AddressOf(inner) => {
                vec![("expr", FieldValue::boxed_expr(inner))]
            }
            Expression::Dereference(inner) => {
                vec![("expr", FieldValue::boxed_expr(inner))]
            }
            Expression::PointerOffset { pointer, offset } => vec![
                ("pointer", FieldValue::boxed_expr(pointer)),
                ("offset", FieldValue::boxed_expr(offset)),
            ],
            Expression::PointerDereference(inner) => {
                vec![("expr", FieldValue::boxed_expr(inner))]
            }
            Expression::PointerAddress(inner) => {
                vec![("expr", FieldValue::boxed_expr(inner))]
            }
            Expression::CreateReference(inner) => {
                vec![("expr", FieldValue::boxed_expr(inner))]
            }
            Expression::CreateMutableReference(inner) => {
                vec![("expr", FieldValue::boxed_expr(inner))]
            }

            Expression::StructLiteral { name, fields: fs } => vec![
                ("name", FieldValue::String(name.clone())),
                (
                    "fields",
                    FieldValue::Array(
                        fs.iter()
                            .map(|(n, e)| {
                                let mut fields = HashMap::new();
                                fields.insert("name".to_string(), FieldValue::String(n.clone()));
                                fields.insert("value".to_string(), FieldValue::expr(e));
                                FieldValue::Struct {
                                    name: "StructFieldInit".to_string(),
                                    fields,
                                }
                            })
                            .collect(),
                    ),
                ),
            ],

            Expression::StructField { struct_, field } => vec![
                ("struct_expr", FieldValue::boxed_expr(struct_)),
                ("field", FieldValue::String(field.clone())),
            ],

            Expression::ArrayLiteral(elems) => {
                vec![("elements", FieldValue::expr_array(elems))]
            }

            Expression::ArrayIndex { array, index } => vec![
                ("array", FieldValue::boxed_expr(array)),
                ("index", FieldValue::boxed_expr(index)),
            ],

            Expression::ArrayConstructor { element_type } => {
                vec![("element_type", FieldValue::ty(element_type))]
            }

            Expression::VecConstructor {
                element_type,
                size,
                initial_values,
            } => vec![
                ("element_type", FieldValue::ty(element_type)),
                ("size", FieldValue::I64(*size as i64)),
                (
                    "initial_values",
                    match initial_values {
                        Some(vals) => FieldValue::expr_array(vals),
                        None => FieldValue::Array(vec![]),
                    },
                ),
            ],

            Expression::DynVecConstructor {
                element_types,
                allocator,
                initial_capacity,
            } => vec![
                ("element_types", FieldValue::type_array(element_types)),
                ("allocator", FieldValue::boxed_expr(allocator)),
                (
                    "initial_capacity",
                    match initial_capacity {
                        Some(cap) => FieldValue::boxed_expr(cap),
                        None => FieldValue::Null,
                    },
                ),
            ],

            Expression::EnumVariant {
                enum_name,
                variant,
                payload,
            } => vec![
                ("enum_name", FieldValue::String(enum_name.clone())),
                ("variant", FieldValue::String(variant.clone())),
                ("payload", FieldValue::opt_expr(payload)),
            ],

            Expression::EnumLiteral { variant, payload } => vec![
                ("variant", FieldValue::String(variant.clone())),
                ("payload", FieldValue::opt_expr(payload)),
            ],

            Expression::MemberAccess { object, member } => vec![
                ("object", FieldValue::boxed_expr(object)),
                ("member", FieldValue::String(member.clone())),
            ],

            Expression::StringLength(inner) => {
                vec![("expr", FieldValue::boxed_expr(inner))]
            }

            Expression::Some(expr) => {
                vec![("inner", FieldValue::boxed_expr(expr))]
            }

            Expression::StringInterpolation { parts } => vec![(
                "parts",
                FieldValue::Array(
                    parts
                        .iter()
                        .map(|part| match part {
                            StringPart::Literal(s) => {
                                let mut fields = HashMap::new();
                                fields.insert(
                                    "kind".to_string(),
                                    FieldValue::String("Literal".to_string()),
                                );
                                fields.insert("value".to_string(), FieldValue::String(s.clone()));
                                FieldValue::Struct {
                                    name: "StringPart".to_string(),
                                    fields,
                                }
                            }
                            StringPart::Interpolation(e) => {
                                let mut fields = HashMap::new();
                                fields.insert(
                                    "kind".to_string(),
                                    FieldValue::String("Interpolation".to_string()),
                                );
                                fields.insert("expr".to_string(), FieldValue::expr(e));
                                FieldValue::Struct {
                                    name: "StringPart".to_string(),
                                    fields,
                                }
                            }
                        })
                        .collect(),
                ),
            )],

            Expression::Comptime(inner) => {
                vec![("expr", FieldValue::boxed_expr(inner))]
            }

            Expression::Range {
                start,
                end,
                inclusive,
            } => vec![
                ("start", FieldValue::boxed_expr(start)),
                ("end", FieldValue::boxed_expr(end)),
                ("inclusive", FieldValue::Bool(*inclusive)),
            ],

            Expression::Loop { body } => {
                vec![("body", FieldValue::boxed_expr(body))]
            }

            Expression::CollectionLoop {
                collection,
                param,
                index_param,
                body,
            } => vec![
                ("collection", FieldValue::boxed_expr(collection)),
                ("param_name", FieldValue::String(param.0.clone())),
                (
                    "param_type",
                    match &param.1 {
                        Some(t) => FieldValue::ty(t),
                        None => FieldValue::Null,
                    },
                ),
                (
                    "index_name",
                    match index_param {
                        Some((name, _)) => FieldValue::String(name.clone()),
                        None => FieldValue::String(String::new()),
                    },
                ),
                (
                    "index_type",
                    match index_param {
                        Some((_, Some(t))) => FieldValue::ty(t),
                        _ => FieldValue::Null,
                    },
                ),
                ("body", FieldValue::boxed_expr(body)),
            ],

            Expression::Closure {
                params,
                return_type,
                body,
            } => vec![
                (
                    "params",
                    FieldValue::Array(
                        params
                            .iter()
                            .map(|(name, ty)| {
                                let mut fields = HashMap::new();
                                fields.insert("name".to_string(), FieldValue::String(name.clone()));
                                fields.insert(
                                    "param_type".to_string(),
                                    match ty {
                                        Some(t) => FieldValue::ty(t),
                                        None => FieldValue::Null,
                                    },
                                );
                                FieldValue::Struct {
                                    name: "ClosureParam".to_string(),
                                    fields,
                                }
                            })
                            .collect(),
                    ),
                ),
                (
                    "return_type",
                    match return_type {
                        Some(t) => FieldValue::ty(t),
                        None => FieldValue::Null,
                    },
                ),
                ("body", FieldValue::boxed_expr(body)),
            ],

            Expression::Block(stmts) => {
                vec![("statements", FieldValue::stmt_array(stmts))]
            }

            Expression::Return(expr) => {
                vec![("expr", FieldValue::boxed_expr(expr))]
            }
            Expression::Raise(expr) => {
                vec![("expr", FieldValue::boxed_expr(expr))]
            }
            Expression::Defer(expr) => {
                vec![("expr", FieldValue::boxed_expr(expr))]
            }

            Expression::Break { label, value } => vec![
                ("label", FieldValue::opt_label(label)),
                ("value", FieldValue::opt_expr(value)),
            ],
            Expression::Continue { label } => {
                vec![("label", FieldValue::opt_label(label))]
            }
        }
    }
}

/// Expression with optional resolved type information.
///
/// This wrapper carries type information computed by the typechecker,
/// allowing codegen and other phases to access it without re-inferring.
///
/// # Usage
///
/// After parsing, expressions have no resolved type:
/// ```ignore
/// let expr = TypedExpr::untyped(Expression::Integer32(42));
/// assert!(expr.resolved_type.is_none());
/// ```
///
/// After type checking, the type is populated:
/// ```ignore
/// expr.resolved_type = Some(AstType::I32);
/// assert_eq!(expr.get_type(), Some(&AstType::I32));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TypedExpr {
    pub expr: Expression,
    pub resolved_type: Option<AstType>,
}

impl TypedExpr {
    /// Create an expression without type information (used by parser)
    pub fn untyped(expr: Expression) -> Self {
        Self {
            expr,
            resolved_type: None,
        }
    }

    /// Create an expression with known type (used by typechecker)
    pub fn typed(expr: Expression, ty: AstType) -> Self {
        Self {
            expr,
            resolved_type: Some(ty),
        }
    }

    /// Get the resolved type, if available
    pub fn get_type(&self) -> Option<&AstType> {
        self.resolved_type.as_ref()
    }

    /// Check if type has been resolved
    pub fn is_typed(&self) -> bool {
        self.resolved_type.is_some()
    }

    /// Set the resolved type (used during type checking)
    pub fn set_type(&mut self, ty: AstType) {
        self.resolved_type = Some(ty);
    }
}

/// Allow converting Expression to TypedExpr (untyped)
impl From<Expression> for TypedExpr {
    fn from(expr: Expression) -> Self {
        Self::untyped(expr)
    }
}

/// Allow dereferencing TypedExpr to get the inner Expression
impl std::ops::Deref for TypedExpr {
    type Target = Expression;

    fn deref(&self) -> &Self::Target {
        &self.expr
    }
}

/// Allow mutable dereferencing
impl std::ops::DerefMut for TypedExpr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.expr
    }
}
