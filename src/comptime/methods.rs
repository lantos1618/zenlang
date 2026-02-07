// Method dispatch for the comptime interpreter (member access, method calls,
// AST node methods, array methods, string methods, meta intrinsics)

use crate::ast::builtins::MetaIntrinsic;
use crate::ast::{Declaration, Expression};
use crate::error::{CompileError, Result};
use std::fmt;
use std::rc::Rc;

use super::values::*;
use super::{meta, ComptimeInterpreter};

// ---------------------------------------------------------------------------
// Helpers for argument validation
// ---------------------------------------------------------------------------

impl ComptimeInterpreter {
    /// Evaluate a single argument and assert it's an ASTNode. Used by all meta intrinsics.
    fn require_one_ast_arg(
        &mut self,
        args: &[Expression],
        method: &impl fmt::Display,
        span: Option<crate::error::Span>,
    ) -> Result<Rc<ASTNodeValue>> {
        if args.len() != 1 {
            return Err(CompileError::ComptimeError(
                format!("meta.{}() expects exactly 1 argument", method),
                span,
            ));
        }
        let val = self.evaluate_expression(&args[0], span.clone())?;
        match val {
            ComptimeValue::ASTNode(node) => Ok(node),
            _ => Err(CompileError::ComptimeError(
                format!("meta.{}() expects an ASTNode argument", method),
                span,
            )),
        }
    }

    /// Evaluate a single argument and return it. Validates arity.
    pub(super) fn require_one_arg(
        &mut self,
        args: &[Expression],
        fn_name: &impl fmt::Display,
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        if args.len() != 1 {
            return Err(CompileError::ComptimeError(
                format!("{} expects exactly one argument", fn_name),
                span,
            ));
        }
        self.evaluate_expression(&args[0], span)
    }
}

// ---------------------------------------------------------------------------
// Member access
// ---------------------------------------------------------------------------

