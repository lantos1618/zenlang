// Meta-programming introspection for Zen
// Enables Zen programs to walk and inspect AST nodes at compile time.
//
// This implements the Rust side of:
//   meta.type_info(node)    -> TypeInfo struct
//   meta.fields(node)       -> []FieldInfo
//   meta.variant_name(node) -> String
//   meta.children(node)     -> []ASTNode

use crate::ast::{self, AstType, BinaryOperator, Declaration, Expression, Pattern, Statement};
use crate::error::Result;
use std::collections::HashMap;
use std::rc::Rc;

use super::{ASTNodeValue, ComptimeValue};

// ---------------------------------------------------------------------------
// Helper: build a ComptimeValue::Struct representing a FieldInfo
// ---------------------------------------------------------------------------

fn field_info(name: &str, value: ComptimeValue) -> ComptimeValue {
    ComptimeValue::Struct {
        name: "FieldInfo".to_string(),
        fields: HashMap::from([
            ("name".to_string(), ComptimeValue::String(name.to_string())),
            ("value".to_string(), value),
        ]),
    }
}

fn ast_node(value: ASTNodeValue) -> ComptimeValue {
    ComptimeValue::ASTNode(Rc::new(value))
}

fn ast_expr(e: Expression) -> ComptimeValue {
    ast_node(ASTNodeValue::Expression(e))
}

fn ast_stmt(s: Statement) -> ComptimeValue {
    ast_node(ASTNodeValue::Statement(s))
}

fn ast_type(t: AstType) -> ComptimeValue {
    ast_node(ASTNodeValue::Type(t))
}

fn ast_pattern(p: Pattern) -> ComptimeValue {
    ast_node(ASTNodeValue::Pattern(p))
}

fn opt_expr(e: &Option<Box<Expression>>) -> ComptimeValue {
    match e {
        Some(expr) => ast_expr(*expr.clone()),
        None => ComptimeValue::Null,
    }
}

fn opt_string(s: &Option<String>) -> ComptimeValue {
    match s {
        Some(s) => ComptimeValue::String(s.clone()),
        None => ComptimeValue::Null,
    }
}

// ---------------------------------------------------------------------------
// meta.variant_name(node) -> String
// Returns the variant name of an AST node enum value.
// ---------------------------------------------------------------------------

pub fn variant_name(node: &ASTNodeValue) -> String {
    match node {
        ASTNodeValue::Expression(expr) => expression_variant_name(expr),
        ASTNodeValue::Statement(stmt) => statement_variant_name(stmt),
        ASTNodeValue::Declaration(decl) => declaration_variant_name(decl),
        ASTNodeValue::Type(ty) => type_variant_name(ty),
        ASTNodeValue::Pattern(pat) => pattern_variant_name(pat),
        ASTNodeValue::Program(_) => "Program".to_string(),
    }
}

