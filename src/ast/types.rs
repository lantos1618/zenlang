use crate::error::Span;
use serde::Serialize;

use super::declarations::TypeParam;

mod names;

pub use names::{BuiltinGenericTypeName, BuiltinTypeName};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AstType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Usize,

    F32,
    F64,

    Bool,
    Void,

    Str, // static string view over baked program storage: { ptr, len }

    Named(String),

    Generic {
        name: String,
        type_args: Vec<AstType>,
    },

    Array {
        elem: Box<AstType>,
        size: Option<usize>,
    },
    Slice(Box<AstType>),

    Ptr(Box<AstType>),
    MutPtr(Box<AstType>),
    RawPtr(Box<AstType>),

    /// The surface spelling of a suspendable computation, `Future<T>`. A
    /// well-known builtin generic type (like `RawPtr<T>`): resolvable without an
    /// import so `@async`/`@await` work in any program, and nameable in stdlib
    /// signatures so it resolves to the SAME `Type::Future(T)` the compiler
    /// produces for an `@async` call.
    Future(Box<AstType>),

    Function {
        params: Vec<AstType>,
        ret: Box<AstType>,
    },

    SelfType,
    Inferred,
}

impl AstType {
    pub fn display_name(&self) -> String {
        if let Some(builtin) = BuiltinTypeName::from_ast_type(self) {
            return builtin.to_string();
        }

        match self {
            AstType::Named(n) => n.clone(),
            AstType::Generic { name, type_args } => format!("{name}<{}>", type_list(type_args)),
            AstType::Array { elem, size } => match size {
                Some(n) => format!("[{}; {}]", elem.display_name(), n),
                None => format!("[{}]", elem.display_name()),
            },
            AstType::Slice(elem) => format!("[{}]", elem.display_name()),
            AstType::Ptr(inner) => format!("Ptr<{}>", inner.display_name()),
            AstType::MutPtr(inner) => format!("MutPtr<{}>", inner.display_name()),
            AstType::RawPtr(inner) => format!("RawPtr<{}>", inner.display_name()),
            AstType::Future(inner) => format!("Future<{}>", inner.display_name()),
            AstType::Function { params, ret } => {
                format!("({}) {}", type_list(params), ret.display_name())
            }
            AstType::Inferred => "_".into(),
            _ => unreachable!("handled by BuiltinTypeName"),
        }
    }

    pub(crate) fn any(&self, predicate: &mut impl FnMut(&AstType) -> bool) -> bool {
        if predicate(self) {
            return true;
        }

        match self {
            AstType::Generic { type_args, .. } => type_args.iter().any(|arg| arg.any(predicate)),
            AstType::Array { elem, .. }
            | AstType::Slice(elem)
            | AstType::Ptr(elem)
            | AstType::MutPtr(elem)
            | AstType::RawPtr(elem)
            | AstType::Future(elem) => elem.any(predicate),
            AstType::Function { params, ret } => {
                params.iter().any(|param| param.any(predicate)) || ret.any(predicate)
            }
            _ => false,
        }
    }
}

pub(crate) fn behavior_ref_display(behavior: &str, type_args: &[AstType]) -> String {
    if type_args.is_empty() {
        behavior.to_string()
    } else {
        format!("{behavior}<{}>", type_list(type_args))
    }
}

pub(crate) fn method_symbol_key(type_name: &str, method_name: &str) -> String {
    format!("{type_name}.{method_name}")
}

pub(crate) fn behavior_impl_method_symbol_key(
    type_name: &str,
    method_name: &str,
    behavior: Option<&str>,
    behavior_type_args: &[AstType],
    target_type_args: &[AstType],
) -> String {
    let key = method_symbol_key(type_name, method_name);
    let Some(behavior) = behavior else {
        return key;
    };
    if behavior_type_args.is_empty() {
        return key;
    }

    if !target_type_args.is_empty()
        && behavior_type_args == target_type_args
        && behavior_type_args
            .iter()
            .all(|arg| matches!(arg, AstType::Named(_)))
    {
        format!("{key}__{behavior}")
    } else {
        format!(
            "{}__{}",
            key,
            behavior_ref_symbol_suffix(behavior, behavior_type_args)
        )
    }
}

fn type_list(types: &[AstType]) -> String {
    types
        .iter()
        .map(AstType::display_name)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn behavior_ref_symbol_suffix(behavior: &str, type_args: &[AstType]) -> String {
    std::iter::once(behavior.to_string())
        .chain(type_args.iter().map(AstType::display_name))
        .map(|name| symbol_key_part(&name))
        .collect::<Vec<_>>()
        .join("_")
}

pub(crate) fn symbol_key_part(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub(crate) fn named_type_arg_names(type_args: &[AstType]) -> Vec<String> {
    type_args
        .iter()
        .filter_map(|arg| match arg {
            AstType::Named(name) => Some(name.clone()),
            _ => None,
        })
        .collect()
}

pub(crate) fn named_type_arg_params(type_args: &[AstType]) -> Vec<TypeParam> {
    type_params_from_names(named_type_arg_names(type_args))
}

pub(crate) fn type_params_from_names(names: impl IntoIterator<Item = String>) -> Vec<TypeParam> {
    names
        .into_iter()
        .map(|name| TypeParam {
            name,
            constraint: None,
            constraint_type_args: Vec::new(),
            default: None,
            span: Span::dummy(),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Param {
    pub name: String,
    pub ty: AstType,
    pub mutable: bool,
    pub span: Span,
}
