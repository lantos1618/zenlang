//! Monomorphization helpers — generic type argument inference and substitution.

use std::collections::HashMap;

use crate::ast::typed::{Type, TypeDefKind, TypedTypeDef};
use crate::ast::AstType;
use crate::error::{Diagnostic, Span};

use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn mangle_generic_type_name(&self, name: &str, type_args: &[AstType]) -> String {
        if type_args.is_empty() {
            return name.to_string();
        }
        let suffix: Vec<String> = type_args
            .iter()
            .map(|arg| type_mangle_key(&self.resolve_type(arg)))
            .collect();
        format!("{}_{}", name, suffix.join("_"))
    }

    pub(crate) fn generic_function_mangled_name(
        &self,
        name: &str,
        type_params: &[String],
        substitutions: &HashMap<String, Type>,
    ) -> String {
        let suffix: Vec<String> = type_params
            .iter()
            .filter_map(|param| substitutions.get(param).map(type_mangle_key))
            .collect();
        if suffix.is_empty() {
            name.to_string()
        } else {
            format!("{}_{}", name, suffix.join("_"))
        }
    }

    pub(crate) fn specialize_generic_function(
        &mut self,
        name: &str,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) -> Option<String> {
        let template = self.generic_functions.get(name).cloned()?;
        let missing: Vec<&str> = template
            .type_params
            .iter()
            .map(String::as_str)
            .filter(|param| !substitutions.contains_key(*param))
            .collect();
        if !missing.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E5000",
                format!(
                    "cannot infer type argument{} {} for generic function `{}`",
                    if missing.len() == 1 {
                        ""
                    } else {
                        "s"
                    },
                    missing
                        .iter()
                        .map(|name| format!("`{name}`"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    name
                ),
                span,
            ));
            return None;
        }

        let mangled =
            self.generic_function_mangled_name(name, &template.type_params, substitutions);
        if self.specializations_seen.contains(&mangled) {
            return Some(mangled);
        }

        self.specializations_seen.insert(mangled.clone());
        let saved_return_type = self.current_return_type.clone();
        let saved_self_type = self.current_self_type.clone();
        let saved_defers = std::mem::take(&mut self.pending_defers);
        self.type_substitutions.push(substitutions.clone());
        match self.check_function(
            &mangled,
            &template.params,
            &template.return_type,
            &template.body,
            &template.span,
        ) {
            Ok(function) => self.specialized_functions.push(function),
            Err(diagnostic) => self.diagnostics.push(diagnostic),
        }
        self.type_substitutions.pop();
        self.pending_defers = saved_defers;
        self.current_return_type = saved_return_type;
        self.current_self_type = saved_self_type;

        Some(mangled)
    }

    pub(crate) fn specialize_generic_method(
        &mut self,
        name: &str,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) -> Option<String> {
        let template = self.generic_methods.get(name).cloned()?;
        let missing: Vec<&str> = template
            .type_params
            .iter()
            .map(String::as_str)
            .filter(|param| !substitutions.contains_key(*param))
            .collect();
        if !missing.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E5000",
                format!(
                    "cannot infer type argument{} {} for generic method `{}`",
                    if missing.len() == 1 {
                        ""
                    } else {
                        "s"
                    },
                    missing
                        .iter()
                        .map(|name| format!("`{name}`"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    name
                ),
                span,
            ));
            return None;
        }

        let mangled =
            self.generic_function_mangled_name(name, &template.type_params, substitutions);
        if self.specializations_seen.contains(&mangled) {
            return Some(mangled);
        }

        self.specializations_seen.insert(mangled.clone());
        let saved_return_type = self.current_return_type.clone();
        let saved_self_type = self.current_self_type.clone();
        let saved_defers = std::mem::take(&mut self.pending_defers);
        self.type_substitutions.push(substitutions.clone());
        match self.check_function(
            &mangled,
            &template.params,
            &template.return_type,
            &template.body,
            &template.span,
        ) {
            Ok(function) => self.specialized_functions.push(function),
            Err(diagnostic) => self.diagnostics.push(diagnostic),
        }
        self.type_substitutions.pop();
        self.pending_defers = saved_defers;
        self.current_return_type = saved_return_type;
        self.current_self_type = saved_self_type;

        Some(mangled)
    }

    pub(crate) fn specialize_generic_struct(
        &mut self,
        name: &str,
        type_args: &[AstType],
        span: Span,
    ) -> HashMap<String, Type> {
        let Some(info) = self.structs.get(name).cloned() else {
            return HashMap::new();
        };
        let substitutions =
            self.type_param_substitutions(&info.type_params, type_args, "struct", name, span);
        let fields: Vec<(String, Type)> = info
            .fields
            .iter()
            .map(|(field_name, field_type)| {
                (
                    field_name.clone(),
                    self.substitute_type(field_type, &substitutions),
                )
            })
            .collect();
        let field_map = fields.iter().cloned().collect();
        let mangled = self.mangle_generic_type_name(name, type_args);
        if self.specialized_types_seen.insert(mangled.clone()) {
            self.specialized_types.push(TypedTypeDef {
                name: mangled,
                kind: TypeDefKind::Struct { fields },
                methods: Vec::new(),
                span,
            });
        }
        field_map
    }

    pub(crate) fn specialize_generic_enum(
        &mut self,
        name: &str,
        type_args: &[AstType],
        span: Span,
    ) -> HashMap<String, Option<Type>> {
        let Some(info) = self.enums.get(name).cloned() else {
            return HashMap::new();
        };
        let substitutions =
            self.type_param_substitutions(&info.type_params, type_args, "enum", name, span);
        let variants: Vec<(String, Option<Type>)> = info
            .variants
            .iter()
            .map(|(variant_name, payload)| {
                (
                    variant_name.clone(),
                    payload
                        .as_ref()
                        .map(|payload| self.substitute_type(payload, &substitutions)),
                )
            })
            .collect();
        let variant_map = variants.iter().cloned().collect();
        let mangled = self.mangle_generic_type_name(name, type_args);
        if self.specialized_types_seen.insert(mangled.clone()) {
            let typed_variants = variants
                .into_iter()
                .enumerate()
                .map(
                    |(tag, (variant_name, payload))| crate::ast::typed::TypedVariant {
                        name: variant_name,
                        tag: tag as u32,
                        payload: payload.map(|ty| vec![("payload".to_string(), ty)]),
                    },
                )
                .collect();
            self.specialized_types.push(TypedTypeDef {
                name: mangled,
                kind: TypeDefKind::Enum {
                    variants: typed_variants,
                },
                methods: Vec::new(),
                span,
            });
        }
        variant_map
    }

    pub(crate) fn type_param_substitutions(
        &mut self,
        type_params: &[String],
        type_args: &[AstType],
        kind: &str,
        name: &str,
        span: Span,
    ) -> HashMap<String, Type> {
        if type_params.len() != type_args.len() {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic {} `{}` expects {} type arguments, found {}",
                    kind,
                    name,
                    type_params.len(),
                    type_args.len()
                ),
                span,
            ));
        }

        type_params
            .iter()
            .zip(type_args.iter())
            .map(|(param, arg)| (param.clone(), self.resolve_type(arg)))
            .collect()
    }

    /// Infer type arguments for a generic function by matching actual arg types
    /// against parameter types containing type params.
    pub(crate) fn infer_type_args(
        &self,
        type_params: &[String],
        param_types: &[(String, AstType)],
        arg_types: &[Type],
    ) -> HashMap<String, Type> {
        let mut map = HashMap::new();
        for ((_name, param_ty), arg_ty) in param_types.iter().zip(arg_types.iter()) {
            match_type_param(param_ty, arg_ty, type_params, &mut map);
        }
        map
    }

    /// Substitute type parameters in an AstType, returning a resolved Type.
    pub(crate) fn substitute_type(
        &self,
        ast_type: &AstType,
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        match ast_type {
            AstType::Named(name) => {
                if let Some(concrete) = substitutions.get(name) {
                    concrete.clone()
                } else {
                    self.resolve_type(ast_type)
                }
            }
            AstType::Ptr(inner) => Type::Ptr(Box::new(self.substitute_type(inner, substitutions))),
            AstType::MutPtr(inner) => {
                Type::MutPtr(Box::new(self.substitute_type(inner, substitutions)))
            }
            AstType::Slice(inner) => {
                Type::Slice(Box::new(self.substitute_type(inner, substitutions)))
            }
            AstType::Generic { name, type_args } => {
                let subst_args: Vec<AstType> = type_args
                    .iter()
                    .map(|a| substitute_ast_type(a, substitutions))
                    .collect();
                self.resolve_type(&AstType::Generic {
                    name: name.clone(),
                    type_args: subst_args,
                })
            }
            _ => self.resolve_type(ast_type),
        }
    }
}

