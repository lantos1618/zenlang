use std::collections::HashMap;
use std::path::Path;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::monomorphize_types::type_mangle_key;
use super::super::TypeChecker;

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
