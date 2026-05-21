use std::collections::HashMap;

use crate::ast::{is_builtin_type_name, AstType};

use super::{Type, TypeChecker};

impl TypeChecker {
    pub(super) fn resolve_named_type(&self, name: &str) -> Type {
        if let Some(concrete) = self
            .type_substitutions
            .iter()
            .rev()
            .find_map(|subs| subs.get(name))
        {
            return concrete.clone();
        }
        if is_builtin_type_name(name) {
            return Type::String;
        }
        if let Some(info) = self.structs.get(name) {
            return self.resolve_struct_type(&info.name, &info.fields, None);
        }
        if let Some(info) = self.enums.get(name) {
            return self.resolve_enum_type(&info.name, &info.variants, None);
        }
        Type::Named(name.to_string())
    }

    pub(super) fn resolve_generic_type(&self, name: &str, type_args: &[AstType]) -> Type {
        let mangled = self.mangle_generic_type_name(name, type_args);
        if let Some(info) = self.structs.get(name) {
            let substitutions = self.generic_type_substitutions(&info.type_params, type_args);
            return self.resolve_struct_type(&mangled, &info.fields, Some(&substitutions));
        }
        if let Some(info) = self.enums.get(name) {
            let substitutions = self.generic_type_substitutions(&info.type_params, type_args);
            return self.resolve_enum_type(&mangled, &info.variants, Some(&substitutions));
        }
        Type::Named(mangled)
    }

    fn generic_type_substitutions(
        &self,
        type_params: &[String],
        type_args: &[AstType],
    ) -> HashMap<String, Type> {
        type_params
            .iter()
            .zip(type_args.iter())
            .map(|(param, arg)| (param.clone(), self.resolve_type(arg)))
            .collect()
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
                        self.resolve_aggregate_member_type(field_type, substitutions),
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
                            .map(|ty| self.resolve_aggregate_member_type(ty, substitutions)),
                    )
                })
                .collect(),
        }
    }

    fn resolve_aggregate_member_type(
        &self,
        member_type: &AstType,
        substitutions: Option<&HashMap<String, Type>>,
    ) -> Type {
        substitutions.map_or_else(
            || self.resolve_type(member_type),
            |substitutions| self.substitute_type(member_type, substitutions),
        )
    }
}