fn type_mangle_key(ty: &Type) -> String {
    match ty {
        Type::I8 => "i8".into(),
        Type::I16 => "i16".into(),
        Type::I32 => "i32".into(),
        Type::I64 => "i64".into(),
        Type::U8 => "u8".into(),
        Type::U16 => "u16".into(),
        Type::U32 => "u32".into(),
        Type::U64 => "u64".into(),
        Type::Usize => "usize".into(),
        Type::F32 => "f32".into(),
        Type::F64 => "f64".into(),
        Type::Bool => "bool".into(),
        Type::Void => "void".into(),
        Type::Str => "str".into(),
        Type::String => "String".into(),
        Type::Named(name) | Type::Struct { name, .. } | Type::Enum { name, .. } => {
            symbol_mangle_key(name)
        }
        Type::Array { elem, size } => match size {
            Some(size) => format!("array_{}_{}", type_mangle_key(elem), size),
            None => format!("array_{}", type_mangle_key(elem)),
        },
        Type::Slice(elem) => format!("slice_{}", type_mangle_key(elem)),
        Type::Ptr(inner) => format!("ptr_{}", type_mangle_key(inner)),
        Type::MutPtr(inner) => format!("mutptr_{}", type_mangle_key(inner)),
        Type::RawPtr(inner) => format!("rawptr_{}", type_mangle_key(inner)),
        Type::Function { params, ret } => {
            let params = params
                .iter()
                .map(type_mangle_key)
                .collect::<Vec<_>>()
                .join("_");
            format!("fn_{}_ret_{}", params, type_mangle_key(ret))
        }
        Type::Never => "never".into(),
        Type::Unknown => "unknown".into(),
    }
}