fn expression_variant_name(expr: &Expression) -> String {
    match expr {
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
    .to_string()
}

fn statement_variant_name(stmt: &Statement) -> String {
    match stmt {
        Statement::Expression { .. } => "Expression",
        Statement::Return { .. } => "Return",
        Statement::VariableDeclaration { .. } => "VariableDeclaration",
        Statement::VariableAssignment { .. } => "VariableAssignment",
        Statement::PointerAssignment { .. } => "PointerAssignment",
        Statement::Loop { .. } => "Loop",
        Statement::Break { .. } => "Break",
        Statement::Continue { .. } => "Continue",
        Statement::ComptimeBlock { .. } => "ComptimeBlock",
        Statement::ModuleImport { .. } => "ModuleImport",
        Statement::Defer { .. } => "Defer",
        Statement::ThisDefer { .. } => "ThisDefer",
        Statement::DestructuringImport { .. } => "DestructuringImport",
        Statement::Block { .. } => "Block",
    }
    .to_string()
}

fn declaration_variant_name(decl: &Declaration) -> String {
    match decl {
        Declaration::Function(_) => "Function",
        Declaration::ExternalFunction(_) => "ExternalFunction",
        Declaration::Struct(_) => "Struct",
        Declaration::Enum(_) => "Enum",
        Declaration::Behavior(_) => "Behavior",
        Declaration::Trait(_) => "Trait",
        Declaration::TraitImplementation(_) => "TraitImplementation",
        Declaration::TraitRequirement(_) => "TraitRequirement",
        Declaration::ImplBlock(_) => "ImplBlock",
        Declaration::ComptimeBlock(_) => "ComptimeBlock",
        Declaration::Constant { .. } => "Constant",
        Declaration::ModuleImport { .. } => "ModuleImport",
        Declaration::Export { .. } => "Export",
        Declaration::TypeAlias(_) => "TypeAlias",
    }
    .to_string()
}

fn type_variant_name(ty: &AstType) -> String {
    match ty {
        AstType::I8 => "I8",
        AstType::I16 => "I16",
        AstType::I32 => "I32",
        AstType::I64 => "I64",
        AstType::U8 => "U8",
        AstType::U16 => "U16",
        AstType::U32 => "U32",
        AstType::U64 => "U64",
        AstType::Usize => "Usize",
        AstType::F32 => "F32",
        AstType::F64 => "F64",
        AstType::Bool => "Bool",
        AstType::StaticLiteral => "StaticLiteral",
        AstType::StaticString => "StaticString",
        AstType::Void => "Void",
        AstType::Slice(_) => "Slice",
        AstType::FixedArray { .. } => "FixedArray",
        AstType::Function { .. } => "Function",
        AstType::FunctionPointer { .. } => "FunctionPointer",
        AstType::Struct { .. } => "Struct",
        AstType::Enum { .. } => "Enum",
        AstType::Ref(_) => "Ref",
        AstType::Range { .. } => "Range",
        AstType::Generic { .. } => "Generic",
        AstType::EnumType { .. } => "EnumType",
        AstType::StdModule => "StdModule",
    }
    .to_string()
}

fn pattern_variant_name(pat: &Pattern) -> String {
    match pat {
        Pattern::Literal(_) => "Literal",
        Pattern::Identifier(_) => "Identifier",
        Pattern::Struct { .. } => "Struct",
        Pattern::EnumVariant { .. } => "EnumVariant",
        Pattern::Wildcard => "Wildcard",
        Pattern::EnumLiteral { .. } => "EnumLiteral",
        Pattern::Or(_) => "Or",
        Pattern::Tuple(_) => "Tuple",
        Pattern::Range { .. } => "Range",
        Pattern::Binding { .. } => "Binding",
        Pattern::Type { .. } => "Type",
        Pattern::Guard { .. } => "Guard",
    }
    .to_string()
}

// ---------------------------------------------------------------------------
// meta.fields(node) -> []FieldInfo
// Returns fields of an AST node as an array of FieldInfo structs.
// ---------------------------------------------------------------------------

pub fn fields(node: &ASTNodeValue) -> Result<Vec<ComptimeValue>> {
    match node {
        ASTNodeValue::Expression(expr) => expression_fields(expr),
        ASTNodeValue::Statement(stmt) => statement_fields(stmt),
        ASTNodeValue::Declaration(decl) => declaration_fields(decl),
        ASTNodeValue::Type(ty) => type_fields(ty),
        ASTNodeValue::Pattern(pat) => pattern_fields(pat),
        ASTNodeValue::Program(prog) => Ok(vec![
            field_info(
                "declarations",
                ComptimeValue::Array(
                    prog.declarations
                        .iter()
                        .map(|d| ast_node(ASTNodeValue::Declaration(d.clone())))
                        .collect(),
                ),
            ),
            field_info(
                "statements",
                ComptimeValue::Array(
                    prog.statements
                        .iter()
                        .map(|s| ast_node(ASTNodeValue::Statement(s.clone())))
                        .collect(),
                ),
            ),
        ]),
    }
}

fn expression_fields(expr: &Expression) -> Result<Vec<ComptimeValue>> {
    Ok(match expr {
        // Literals - single value field
        Expression::Integer8(v) => vec![field_info("value", ComptimeValue::I8(*v))],
        Expression::Integer16(v) => vec![field_info("value", ComptimeValue::I16(*v))],
        Expression::Integer32(v) => vec![field_info("value", ComptimeValue::I32(*v))],
        Expression::Integer64(v) => vec![field_info("value", ComptimeValue::I64(*v))],
        Expression::Unsigned8(v) => vec![field_info("value", ComptimeValue::U8(*v))],
        Expression::Unsigned16(v) => vec![field_info("value", ComptimeValue::U16(*v))],
        Expression::Unsigned32(v) => vec![field_info("value", ComptimeValue::U32(*v))],
        Expression::Unsigned64(v) => vec![field_info("value", ComptimeValue::U64(*v))],
        Expression::Float32(v) => vec![field_info("value", ComptimeValue::F32(*v))],
        Expression::Float64(v) => vec![field_info("value", ComptimeValue::F64(*v))],
        Expression::Boolean(v) => vec![field_info("value", ComptimeValue::Bool(*v))],
        Expression::String(v) => vec![field_info("value", ComptimeValue::String(v.clone()))],
        Expression::Identifier(name) => {
            vec![field_info("name", ComptimeValue::String(name.clone()))]
        }
        Expression::Unit
        | Expression::None
        | Expression::StdReference
        | Expression::BuiltinReference
        | Expression::ThisReference => vec![],

        // Binary operations
        Expression::BinaryOp { left, op, right } => vec![
            field_info("left", ast_expr(*left.clone())),
            field_info("op", ComptimeValue::String(op.to_string())),
            field_info("right", ast_expr(*right.clone())),
        ],

        // Function call
        Expression::FunctionCall {
            name,
            type_args,
            args,
        } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info(
                "type_args",
                ComptimeValue::Array(type_args.iter().map(|t| ast_type(t.clone())).collect()),
            ),
            field_info(
                "args",
                ComptimeValue::Array(args.iter().map(|a| ast_expr(a.clone())).collect()),
            ),
        ],

        // Method call
        Expression::MethodCall {
            object,
            method,
            type_args,
            args,
        } => vec![
            field_info("object", ast_expr(*object.clone())),
            field_info("method", ComptimeValue::String(method.clone())),
            field_info(
                "type_args",
                ComptimeValue::Array(type_args.iter().map(|t| ast_type(t.clone())).collect()),
            ),
            field_info(
                "args",
                ComptimeValue::Array(args.iter().map(|a| ast_expr(a.clone())).collect()),
            ),
        ],

        // Pattern matching
        Expression::QuestionMatch { scrutinee, arms } => vec![
            field_info("scrutinee", ast_expr(*scrutinee.clone())),
            field_info(
                "arms",
                ComptimeValue::Array(
                    arms.iter()
                        .map(|arm| ComptimeValue::Struct {
                            name: "MatchArm".to_string(),
                            fields: HashMap::from([
                                ("pattern".to_string(), ast_pattern(arm.pattern.clone())),
                                (
                                    "guard".to_string(),
                                    match &arm.guard {
                                        Some(g) => ast_expr(g.clone()),
                                        None => ComptimeValue::Null,
                                    },
                                ),
                                ("body".to_string(), ast_expr(arm.body.clone())),
                            ]),
                        })
                        .collect(),
                ),
            ),
        ],

        // Struct literal
        Expression::StructLiteral { name, fields: fs } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info(
                "fields",
                ComptimeValue::Array(
                    fs.iter()
                        .map(|(n, e)| ComptimeValue::Struct {
                            name: "FieldInit".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(n.clone())),
                                ("value".to_string(), ast_expr(e.clone())),
                            ]),
                        })
                        .collect(),
                ),
            ),
        ],

        // Array literal
        Expression::ArrayLiteral(elems) => vec![field_info(
            "elements",
            ComptimeValue::Array(elems.iter().map(|e| ast_expr(e.clone())).collect()),
        )],

        // Array index
        Expression::ArrayIndex { array, index } => vec![
            field_info("array", ast_expr(*array.clone())),
            field_info("index", ast_expr(*index.clone())),
        ],

        // Enum variant
        Expression::EnumVariant {
            enum_name,
            variant,
            payload,
        } => vec![
            field_info("enum_name", ComptimeValue::String(enum_name.clone())),
            field_info("variant", ComptimeValue::String(variant.clone())),
            field_info("payload", opt_expr(payload)),
        ],

        Expression::EnumLiteral { variant, payload } => vec![
            field_info("variant", ComptimeValue::String(variant.clone())),
            field_info("payload", opt_expr(payload)),
        ],

        // Member access
        Expression::MemberAccess { object, member } => vec![
            field_info("object", ast_expr(*object.clone())),
            field_info("member", ComptimeValue::String(member.clone())),
        ],

        // Block
        Expression::Block(stmts) => vec![field_info(
            "statements",
            ComptimeValue::Array(stmts.iter().map(|s| ast_stmt(s.clone())).collect()),
        )],

        // Return
        Expression::Return(expr) => {
            vec![field_info("expr", ast_expr(*expr.clone()))]
        }

        // Raise
        Expression::Raise(expr) => {
            vec![field_info("expr", ast_expr(*expr.clone()))]
        }

        // Some
        Expression::Some(expr) => {
            vec![field_info("value", ast_expr(*expr.clone()))]
        }

        // Loop
        Expression::Loop { body } => {
            vec![field_info("body", ast_expr(*body.clone()))]
        }

        // Collection loop
        Expression::CollectionLoop {
            collection,
            param,
            index_param,
            body,
        } => vec![
            field_info("collection", ast_expr(*collection.clone())),
            field_info("param_name", ComptimeValue::String(param.0.clone())),
            field_info(
                "index_param_name",
                match index_param {
                    Some((name, _)) => ComptimeValue::String(name.clone()),
                    None => ComptimeValue::Null,
                },
            ),
            field_info("body", ast_expr(*body.clone())),
        ],

        // Closure
        Expression::Closure {
            params,
            return_type,
            body,
        } => vec![
            field_info(
                "params",
                ComptimeValue::Array(
                    params
                        .iter()
                        .map(|(name, ty)| ComptimeValue::Struct {
                            name: "Param".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(name.clone())),
                                (
                                    "type".to_string(),
                                    match ty {
                                        Some(t) => ast_type(t.clone()),
                                        None => ComptimeValue::Null,
                                    },
                                ),
                            ]),
                        })
                        .collect(),
                ),
            ),
            field_info(
                "return_type",
                match return_type {
                    Some(t) => ast_type(t.clone()),
                    None => ComptimeValue::Null,
                },
            ),
            field_info("body", ast_expr(*body.clone())),
        ],

        // Range
        Expression::Range {
            start,
            end,
            inclusive,
        } => vec![
            field_info("start", ast_expr(*start.clone())),
            field_info("end", ast_expr(*end.clone())),
            field_info("inclusive", ComptimeValue::Bool(*inclusive)),
        ],

        // Break / Continue
        Expression::Break { label, value } => vec![
            field_info("label", opt_string(label)),
            field_info("value", opt_expr(value)),
        ],
        Expression::Continue { label } => {
            vec![field_info("label", opt_string(label))]
        }

        // String interpolation
        Expression::StringInterpolation { parts } => vec![field_info(
            "parts",
            ComptimeValue::Array(
                parts
                    .iter()
                    .map(|part| match part {
                        ast::StringPart::Literal(s) => ComptimeValue::Struct {
                            name: "StringPart".to_string(),
                            fields: HashMap::from([
                                (
                                    "kind".to_string(),
                                    ComptimeValue::String("Literal".to_string()),
                                ),
                                ("value".to_string(), ComptimeValue::String(s.clone())),
                            ]),
                        },
                        ast::StringPart::Interpolation(e) => ComptimeValue::Struct {
                            name: "StringPart".to_string(),
                            fields: HashMap::from([
                                (
                                    "kind".to_string(),
                                    ComptimeValue::String("Interpolation".to_string()),
                                ),
                                ("value".to_string(), ast_expr(e.clone())),
                            ]),
                        },
                    })
                    .collect(),
            ),
        )],

        // Fallback for variants not yet detailed
        _ => vec![],
    })
}

