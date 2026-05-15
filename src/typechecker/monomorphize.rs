//! Monomorphization helpers — generic type argument inference and substitution.

use std::collections::HashMap;

use crate::ast::typed::{Type, TypeDefKind, TypedTypeDef};
use crate::ast::AstType;
use crate::error::{Diagnostic, Span};

use super::TypeChecker;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InferenceConflict {
    pub(crate) param: String,
    pub(crate) inferred: Type,
    pub(crate) actual: Type,
}

fn install_dependency_map<T: Clone>(
    target: &mut HashMap<String, T>,
    dependencies: &HashMap<String, T>,
) -> Vec<(String, Option<T>)> {
    dependencies
        .iter()
        .map(|(name, value)| (name.clone(), target.insert(name.clone(), value.clone())))
        .collect()
}

fn restore_dependency_map<T>(target: &mut HashMap<String, T>, state: Vec<(String, Option<T>)>) {
    for (name, previous) in state {
        if let Some(previous) = previous {
            target.insert(name, previous);
        } else {
            target.remove(&name);
        }
    }
}

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

    fn install_template_dependencies(
        &mut self,
        template: &super::GenericFunctionTemplate,
    ) -> super::TemplateDependencyState {
        super::TemplateDependencyState {
            structs: install_dependency_map(&mut self.structs, &template.dependency_structs),
            enums: install_dependency_map(&mut self.enums, &template.dependency_enums),
            functions: install_dependency_map(&mut self.functions, &template.dependency_functions),
            generic_functions: install_dependency_map(
                &mut self.generic_functions,
                &template.dependency_generic_functions,
            ),
            methods: install_dependency_map(&mut self.methods, &template.dependency_methods),
            generic_methods: install_dependency_map(
                &mut self.generic_methods,
                &template.dependency_generic_methods,
            ),
        }
    }

    fn restore_template_dependencies(&mut self, state: super::TemplateDependencyState) {
        restore_dependency_map(&mut self.structs, state.structs);
        restore_dependency_map(&mut self.enums, state.enums);
        restore_dependency_map(&mut self.functions, state.functions);
        restore_dependency_map(&mut self.generic_functions, state.generic_functions);
        restore_dependency_map(&mut self.methods, state.methods);
        restore_dependency_map(&mut self.generic_methods, state.generic_methods);
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

    /// Infer type arguments for a generic function by matching actual arg types
    /// against parameter types containing type params.
    #[cfg(test)]
    pub(crate) fn infer_type_args(
        &self,
        type_params: &[String],
        param_types: &[(String, AstType)],
        arg_types: &[Type],
    ) -> HashMap<String, Type> {
        self.infer_type_args_with_conflicts(type_params, param_types, arg_types)
            .0
    }

    pub(crate) fn infer_type_args_with_conflicts(
        &self,
        type_params: &[String],
        param_types: &[(String, AstType)],
        arg_types: &[Type],
    ) -> (HashMap<String, Type>, Vec<InferenceConflict>) {
        let mut map = HashMap::new();
        let mut conflicts = Vec::new();
        for ((_name, param_ty), arg_ty) in param_types.iter().zip(arg_types.iter()) {
            self.match_type_param(param_ty, arg_ty, type_params, &mut map, &mut conflicts);
        }
        (map, conflicts)
    }

    pub(crate) fn infer_method_type_args(
        &self,
        method_name: &str,
        type_params: &[String],
        param_types: &[(String, AstType)],
        arg_types: &[Type],
    ) -> (HashMap<String, Type>, Vec<InferenceConflict>) {
        let (mut map, mut conflicts) =
            self.infer_type_args_with_conflicts(type_params, param_types, arg_types);
        if let (Some(receiver_name), Some(receiver_ty)) = (
            super::method_signature_receiver_name(method_name),
            arg_types.first(),
        ) {
            self.match_generic_type_params(
                receiver_name,
                receiver_ty,
                type_params,
                &mut map,
                &mut conflicts,
            );
        }
        (map, conflicts)
    }

    fn match_type_param(
        &self,
        param: &AstType,
        actual: &Type,
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        match param {
            AstType::Named(name) if type_params.contains(name) => {
                self.set_inferred_type_param(name, actual, map, conflicts);
            }
            AstType::Ptr(inner) => {
                if let Type::Ptr(actual_inner) = actual {
                    self.match_type_param(inner, actual_inner, type_params, map, conflicts);
                }
            }
            AstType::MutPtr(inner) => {
                if let Type::MutPtr(actual_inner) = actual {
                    self.match_type_param(inner, actual_inner, type_params, map, conflicts);
                }
            }
            AstType::RawPtr(inner) => {
                if let Type::RawPtr(actual_inner) = actual {
                    self.match_type_param(inner, actual_inner, type_params, map, conflicts);
                }
            }
            AstType::Slice(inner) => {
                if let Type::Slice(actual_inner) = actual {
                    self.match_type_param(inner, actual_inner, type_params, map, conflicts);
                }
            }
            AstType::Array { elem, .. } => {
                if let Type::Array {
                    elem: actual_elem, ..
                } = actual
                {
                    self.match_type_param(elem, actual_elem, type_params, map, conflicts);
                }
            }
            AstType::Function { params, ret } => {
                if let Type::Function {
                    params: actual_params,
                    ret: actual_ret,
                } = actual
                {
                    for (param, actual_param) in params.iter().zip(actual_params.iter()) {
                        self.match_type_param(param, actual_param, type_params, map, conflicts);
                    }
                    self.match_type_param(ret, actual_ret, type_params, map, conflicts);
                }
            }
            AstType::Generic { name, .. } => {
                self.match_generic_type_params(name, actual, type_params, map, conflicts);
            }
            _ => {}
        }
    }

    fn set_inferred_type_param(
        &self,
        name: &str,
        actual: &Type,
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        if let Some(inferred) = map.get(name) {
            if !self.types_compatible(inferred, actual) {
                conflicts.push(InferenceConflict {
                    param: name.to_string(),
                    inferred: inferred.clone(),
                    actual: actual.clone(),
                });
            }
            return;
        }

        map.insert(name.to_string(), actual.clone());
    }

    fn match_generic_type_params(
        &self,
        generic_name: &str,
        actual: &Type,
        type_params: &[String],
        map: &mut HashMap<String, Type>,
        conflicts: &mut Vec<InferenceConflict>,
    ) {
        match actual {
            Type::Struct {
                name: actual_name,
                fields: actual_fields,
            } if concrete_name_matches_generic(actual_name, generic_name) => {
                if let Some(info) = self.structs.get(generic_name) {
                    for ((_, expected), (_, actual)) in info.fields.iter().zip(actual_fields.iter())
                    {
                        self.match_type_param(expected, actual, type_params, map, conflicts);
                    }
                }
            }
            Type::Enum {
                name: actual_name,
                variants: actual_variants,
            } if concrete_name_matches_generic(actual_name, generic_name) => {
                if let Some(info) = self.enums.get(generic_name) {
                    for ((_, expected_payload), (_, actual_payload)) in
                        info.variants.iter().zip(actual_variants.iter())
                    {
                        if let (Some(expected), Some(actual)) = (expected_payload, actual_payload) {
                            self.match_type_param(expected, actual, type_params, map, conflicts);
                        }
                    }
                }
            }
            _ => {}
        }
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

fn concrete_name_matches_generic(concrete_name: &str, generic_name: &str) -> bool {
    concrete_name == generic_name || concrete_name.starts_with(&format!("{generic_name}_"))
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
