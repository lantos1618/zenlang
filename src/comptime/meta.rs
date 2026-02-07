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

fn opt_pattern(p: &Option<Box<Pattern>>) -> ComptimeValue {
    match p {
        Some(pat) => ast_pattern(*pat.clone()),
        None => ComptimeValue::Null,
    }
}

fn opt_type(t: &Option<AstType>) -> ComptimeValue {
    match t {
        Some(ty) => ast_type(ty.clone()),
        None => ComptimeValue::Null,
    }
}

// decl_type_to_string is replaced by VariableDeclarationType's Display impl

fn function_arg(name: &str, ty: &AstType) -> ComptimeValue {
    ComptimeValue::Struct {
        name: "FunctionArg".to_string(),
        fields: HashMap::from([
            ("name".to_string(), ComptimeValue::String(name.to_string())),
            ("arg_type".to_string(), ast_type(ty.clone())),
        ]),
    }
}

fn function_to_fields(f: &ast::Function) -> Vec<ComptimeValue> {
    vec![
        field_info("name", ComptimeValue::String(f.name.clone())),
        field_info(
            "type_params",
            ComptimeValue::Array(
                f.type_params
                    .iter()
                    .map(|tp| ComptimeValue::Struct {
                        name: "TypeParameter".to_string(),
                        fields: HashMap::from([
                            ("name".to_string(), ComptimeValue::String(tp.name.clone())),
                            (
                                "constraints".to_string(),
                                ComptimeValue::Array(
                                    tp.constraints
                                        .iter()
                                        .map(|c| ComptimeValue::Struct {
                                            name: "TraitConstraint".to_string(),
                                            fields: HashMap::from([(
                                                "trait_name".to_string(),
                                                ComptimeValue::String(c.trait_name.clone()),
                                            )]),
                                        })
                                        .collect(),
                                ),
                            ),
                        ]),
                    })
                    .collect(),
            ),
        ),
        field_info(
            "args",
            ComptimeValue::Array(
                f.args
                    .iter()
                    .map(|(name, ty)| function_arg(name, ty))
                    .collect(),
            ),
        ),
        field_info("return_type", ast_type(f.return_type.clone())),
        field_info(
            "body",
            ComptimeValue::Array(f.body.iter().map(|s| ast_stmt(s.clone())).collect()),
        ),
        field_info("is_varargs", ComptimeValue::Bool(f.is_varargs)),
        field_info("is_public", ComptimeValue::Bool(f.is_public)),
    ]
}

fn type_params_to_array(tps: &[ast::TypeParameter]) -> ComptimeValue {
    ComptimeValue::Array(
        tps.iter()
            .map(|tp| ComptimeValue::Struct {
                name: "TypeParameter".to_string(),
                fields: HashMap::from([
                    ("name".to_string(), ComptimeValue::String(tp.name.clone())),
                    (
                        "constraints".to_string(),
                        ComptimeValue::Array(
                            tp.constraints
                                .iter()
                                .map(|c| ComptimeValue::Struct {
                                    name: "TraitConstraint".to_string(),
                                    fields: HashMap::from([(
                                        "trait_name".to_string(),
                                        ComptimeValue::String(c.trait_name.clone()),
                                    )]),
                                })
                                .collect(),
                        ),
                    ),
                ]),
            })
            .collect(),
    )
}

fn methods_to_array(methods: &[ast::Function]) -> ComptimeValue {
    ComptimeValue::Array(
        methods
            .iter()
            .map(|m| ast_node(ASTNodeValue::Declaration(Declaration::Function(m.clone()))))
            .collect(),
    )
}