fn statement_fields(stmt: &Statement) -> Result<Vec<ComptimeValue>> {
    Ok(match stmt {
        Statement::Expression { expr, .. } => {
            vec![field_info("expr", ast_expr(expr.clone()))]
        }
        Statement::Return { expr, .. } => {
            vec![field_info("expr", ast_expr(expr.clone()))]
        }
        Statement::VariableDeclaration {
            name,
            type_,
            initializer,
            is_mutable,
            ..
        } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info(
                "type",
                match type_ {
                    Some(t) => ast_type(t.clone()),
                    None => ComptimeValue::Null,
                },
            ),
            field_info(
                "initializer",
                match initializer {
                    Some(e) => ast_expr(e.clone()),
                    None => ComptimeValue::Null,
                },
            ),
            field_info("is_mutable", ComptimeValue::Bool(*is_mutable)),
        ],
        Statement::VariableAssignment { name, value, .. } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info("value", ast_expr(value.clone())),
        ],
        Statement::PointerAssignment { pointer, value, .. } => vec![
            field_info("pointer", ast_expr(pointer.clone())),
            field_info("value", ast_expr(value.clone())),
        ],
        Statement::Loop {
            kind, label, body, ..
        } => vec![
            field_info(
                "kind",
                match kind {
                    ast::LoopKind::Infinite => ComptimeValue::String("Infinite".to_string()),
                    ast::LoopKind::Condition(cond) => ComptimeValue::Struct {
                        name: "LoopKind".to_string(),
                        fields: HashMap::from([
                            (
                                "kind".to_string(),
                                ComptimeValue::String("Condition".to_string()),
                            ),
                            ("condition".to_string(), ast_expr(cond.clone())),
                        ]),
                    },
                },
            ),
            field_info("label", opt_string(label)),
            field_info(
                "body",
                ComptimeValue::Array(body.iter().map(|s| ast_stmt(s.clone())).collect()),
            ),
        ],
        Statement::Break { label, .. } => {
            vec![field_info("label", opt_string(label))]
        }
        Statement::Continue { label, .. } => {
            vec![field_info("label", opt_string(label))]
        }
        Statement::DestructuringImport { names, source, .. } => vec![
            field_info(
                "names",
                ComptimeValue::Array(
                    names
                        .iter()
                        .map(|n| ComptimeValue::String(n.clone()))
                        .collect(),
                ),
            ),
            field_info("source", ast_expr(source.clone())),
        ],
        Statement::Block { statements, .. } => vec![field_info(
            "statements",
            ComptimeValue::Array(statements.iter().map(|s| ast_stmt(s.clone())).collect()),
        )],
        _ => vec![],
    })
}

