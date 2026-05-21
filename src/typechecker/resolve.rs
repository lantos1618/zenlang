//! Type resolution — AstType → Type, field lookups, and compatibility checks.
#![allow(clippy::result_large_err)]

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::TypeChecker;

impl TypeChecker {
    /// Resolve an AstType to a concrete Type.
    pub(crate) fn resolve_type(&self, ast_ty: &AstType) -> Type {
        match ast_ty {
            AstType::I8 => Type::I8,
            AstType::I16 => Type::I16,
            AstType::I32 => Type::I32,
            AstType::I64 => Type::I64,
            AstType::U8 => Type::U8,
            AstType::U16 => Type::U16,
            AstType::U32 => Type::U32,
            AstType::U64 => Type::U64,
            AstType::Usize => Type::Usize,
            AstType::F32 => Type::F32,
            AstType::F64 => Type::F64,
            AstType::Bool => Type::Bool,
            AstType::Void => Type::Void,
            AstType::Str => Type::Str,
            AstType::String => Type::String,
            AstType::Named(name) => self.resolve_named_type(name),
            AstType::Generic { name, type_args } => self.resolve_generic_type(name, type_args),
            AstType::Ptr(inner) => Type::Ptr(Box::new(self.resolve_type(inner))),
            AstType::MutPtr(inner) => Type::MutPtr(Box::new(self.resolve_type(inner))),
            AstType::RawPtr(inner) => Type::RawPtr(Box::new(self.resolve_type(inner))),
            AstType::Array { elem, size } => Type::Array {
                elem: Box::new(self.resolve_type(elem)),
                size: *size,
            },
            AstType::Slice(inner) => Type::Slice(Box::new(self.resolve_type(inner))),
            AstType::Function { params, ret } => Type::Function {
                params: params.iter().map(|p| self.resolve_type(p)).collect(),
                ret: Box::new(self.resolve_type(ret)),
            },
            AstType::SelfType => self.current_self_type.clone().unwrap_or(Type::Unknown),
            AstType::Inferred => Type::Unknown,
        }
    }

    pub(crate) fn lookup_field_type(&self, ty: &Type, field: &str) -> Type {
        match ty {
            Type::Struct { fields, .. } => {
                for (name, field_ty) in fields {
                    if name == field {
                        return field_ty.clone();
                    }
                }
                Type::Unknown
            }
            Type::Named(name) => {
                if let Some(info) = self.structs.get(name) {
                    for (fname, ftype) in &info.fields {
                        if fname == field {
                            return self.resolve_type(ftype);
                        }
                    }
                }
                Type::Unknown
            }
            Type::Ptr(inner) | Type::MutPtr(inner) => {
                // Auto-deref through pointers for field access
                self.lookup_field_type(inner, field)
            }
            _ => Type::Unknown,
        }
    }

    /// Check if two types are compatible (for assignment/return contexts).
    /// Returns true if the types are clearly compatible or if either is ambiguous.
    /// Returns false only for clear mismatches between concrete primitive types.
    pub(crate) fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }
        // Unknown types are always compatible (error recovery)
        if *expected == Type::Unknown || *actual == Type::Unknown {
            return true;
        }
        // Named/nominal types match only by explicit identity.
        match (expected, actual) {
            (Type::Named(a), Type::Named(b)) if a == b => return true,
            (Type::Struct { name: a, .. }, Type::Struct { name: b, .. }) if a == b => return true,
            (Type::Struct { name, .. }, Type::Named(n))
            | (Type::Named(n), Type::Struct { name, .. })
                if name == n =>
            {
                return true;
            }
            (Type::Enum { name: a, .. }, Type::Enum { name: b, .. }) if a == b => return true,
            (Type::Enum { name, .. }, Type::Named(n))
            | (Type::Named(n), Type::Enum { name, .. })
                if name == n =>
            {
                return true;
            }
            _ => {}
        }
        // Never type is compatible with anything (diverging expression)
        if *expected == Type::Never || *actual == Type::Never {
            return true;
        }
        // Numeric width/sign conversions require explicit casts. Literal
        // coercion is handled before this check at declaration sites.
        false
    }
}
