use super::*;
use crate::module_system::import_errors::{missing_export_error, private_export_error};

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
                    return Err(private_export_error(name, module_name, import_span));
                }
                return Err(missing_export_error(name, module_name, import_span));
            }
        }

        Ok(())
    }
}