fn declaration_fields(decl: &Declaration) -> Result<Vec<ComptimeValue>> {
    Ok(match decl {
        Declaration::Function(f) => vec![
            field_info("name", ComptimeValue::String(f.name.clone())),
            field_info(
                "args",
                ComptimeValue::Array(
                    f.args
                        .iter()
                        .map(|(name, ty)| ComptimeValue::Struct {
                            name: "Param".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(name.clone())),
                                ("type".to_string(), ast_type(ty.clone())),
                            ]),
                        })
                        .collect(),
                ),
            ),
            field_info("return_type", ast_type(f.return_type.clone())),
            field_info(
                "body",
                ComptimeValue::Array(f.body.iter().map(|s| ast_stmt(s.clone())).collect()),
            ),
            field_info("is_public", ComptimeValue::Bool(f.is_public)),
            field_info("is_varargs", ComptimeValue::Bool(f.is_varargs)),
        ],
        Declaration::Struct(s) => vec![
            field_info("name", ComptimeValue::String(s.name.clone())),
            field_info(
                "fields",
                ComptimeValue::Array(
                    s.fields
                        .iter()
                        .map(|f| ComptimeValue::Struct {
                            name: "StructField".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(f.name.clone())),
                                ("type".to_string(), ast_type(f.type_.clone())),
                                ("is_mutable".to_string(), ComptimeValue::Bool(f.is_mutable)),
                            ]),
                        })
                        .collect(),
                ),
            ),
            field_info(
                "methods",
                ComptimeValue::Array(
                    s.methods
                        .iter()
                        .map(|m| {
                            ast_node(ASTNodeValue::Declaration(Declaration::Function(m.clone())))
                        })
                        .collect(),
                ),
            ),
        ],
        Declaration::Enum(e) => vec![
            field_info("name", ComptimeValue::String(e.name.clone())),
            field_info(
                "variants",
                ComptimeValue::Array(
                    e.variants
                        .iter()
                        .map(|v| ComptimeValue::Struct {
                            name: "EnumVariant".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(v.name.clone())),
                                (
                                    "payload".to_string(),
                                    match &v.payload {
                                        Some(t) => ast_type(t.clone()),
                                        None => ComptimeValue::Null,
                                    },
                                ),
                            ]),
                        })
                        .collect(),
                ),
            ),
            field_info(
                "methods",
                ComptimeValue::Array(
                    e.methods
                        .iter()
                        .map(|m| {
                            ast_node(ASTNodeValue::Declaration(Declaration::Function(m.clone())))
                        })
                        .collect(),
                ),
            ),
        ],
        Declaration::Constant { name, value, .. } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info("value", ast_expr(value.clone())),
        ],
        Declaration::ImplBlock(imp) => vec![
            field_info("type_name", ComptimeValue::String(imp.type_name.clone())),
            field_info(
                "methods",
                ComptimeValue::Array(
                    imp.methods
                        .iter()
                        .map(|m| {
                            ast_node(ASTNodeValue::Declaration(Declaration::Function(m.clone())))
                        })
                        .collect(),
                ),
            ),
        ],
        Declaration::TraitImplementation(ti) => vec![
            field_info("type_name", ComptimeValue::String(ti.type_name.clone())),
            field_info("trait_name", ComptimeValue::String(ti.trait_name.clone())),
            field_info(
                "methods",
                ComptimeValue::Array(
                    ti.methods
                        .iter()
                        .map(|m| {
                            ast_node(ASTNodeValue::Declaration(Declaration::Function(m.clone())))
                        })
                        .collect(),
                ),
            ),
        ],
        Declaration::ExternalFunction(ef) => vec![
            field_info("name", ComptimeValue::String(ef.name.clone())),
            field_info(
                "args",
                ComptimeValue::Array(ef.args.iter().map(|t| ast_type(t.clone())).collect()),
            ),
            field_info("return_type", ast_type(ef.return_type.clone())),
            field_info("is_varargs", ComptimeValue::Bool(ef.is_varargs)),
        ],
        _ => vec![],
    })
}

