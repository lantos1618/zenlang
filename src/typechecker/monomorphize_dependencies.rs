use std::collections::HashMap;

use super::{TemplateDependencyState, TypeChecker};

fn install_dependency_map<T: Clone>(
    target: &mut HashMap<String, T>,
    dependencies: &HashMap<String, T>,
) -> Vec<(String, Option<T>)> {
    dependencies
        .iter()
        .map(|(name, value)| (name.clone(), target.insert(name.clone(), value.clone())))
        .collect()
}

fn restore_dependency_map<T>(target: &mut HashMap<String, T>, state: Vec<(String, Option<T>)>) {
    for (name, previous) in state {
        if let Some(previous) = previous {
            target.insert(name, previous);
        } else {
            target.remove(&name);
        }
    }
}

impl TypeChecker {
    pub(super) fn install_template_dependencies(
        &mut self,
        template: &super::GenericFunctionTemplate,
    ) -> TemplateDependencyState {
        let dependencies = &template.dependencies;
        TemplateDependencyState {
            structs: install_dependency_map(&mut self.structs, &dependencies.structs),
            enums: install_dependency_map(&mut self.enums, &dependencies.enums),
            functions: install_dependency_map(&mut self.functions, &dependencies.functions),
            generic_functions: install_dependency_map(
                &mut self.generic_functions,
                &dependencies.generic_functions,
            ),
            methods: install_dependency_map(&mut self.methods, &dependencies.methods),
            generic_methods: install_dependency_map(
                &mut self.generic_methods,
                &dependencies.generic_methods,
            ),
        }
    }

    pub(super) fn restore_template_dependencies(&mut self, state: TemplateDependencyState) {
        restore_dependency_map(&mut self.structs, state.structs);
        restore_dependency_map(&mut self.enums, state.enums);
        restore_dependency_map(&mut self.functions, state.functions);
        restore_dependency_map(&mut self.generic_functions, state.generic_functions);
        restore_dependency_map(&mut self.methods, state.methods);
        restore_dependency_map(&mut self.generic_methods, state.generic_methods);
    }
}
