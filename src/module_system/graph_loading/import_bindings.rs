use crate::error::{CompileError, Span};
use crate::module_system::{ImportBinding, ResolvedModule};

use super::exported_symbols::{exported_module_symbol, ExportedModuleSymbol};

pub(super) fn collect_import_bindings(
    dep_module: &ResolvedModule,
    names: &[String],
    module_name: &str,
    import_span: Span,
    bindings: &mut Vec<ImportBinding>,
) -> Result<(), Vec<CompileError>> {
    for name in names {
        match exported_module_symbol(&dep_module.symbols, name) {
            ExportedModuleSymbol::Public => {
                bindings.push(ImportBinding {
                    local_name: name.clone(),
                    source_module: dep_module.info.id,
                    source_symbol: name.clone(),
                    span: import_span,
                });
            }
            ExportedModuleSymbol::Private => {
                return Err(vec![CompileError::Resolution(
                    format!(
                        "symbol '{}' in module '{}' is not exported",
                        name, module_name
                    ),
                    Some(import_span),
                )]);
            }
            ExportedModuleSymbol::Missing => {
                return Err(vec![CompileError::Resolution(
                    format!("module '{}' does not export '{}'", module_name, name),
                    Some(import_span),
                )]);
            }
        }
    }

    Ok(())
}
