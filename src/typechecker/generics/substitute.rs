//! Type substitution + specialization for monomorphization. One place for the
//! whole "given concrete type args, produce the specialized type" path: mangled
//! naming, key/scope reservation, struct/enum specialization, the generic↔concrete
//! type-arg recovery, and method self-type derivation. The call-site type-arg
//! *inference* (which args to substitute) lives in `monomorphize_inference`; the
//! orchestration of specializing a callable lives in `monomorphize`.

use std::collections::HashMap;
use std::path::Path;

use crate::ast::typed::{Type, TypeDefKind, TypedTypeDef};
use crate::ast::{symbol_key_part, AstType};
use crate::error::Span;

use super::super::ast_type_substitution::substitute_ast_type_names;
use super::super::{BehaviorBound, TypeChecker};

// ── Type → mangled key / AST ────────────────────────────────────────────────

pub(super) fn type_mangle_key(ty: &Type) -> String {
    if let Some(name) = ty.builtin_source_name() {
        return name.into();
    }
    if let Some(name) = ty.nominal_name() {
        return symbol_key_part(name);
    }

    match ty {
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
        _ => unreachable!("handled by builtin_source_name"),
    }
}

pub(crate) fn substitute_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, Type>,
) -> AstType {
    substitute_ast_type_names(ast_type, &|name| substitutions.get(name).map(type_to_ast))
}

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
        Type::Void | Type::Never | Type::Unknown => AstType::Void,
        Type::Str => AstType::Str,
        Type::Named(name) | Type::Struct { name, .. } | Type::Enum { name, .. } => {
            AstType::Named(name.clone())
        }
        Type::Ptr(inner) => AstType::Ptr(Box::new(type_to_ast(inner))),
        Type::MutPtr(inner) => AstType::MutPtr(Box::new(type_to_ast(inner))),
        Type::RawPtr(inner) => AstType::RawPtr(Box::new(type_to_ast(inner))),
        Type::Slice(inner) => AstType::Slice(Box::new(type_to_ast(inner))),
        Type::Array { elem, size } => AstType::Array {
            elem: Box::new(type_to_ast(elem)),
            size: *size,
        },
        Type::Function { params, ret } => AstType::Function {
            params: params.iter().map(type_to_ast).collect(),
            ret: Box::new(type_to_ast(ret)),
        },
    }
}

// ── Mangled names + specialization-key reservation ──────────────────────────

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

    pub(crate) fn generic_specialization_key(
        &self,
        kind: &str,
        scope: Option<&str>,
        mangled: &str,
    ) -> String {
        format!("{kind}:{}:{mangled}", scope.unwrap_or("local"))
    }

    pub(crate) fn reserved_or_requested_generic_type_name(
        &self,
        kind: &str,
        scope: Option<&str>,
        requested: String,
    ) -> String {
        let key = self.generic_specialization_key(kind, scope, &requested);
        self.specialized_types_seen
            .get(&key)
            .cloned()
            .unwrap_or(requested)
    }

    pub(crate) fn concrete_type_name_matches_generic(
        &self,
        concrete_name: &str,
        generic_name: &str,
    ) -> bool {
        concrete_name == generic_name
            || concrete_name.starts_with(&format!("{generic_name}_"))
            || self
                .specialized_type_generic_names
                .get(concrete_name)
                .is_some_and(|source| source == generic_name)
    }
}

pub(in crate::typechecker) fn reserve_specialization_name(
    seen: &mut HashMap<String, String>,
    owners: &mut HashMap<String, String>,
    specialization_key: &str,
    requested: &str,
    scope: Option<&str>,
) -> String {
    if let Some(existing) = seen.get(specialization_key) {
        return existing.clone();
    }

    let mut mangled = requested.to_string();
    if owners
        .get(requested)
        .is_some_and(|owner| owner != specialization_key)
    {
        mangled = scoped_generic_specialization_name(requested, scope, specialization_key);
        let base = mangled.clone();
        let mut suffix = 2;
        while owners
            .get(&mangled)
            .is_some_and(|owner| owner != specialization_key)
        {
            mangled = format!("{base}_{suffix}");
            suffix += 1;
        }
    }

    seen.insert(specialization_key.to_string(), mangled.clone());
    owners.insert(mangled.clone(), specialization_key.to_string());
    mangled
}

