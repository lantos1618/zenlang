use crate::ast::{AstType, Expression, Statement};
use crate::intrinsics::well_known;
use crate::lsp::utils::format_type;
use crate::stdlib_types::stdlib_types;
use crate::type_context::TypeContext;

use super::types::Document;

/// Primary interface for type queries in the LSP.
/// Delegates to TypeContext (SEMA output) with minimal literal-only fallback.
pub struct TypeQuery<'a> {
    type_ctx: Option<&'a TypeContext>,
}

impl<'a> TypeQuery<'a> {
    pub fn new(doc: &'a Document) -> Self {
        Self {
            type_ctx: doc.type_context.as_deref(),
        }
    }

    pub fn from_ctx(type_ctx: Option<&'a TypeContext>) -> Self {
        Self { type_ctx }
    }

    pub fn has_sema(&self) -> bool {
        self.type_ctx.is_some()
    }

    // ========================================================================
    // Variable queries
    // ========================================================================

    pub fn variable_type(&self, func_name: &str, var_name: &str) -> Option<String> {
        self.type_ctx?
            .get_variable_type(func_name, var_name)
            .map(|t| format_type(&t))
    }

    pub fn variable_type_ast(&self, func_name: &str, var_name: &str) -> Option<AstType> {
        self.type_ctx?.get_variable_type(func_name, var_name)
    }

    pub fn is_variable_mutable(&self, func_name: &str, var_name: &str) -> Option<bool> {
        self.type_ctx?.is_variable_mutable(func_name, var_name)
    }

    /// Find variable type searching all scopes (when function name unknown)
    pub fn find_variable_type(&self, var_name: &str) -> Option<String> {
        self.find_variable_type_ast(var_name)
            .map(|t| format_type(&t))
    }

    /// Find variable type as AstType searching all scopes (when function name unknown)
    pub fn find_variable_type_ast(&self, var_name: &str) -> Option<AstType> {
        let ctx = self.type_ctx?;
        let suffix = format!("::{}", var_name);
        for (key, var_type) in &ctx.variables {
            if key.ends_with(&suffix) {
                return Some(var_type.clone());
            }
        }
        None
    }

    // ========================================================================
    // Function queries
    // ========================================================================

    pub fn function_return_type(&self, name: &str) -> Option<String> {
        self.type_ctx?
            .get_function_return_type(name)
            .map(|t| format_type(&t))
    }

    pub fn function_return_type_ast(&self, name: &str) -> Option<AstType> {
        self.type_ctx?.get_function_return_type(name)
    }

    pub fn function_params(&self, name: &str) -> Option<&Vec<(String, AstType)>> {
        self.type_ctx?.get_function_params(name)
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.type_ctx
            .map(|ctx| ctx.has_function(name))
            .unwrap_or(false)
    }

    // ========================================================================
    // Method queries
    // ========================================================================

    pub fn method_return_type(&self, recv_type: &str, method: &str) -> Option<String> {
        if let Some(ctx) = self.type_ctx {
            if let Some(t) = ctx.get_method_return_type(recv_type, method) {
                return Some(format_type(&t));
            }
        }
        // Fallback to stdlib registry (for types not in user code)
        stdlib_types()
            .get_method_return_type(recv_type, method)
            .map(format_type)
    }

    pub fn method_params(&self, recv_type: &str, method: &str) -> Option<&Vec<(String, AstType)>> {
        self.type_ctx?.get_method_params(recv_type, method)
    }

    pub fn constructor_type(&self, type_name: &str, method: &str) -> Option<String> {
        self.type_ctx?
            .get_constructor_type(type_name, method)
            .map(|t| format_type(&t))
    }

    // ========================================================================
    // Struct queries
    // ========================================================================

    pub fn struct_fields(&self, name: &str) -> Option<&Vec<(String, AstType)>> {
        self.type_ctx?.get_struct_fields(name)
    }

    pub fn struct_field_type(&self, struct_name: &str, field_name: &str) -> Option<String> {
        self.type_ctx?
            .get_struct_field_type(struct_name, field_name)
            .map(|t| format_type(&t))
    }

    pub fn has_struct(&self, name: &str) -> bool {
        self.type_ctx
            .map(|ctx| ctx.has_struct(name))
            .unwrap_or(false)
    }

    // ========================================================================
    // Enum queries
    // ========================================================================

    pub fn enum_variants(&self, name: &str) -> Option<&Vec<(String, Option<AstType>)>> {
        self.type_ctx?.get_enum_variants(name)
    }

    pub fn has_enum(&self, name: &str) -> bool {
        self.type_ctx.map(|ctx| ctx.has_enum(name)).unwrap_or(false)
    }

    // ========================================================================
    // Expression type inference (literal-only fallback when no SEMA)
    // ========================================================================

