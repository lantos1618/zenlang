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
mod import_seeding;
mod patterns;
mod program_checking;
mod program_module_graph;
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

impl TypeChecker {
    pub(in crate::typechecker) fn literal_coerced_type(
        &mut self,
        expected: &Type,
        actual: &TypedExpression,
    ) -> Type {
        match &actual.kind {
            // Numeric literals adopt an expected primitive numeric type only when
            // the compile-time value fits that target. An int literal may adopt a
            // float expected type (`ratio: f64 = 2` becomes 2.0), matching binary-
            // op coercion (`1 + 2.5`). A float literal only adopts a float —
            // never silently an integer.
            TypedExprKind::IntLiteral(value) if expected.is_integer() => {
                if int_literal_fits_integer_type(*value, expected) {
                    expected.clone()
                } else {
                    self.push_numeric_literal_overflow(
                        "integer",
                        &value.to_string(),
                        expected,
                        actual.span,
                    );
                    Type::Unknown
                }
            }
            TypedExprKind::IntLiteral(value) if expected.is_float() => {
                if int_literal_fits_float_type(*value, expected) {
                    expected.clone()
                } else {
                    self.push_numeric_literal_overflow(
                        "integer",
                        &value.to_string(),
                        expected,
                        actual.span,
                    );
                    Type::Unknown
                }
            }
            TypedExprKind::FloatLiteral(value) if expected.is_float() => {
                if float_literal_fits_float_type(*value, expected) {
                    expected.clone()
                } else {
                    self.push_numeric_literal_overflow(
                        "float",
                        &value.to_string(),
                        expected,
                        actual.span,
                    );
                    Type::Unknown
                }
            }
            _ => actual.ty.clone(),
        }
    }

    fn push_numeric_literal_overflow(
        &mut self,
        literal_kind: &str,
        value: &str,
        expected: &Type,
        span: Span,
    ) {
        self.push_error(
            E3074,
            format!(
                "{literal_kind} literal `{value}` does not fit in `{}`",
                expected.display_name()
            ),
            span,
        );
    }
}

fn int_literal_fits_integer_type(value: i128, ty: &Type) -> bool {
    let Some((min, max)) = integer_type_range(ty) else {
        return false;
    };
    min <= value && value <= max
}

fn integer_type_range(ty: &Type) -> Option<(i128, i128)> {
    let range = match ty {
        Type::I8 => (i128::from(i8::MIN), i128::from(i8::MAX)),
        Type::I16 => (i128::from(i16::MIN), i128::from(i16::MAX)),
        Type::I32 => (i128::from(i32::MIN), i128::from(i32::MAX)),
        Type::I64 => (i128::from(i64::MIN), i128::from(i64::MAX)),
        Type::U8 => (0, i128::from(u8::MAX)),
        Type::U16 => (0, i128::from(u16::MAX)),
        Type::U32 => (0, i128::from(u32::MAX)),
        Type::U64 | Type::Usize => (0, u64::MAX.into()),
        _ => return None,
    };
    Some(range)
}

fn int_literal_fits_float_type(value: i128, ty: &Type) -> bool {
    let value = value as f64;
    float_literal_fits_float_type(value, ty)
}

fn float_literal_fits_float_type(value: f64, ty: &Type) -> bool {
    if !value.is_finite() {
        return false;
    }
    let Some(max) = float_type_abs_max(ty) else {
        return false;
    };
    value.abs() <= max
}

fn float_type_abs_max(ty: &Type) -> Option<f64> {
    let max = match ty {
        Type::F32 => f64::from(f32::MAX),
        Type::F64 => f64::MAX,
        _ => return None,
    };
    Some(max)
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
