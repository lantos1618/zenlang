// AST field extraction for compile-time introspection.
// Each function converts an AST node into an array of FieldInfo structs.

use crate::ast::{self, AstType, Declaration, Expression, Pattern, Statement};
use crate::error::Result;
use std::collections::HashMap;

use super::helpers::*;
use crate::comptime::values::ComptimeValue;

pub fn expression_fields(expr: &Expression) -> Result<Vec<ComptimeValue>> {
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

        Expression::BinaryOp { left, op, right } => vec![
            field_info("left", ast_expr(*left.clone())),
            field_info("op", ComptimeValue::String(op.to_string())),
            field_info("right", ast_expr(*right.clone())),
        ],

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

        // Pattern matching — all three variants share the same arm structure
        Expression::QuestionMatch { scrutinee, arms } => {
            match_arms_to_fields("MatchArm", scrutinee, arms)
        }
        Expression::Conditional { scrutinee, arms } => {
            match_arms_to_fields("ConditionalArm", scrutinee, arms)
        }
        Expression::PatternMatch { scrutinee, arms } => {
            match_arms_to_fields("PatternArm", scrutinee, arms)
        }

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

        Expression::StructField { struct_, field } => vec![
            field_info("struct_expr", ast_expr(*struct_.clone())),
            field_info("field", ComptimeValue::String(field.clone())),
        ],

        Expression::ArrayLiteral(elems) => vec![field_info(
            "elements",
            ComptimeValue::Array(elems.iter().map(|e| ast_expr(e.clone())).collect()),
        )],

        Expression::ArrayIndex { array, index } => vec![
            field_info("array", ast_expr(*array.clone())),
            field_info("index", ast_expr(*index.clone())),
        ],

        Expression::ArrayConstructor { element_type } => {
            vec![field_info("element_type", ast_type(element_type.clone()))]
        }

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

        Expression::MemberAccess { object, member } => vec![
            field_info("object", ast_expr(*object.clone())),
            field_info("member", ComptimeValue::String(member.clone())),
        ],

        Expression::StringLength(inner) => {
            vec![field_info("expr", ast_expr(*inner.clone()))]
        }

        Expression::Some(expr) => {
            vec![field_info("inner", ast_expr(*expr.clone()))]
        }

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

        Expression::Comptime(inner) => {
            vec![field_info("expr", ast_expr(*inner.clone()))]
        }

        Expression::Range {
            start,
            end,
            inclusive,
        } => vec![
            field_info("start", ast_expr(*start.clone())),
            field_info("end", ast_expr(*end.clone())),
            field_info("inclusive", ComptimeValue::Bool(*inclusive)),
        ],

        Expression::Loop { body } => {
            vec![field_info("body", ast_expr(*body.clone()))]
        }

        Expression::CollectionLoop {
            collection,
            param,
            index_param,
            body,
        } => vec![
            field_info("collection", ast_expr(*collection.clone())),
            field_info("param_name", ComptimeValue::String(param.0.clone())),
            field_info("param_type", opt_type(&param.1)),
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
                                ("param_type".to_string(), opt_type(ty)),
                            ]),
                        })
                        .collect(),
                ),
            ),
            field_info("return_type", opt_type(return_type)),
            field_info("body", ast_expr(*body.clone())),
        ],

        Expression::Block(stmts) => vec![field_info(
            "statements",
            ComptimeValue::Array(stmts.iter().map(|s| ast_stmt(s.clone())).collect()),
        )],

        Expression::Return(expr) => {
            vec![field_info("expr", ast_expr(*expr.clone()))]
        }
        Expression::Raise(expr) => {
            vec![field_info("expr", ast_expr(*expr.clone()))]
        }
        Expression::Defer(expr) => {
            vec![field_info("expr", ast_expr(*expr.clone()))]
        }

        Expression::Break { label, value } => vec![
            field_info("label", opt_label(label)),
            field_info("value", opt_expr(value)),
        ],
        Expression::Continue { label } => {
            vec![field_info("label", opt_label(label))]
        }
    })
}

pub fn statement_fields(stmt: &Statement) -> Result<Vec<ComptimeValue>> {
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
            field_info("label", opt_label(label)),
            field_info(
                "body",
                ComptimeValue::Array(body.iter().map(|s| ast_stmt(s.clone())).collect()),
            ),
        ],
        Statement::Break { label, .. } => {
            vec![field_info("label", opt_label(label))]
        }
        Statement::Continue { label, .. } => {
            vec![field_info("label", opt_label(label))]
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

pub fn declaration_fields(decl: &Declaration) -> Result<Vec<ComptimeValue>> {
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
                                ("payload".to_string(), opt_type(&v.payload)),
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

pub fn type_fields(ty: &AstType) -> Result<Vec<ComptimeValue>> {
    Ok(match ty {
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
                                ("payload".to_string(), opt_type(&v.payload)),
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

pub fn pattern_fields(pat: &Pattern) -> Result<Vec<ComptimeValue>> {
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
            field_info("binding", opt_label(binding)),
        ],
        Pattern::Guard { pattern, condition } => vec![
            field_info("pattern", ast_pattern(*pattern.clone())),
            field_info("condition", ast_expr(*condition.clone())),
        ],
    })
}