    pub fn infer_literal_type(expr: &Expression) -> Option<String> {
        let wk = well_known();
        match expr {
            Expression::Integer32(_) => Some("i32".to_string()),
            Expression::Integer64(_) => Some("i64".to_string()),
            Expression::Float32(_) => Some("f32".to_string()),
            Expression::Float64(_) => Some("f64".to_string()),
            Expression::Boolean(_) => Some("bool".to_string()),
            Expression::String(_) => Some("StaticString".to_string()),
            Expression::StructLiteral { name, .. } => Some(name.clone()),
            Expression::ArrayLiteral(_) => Some("Array".to_string()),
            Expression::EnumVariant {
                enum_name, variant, ..
            } => {
                if wk.is_option(enum_name) || wk.is_option_variant(variant) {
                    Some(wk.option_name().to_string())
                } else if wk.is_result(enum_name) || wk.is_result_variant(variant) {
                    Some(wk.result_name().to_string())
                } else {
                    Some(enum_name.clone())
                }
            }
            _ => None,
        }
    }

    // ========================================================================
    // Behavior queries
    // ========================================================================

    pub fn type_implements_behavior(&self, type_name: &str, behavior_name: &str) -> bool {
        self.type_ctx
            .map(|ctx| ctx.type_implements_behavior(type_name, behavior_name))
            .unwrap_or(false)
    }

    // ========================================================================
    // Receiver type resolution (for method chains)
    // ========================================================================

    /// Resolve the type of a receiver expression using SEMA variable data.
    /// For `self`, looks up the enclosing function's self parameter.
    /// For identifiers, looks up in TypeContext variables.
    pub fn resolve_receiver_in_function(
        &self,
        receiver_name: &str,
        func_name: &str,
    ) -> Option<String> {
        if receiver_name == "self" {
            // Look for self parameter type in function params
            if let Some(ctx) = self.type_ctx {
                if let Some(func) = ctx.functions.get(func_name) {
                    for (param_name, param_type) in &func.params {
                        if param_name == "self" {
                            return Some(format_type(param_type));
                        }
                    }
                }
            }
            return None;
        }
        self.variable_type(func_name, receiver_name)
    }

    /// Resolve chained field access: given "a.b.c", resolve type step by step
    pub fn resolve_chain(&self, base_type: &str, fields: &[&str]) -> Option<String> {
        let mut current = base_type.to_string();
        for field in fields {
            current = self.struct_field_type(&current, field)?;
        }
        Some(current)
    }

    // ========================================================================
    // Unified variable type inference
    // ========================================================================

    /// Unified variable type inference: TypeContext → AST explicit type → literal inference.
    pub fn infer_variable_type_unified(
        &self,
        name: &str,
        type_annotation: Option<&AstType>,
        initializer: Option<&Expression>,
    ) -> Option<AstType> {
        // 1. Try TypeContext first (canonical path from typechecker)
        if let Some(ty) = self.find_variable_type_ast(name) {
            return Some(ty);
        }

        // 2. Use explicit type annotation if present
        if let Some(ty) = type_annotation {
            return Some(ty.clone());
        }

        // 3. Try literal type inference from initializer
        if let Some(init) = initializer {
            if let Some(type_str) = Self::infer_literal_type(init) {
                return crate::parser::parse_type_from_string(&type_str).ok();
            }
        }

        None
    }

    /// Walk statements to find a variable declaration, then infer its type.
    pub fn infer_variable_type_from_statements(
        &self,
        name: &str,
        statements: &[Statement],
    ) -> Option<AstType> {
        // 1. Try TypeContext first
        if let Some(ty) = self.find_variable_type_ast(name) {
            return Some(ty);
        }

        // 2. Walk AST for variable declaration
        if let Some((type_opt, init_opt)) = Self::find_variable_in_statements(statements, name) {
            return self.infer_variable_type_unified(name, type_opt.as_ref(), init_opt.as_ref());
        }

        None
    }

    fn find_variable_in_statements(
        stmts: &[Statement],
        var_name: &str,
    ) -> Option<(Option<AstType>, Option<Expression>)> {
        for stmt in stmts {
            match stmt {
                Statement::VariableDeclaration {
                    name,
                    type_,
                    initializer,
                    ..
                } if name == var_name => {
                    return Some((type_.clone(), initializer.clone()));
                }
                Statement::Loop { body, .. } => {
                    if let Some(result) = Self::find_variable_in_statements(body, var_name) {
                        return Some(result);
                    }
                }
                Statement::Block { statements, .. } => {
                    if let Some(result) = Self::find_variable_in_statements(statements, var_name) {
                        return Some(result);
                    }
                }
                _ => {}
            }
        }
        None
    }
}