fn parameter_to_value(p: &ast::Parameter) -> ComptimeValue {
    ComptimeValue::Struct {
        name: "Parameter".to_string(),
        fields: HashMap::from([
            ("name".to_string(), ComptimeValue::String(p.name.clone())),
            ("param_type".to_string(), ast_type(p.type_.clone())),
            ("is_mutable".to_string(), ComptimeValue::Bool(p.is_mutable)),
        ]),
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

// Variant name functions delegate to the methods on the AST enums themselves.
// The canonical implementation lives in src/ast/*.rs — these are thin wrappers
// kept for the meta::variant_name(ASTNodeValue) public API.

fn expression_variant_name(expr: &Expression) -> String {
    expr.variant_name().to_string()
}

fn statement_variant_name(stmt: &Statement) -> String {
    stmt.variant_name().to_string()
}

fn declaration_variant_name(decl: &Declaration) -> String {
    decl.variant_name().to_string()
}

fn type_variant_name(ty: &AstType) -> String {
    ty.variant_name().to_string()
}

fn pattern_variant_name(pat: &Pattern) -> String {
    pat.variant_name().to_string()
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

        // Conditional expression
        Expression::Conditional { scrutinee, arms } => vec![
            field_info("scrutinee", ast_expr(*scrutinee.clone())),
            field_info(
                "arms",
                ComptimeValue::Array(
                    arms.iter()
                        .map(|arm| ComptimeValue::Struct {
                            name: "ConditionalArm".to_string(),
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

        // Pattern match expression
        Expression::PatternMatch { scrutinee, arms } => vec![
            field_info("scrutinee", ast_expr(*scrutinee.clone())),
            field_info(
                "arms",
                ComptimeValue::Array(
                    arms.iter()
                        .map(|arm| ComptimeValue::Struct {
                            name: "PatternArm".to_string(),
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

        // Memory operations
        Expression::AddressOf(inner) => {
            vec![field_info("expr", ast_expr(*inner.clone()))]
        }
        Expression::Dereference(inner) => {
            vec![field_info("expr", ast_expr(*inner.clone()))]
        }
        Expression::PointerOffset { pointer, offset } => vec![
            field_info("pointer", ast_expr(*pointer.clone())),
            field_info("offset", ast_expr(*offset.clone())),
        ],
        Expression::PointerDereference(inner) => {
            vec![field_info("expr", ast_expr(*inner.clone()))]
        }
        Expression::PointerAddress(inner) => {
            vec![field_info("expr", ast_expr(*inner.clone()))]
        }
        Expression::CreateReference(inner) => {
            vec![field_info("expr", ast_expr(*inner.clone()))]
        }
        Expression::CreateMutableReference(inner) => {
            vec![field_info("expr", ast_expr(*inner.clone()))]
        }

        // Struct literal
        Expression::StructLiteral { name, fields: fs } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info(
                "fields",
                ComptimeValue::Array(
                    fs.iter()
                        .map(|(n, e)| ComptimeValue::Struct {
                            name: "StructFieldInit".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(n.clone())),
                                ("value".to_string(), ast_expr(e.clone())),
                            ]),
                        })
                        .collect(),
                ),
            ),
        ],

        // Struct field access
        Expression::StructField { struct_, field } => vec![
            field_info("struct_expr", ast_expr(*struct_.clone())),
            field_info("field", ComptimeValue::String(field.clone())),
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

        // Array constructor
        Expression::ArrayConstructor { element_type } => {
            vec![field_info("element_type", ast_type(element_type.clone()))]
        }

        // Vec constructor
        Expression::VecConstructor {
            element_type,
            size,
            initial_values,
        } => vec![
            field_info("element_type", ast_type(element_type.clone())),
            field_info("size", ComptimeValue::I64(*size as i64)),
            field_info(
                "initial_values",
                match initial_values {
                    Some(vals) => {
                        ComptimeValue::Array(vals.iter().map(|e| ast_expr(e.clone())).collect())
                    }
                    None => ComptimeValue::Array(vec![]),
                },
            ),
        ],

        // DynVec constructor
        Expression::DynVecConstructor {
            element_types,
            allocator,
            initial_capacity,
        } => vec![
            field_info(
                "element_types",
                ComptimeValue::Array(element_types.iter().map(|t| ast_type(t.clone())).collect()),
            ),
            field_info("allocator", ast_expr(*allocator.clone())),
            field_info(
                "initial_capacity",
                match initial_capacity {
                    Some(cap) => ast_expr(*cap.clone()),
                    None => ComptimeValue::Null,
                },
            ),
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

        // String length
        Expression::StringLength(inner) => {
            vec![field_info("expr", ast_expr(*inner.clone()))]
        }

        // Some
        Expression::Some(expr) => {
            vec![field_info("inner", ast_expr(*expr.clone()))]
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
                                ("expr".to_string(), ast_expr(e.clone())),
                            ]),
                        },
                    })
                    .collect(),
            ),
        )],

        // Comptime
        Expression::Comptime(inner) => {
            vec![field_info("expr", ast_expr(*inner.clone()))]
        }

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
                "param_type",
                match &param.1 {
                    Some(t) => ast_type(t.clone()),
                    None => ComptimeValue::Null,
                },
            ),
            field_info(
                "index_name",
                match index_param {
                    Some((name, _)) => ComptimeValue::String(name.clone()),
                    None => ComptimeValue::String(String::new()),
                },
            ),
            field_info(
                "index_type",
                match index_param {
                    Some((_, Some(t))) => ast_type(t.clone()),
                    _ => ComptimeValue::Null,
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
                            name: "ClosureParam".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(name.clone())),
                                (
                                    "param_type".to_string(),
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

        // Defer
        Expression::Defer(expr) => {
            vec![field_info("expr", ast_expr(*expr.clone()))]
        }

        // Break / Continue
        Expression::Break { label, value } => vec![
            field_info(
                "label",
                match label {
                    Some(l) => ComptimeValue::String(l.clone()),
                    None => ComptimeValue::String(String::new()),
                },
            ),
            field_info("value", opt_expr(value)),
        ],
        Expression::Continue { label } => {
            vec![field_info(
                "label",
                match label {
                    Some(l) => ComptimeValue::String(l.clone()),
                    None => ComptimeValue::String(String::new()),
                },
            )]
        }
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
            declaration_type,
            ..
        } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info("var_type", opt_type(type_)),
            field_info(
                "initializer",
                match initializer {
                    Some(e) => ast_expr(e.clone()),
                    None => ComptimeValue::Null,
                },
            ),
            field_info("is_mutable", ComptimeValue::Bool(*is_mutable)),
            field_info(
                "declaration_type",
                ComptimeValue::String(declaration_type.to_string()),
            ),
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
            field_info(
                "label",
                match label {
                    Some(l) => ComptimeValue::String(l.clone()),
                    None => ComptimeValue::String(String::new()),
                },
            ),
            field_info(
                "body",
                ComptimeValue::Array(body.iter().map(|s| ast_stmt(s.clone())).collect()),
            ),
        ],
        Statement::Break { label, .. } => {
            vec![field_info(
                "label",
                match label {
                    Some(l) => ComptimeValue::String(l.clone()),
                    None => ComptimeValue::String(String::new()),
                },
            )]
        }
        Statement::Continue { label, .. } => {
            vec![field_info(
                "label",
                match label {
                    Some(l) => ComptimeValue::String(l.clone()),
                    None => ComptimeValue::String(String::new()),
                },
            )]
        }
        Statement::ComptimeBlock { statements, .. } => vec![field_info(
            "statements",
            ComptimeValue::Array(statements.iter().map(|s| ast_stmt(s.clone())).collect()),
        )],
        Statement::ModuleImport { alias, module_path } => vec![
            field_info("alias", ComptimeValue::String(alias.clone())),
            field_info("module_path", ComptimeValue::String(module_path.clone())),
        ],
        Statement::Defer { statement, .. } => {
            vec![field_info("statement", ast_stmt(*statement.clone()))]
        }
        Statement::ThisDefer { expr, .. } => {
            vec![field_info("expr", ast_expr(expr.clone()))]
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
    })
}

fn declaration_fields(decl: &Declaration) -> Result<Vec<ComptimeValue>> {
    Ok(match decl {
        Declaration::Function(f) => function_to_fields(f),
        Declaration::ExternalFunction(ef) => vec![
            field_info("name", ComptimeValue::String(ef.name.clone())),
            field_info(
                "args",
                ComptimeValue::Array(ef.args.iter().map(|t| ast_type(t.clone())).collect()),
            ),
            field_info("return_type", ast_type(ef.return_type.clone())),
            field_info("is_varargs", ComptimeValue::Bool(ef.is_varargs)),
        ],
        Declaration::Struct(s) => vec![
            field_info("name", ComptimeValue::String(s.name.clone())),
            field_info("type_params", type_params_to_array(&s.type_params)),
            field_info(
                "fields",
                ComptimeValue::Array(
                    s.fields
                        .iter()
                        .map(|f| ComptimeValue::Struct {
                            name: "StructField".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(f.name.clone())),
                                ("field_type".to_string(), ast_type(f.type_.clone())),
                                ("is_mutable".to_string(), ComptimeValue::Bool(f.is_mutable)),
                                (
                                    "default_value".to_string(),
                                    match &f.default_value {
                                        Some(e) => ast_expr(e.clone()),
                                        None => ComptimeValue::Null,
                                    },
                                ),
                            ]),
                        })
                        .collect(),
                ),
            ),
            field_info("methods", methods_to_array(&s.methods)),
        ],
        Declaration::Enum(e) => vec![
            field_info("name", ComptimeValue::String(e.name.clone())),
            field_info("type_params", type_params_to_array(&e.type_params)),
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
            field_info("methods", methods_to_array(&e.methods)),
            field_info(
                "required_traits",
                ComptimeValue::Array(
                    e.required_traits
                        .iter()
                        .map(|t| ComptimeValue::String(t.clone()))
                        .collect(),
                ),
            ),
        ],
        Declaration::Behavior(b) => vec![
            field_info("name", ComptimeValue::String(b.name.clone())),
            field_info("type_params", type_params_to_array(&b.type_params)),
            field_info(
                "methods",
                ComptimeValue::Array(
                    b.methods
                        .iter()
                        .map(|m| ComptimeValue::Struct {
                            name: "BehaviorMethod".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(m.name.clone())),
                                (
                                    "params".to_string(),
                                    ComptimeValue::Array(
                                        m.params.iter().map(parameter_to_value).collect(),
                                    ),
                                ),
                                ("return_type".to_string(), ast_type(m.return_type.clone())),
                            ]),
                        })
                        .collect(),
                ),
            ),
        ],
        Declaration::Trait(t) => vec![
            field_info("name", ComptimeValue::String(t.name.clone())),
            field_info("type_params", type_params_to_array(&t.type_params)),
            field_info(
                "methods",
                ComptimeValue::Array(
                    t.methods
                        .iter()
                        .map(|m| ComptimeValue::Struct {
                            name: "TraitMethod".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(m.name.clone())),
                                (
                                    "params".to_string(),
                                    ComptimeValue::Array(
                                        m.params.iter().map(parameter_to_value).collect(),
                                    ),
                                ),
                                ("return_type".to_string(), ast_type(m.return_type.clone())),
                            ]),
                        })
                        .collect(),
                ),
            ),
        ],
        Declaration::TraitImplementation(ti) => vec![
            field_info("type_name", ComptimeValue::String(ti.type_name.clone())),
            field_info("trait_name", ComptimeValue::String(ti.trait_name.clone())),
            field_info("type_params", type_params_to_array(&ti.type_params)),
            field_info("methods", methods_to_array(&ti.methods)),
        ],
        Declaration::TraitRequirement(tr) => vec![
            field_info("type_name", ComptimeValue::String(tr.type_name.clone())),
            field_info("trait_name", ComptimeValue::String(tr.trait_name.clone())),
        ],
        Declaration::ImplBlock(imp) => vec![
            field_info("type_name", ComptimeValue::String(imp.type_name.clone())),
            field_info("type_params", type_params_to_array(&imp.type_params)),
            field_info("methods", methods_to_array(&imp.methods)),
        ],
        Declaration::ComptimeBlock(stmts) => vec![field_info(
            "statements",
            ComptimeValue::Array(stmts.iter().map(|s| ast_stmt(s.clone())).collect()),
        )],
        Declaration::Constant {
            name, value, type_, ..
        } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info("value", ast_expr(value.clone())),
            field_info("const_type", opt_type(type_)),
        ],
        Declaration::ModuleImport {
            alias, module_path, ..
        } => vec![
            field_info("alias", ComptimeValue::String(alias.clone())),
            field_info("module_path", ComptimeValue::String(module_path.clone())),
        ],
        Declaration::Export { symbols } => vec![field_info(
            "symbols",
            ComptimeValue::Array(
                symbols
                    .iter()
                    .map(|s| ComptimeValue::String(s.clone()))
                    .collect(),
            ),
        )],
        Declaration::TypeAlias(ta) => vec![
            field_info("name", ComptimeValue::String(ta.name.clone())),
            field_info("type_params", type_params_to_array(&ta.type_params)),
            field_info("target_type", ast_type(ta.target_type.clone())),
        ],
    })
}

