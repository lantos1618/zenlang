use super::*;

impl TypeChecker {
    pub(super) fn validate_collected_declaration_semantics(&mut self, decls: &[Declaration]) {
        for decl in decls {
            match decl {
                Declaration::ImplBlock {
                    type_name,
                    type_args,
                    behavior: Some(behavior),
                    behavior_type_args,
                    methods,
                    span,
                    ..
                } => {
                    self.check_behavior_impl(
                        type_name,
                        type_args,
                        behavior,
                        behavior_type_args,
                        methods,
                        *span,
                    );
                }
                Declaration::Requires {
                    type_name,
                    behavior,
                    behavior_type_args,
                    span,
                } => self.check_behavior_requires(type_name, behavior, behavior_type_args, *span),
                Declaration::Struct {
                    type_params,
                    fields,
                    ..
                } if type_params.is_empty() => {
                    for field in fields {
                        let Some(default) = &field.default else {
                            continue;
                        };
                        let expected = self.resolve_type(&field.ty);
                        let actual = self.with_scope(|checker| checker.check_expr(default));

                        let Ok(actual) = actual else {
                            self.diagnostics.push(actual.expect_err("checked error"));
                            continue;
                        };
                        let actual_ty = literal_coerced_type(&expected, &actual);
                        if !self.types_compatible(&expected, &actual_ty) {
                            let (expected, actual_display) =
                                type_display_pair(&expected, &actual.ty);
                            self.push_error(
                                E3073,
                                format!(
                                    "field `{}` default expects `{expected}`, found `{actual_display}`",
                                    field.name
                                ),
                                actual.span,
                            );
                        }
                    }
                }
                _ => {}
            }
            self.validate_ast_type_references(decl);
        }
    }
}
