//! Monomorphization helpers — generic type argument inference and substitution.

use std::collections::HashMap;

use crate::ast::typed::{Type, TypeDefKind, TypedTypeDef};
use crate::ast::AstType;
use crate::error::{Diagnostic, Span};

pub(super) use super::monomorphize_types::concrete_name_matches_generic;
pub(crate) use super::monomorphize_types::type_to_ast;
use super::monomorphize_types::{substitute_ast_type, type_mangle_key};
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
        if self.reject_missing_generic_substitutions(
            "function",
            name,
            &template.type_params,
            substitutions,
            span,
        ) {
            return None;
        }

        let mangled =
            self.generic_function_mangled_name(name, &template.type_params, substitutions);
        if self.specializations_seen.contains(&mangled) {
            return Some(mangled);
        }

        self.specializations_seen.insert(mangled.clone());
        self.specialize_generic_template_body(&mangled, &template, substitutions, None);

        Some(mangled)
    }

    pub(crate) fn specialize_generic_method(
        &mut self,
        name: &str,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) -> Option<String> {
        let template = self.generic_methods.get(name).cloned()?;
        if self.reject_missing_generic_substitutions(
            "method",
            name,
            &template.type_params,
            substitutions,
            span,
        ) {
            return None;
        }

        let mangled =
            self.generic_function_mangled_name(name, &template.type_params, substitutions);
        if self.specializations_seen.contains(&mangled) {
            return Some(mangled);
        }

        self.specializations_seen.insert(mangled.clone());
        let self_type = self.generic_method_self_type(name, substitutions);
        self.specialize_generic_template_body(&mangled, &template, substitutions, self_type);

        Some(mangled)
    }

    fn specialize_generic_template_body(
        &mut self,
        mangled: &str,
        template: &super::GenericFunctionTemplate,
        substitutions: &HashMap<String, Type>,
        self_type: Option<Type>,
    ) {
        let saved_return_type = self.current_return_type.clone();
        let saved_self_type = self.current_self_type.clone();
        let saved_defers = std::mem::take(&mut self.pending_defers);
        let dependency_state = self.install_template_dependencies(template);
        self.current_self_type = self_type;
        self.type_substitutions.push(substitutions.clone());
        match self.check_function(
            mangled,
            &template.params,
            &template.return_type,
            &template.body,
            &template.span,
        ) {
            Ok(function) => self.specialized_functions.push(function),
            Err(diagnostic) => self.diagnostics.push(diagnostic),
        }
        self.type_substitutions.pop();
        self.restore_template_dependencies(dependency_state);
        self.pending_defers = saved_defers;
        self.current_return_type = saved_return_type;
        self.current_self_type = saved_self_type;
    }

    fn reject_missing_generic_substitutions(
        &mut self,
        kind: &str,
        name: &str,
        type_params: &[String],
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) -> bool {
        let missing: Vec<&str> = type_params
            .iter()
            .map(String::as_str)
            .filter(|param| !substitutions.contains_key(*param))
            .collect();
        if missing.is_empty() {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            "E5000",
            format!(
                "cannot infer type argument{} {} for generic {} `{}`",
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
                kind,
                name
            ),
            span,
        ));
        true
    }

    pub(crate) fn generic_method_self_type(
        &mut self,
        method_name: &str,
        substitutions: &HashMap<String, Type>,
    ) -> Option<Type> {
        let receiver_name = super::method_signature_receiver_name(method_name)?;
        if let Some(info) = self.structs.get(receiver_name).cloned() {
            return Some(self.generic_receiver_self_type(
                receiver_name,
                &info.type_params,
                substitutions,
            ));
        }
        if let Some(info) = self.enums.get(receiver_name).cloned() {
            return Some(self.generic_receiver_self_type(
                receiver_name,
                &info.type_params,
                substitutions,
            ));
        }
        Some(self.resolve_type(&AstType::Named(receiver_name.to_string())))
    }

    fn generic_receiver_self_type(
        &mut self,
        receiver_name: &str,
        type_params: &[String],
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        if type_params.is_empty() {
            return self.resolve_type(&AstType::Named(receiver_name.to_string()));
        }

        let type_args: Vec<AstType> = type_params
            .iter()
            .filter_map(|param| substitutions.get(param).map(|ty| self.type_to_ast_ref(ty)))
            .collect();
        if type_args.len() == type_params.len() {
            self.resolve_type(&AstType::Generic {
                name: receiver_name.to_string(),
                type_args,
            })
        } else {
            Type::Unknown
        }
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
        self.check_generic_bounds(&info.type_param_bounds, &substitutions, span);
        for (_, field_type) in &info.fields {
            self.ensure_specialized_type_refs(field_type, &substitutions, span);
        }
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
        for (_, field_type) in &fields {
            self.ensure_specialized_type_refs_for_type(field_type, span);
        }
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
        self.check_generic_bounds(&info.type_param_bounds, &substitutions, span);
        for (_, payload) in &info.variants {
            if let Some(payload) = payload {
                self.ensure_specialized_type_refs(payload, &substitutions, span);
            }
        }
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
        for (_, payload) in &variants {
            if let Some(payload) = payload {
                self.ensure_specialized_type_refs_for_type(payload, span);
            }
        }
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

    fn ensure_specialized_type_refs(
        &mut self,
        ast_type: &AstType,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) {
        match substitute_ast_type(ast_type, substitutions) {
            AstType::Generic { name, type_args } => {
                for type_arg in &type_args {
                    self.ensure_specialized_type_refs(type_arg, substitutions, span);
                }
                if self.structs.contains_key(&name) {
                    self.specialize_generic_struct(&name, &type_args, span);
                } else if self.enums.contains_key(&name) {
                    self.specialize_generic_enum(&name, &type_args, span);
                }
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => {
                self.ensure_specialized_type_refs(&inner, substitutions, span);
            }
            AstType::Array { elem, .. } => {
                self.ensure_specialized_type_refs(&elem, substitutions, span);
            }
            AstType::Function { params, ret } => {
                for param in &params {
                    self.ensure_specialized_type_refs(param, substitutions, span);
                }
                self.ensure_specialized_type_refs(&ret, substitutions, span);
            }
            _ => {}
        }
    }

    fn ensure_specialized_type_refs_for_type(&mut self, ty: &Type, span: Span) {
        match ty {
            Type::Struct { name, fields } => {
                if let Some((generic_name, type_args)) = self.generic_type_args_from_type(name, ty)
                {
                    self.specialize_generic_struct(&generic_name, &type_args, span);
                }
                for (_, field_type) in fields {
                    self.ensure_specialized_type_refs_for_type(field_type, span);
                }
            }
            Type::Enum { name, variants } => {
                if let Some((generic_name, type_args)) = self.generic_type_args_from_type(name, ty)
                {
                    self.specialize_generic_enum(&generic_name, &type_args, span);
                }
                for (_, payload) in variants {
                    if let Some(payload) = payload {
                        self.ensure_specialized_type_refs_for_type(payload, span);
                    }
                }
            }
            Type::Array { elem, .. }
            | Type::Slice(elem)
            | Type::Ptr(elem)
            | Type::MutPtr(elem)
            | Type::RawPtr(elem) => self.ensure_specialized_type_refs_for_type(elem, span),
            Type::Function { params, ret } => {
                for param in params {
                    self.ensure_specialized_type_refs_for_type(param, span);
                }
                self.ensure_specialized_type_refs_for_type(ret, span);
            }
            _ => {}
        }
    }

    fn generic_type_args_from_type(
        &self,
        concrete_name: &str,
        ty: &Type,
    ) -> Option<(String, Vec<AstType>)> {
        if let Some((name, params)) = self
            .structs
            .iter()
            .find(|(name, info)| {
                concrete_name != name.as_str()
                    && concrete_name_matches_generic(concrete_name, name)
                    && !info.type_params.is_empty()
            })
            .map(|(name, info)| (name.clone(), info.type_params.clone()))
        {
            let mut inferred = HashMap::new();
            let mut conflicts = Vec::new();
            self.match_generic_type_params(&name, ty, &params, &mut inferred, &mut conflicts);
            let type_args = params
                .iter()
                .filter_map(|param| inferred.get(param).map(|ty| self.type_to_ast_ref(ty)))
                .collect::<Vec<_>>();
            if type_args.len() == params.len() {
                return Some((name, type_args));
            }
        }

        if let Some((name, params)) = self
            .enums
            .iter()
            .find(|(name, info)| {
                concrete_name != name.as_str()
                    && concrete_name_matches_generic(concrete_name, name)
                    && !info.type_params.is_empty()
            })
            .map(|(name, info)| (name.clone(), info.type_params.clone()))
        {
            let mut inferred = HashMap::new();
            let mut conflicts = Vec::new();
            self.match_generic_type_params(&name, ty, &params, &mut inferred, &mut conflicts);
            let type_args = params
                .iter()
                .filter_map(|param| inferred.get(param).map(|ty| self.type_to_ast_ref(ty)))
                .collect::<Vec<_>>();
            if type_args.len() == params.len() {
                return Some((name, type_args));
            }
        }

        None
    }

    fn type_to_ast_ref(&self, ty: &Type) -> AstType {
        match ty {
            Type::Struct { name, .. } | Type::Enum { name, .. } => {
                if let Some((generic_name, type_args)) = self.generic_type_args_from_type(name, ty)
                {
                    AstType::Generic {
                        name: generic_name,
                        type_args,
                    }
                } else {
                    type_to_ast(ty)
                }
            }
            _ => type_to_ast(ty),
        }
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
            AstType::RawPtr(inner) => {
                Type::RawPtr(Box::new(self.substitute_type(inner, substitutions)))
            }
            AstType::Slice(inner) => {
                Type::Slice(Box::new(self.substitute_type(inner, substitutions)))
            }
            AstType::Array { elem, size } => Type::Array {
                elem: Box::new(self.substitute_type(elem, substitutions)),
                size: *size,
            },
            AstType::Function { params, ret } => Type::Function {
                params: params
                    .iter()
                    .map(|param| self.substitute_type(param, substitutions))
                    .collect(),
                ret: Box::new(self.substitute_type(ret, substitutions)),
            },
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
