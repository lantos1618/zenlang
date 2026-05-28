use super::*;

impl TypeChecker {
    pub fn check_module_graph_entry(
        &mut self,
        graph: &ResolvedModuleGraph,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        let Some(entry) = graph.module(graph.entry) else {
            self.push_error(
                E0232,
                format!("module graph missing entry module {:?}", graph.entry),
                Span::dummy(),
            );
            return Err(self.diagnostics.clone());
        };

        let mut dependency_programs = Vec::new();
        for module in graph.sorted_modules() {
            if module.info.id == graph.entry {
                continue;
            }

            let mut checker = TypeChecker::new();
            match checker.check_module_graph_module(graph, module) {
                Ok(typed) => dependency_programs.push(typed),
                Err(diags) => self.diagnostics.extend(diags),
            }
        }

        self.fail_if_errors()?;

        let mut typed = self.check_module_graph_module(graph, entry)?;
        for mut dependency in dependency_programs {
            typed.functions.append(&mut dependency.functions);
            typed.types.append(&mut dependency.types);
            typed.globals.append(&mut dependency.globals);
        }
        Ok(typed)
    }

    fn check_module_graph_module(
        &mut self,
        graph: &ResolvedModuleGraph,
        module: &ResolvedModule,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        self.collect_module_graph_imports(graph, module);
        self.fail_if_errors()?;

        self.check_program(&module.program)
    }
}
