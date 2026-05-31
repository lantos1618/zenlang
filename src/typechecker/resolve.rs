use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;

use super::generics::monomorphize_types::substitute_ast_type;
use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn resolve_type(&self, ast_ty: &AstType) -> Type {
        self.resolve_type_with_substitutions(ast_ty, None)
    }

    pub(crate) fn substitute_type(
        &self,
        ast_type: &AstType,
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        self.resolve_type_with_substitutions(ast_type, Some(substitutions))
    }

    pub(super) fn resolve_type_with_substitutions(
        &self,
        ast_ty: &AstType,
        substitutions: Option<&HashMap<String, Type>>,
    ) -> Type {
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
            AstType::Named(name) => {
                if let Some(concrete) =
                    substitutions.and_then(|substitutions| substitutions.get(name))
                {
                    concrete.clone()
                } else if let Some(concrete) = self
                    .type_substitutions
                    .iter()
                    .rev()
                    .find_map(|subs| subs.get(name))
                {
                    concrete.clone()
                } else if let Some(info) = self.structs.get(name) {
                    self.resolve_struct_type(name, &info.fields, None)
                } else if let Some(info) = self.enums.get(name) {
                    self.resolve_enum_type(name, &info.variants, None)
                } else {
                    Type::Named(name.to_string())
                }
            }
            AstType::Generic { name, type_args } => {
                if let Some(substitutions) = substitutions {
                    let type_args = type_args
                        .iter()
                        .map(|arg| substitute_ast_type(arg, substitutions))
                        .collect::<Vec<_>>();
                    self.resolve_generic_type(name, &type_args)
                } else {
                    self.resolve_generic_type(name, type_args)
                }
            }
            AstType::Ptr(inner) => Type::Ptr(self.resolve_boxed_type(inner, substitutions)),
            AstType::MutPtr(inner) => Type::MutPtr(self.resolve_boxed_type(inner, substitutions)),
            AstType::RawPtr(inner) => Type::RawPtr(self.resolve_boxed_type(inner, substitutions)),
            AstType::Future(inner) => Type::Future(self.resolve_boxed_type(inner, substitutions)),
            AstType::Array { elem, size } => Type::Array {
                elem: self.resolve_boxed_type(elem, substitutions),
                size: *size,
            },
            AstType::Slice(inner) => Type::Slice(self.resolve_boxed_type(inner, substitutions)),
            AstType::Function { params, ret } => Type::Function {
                params: params
                    .iter()
                    .map(|param| self.resolve_type_with_substitutions(param, substitutions))
                    .collect(),
                ret: Box::new(self.resolve_type_with_substitutions(ret, substitutions)),
            },
            AstType::SelfType => self.current_self_type.clone().unwrap_or(Type::Unknown),
            AstType::Inferred => Type::Unknown,
        }
    }

    fn resolve_boxed_type(
        &self,
        ast_ty: &AstType,
        substitutions: Option<&HashMap<String, Type>>,
    ) -> Box<Type> {
        Box::new(self.resolve_type_with_substitutions(ast_ty, substitutions))
    }

    fn resolve_generic_type(&self, name: &str, type_args: &[AstType]) -> Type {
        let type_args = self.fill_type_arg_defaults(name, type_args);
        let type_args = type_args.as_slice();
        let requested = self.mangle_generic_type_name(name, type_args);
        if let Some(info) = self.structs.get(name) {
            let mangled = self.reserved_or_requested_generic_type_name(
                "struct",
                info.specialization_scope.as_deref(),
                requested,
            );
            let substitutions = self.type_arg_substitutions(&info.type_params, type_args);
            return self.resolve_struct_type(&mangled, &info.fields, Some(&substitutions));
        }
        if let Some(info) = self.enums.get(name) {
            let mangled = self.reserved_or_requested_generic_type_name(
                "enum",
                info.specialization_scope.as_deref(),
                requested,
            );
            let substitutions = self.type_arg_substitutions(&info.type_params, type_args);
            return self.resolve_enum_type(&mangled, &info.variants, Some(&substitutions));
        }
        Type::Named(requested)
    }

    fn resolve_struct_type(
        &self,
        name: &str,
        fields: &[(String, AstType)],
        substitutions: Option<&HashMap<String, Type>>,
    ) -> Type {
        Type::Struct {
            name: name.to_string(),
            fields: fields
                .iter()
                .map(|(field_name, field_type)| {
                    (
                        field_name.clone(),
                        self.resolve_type_with_substitutions(field_type, substitutions),
                    )
                })
                .collect(),
        }
    }

    fn resolve_enum_type(
        &self,
        name: &str,
        variants: &[(String, Option<AstType>)],
        substitutions: Option<&HashMap<String, Type>>,
    ) -> Type {
        Type::Enum {
            name: name.to_string(),
            variants: variants
                .iter()
                .map(|(variant_name, payload)| {
                    (
                        variant_name.clone(),
                        payload
                            .as_ref()
                            .map(|ty| self.resolve_type_with_substitutions(ty, substitutions)),
                    )
                })
                .collect(),
        }
    }

    pub(crate) fn lookup_field_type(&self, ty: &Type, field: &str) -> Type {
        match ty {
            Type::Struct { fields, .. } => fields
                .iter()
                .find_map(|(name, field_ty)| (name == field).then(|| field_ty.clone()))
                .unwrap_or(Type::Unknown),
            Type::Named(name) => self
                .structs
                .get(name)
                .and_then(|info| {
                    info.fields
                        .iter()
                        .find_map(|(name, ty)| (name == field).then(|| self.resolve_type(ty)))
                })
                .unwrap_or(Type::Unknown),
            Type::Ptr(inner) | Type::MutPtr(inner) => self.lookup_field_type(inner, field),
            // A `Future<T>` exposes a single field `frame: RawPtr<u8>` — the
            // coroutine-frame pointer the `@async`-call transform allocates. This
            // is the entire handle stdlib (`block_on`, `Scheduler`) reads to drive
            // the future through `@builtin.poll`. The C value of a future *is* that
            // pointer, so `.frame` lowers to identity (see the C backend).
            Type::Future(_) if field == "frame" => Type::RawPtr(Box::new(Type::U8)),
            _ => Type::Unknown,
        }
    }

    pub(crate) fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        match (expected, actual) {
            (a, b) if a == b => true,
            (Type::Unknown | Type::Never, _) | (_, Type::Unknown | Type::Never) => true,
            (Type::Named(name), ty) | (ty, Type::Named(name)) => {
                ty.nominal_name().is_some_and(|ty_name| ty_name == name)
            }
            (Type::Struct { name: a, .. }, Type::Struct { name: b, .. }) => a == b,
            (Type::Enum { name: a, .. }, Type::Enum { name: b, .. }) => a == b,
            _ => false,
        }
    }
}