impl ComptimeInterpreter {
    pub(super) fn evaluate_member_access(
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
}

// ---------------------------------------------------------------------------
// Method call routing
// ---------------------------------------------------------------------------

impl ComptimeInterpreter {
    pub(super) fn evaluate_method_call(
        &mut self,
        object: ComptimeValue,
        method: &str,
        args: &[Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        if let ComptimeValue::Struct { name, .. } = &object {
            if name == "meta" {
                return self.evaluate_meta_intrinsic(method, args, span);
            }
        }
        if let ComptimeValue::ASTNode(ref node) = object {
            return self.evaluate_ast_node_method(node, method, args, span);
        }
        if let ComptimeValue::Array(ref items) = object {
            return self.evaluate_array_method(items, method, args, span);
        }
        if let ComptimeValue::String(ref s) = object {
            return self.evaluate_string_method(s, method, args, span);
        }

        Err(CompileError::ComptimeError(
            format!("Cannot call method '{}' on {:?}", method, object.get_type()),
            span,
        ))
    }
}

// ---------------------------------------------------------------------------
// Meta intrinsics — meta.type_info(), meta.fields(), etc.
// ---------------------------------------------------------------------------

impl ComptimeInterpreter {
    fn evaluate_meta_intrinsic(
        &mut self,
        method: &str,
        args: &[Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        let Some(intrinsic) = MetaIntrinsic::from_name(method) else {
            return Err(CompileError::ComptimeError(
                format!("Unknown meta intrinsic: meta.{}()", method),
                span,
            ));
        };

        match intrinsic {
            MetaIntrinsic::TypeInfo => {
                let node = self.require_one_ast_arg(args, &intrinsic, span)?;
                meta::type_info(&node)
            }
            MetaIntrinsic::Fields => {
                let node = self.require_one_ast_arg(args, &intrinsic, span)?;
                Ok(ComptimeValue::Array(meta::fields(&node)?))
            }
            MetaIntrinsic::VariantName => {
                let node = self.require_one_ast_arg(args, &intrinsic, span)?;
                Ok(ComptimeValue::String(meta::variant_name(&node)))
            }
            MetaIntrinsic::Children => {
                let node = self.require_one_ast_arg(args, &intrinsic, span)?;
                Ok(ComptimeValue::Array(meta::children(&node)?))
            }
            MetaIntrinsic::Parse => {
                let val = self.require_one_arg(args, &intrinsic, span.clone())?;
                match val {
                    ComptimeValue::String(source) => {
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
        }
    }
}

// ---------------------------------------------------------------------------
// AST node methods — node.functions(), node.find_function(), etc.
// ---------------------------------------------------------------------------

impl ComptimeInterpreter {
    fn evaluate_ast_node_method(
        &mut self,
        node: &ASTNodeValue,
        method: &str,
        args: &[Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        match method {
            "type_info" => meta::type_info(node),
            "fields" => Ok(ComptimeValue::Array(meta::fields(node)?)),
            "variant_name" => Ok(ComptimeValue::String(meta::variant_name(node))),
            "children" => Ok(ComptimeValue::Array(meta::children(node)?)),

            "functions" => Self::filter_program_declarations(
                node,
                |d| matches!(d, Declaration::Function(_)),
                "functions",
                span,
            ),
            "structs" => Self::filter_program_declarations(
                node,
                |d| matches!(d, Declaration::Struct(_)),
                "structs",
                span,
            ),
            "enums" => Self::filter_program_declarations(
                node,
                |d| matches!(d, Declaration::Enum(_)),
                "enums",
                span,
            ),
            "constants" => Self::filter_program_declarations(
                node,
                |d| matches!(d, Declaration::Constant { .. }),
                "constants",
                span,
            ),
            "imports" => Self::filter_program_declarations(
                node,
                |d| matches!(d, Declaration::ModuleImport { .. }),
                "imports",
                span,
            ),
            "traits" => Self::filter_program_declarations(
                node,
                |d| matches!(d, Declaration::Trait(_)),
                "traits",
                span,
            ),
            "behaviors" => Self::filter_program_declarations(
                node,
                |d| matches!(d, Declaration::Behavior(_)),
                "behaviors",
                span,
            ),
            "type_aliases" => Self::filter_program_declarations(
                node,
                |d| matches!(d, Declaration::TypeAlias(_)),
                "type_aliases",
                span,
            ),
            "impl_blocks" => Self::filter_program_declarations(
                node,
                |d| matches!(d, Declaration::ImplBlock(_)),
                "impl_blocks",
                span,
            ),

            "find_function" => {
                let target = self.eval_string_arg(args, "find_function", span.clone())?;
                Self::find_program_declaration(
                    node,
                    &target,
                    |d| {
                        if let Declaration::Function(f) = d {
                            Some(f.name.as_str())
                        } else {
                            None
                        }
                    },
                    "find_function",
                    span,
                )
            }
            "find_struct" => {
                let target = self.eval_string_arg(args, "find_struct", span.clone())?;
                Self::find_program_declaration(
                    node,
                    &target,
                    |d| {
                        if let Declaration::Struct(s) = d {
                            Some(s.name.as_str())
                        } else {
                            None
                        }
                    },
                    "find_struct",
                    span,
                )
            }
            "find_enum" => {
                let target = self.eval_string_arg(args, "find_enum", span.clone())?;
                Self::find_program_declaration(
                    node,
                    &target,
                    |d| {
                        if let Declaration::Enum(e) = d {
                            Some(e.name.as_str())
                        } else {
                            None
                        }
                    },
                    "find_enum",
                    span,
                )
            }

            "find_by_variant" => {
                let target = self.eval_string_arg(args, "find_by_variant", span)?;
                let children = meta::children(node)?;
                let mut results = Vec::new();
                Self::collect_by_variant(&children, &target, &mut results);
                Ok(ComptimeValue::Array(results))
            }

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
                if let Ok(children) = meta::children(node) {
                    Self::collect_by_variant(&children, target, results);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Array methods
// ---------------------------------------------------------------------------

impl ComptimeInterpreter {
    fn evaluate_array_method(
        &mut self,
        items: &[ComptimeValue],
        method: &str,
        args: &[Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        match method {
            "len" => Ok(ComptimeValue::I64(items.len() as i64)),
            "first" => Ok(items.first().cloned().unwrap_or(ComptimeValue::Null)),
            "last" => Ok(items.last().cloned().unwrap_or(ComptimeValue::Null)),
            "is_empty" => Ok(ComptimeValue::Bool(items.is_empty())),

            "filter_by_variant" => {
                let target = self.eval_string_arg(args, "filter_by_variant", span)?;
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
                let idx = Self::value_to_index(&idx_val).ok_or_else(|| {
                    CompileError::ComptimeError(
                        "at() expects an integer index".to_string(),
                        span.clone(),
                    )
                })?;
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
}

// ---------------------------------------------------------------------------
// String methods
// ---------------------------------------------------------------------------

impl ComptimeInterpreter {
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
                let val = self.require_one_arg(args, &"String.append", span.clone())?;
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
}
