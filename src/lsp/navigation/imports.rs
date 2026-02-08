// Import-related helper functions

use crate::ast::Declaration;

pub struct ImportInfo {
    pub import_line: String,
    pub source: String,
}

/// Find import information for a symbol using AST declarations
pub fn find_import_info_from_ast(ast: &[Declaration], symbol_name: &str) -> Option<ImportInfo> {
    for decl in ast {
        if let Declaration::ModuleImport {
            alias, module_path, ..
        } = decl
        {
            if alias == symbol_name {
                return Some(ImportInfo {
                    import_line: format!("{{ {} }} = {}", alias, module_path),
                    source: module_path.clone(),
                });
            }
        }
    }
    None
}
