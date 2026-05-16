use std::collections::HashMap;

use super::{TemplateDependencyEntry, TemplateDependencyState, TypeChecker};

fn install_dependency_map<T: Clone>(
    target: &mut HashMap<String, T>,
    dependencies: &HashMap<String, T>,
) -> Vec<TemplateDependencyEntry<T>> {
    dependencies
        .iter()
        .map(|(name, value)| TemplateDependencyEntry {
            name: name.clone(),
            previous: target.insert(name.clone(), value.clone()),
        })
        .collect()
}

fn restore_dependency_map<T>(
    target: &mut HashMap<String, T>,
    state: Vec<TemplateDependencyEntry<T>>,
) {
    for entry in state {
        if let Some(previous) = entry.previous {
            target.insert(entry.name, previous);
        } else {
            target.remove(&entry.name);
        }
    }
}

impl TypeChecker {
    pub(super) fn install_template_dependencies(
        &mut self,
        template: &super::GenericFunctionTemplate,
    ) -> TemplateDependencyState {
        TemplateDependencyState {
            structs: install_dependency_map(&mut self.structs, &template.dependency_structs),
            enums: install_dependency_map(&mut self.enums, &template.dependency_enums),
            functions: install_dependency_map(&mut self.functions, &template.dependency_functions),
            generic_functions: install_dependency_map(
                &mut self.generic_functions,
                &template.dependency_generic_functions,
            ),
            methods: install_dependency_map(&mut self.methods, &template.dependency_methods),
            generic_methods: install_dependency_map(
                &mut self.generic_methods,
                &template.dependency_generic_methods,
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
