use super::*;
use crate::typechecker::EnumInfo;

pub(super) struct EnumVariantTypeArgs {
    pub(super) type_name: String,
    pub(super) ty: Type,
    pub(super) variant_defs: std::collections::HashMap<String, Option<Type>>,
    pub(super) type_args_valid: bool,
}

impl TypeChecker {
    pub(super) fn resolve_enum_variant_type_args(
        &mut self,
        enum_name: &str,
        type_args: &[AstType],
        enum_info: Option<&EnumInfo>,
        span: Span,
    ) -> EnumVariantTypeArgs {
        let type_arg_count = enum_info.map(|info| info.type_params.len());
        let type_args_valid = type_arg_count.is_none_or(|expected| expected == type_args.len());

        self.diagnose_enum_variant_type_arg_arity(enum_name, type_args, type_arg_count, span);

        let (type_name, ty, variant_defs) = if type_args.is_empty() {
            let ty = self.resolve_type(&AstType::Named(enum_name.to_string()));
            if let Some(expected) = type_arg_count.filter(|expected| *expected > 0) {
                self.diagnostics.push(Diagnostic::error(
                    "E5001",
                    format!(
                        "generic enum `{}` expects {} type arguments, found 0",
                        enum_name, expected
                    ),
                    span,
                ));
            }
            let variant_defs = enum_info
                .filter(|_| type_args_valid)
                .map(|info| {
                    info.variants
                        .iter()
                        .map(|(variant_name, payload)| {
                            (
                                variant_name.clone(),
                                payload.as_ref().map(|ty| self.resolve_type(ty)),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            (enum_name.to_string(), ty, variant_defs)
        } else {
            let type_name = self.mangle_generic_type_name(enum_name, type_args);
            let ty = if type_args_valid {
                self.resolve_type(&AstType::Generic {
                    name: enum_name.to_string(),
                    type_args: type_args.to_vec(),
                })
            } else {
                Type::Unknown
            };
            let variant_defs = if type_args_valid {
                self.specialize_generic_enum(enum_name, type_args, span)
            } else {
                std::collections::HashMap::new()
            };
            (type_name, ty, variant_defs)
        };

        EnumVariantTypeArgs {
            type_name,
            ty,
            variant_defs,
            type_args_valid,
        }
    }

    fn diagnose_enum_variant_type_arg_arity(
        &mut self,
        enum_name: &str,
        type_args: &[AstType],
        type_arg_count: Option<usize>,
        span: Span,
    ) {
        if !type_args.is_empty() && type_arg_count == Some(0) {
            self.diagnostics.push(Diagnostic::error(
                "E5002",
                format!(
                    "non-generic enum `{}` does not accept type arguments",
                    enum_name
                ),
                span,
            ));
        } else if let Some(expected) = type_arg_count.filter(|expected| {
            !type_args.is_empty() && *expected > 0 && *expected != type_args.len()
        }) {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic enum `{}` expects {} type arguments, found {}",
                    enum_name,
                    expected,
                    type_args.len()
                ),
                span,
            ));
        }
    }
}
