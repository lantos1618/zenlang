use super::*;

impl TypeChecker {
    pub(super) fn collect_declarations(&mut self, decls: &[Declaration]) {
        for decl in decls {
            let Declaration::Behavior {
                name, type_params, ..
            } = decl
            else {
                continue;
            };
            self.validate_generic_bounds(type_params);
            self.seed_declaration_info(name, decl);
        }

        for decl in decls {
            let Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } = decl
            else {
                continue;
            };
            self.check_behavior_extends(behavior, parent, parent_type_args, *span);
        }
        self.validate_behavior_extends_cycles();
        self.validate_behavior_method_coherence();

        for decl in decls {
            if let Some(callable) = decl.as_callable() {
                let seed_name = match decl {
                    Declaration::Method { type_name, .. } => type_name,
                    _ => callable.name,
                };
                self.validate_generic_bounds(callable.type_params);
                self.seed_declaration_info(seed_name, decl);
                continue;
            }

            match decl {
                Declaration::Struct {
                    name, type_params, ..
                }
                | Declaration::Enum {
                    name, type_params, ..
                } => {
                    self.validate_generic_bounds(type_params);
                    self.seed_declaration_info(name, decl);
                }
                Declaration::Import { names, .. } => {
                    // Real `io_*` (and every other `{ x } = std` namespace) function
                    // is spliced from stdlib as an actual Zen declaration before this
                    // runs, so no compiler-side stub seeding is needed.
                    for name in names {
                        self.imports.insert(name.to_string());
                    }
                }
                Declaration::ImplBlock {
                    type_name,
                    type_args,
                    behavior,
                    behavior_type_args,
                    methods,
                    ..
                } => {
                    for method in methods {
                        let Some(method_decl) = method.as_callable() else {
                            continue;
                        };
                        self.validate_generic_bounds(method_decl.type_params);
                        let key = behavior_impl_method_signature_key_with_target_args(
                            type_name,
                            method_decl.name,
                            behavior.as_deref(),
                            behavior_type_args,
                            type_args,
                        );
                        insert_callable_signature(
                            key,
                            method,
                            &mut self.methods,
                            &mut self.generic_methods,
                        );
                    }
                    if let Some(behavior) = behavior {
                        for default in self.behavior_default_methods_for_impl(
                            type_name,
                            type_args,
                            behavior,
                            behavior_type_args,
                            methods,
                        ) {
                            self.seed_behavior_default_method_signature(type_name, &default);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
