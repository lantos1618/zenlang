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
        // Deduplicate by name to avoid C redefinition errors when multiple
        // modules import the same wrapper function from std.compiler.
        let mut seen_fns: std::collections::HashSet<String> =
            typed.functions.iter().map(|f| f.name.clone()).collect();
        let mut seen_types: std::collections::HashSet<String> =
            typed.types.iter().map(|t| t.name.clone()).collect();
        let mut seen_globals: std::collections::HashSet<String> =
            typed.globals.iter().map(|g| g.name.clone()).collect();
        for mut dependency in dependency_programs {
            for f in dependency.functions.drain(..) {
                if seen_fns.insert(f.name.clone()) {
                    typed.functions.push(f);
                }
            }
            for t in dependency.types.drain(..) {
                if seen_types.insert(t.name.clone()) {
                    typed.types.push(t);
                }
            }
            for g in dependency.globals.drain(..) {
                if seen_globals.insert(g.name.clone()) {
                    typed.globals.push(g);
                }
            }
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
