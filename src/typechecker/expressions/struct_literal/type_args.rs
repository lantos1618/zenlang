use super::*;
use crate::typechecker::StructInfo;

pub(super) struct StructLiteralTypeArgs {
    pub(super) type_name: String,
    pub(super) ty: Type,
    pub(super) field_defs: std::collections::HashMap<String, Type>,
    pub(super) constructor_type_args_valid: bool,
    pub(super) default_substitutions: Option<std::collections::HashMap<String, Type>>,
}

impl TypeChecker {
    pub(super) fn resolve_struct_literal_type_args(
        &mut self,
        name: &str,
        type_args: &[AstType],
        struct_info: Option<&StructInfo>,
        span: Span,
    ) -> StructLiteralTypeArgs {
        let type_arg_count = struct_info.map(|info| info.type_params.len());
        let type_args_valid = type_arg_count.is_none_or(|expected| expected == type_args.len());
        let type_arg_annotations_valid = type_args
            .iter()
            .all(|type_arg| self.generic_type_annotation_arities_valid(type_arg));
        let constructor_type_args_valid = type_args_valid && type_arg_annotations_valid;

        self.diagnose_struct_literal_type_arg_arity(name, type_args, type_arg_count, span);

        let (type_name, ty, field_defs) = if type_args.is_empty() {
            let ty = if let Some(expected) = type_arg_count.filter(|expected| *expected > 0) {
                self.diagnostics.push(Diagnostic::error(
                    "E5001",
                    format!(
                        "generic struct `{}` expects {} type arguments, found 0",
                        name, expected
                    ),
                    span,
                ));
                Type::Unknown
            } else {
                self.resolve_type(&AstType::Named(name.to_string()))
            };
            let field_defs = struct_info
                .filter(|_| type_args_valid)
                .map(|info| {
                    info.fields
                        .iter()
                        .map(|(field_name, field_type)| {
                            (field_name.clone(), self.resolve_type(field_type))
                        })
                        .collect()
                })
                .unwrap_or_default();
            (name.to_string(), ty, field_defs)
        } else {
            let type_name = self.mangle_generic_type_name(name, type_args);
            let ty = if type_args_valid {
                self.resolve_type(&AstType::Generic {
                    name: name.to_string(),
                    type_args: type_args.to_vec(),
                })
            } else {
                Type::Unknown
            };
            let field_defs = if constructor_type_args_valid {
                self.specialize_generic_struct(name, type_args, span)
            } else {
                std::collections::HashMap::new()
            };
            (type_name, ty, field_defs)
        };

        let default_substitutions = self.generic_struct_default_substitutions(
            type_args,
            constructor_type_args_valid,
            struct_info,
        );

        StructLiteralTypeArgs {
            type_name,
            ty,
            field_defs,
            constructor_type_args_valid,
            default_substitutions,
        }
    }

    fn diagnose_struct_literal_type_arg_arity(
        &mut self,
        name: &str,
        type_args: &[AstType],
        type_arg_count: Option<usize>,
        span: Span,
    ) {
        if !type_args.is_empty() && type_arg_count == Some(0) {
            self.diagnostics.push(Diagnostic::error(
                "E5002",
                format!(
                    "non-generic struct `{}` does not accept type arguments",
                    name
                ),
                span,
            ));
        } else if let Some(expected) = type_arg_count.filter(|expected| {
            !type_args.is_empty() && *expected > 0 && *expected != type_args.len()
        }) {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic struct `{}` expects {} type arguments, found {}",
                    name,
                    expected,
                    type_args.len()
                ),
                span,
            ));
        }
    }

    fn generic_struct_default_substitutions(
        &self,
        type_args: &[AstType],
        constructor_type_args_valid: bool,
        struct_info: Option<&StructInfo>,
    ) -> Option<std::collections::HashMap<String, Type>> {
        if type_args.is_empty() || !constructor_type_args_valid {
            return None;
        }

        struct_info.and_then(|info| {
            (info.type_params.len() == type_args.len()).then(|| {
                info.type_params
                    .iter()
                    .zip(type_args.iter())
                    .map(|(param, arg)| (param.clone(), self.resolve_type(arg)))
                    .collect()
            })
        })
    }
}