fn symbol_mangle_key(symbol: &str) -> String {
    let mut out = String::with_capacity(symbol.len());
    for ch in symbol.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    out
}

/// Recursively match a parameter AstType against an actual Type to discover
/// type parameter bindings.
fn match_type_param(
    param: &AstType,
    actual: &Type,
    type_params: &[String],
    map: &mut HashMap<String, Type>,
) {
    match param {
        AstType::Named(name) if type_params.contains(name) => {
            map.entry(name.clone()).or_insert_with(|| actual.clone());
        }
        AstType::Ptr(inner) => {
            if let Type::Ptr(actual_inner) = actual {
                match_type_param(inner, actual_inner, type_params, map);
            }
        }
        AstType::MutPtr(inner) => {
            if let Type::MutPtr(actual_inner) = actual {
                match_type_param(inner, actual_inner, type_params, map);
            }
        }
        AstType::Slice(inner) => {
            if let Type::Slice(actual_inner) = actual {
                match_type_param(inner, actual_inner, type_params, map);
            }
        }
        AstType::Generic { name: _, type_args } => {
            let actual_args = generic_type_args_from_concrete_name(actual, type_args.len());
            for (arg, actual_arg) in type_args.iter().zip(actual_args.iter()) {
                match_type_param(arg, actual_arg, type_params, map);
            }
        }
        _ => {}
    }
}

fn generic_type_args_from_concrete_name(actual: &Type, expected_len: usize) -> Vec<Type> {
    let concrete_name = match actual {
        Type::Named(name) | Type::Struct { name, .. } | Type::Enum { name, .. } => name,
        _ => return Vec::new(),
    };
    let Some(suffix) = concrete_name.rsplit_once('_').map(|(_, suffix)| suffix) else {
        return Vec::new();
    };
    if expected_len != 1 {
        return Vec::new();
    }
    match suffix {
        "i8" => vec![Type::I8],
        "i16" => vec![Type::I16],
        "i32" => vec![Type::I32],
        "i64" => vec![Type::I64],
        "u8" => vec![Type::U8],
        "u16" => vec![Type::U16],
        "u32" => vec![Type::U32],
        "u64" => vec![Type::U64],
        "usize" => vec![Type::Usize],
        "f32" => vec![Type::F32],
        "f64" => vec![Type::F64],
        "bool" => vec![Type::Bool],
        "str" => vec![Type::Str],
        other => vec![Type::Named(other.to_string())],
    }
}

/// Substitute type parameters in an AstType, returning a new AstType.
fn substitute_ast_type(ast_type: &AstType, substitutions: &HashMap<String, Type>) -> AstType {
    match ast_type {
        AstType::Named(name) => {
            if let Some(concrete) = substitutions.get(name) {
                type_to_ast(concrete)
            } else {
                ast_type.clone()
            }
        }
        AstType::Ptr(inner) => AstType::Ptr(Box::new(substitute_ast_type(inner, substitutions))),
        AstType::MutPtr(inner) => {
            AstType::MutPtr(Box::new(substitute_ast_type(inner, substitutions)))
        }
        AstType::Slice(inner) => {
            AstType::Slice(Box::new(substitute_ast_type(inner, substitutions)))
        }
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|a| substitute_ast_type(a, substitutions))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}

/// Convert a resolved Type back to an AstType (best-effort, for substitution).
pub(crate) fn type_to_ast(ty: &Type) -> AstType {
    match ty {
        Type::I8 => AstType::I8,
        Type::I16 => AstType::I16,
        Type::I32 => AstType::I32,
        Type::I64 => AstType::I64,
        Type::U8 => AstType::U8,
        Type::U16 => AstType::U16,
        Type::U32 => AstType::U32,
        Type::U64 => AstType::U64,
        Type::Usize => AstType::Usize,
        Type::F32 => AstType::F32,
        Type::F64 => AstType::F64,
        Type::Bool => AstType::Bool,
        Type::Void => AstType::Void,
        Type::Str => AstType::Str,
        Type::String => AstType::Named("String".into()),
        Type::Named(n) => AstType::Named(n.clone()),
        Type::Struct { name, .. } => AstType::Named(name.clone()),
        Type::Enum { name, .. } => AstType::Named(name.clone()),
        Type::Ptr(inner) => AstType::Ptr(Box::new(type_to_ast(inner))),
        Type::MutPtr(inner) => AstType::MutPtr(Box::new(type_to_ast(inner))),
        Type::RawPtr(inner) => AstType::RawPtr(Box::new(type_to_ast(inner))),
        Type::Slice(inner) => AstType::Slice(Box::new(type_to_ast(inner))),
        Type::Array { elem, size } => AstType::Array {
            elem: Box::new(type_to_ast(elem)),
            size: *size,
        },
        _ => AstType::Void,
    }
}