fn type_fields(ty: &AstType) -> Result<Vec<ComptimeValue>> {
    Ok(match ty {
        // Primitives have no fields
        AstType::I8
        | AstType::I16
        | AstType::I32
        | AstType::I64
        | AstType::U8
        | AstType::U16
        | AstType::U32
        | AstType::U64
        | AstType::Usize
        | AstType::F32
        | AstType::F64
        | AstType::Bool
        | AstType::StaticLiteral
        | AstType::StaticString
        | AstType::Void
        | AstType::StdModule => vec![],

        AstType::Slice(inner) => {
            vec![field_info("element_type", ast_type(*inner.clone()))]
        }
        AstType::FixedArray { element_type, size } => vec![
            field_info("element_type", ast_type(*element_type.clone())),
            field_info("size", ComptimeValue::I64(*size as i64)),
        ],
        AstType::Function { args, return_type } => vec![
            field_info(
                "args",
                ComptimeValue::Array(args.iter().map(|t| ast_type(t.clone())).collect()),
            ),
            field_info("return_type", ast_type(*return_type.clone())),
        ],
        AstType::FunctionPointer {
            param_types,
            return_type,
        } => vec![
            field_info(
                "param_types",
                ComptimeValue::Array(param_types.iter().map(|t| ast_type(t.clone())).collect()),
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
                            name: "StructTypeField".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(n.clone())),
                                ("field_type".to_string(), ast_type(t.clone())),
                            ]),
                        })
                        .collect(),
                ),
            ),
        ],
        AstType::Enum { name, variants } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info(
                "variants",
                ComptimeValue::Array(
                    variants
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
        AstType::Generic { name, type_args } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info(
                "type_args",
                ComptimeValue::Array(type_args.iter().map(|t| ast_type(t.clone())).collect()),
            ),
        ],
        AstType::EnumType { name } => {
            vec![field_info("name", ComptimeValue::String(name.clone()))]
        }
    })
}

