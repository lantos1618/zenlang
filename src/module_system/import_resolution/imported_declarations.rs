use super::*;

impl ModuleSystem {
    pub(super) fn collect_imported_declarations(
        &self,
        dep_program: &Program,
        names: &[String],
        module_name: &str,
        import_span: Span,
        imported_decls: &mut Vec<Declaration>,
    ) -> Result<(), Vec<CompileError>> {
        for name in names {
            let mut found_private = false;
            let mut found_public = false;

            for decl in &dep_program.declarations {
                if decl.name() == Some(name.as_str()) {
                    if decl.is_public() {
                        found_public = true;
                        imported_decls.push(decl.clone());
                    } else {
                        found_private = true;
                    }
                }

                if let Declaration::Method {
                    type_name, public, ..
                } = decl
                {
                    if type_name == name && *public {
                        imported_decls.push(decl.clone());
                    }
                }
            }

            if !found_public {
                if found_private {
                    return Err(vec![CompileError::Resolution(
                        format!(
                            "symbol '{}' in module '{}' is not exported",
                            name, module_name
                        ),
                        Some(import_span),
                    )]);
                }
                return Err(vec![CompileError::Resolution(
                    format!("module '{}' does not export '{}'", module_name, name),
                    Some(import_span),
                )]);
            }
        }

        Ok(())
    }
}
