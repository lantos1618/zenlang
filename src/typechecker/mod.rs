pub(in crate::typechecker) mod ast_type_substitution;
mod behavior_associations;
mod behavior_impl_support;
mod behavior_impl_validation;
mod declaration_collection_ast;
mod environment;
mod expressions;
mod generic_bound_validation;
mod generic_type_arg_diagnostics;
mod generic_type_reference_walker;
mod generic_type_validation;
mod generics;
mod patterns;
mod program_checking;
mod program_module_graph;
mod import_seeding;
mod resolve;
mod resolve_binary_ops;
mod scope_management;
mod semantic_validation;
mod statements;

use std::collections::{HashMap, HashSet};

use crate::ast::typed::*;
use crate::ast::{
    self, behavior_impl_method_symbol_key as behavior_impl_method_signature_key_with_target_args,
    behavior_ref_display, method_symbol_key as method_signature_key, named_type_arg_names,
    named_type_arg_params, type_param_names, AstType, Declaration, EnumVariant, Expression, Param,
    StructField,
};
use crate::error::CompilerDiagnosticCode::*;
use crate::error::{CompilerDiagnosticCode, Diagnostic, Span};
use crate::module_system::{ResolvedModule, ResolvedModuleGraph};

pub(crate) use environment::{
    BehaviorBound, BehaviorInfo, EnumInfo, FuncInfo, GenericBehaviorImplTemplate,
    GenericFunctionTemplate, SourceModuleDependencies, StructInfo, TemplateDependencyState,
};

include!("info_builders.rs");
include!("state.rs");

impl TypeChecker {
    fn push_error(&mut self, code: CompilerDiagnosticCode, message: impl Into<String>, span: Span) {
        self.diagnostics
            .push(Diagnostic::error_code(code, message, span));
    }

    fn fail_if_errors(&self) -> Result<(), Vec<Diagnostic>> {
        self.diagnostics
            .is_empty()
            .then_some(())
            .ok_or_else(|| self.diagnostics.clone())
    }

    pub(in crate::typechecker) fn type_params_for_type(&self, name: &str) -> Option<&[String]> {
        self.generic_type_decl(name)
            .map(|(_, type_params, _)| type_params)
    }

    fn generic_type_decl(
        &self,
        name: &str,
    ) -> Option<(&'static str, &[String], &HashMap<String, BehaviorBound>)> {
        if let Some(info) = self.structs.get(name) {
            return Some(("struct", &info.type_params, &info.type_param_bounds));
        }
        self.enums
            .get(name)
            .map(|info| ("enum", info.type_params.as_slice(), &info.type_param_bounds))
    }

    fn type_arg_substitutions(
        &self,
        type_params: &[String],
        type_args: &[AstType],
    ) -> HashMap<String, Type> {
        self.concrete_type_arg_substitutions(type_params, type_args, &HashSet::new())
    }

    // Pad a generic type reference's args with the declaration's defaults for any
    // omitted trailing params: `Vec<i64>` becomes `Vec<i64, Mallocator>`. Returns
    // the args unchanged when the type is unknown, already saturated, or has no
    // defaults. A missing param without a default stops padding so the normal
    // arity check still reports it.
    pub(in crate::typechecker) fn fill_type_arg_defaults(
        &self,
        name: &str,
        type_args: &[AstType],
    ) -> Vec<AstType> {
        let (type_params, defaults) = if let Some(info) = self.structs.get(name) {
            (&info.type_params, &info.type_param_defaults)
        } else if let Some(info) = self.enums.get(name) {
            (&info.type_params, &info.type_param_defaults)
        } else {
            return type_args.to_vec();
        };
        if defaults.is_empty() || type_args.len() >= type_params.len() {
            return type_args.to_vec();
        }
        let mut filled = type_args.to_vec();
        for param in type_params.iter().skip(type_args.len()) {
            match defaults.get(param) {
                Some(default) => filled.push(default.clone()),
                None => break,
            }
        }
        filled
    }

    fn behavior_type_arg_substitutions(
        &mut self,
        behavior: &str,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) -> Option<HashMap<String, AstType>> {
        let Some(info) = self.behaviors.get(behavior).cloned() else {
            self.push_error(E6006, format!("undefined behavior `{}`", behavior), span);
            return None;
        };

        if !self.validate_type_arg_arity(
            "behavior",
            behavior,
            info.type_params.len(),
            type_args,
            span,
        ) {
            return None;
        }

        let ast_substitutions = self.behavior_type_param_substitutions(behavior, type_args);
        let type_substitutions =
            self.concrete_type_arg_substitutions(&info.type_params, type_args, scoped_type_params);
        let diagnostic_count = self.diagnostics.len();
        self.check_generic_bounds(&info.type_param_bounds, &type_substitutions, span);
        if self.diagnostics.len() > diagnostic_count {
            return None;
        }

        Some(ast_substitutions)
    }

    fn concrete_type_arg_substitutions(
        &self,
        type_params: &[String],
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
    ) -> HashMap<String, Type> {
        type_params
            .iter()
            .zip(type_args.iter())
            .filter(|(_, arg)| !ast_type_references_type_param(arg, scoped_type_params))
            .map(|(param, arg)| (param.clone(), self.resolve_type(arg)))
            .collect()
    }
}

fn literal_coerced_type(expected: &Type, actual: &TypedExpression) -> Type {
    match &actual.kind {
        // An int literal soundly adopts an integer OR a float expected type
        // (`ratio: f64 = 2` becomes 2.0), matching binary-op coercion (`1 + 2.5`).
        // A float literal only adopts a float — never silently an integer.
        TypedExprKind::IntLiteral(_) if expected.is_integer() || expected.is_float() => {
            expected.clone()
        }
        TypedExprKind::FloatLiteral(_) if expected.is_float() => expected.clone(),
        _ => actual.ty.clone(),
    }
}

fn type_display_pair(expected: &Type, actual: &Type) -> (String, String) {
    (expected.display_name(), actual.display_name())
}

fn quoted_list(items: &[&str]) -> String {
    items
        .iter()
        .map(|item| format!("`{item}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