fn type_fields(ty: &AstType) -> Result<Vec<ComptimeValue>> {
    Ok(match ty {
        AstType::Slice(inner) => {
            vec![field_info("element_type", ast_type(*inner.clone()))]
        }
        AstType::FixedArray { element_type, size } => vec![
            field_info("element_type", ast_type(*element_type.clone())),
            field_info("size", ComptimeValue::U64(*size as u64)),
        ],
        AstType::Function { args, return_type } => vec![
            field_info(
                "args",
                ComptimeValue::Array(args.iter().map(|t| ast_type(t.clone())).collect()),
            ),
            field_info("return_type", ast_type(*return_type.clone())),
        ],
        AstType::Struct { name, fields: fs } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info(
                "fields",
                ComptimeValue::Array(
                    fs.iter()
                        .map(|(n, t)| ComptimeValue::Struct {
                            name: "TypeField".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(n.clone())),
                                ("type".to_string(), ast_type(t.clone())),
                            ]),
                        })
                        .collect(),
                ),
            ),
        ],
        AstType::Generic { name, type_args } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info(
                "type_args",
                ComptimeValue::Array(type_args.iter().map(|t| ast_type(t.clone())).collect()),
            ),
        ],
        AstType::Ref(inner) => {
            vec![field_info("inner", ast_type(*inner.clone()))]
        }
        AstType::Range {
            start_type,
            end_type,
            inclusive,
        } => vec![
            field_info("start_type", ast_type(*start_type.clone())),
            field_info("end_type", ast_type(*end_type.clone())),
            field_info("inclusive", ComptimeValue::Bool(*inclusive)),
        ],
        // Primitives and unit types have no fields
        _ => vec![],
    })
}