fn scoped_generic_specialization_name(
    requested: &str,
    scope: Option<&str>,
    specialization_key: &str,
) -> String {
    let prefix = scope
        .map(generic_specialization_scope_prefix)
        .unwrap_or_else(|| format!("generic_{:08x}", stable_hash(specialization_key) as u32));
    format!("{prefix}_{requested}")
}

fn generic_specialization_scope_prefix(scope: &str) -> String {
    let stem = Path::new(scope)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("module");
    ident_fragment(stem)
}

fn ident_fragment(input: &str) -> String {
    let output: String = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();

    let ident = match output.trim_matches('_') {
        "" => "module",
        ident => ident,
    };
    if ident.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        format!("m{ident}")
    } else {
        ident.to_string()
    }
}

fn stable_hash(input: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ── Struct / enum specialization ────────────────────────────────────────────

impl TypeChecker {
    pub(crate) fn specialize_generic_struct(
        &mut self,
        name: &str,
        type_args: &[AstType],
        span: Span,
    ) -> HashMap<String, Type> {
        let Some(info) = self.structs.get(name).cloned() else {
            return HashMap::new();
        };
        let substitutions = self.bound_checked_substitutions(
            &info.type_params,
            &info.type_param_bounds,
            "struct",
            name,
            type_args,
            span,
        );
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
        self.emit_specialized_type(
            "struct",
            name,
            type_args,
            info.specialization_scope.as_deref(),
            span,
            || TypeDefKind::Struct { fields },
        );
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
        let substitutions = self.bound_checked_substitutions(
            &info.type_params,
            &info.type_param_bounds,
            "enum",
            name,
            type_args,
            span,
        );
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
        for payload in variants.iter().filter_map(|(_, payload)| payload.as_ref()) {
            self.ensure_specialized_type_refs_for_type(payload, span);
        }
        let variant_map = variants.iter().cloned().collect();
        self.emit_specialized_type(
            "enum",
            name,
            type_args,
            info.specialization_scope.as_deref(),
            span,
            || {
                let variants = variants
                    .into_iter()
                    .enumerate()
                    .map(
                        |(tag, (variant_name, payload))| crate::ast::typed::TypedVariant {
                            name: variant_name,
                            tag: tag as u32,
                            payload,
                        },
                    )
                    .collect();
                TypeDefKind::Enum { variants }
            },
        );
        variant_map
    }

    /// Shared prefix of struct/enum specialization: map type params to the
    /// concrete args and verify their behavior bounds, returning the
    /// substitution map both then apply to fields/variants.
    fn bound_checked_substitutions(
        &mut self,
        type_params: &[String],
        type_param_bounds: &HashMap<String, BehaviorBound>,
        kind: &str,
        name: &str,
        type_args: &[AstType],
        span: Span,
    ) -> HashMap<String, Type> {
        let substitutions = self.type_param_substitutions(type_params, type_args, kind, name, span);
        self.check_generic_bounds(type_param_bounds, &substitutions, span);
        substitutions
    }

    /// Shared tail of struct/enum specialization: reserve the mangled name and,
    /// if this concrete instantiation hasn't been emitted yet, push its
    /// definition. `make_kind` builds the (struct- or enum-specific) body and is
    /// only invoked on first emission.
    fn emit_specialized_type(
        &mut self,
        kind: &str,
        name: &str,
        type_args: &[AstType],
        specialization_scope: Option<&str>,
        span: Span,
        make_kind: impl FnOnce() -> TypeDefKind,
    ) {
        let (mangled, already_emitted) =
            self.reserve_specialized_type_definition(kind, name, type_args, specialization_scope);
        if !already_emitted {
            self.specialized_types.push(TypedTypeDef {
                name: mangled,
                kind: make_kind(),
                span,
            });
        }
    }

    fn reserve_specialized_type_definition(
        &mut self,
        kind: &str,
        name: &str,
        type_args: &[AstType],
        specialization_scope: Option<&str>,
    ) -> (String, bool) {
        let requested = self.mangle_generic_type_name(name, type_args);
        let specialization_key =
            self.generic_specialization_key(kind, specialization_scope, &requested);
        let already_emitted = self
            .specialized_types_seen
            .contains_key(&specialization_key);
        let mangled = reserve_specialization_name(
            &mut self.specialized_types_seen,
            &mut self.specialized_type_name_owners,
            &specialization_key,
            &requested,
            specialization_scope,
        );
        let concrete_type_args: Vec<AstType> = type_args
            .iter()
            .map(|arg| self.type_to_ast_ref(&self.resolve_type(arg)))
            .collect();
        self.specialized_type_generic_names
            .insert(mangled.clone(), name.to_string());
        self.specialized_type_args
            .insert(mangled.clone(), concrete_type_args);

        (mangled, already_emitted)
    }
}

// ── Generic ↔ concrete type-arg recovery ────────────────────────────────────

impl TypeChecker {
    pub(crate) fn generic_type_args_from_type(
        &self,
        concrete_name: &str,
        ty: &Type,
    ) -> Option<(String, Vec<AstType>)> {
        if let (Some(generic_name), Some(type_args)) = (
            self.specialized_type_generic_names.get(concrete_name),
            self.specialized_type_args.get(concrete_name),
        ) {
            return Some((generic_name.clone(), type_args.clone()));
        }

        let mut candidates = self
            .structs
            .iter()
            .map(|(name, info)| (0u8, name.as_str(), info.type_params.as_slice()))
            .chain(
                self.enums
                    .iter()
                    .map(|(name, info)| (1u8, name.as_str(), info.type_params.as_slice())),
            )
            .filter(|(_, name, params)| {
                concrete_name != *name
                    && self.concrete_type_name_matches_generic(concrete_name, name)
                    && !params.is_empty()
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(right.1)));
        for (_, name, params) in candidates {
            let mut inferred = HashMap::new();
            let mut conflicts = Vec::new();
            let expected_type_args: Vec<_> = params
                .iter()
                .map(|param| AstType::Named(param.clone()))
                .collect();
            self.match_generic_type_with_args(
                name,
                &expected_type_args,
                ty,
                params,
                &mut inferred,
                &mut conflicts,
            );
            let type_args: Vec<_> = params
                .iter()
                .filter_map(|param| inferred.get(param).map(|ty| self.type_to_ast_ref(ty)))
                .collect();
            if type_args.len() == params.len() {
                return Some((name.to_string(), type_args));
            }
        }

        None
    }

    pub(crate) fn type_to_ast_ref(&self, ty: &Type) -> AstType {
        let (Type::Struct { name, .. } | Type::Enum { name, .. }) = ty else {
            return type_to_ast(ty);
        };

        if let Some((generic_name, type_args)) = self.generic_type_args_from_type(name, ty) {
            return AstType::Generic {
                name: generic_name,
                type_args,
            };
        }

        type_to_ast(ty)
    }

    pub(crate) fn ensure_specialized_type_refs_for_type(&mut self, ty: &Type, span: Span) {
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
                for payload in variants.iter().filter_map(|(_, payload)| payload.as_ref()) {
                    self.ensure_specialized_type_refs_for_type(payload, span);
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
}

// ── Method self-type derivation ─────────────────────────────────────────────

impl TypeChecker {
    pub(crate) fn generic_method_self_type(
        &self,
        method_name: &str,
        substitutions: &HashMap<String, Type>,
    ) -> Option<Type> {
        let receiver_name = super::super::method_signature_receiver_name(method_name)?;
        let method_params = self
            .generic_methods
            .get(method_name)
            .map(|template| template.type_params.clone())
            .unwrap_or_default();

        let Some(receiver_params) = self
            .type_params_for_type(receiver_name)
            .filter(|params| !params.is_empty())
        else {
            return Some(self.resolve_type(&AstType::Named(receiver_name.to_string())));
        };

        let type_args: Vec<AstType> = receiver_params
            .iter()
            .enumerate()
            .filter_map(|(idx, param)| {
                let key = if substitutions.contains_key(param) {
                    param
                } else {
                    method_params.get(idx)?
                };
                substitutions.get(key).map(|ty| self.type_to_ast_ref(ty))
            })
            .collect();

        Some(if type_args.len() == receiver_params.len() {
            self.resolve_type(&AstType::Generic {
                name: receiver_name.to_string(),
                type_args,
            })
        } else {
            Type::Unknown
        })
    }
}
