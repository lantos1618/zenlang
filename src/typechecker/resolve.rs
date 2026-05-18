//! Type resolution — AstType → Type, field lookups, binary op checking.
#![allow(clippy::result_large_err)]

use crate::ast::expressions::BinaryOp;
use crate::ast::typed::Type;
use crate::ast::AstType;
use crate::error::{Diagnostic, Span};

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
            AstType::Named(name) => {
                if let Some(concrete) = self
                    .type_substitutions
                    .iter()
                    .rev()
                    .find_map(|subs| subs.get(name))
                {
                    return concrete.clone();
                }
                if name == "String" {
                    return Type::String;
                }
                if let Some(info) = self.structs.get(name) {
                    Type::Struct {
                        name: info.name.clone(),
                        fields: info
                            .fields
                            .iter()
                            .map(|(n, t)| (n.clone(), self.resolve_type(t)))
                            .collect(),
                    }
                } else if let Some(info) = self.enums.get(name) {
                    Type::Enum {
                        name: info.name.clone(),
                        variants: info
                            .variants
                            .iter()
                            .map(|(n, t)| (n.clone(), t.as_ref().map(|ty| self.resolve_type(ty))))
                            .collect(),
                    }
                } else {
                    Type::Named(name.clone()) // forward reference or external type
                }
            }
            AstType::Generic { name, type_args } => {
                let mangled = self.mangle_generic_type_name(name, type_args);
                if let Some(info) = self.structs.get(name) {
                    let substitutions: std::collections::HashMap<String, Type> = info
                        .type_params
                        .iter()
                        .zip(type_args.iter())
                        .map(|(param, arg)| (param.clone(), self.resolve_type(arg)))
                        .collect();
                    Type::Struct {
                        name: mangled,
                        fields: info
                            .fields
                            .iter()
                            .map(|(n, t)| (n.clone(), self.substitute_type(t, &substitutions)))
                            .collect(),
                    }
                } else if let Some(info) = self.enums.get(name) {
                    let substitutions: std::collections::HashMap<String, Type> = info
                        .type_params
                        .iter()
                        .zip(type_args.iter())
                        .map(|(param, arg)| (param.clone(), self.resolve_type(arg)))
                        .collect();
                    Type::Enum {
                        name: mangled,
                        variants: info
                            .variants
                            .iter()
                            .map(|(n, t)| {
                                (
                                    n.clone(),
                                    t.as_ref()
                                        .map(|ty| self.substitute_type(ty, &substitutions)),
                                )
                            })
                            .collect(),
                    }
                } else {
                    Type::Named(mangled)
                }
            }
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

    pub(crate) fn check_binary_op(
        &self,
        op: BinaryOp,
        left: &Type,
        right: &Type,
        span: &Span,
    ) -> Result<Type, Diagnostic> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                // Arithmetic — both sides must be numeric (allow Unknown for error recovery)
                if *left == Type::Unknown || *right == Type::Unknown {
                    let known = if *left != Type::Unknown {
                        left
                    } else {
                        right
                    };
                    return Ok(known.clone());
                }
                if !left.is_numeric() {
                    return Err(Diagnostic::error(
                        "E3010",
                        format!("arithmetic on non-numeric type `{}`", left.display_name()),
                        *span,
                    ));
                }
                if !right.is_numeric() {
                    return Err(Diagnostic::error(
                        "E3010",
                        format!("arithmetic on non-numeric type `{}`", right.display_name()),
                        *span,
                    ));
                }
                if left != right {
                    return Err(Diagnostic::error(
                        "E3013",
                        format!(
                            "arithmetic operands must have the same type, found `{}` and `{}`",
                            left.display_name(),
                            right.display_name()
                        ),
                        *span,
                    ));
                }
                Ok(left.clone())
            }
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::LtEq
            | BinaryOp::GtEq => Ok(Type::Bool),
            BinaryOp::And | BinaryOp::Or => {
                if *left != Type::Bool && *left != Type::Unknown {
                    return Err(Diagnostic::error(
                        "E3011",
                        format!(
                            "logical operator requires `bool`, found `{}`",
                            left.display_name()
                        ),
                        *span,
                    ));
                }
                if *right != Type::Bool && *right != Type::Unknown {
                    return Err(Diagnostic::error(
                        "E3011",
                        format!(
                            "logical operator requires `bool`, found `{}`",
                            right.display_name()
                        ),
                        *span,
                    ));
                }
                Ok(Type::Bool)
            }
            BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => {
                if *left == Type::Unknown || *right == Type::Unknown {
                    let known = if *left != Type::Unknown {
                        left
                    } else {
                        right
                    };
                    return Ok(known.clone());
                }
                if !left.is_integer() {
                    return Err(Diagnostic::error(
                        "E3012",
                        format!(
                            "bitwise operator requires integer type, found `{}`",
                            left.display_name()
                        ),
                        *span,
                    ));
                }
                if !right.is_integer() {
                    return Err(Diagnostic::error(
                        "E3012",
                        format!(
                            "bitwise operator requires integer type, found `{}`",
                            right.display_name()
                        ),
                        *span,
                    ));
                }
                Ok(left.clone())
            }
        }
    }
}