fn pattern_fields(pat: &Pattern) -> Result<Vec<ComptimeValue>> {
    Ok(match pat {
        Pattern::Literal(expr) => vec![field_info("value", ast_expr(expr.clone()))],
        Pattern::Identifier(name) => {
            vec![field_info("name", ComptimeValue::String(name.clone()))]
        }
        Pattern::EnumVariant {
            enum_name,
            variant,
            payload,
        } => vec![
            field_info("enum_name", ComptimeValue::String(enum_name.clone())),
            field_info("variant", ComptimeValue::String(variant.clone())),
            field_info(
                "payload",
                match payload {
                    Some(p) => ast_pattern(*p.clone()),
                    None => ComptimeValue::Null,
                },
            ),
        ],
        Pattern::EnumLiteral { variant, payload } => vec![
            field_info("variant", ComptimeValue::String(variant.clone())),
            field_info(
                "payload",
                match payload {
                    Some(p) => ast_pattern(*p.clone()),
                    None => ComptimeValue::Null,
                },
            ),
        ],
        Pattern::Wildcard => vec![],
        Pattern::Or(pats) => vec![field_info(
            "patterns",
            ComptimeValue::Array(pats.iter().map(|p| ast_pattern(p.clone())).collect()),
        )],
        Pattern::Guard { pattern, condition } => vec![
            field_info("pattern", ast_pattern(*pattern.clone())),
            field_info("condition", ast_expr(*condition.clone())),
        ],
        _ => vec![],
    })
}