fn pattern_fields(pat: &Pattern) -> Result<Vec<ComptimeValue>> {
    Ok(match pat {
        Pattern::Literal(expr) => vec![field_info("value", ast_expr(expr.clone()))],
        Pattern::Identifier(name) => {
            vec![field_info("name", ComptimeValue::String(name.clone()))]
        }
        Pattern::Struct { name, fields: fs } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info(
                "fields",
                ComptimeValue::Array(
                    fs.iter()
                        .map(|(n, p)| ComptimeValue::Struct {
                            name: "PatternField".to_string(),
                            fields: HashMap::from([
                                ("name".to_string(), ComptimeValue::String(n.clone())),
                                ("pattern".to_string(), ast_pattern(p.clone())),
                            ]),
                        })
                        .collect(),
                ),
            ),
        ],
        Pattern::EnumVariant {
            enum_name,
            variant,
            payload,
        } => vec![
            field_info("enum_name", ComptimeValue::String(enum_name.clone())),
            field_info("variant", ComptimeValue::String(variant.clone())),
            field_info("payload", opt_pattern(payload)),
        ],
        Pattern::Wildcard => vec![],
        Pattern::EnumLiteral { variant, payload } => vec![
            field_info("variant", ComptimeValue::String(variant.clone())),
            field_info("payload", opt_pattern(payload)),
        ],
        Pattern::Or(pats) => vec![field_info(
            "patterns",
            ComptimeValue::Array(pats.iter().map(|p| ast_pattern(p.clone())).collect()),
        )],
        Pattern::Tuple(pats) => vec![field_info(
            "patterns",
            ComptimeValue::Array(pats.iter().map(|p| ast_pattern(p.clone())).collect()),
        )],
        Pattern::Range {
            start,
            end,
            inclusive,
        } => vec![
            field_info("start", ast_expr(*start.clone())),
            field_info("end", ast_expr(*end.clone())),
            field_info("inclusive", ComptimeValue::Bool(*inclusive)),
        ],
        Pattern::Binding { name, pattern } => vec![
            field_info("name", ComptimeValue::String(name.clone())),
            field_info("pattern", ast_pattern(*pattern.clone())),
        ],
        Pattern::Type { type_name, binding } => vec![
            field_info("type_name", ComptimeValue::String(type_name.clone())),
            field_info(
                "binding",
                match binding {
                    Some(b) => ComptimeValue::String(b.clone()),
                    None => ComptimeValue::String(String::new()),
                },
            ),
        ],
        Pattern::Guard { pattern, condition } => vec![
            field_info("pattern", ast_pattern(*pattern.clone())),
            field_info("condition", ast_expr(*condition.clone())),
        ],
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
// AST variant name constants for Zen scope
// These are exposed as meta.Expression, meta.Statement, etc. so Zen code
// can write `meta.Expression.BinaryOp` instead of `"BinaryOp"`.
// ---------------------------------------------------------------------------

/// Helper: build a struct of variant name constants (each field maps name → name string)
fn variant_constants(name: &str, variants: &[&str]) -> ComptimeValue {
    ComptimeValue::Struct {
        name: name.to_string(),
        fields: variants
            .iter()
            .map(|v| (v.to_string(), ComptimeValue::String(v.to_string())))
            .collect(),
    }
}

/// Expression variant name constants (meta.Expression.BinaryOp, etc.)
pub fn expression_variants() -> ComptimeValue {
    variant_constants(
        "Expression",
        &[
            "Integer8",
            "Integer16",
            "Integer32",
            "Integer64",
            "Unsigned8",
            "Unsigned16",
            "Unsigned32",
            "Unsigned64",
            "Float32",
            "Float64",
            "Boolean",
            "String",
            "Identifier",
            "Unit",
            "BinaryOp",
            "FunctionCall",
            "QuestionMatch",
            "Conditional",
            "AddressOf",
            "Dereference",
            "PointerOffset",
            "StructLiteral",
            "StructField",
            "ArrayLiteral",
            "ArrayIndex",
            "EnumVariant",
            "EnumLiteral",
            "MemberAccess",
            "PointerDereference",
            "PointerAddress",
            "CreateReference",
            "CreateMutableReference",
            "StringLength",
            "Some",
            "None",
            "StringInterpolation",
            "Comptime",
            "Range",
            "PatternMatch",
            "StdReference",
            "BuiltinReference",
            "ThisReference",
            "MethodCall",
            "Loop",
            "CollectionLoop",
            "Closure",
            "Block",
            "Return",
            "Raise",
            "Defer",
            "Break",
            "Continue",
            "VecConstructor",
            "DynVecConstructor",
            "ArrayConstructor",
        ],
    )
}

/// Statement variant name constants (meta.Statement.VariableDeclaration, etc.)
pub fn statement_variants() -> ComptimeValue {
    variant_constants(
        "Statement",
        &[
            "Expression",
            "Return",
            "VariableDeclaration",
            "VariableAssignment",
            "PointerAssignment",
            "Loop",
            "Break",
            "Continue",
            "ComptimeBlock",
            "ModuleImport",
            "Defer",
            "ThisDefer",
            "DestructuringImport",
            "Block",
        ],
    )
}

/// Declaration variant name constants (meta.Declaration.Function, etc.)
pub fn declaration_variants() -> ComptimeValue {
    variant_constants(
        "Declaration",
        &[
            "Function",
            "ExternalFunction",
            "Struct",
            "Enum",
            "Behavior",
            "Trait",
            "TraitImplementation",
            "TraitRequirement",
            "ImplBlock",
            "ComptimeBlock",
            "Constant",
            "ModuleImport",
            "Export",
            "TypeAlias",
        ],
    )
}

/// AstType variant name constants (meta.AstType.I32, meta.AstType.Generic, etc.)
pub fn type_variants() -> ComptimeValue {
    variant_constants(
        "AstType",
        &[
            "I8",
            "I16",
            "I32",
            "I64",
            "U8",
            "U16",
            "U32",
            "U64",
            "Usize",
            "F32",
            "F64",
            "Bool",
            "StaticLiteral",
            "StaticString",
            "Void",
            "Slice",
            "FixedArray",
            "Function",
            "FunctionPointer",
            "Struct",
            "Enum",
            "Ref",
            "Range",
            "Generic",
            "EnumType",
            "StdModule",
        ],
    )
}

/// Pattern variant name constants (meta.Pattern.Wildcard, etc.)
pub fn pattern_variants() -> ComptimeValue {
    variant_constants(
        "Pattern",
        &[
            "Literal",
            "Identifier",
            "Struct",
            "EnumVariant",
            "Wildcard",
            "EnumLiteral",
            "Or",
            "Tuple",
            "Range",
            "Binding",
            "Type",
            "Guard",
        ],
    )
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
        assert_eq!(flds.len(), 5); // name, var_type, initializer, is_mutable, declaration_type
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
        assert_eq!(flds.len(), 7); // name, type_params, args, return_type, body, is_varargs, is_public
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

    #[test]
    fn test_variant_constants_expression() {
        let variants = expression_variants();
        if let ComptimeValue::Struct { name, fields } = &variants {
            assert_eq!(name, "Expression");
            // Spot-check a few variant constants
            assert_eq!(
                fields.get("BinaryOp"),
                Some(&ComptimeValue::String("BinaryOp".to_string()))
            );
            assert_eq!(
                fields.get("FunctionCall"),
                Some(&ComptimeValue::String("FunctionCall".to_string()))
            );
            assert_eq!(
                fields.get("Integer32"),
                Some(&ComptimeValue::String("Integer32".to_string()))
            );
            // Should not have non-existent variants
            assert_eq!(fields.get("Nonexistent"), None);
        } else {
            panic!("Expected Struct");
        }
    }

    #[test]
    fn test_variant_constants_match_variant_name() {
        // Verify that the constant value matches what variant_name() returns
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Integer32(1)),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Integer32(2)),
        };
        let node = ASTNodeValue::Expression(expr);
        let vname = variant_name(&node);

        let variants = expression_variants();
        if let ComptimeValue::Struct { fields, .. } = &variants {
            // meta.Expression.BinaryOp should equal node.variant_name()
            assert_eq!(
                fields.get(&vname),
                Some(&ComptimeValue::String(vname.clone()))
            );
        } else {
            panic!("Expected Struct");
        }
    }

    #[test]
    fn test_variant_constants_all_enums() {
        // Just verify all 5 build without panic and have correct names
        let e = expression_variants();
        let s = statement_variants();
        let d = declaration_variants();
        let t = type_variants();
        let p = pattern_variants();

        for (val, expected_name) in [
            (&e, "Expression"),
            (&s, "Statement"),
            (&d, "Declaration"),
            (&t, "AstType"),
            (&p, "Pattern"),
        ] {
            if let ComptimeValue::Struct { name, fields } = val {
                assert_eq!(name, expected_name);
                assert!(!fields.is_empty(), "{} should have variants", expected_name);
            } else {
                panic!("Expected Struct for {}", expected_name);
            }
        }
    }
}