// ---------------------------------------------------------------------------
// meta.type_info(node) -> TypeInfo
// Returns a TypeInfo struct with variant_name and fields.
// ---------------------------------------------------------------------------

pub fn type_info(node: &ASTNodeValue) -> Result<ComptimeValue> {
    let vname = variant_name(node);
    let flds = fields(node)?;

    Ok(ComptimeValue::Struct {
        name: "TypeInfo".to_string(),
        fields: HashMap::from([
            ("variant".to_string(), ComptimeValue::String(vname)),
            ("fields".to_string(), ComptimeValue::Array(flds)),
            (
                "kind".to_string(),
                ComptimeValue::String(
                    match node {
                        ASTNodeValue::Expression(_) => "Expression",
                        ASTNodeValue::Statement(_) => "Statement",
                        ASTNodeValue::Declaration(_) => "Declaration",
                        ASTNodeValue::Type(_) => "Type",
                        ASTNodeValue::Pattern(_) => "Pattern",
                        ASTNodeValue::Program(_) => "Program",
                    }
                    .to_string(),
                ),
            ),
        ]),
    })
}

// ---------------------------------------------------------------------------
// meta.children(node) -> []ASTNode
// Returns all child AST nodes (for generic tree traversal).
// ---------------------------------------------------------------------------

pub fn children(node: &ASTNodeValue) -> Result<Vec<ComptimeValue>> {
    let flds = fields(node)?;
    let mut result = Vec::new();

    for f in &flds {
        if let ComptimeValue::Struct { fields, .. } = f {
            if let Some(value) = fields.get("value") {
                collect_ast_nodes(value, &mut result);
            }
        }
    }

    Ok(result)
}

fn collect_ast_nodes(value: &ComptimeValue, out: &mut Vec<ComptimeValue>) {
    match value {
        ComptimeValue::ASTNode(_) => out.push(value.clone()),
        ComptimeValue::Array(items) => {
            for item in items {
                collect_ast_nodes(item, out);
            }
        }
        ComptimeValue::Struct { fields, .. } => {
            for v in fields.values() {
                collect_ast_nodes(v, out);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Binary operator helpers
// ---------------------------------------------------------------------------

pub fn binary_op_to_comptime(op: &BinaryOperator) -> ComptimeValue {
    ComptimeValue::String(op.to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOperator, Expression, Statement};

    #[test]
    fn test_variant_name_expression() {
        let expr = Expression::Integer32(42);
        let node = ASTNodeValue::Expression(expr);
        assert_eq!(variant_name(&node), "Integer32");
    }

    #[test]
    fn test_variant_name_binary_op() {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Integer32(2)),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Integer32(3)),
        };
        let node = ASTNodeValue::Expression(expr);
        assert_eq!(variant_name(&node), "BinaryOp");
    }

    #[test]
    fn test_fields_integer() {
        let expr = Expression::Integer32(42);
        let node = ASTNodeValue::Expression(expr);
        let flds = fields(&node).unwrap();
        assert_eq!(flds.len(), 1);
        // Check the field is named "value"
        if let ComptimeValue::Struct { fields: f, .. } = &flds[0] {
            assert!(f.contains_key("name"));
            if let ComptimeValue::String(name) = &f["name"] {
                assert_eq!(name, "value");
            }
        }
    }

    #[test]
    fn test_fields_binary_op() {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Integer32(2)),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Integer32(3)),
        };
        let node = ASTNodeValue::Expression(expr);
        let flds = fields(&node).unwrap();
        assert_eq!(flds.len(), 3); // left, op, right
    }

    #[test]
    fn test_fields_function_call() {
        let expr = Expression::FunctionCall {
            name: "add".to_string(),
            type_args: vec![],
            args: vec![Expression::Integer32(1), Expression::Integer32(2)],
        };
        let node = ASTNodeValue::Expression(expr);
        let flds = fields(&node).unwrap();
        assert_eq!(flds.len(), 3); // name, type_args, args
    }

    #[test]
    fn test_type_info_returns_struct() {
        let expr = Expression::Integer32(42);
        let node = ASTNodeValue::Expression(expr);
        let info = type_info(&node).unwrap();

        if let ComptimeValue::Struct { name, fields: f } = &info {
            assert_eq!(name, "TypeInfo");
            assert!(f.contains_key("variant"));
            assert!(f.contains_key("fields"));
            assert!(f.contains_key("kind"));

            if let ComptimeValue::String(v) = &f["variant"] {
                assert_eq!(v, "Integer32");
            }
            if let ComptimeValue::String(k) = &f["kind"] {
                assert_eq!(k, "Expression");
            }
        } else {
            panic!("Expected TypeInfo struct");
        }
    }

    #[test]
    fn test_fields_variable_declaration() {
        let stmt = Statement::VariableDeclaration {
            name: "x".to_string(),
            type_: Some(AstType::I32),
            initializer: Some(Expression::Integer32(10)),
            is_mutable: false,
            declaration_type: crate::ast::VariableDeclarationType::InferredImmutable,
            span: None,
        };
        let node = ASTNodeValue::Statement(stmt);
        let flds = fields(&node).unwrap();
        assert_eq!(flds.len(), 4); // name, type, initializer, is_mutable
    }

    #[test]
    fn test_fields_function_declaration() {
        let func = ast::Function {
            name: "add".to_string(),
            type_params: vec![],
            args: vec![
                ("a".to_string(), AstType::I32),
                ("b".to_string(), AstType::I32),
            ],
            return_type: AstType::I32,
            body: vec![Statement::Return {
                expr: Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                },
                span: None,
            }],
            is_varargs: false,
            is_public: false,
        };
        let node = ASTNodeValue::Declaration(Declaration::Function(func));
        let flds = fields(&node).unwrap();
        assert_eq!(flds.len(), 6); // name, args, return_type, body, is_public, is_varargs
    }

    #[test]
    fn test_fields_program() {
        let prog = ast::Program {
            declarations: vec![],
            statements: vec![],
        };
        let node = ASTNodeValue::Program(prog);
        let flds = fields(&node).unwrap();
        assert_eq!(flds.len(), 2); // declarations, statements
    }

    #[test]
    fn test_variant_name_all_statement_types() {
        // Ensure variant_name covers all statement types
        let stmts = vec![
            Statement::Expression {
                expr: Expression::Unit,
                span: None,
            },
            Statement::Return {
                expr: Expression::Unit,
                span: None,
            },
            Statement::Break {
                label: None,
                span: None,
            },
            Statement::Continue {
                label: None,
                span: None,
            },
        ];

        let expected = vec!["Expression", "Return", "Break", "Continue"];

        for (stmt, exp) in stmts.iter().zip(expected.iter()) {
            let node = ASTNodeValue::Statement(stmt.clone());
            assert_eq!(variant_name(&node), *exp);
        }
    }
}
