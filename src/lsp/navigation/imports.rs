// Import-related helper functions

use crate::ast::Declaration;
use lsp_types::Position;

pub struct ImportInfo {
    pub import_line: String,
    pub source: String,
}

/// Find import information for a symbol using AST (preferred method)
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

/// Find import information for a symbol (fallback to string parsing when AST unavailable)
pub fn find_import_info(
    content: &str,
    symbol_name: &str,
    position: Position,
) -> Option<ImportInfo> {
    let start_line = position.line as usize;
    let lines: Vec<&str> = content.lines().take(start_line + 1).collect();

    for (_line_num, line) in lines.iter().enumerate().rev() {
        let trimmed = line.trim();

        if trimmed.starts_with('{') && trimmed.contains('}') && trimmed.contains('=') {
            if let Some(brace_end) = trimmed.find('}') {
                let import_part = &trimmed[1..brace_end];
                let imports: Vec<&str> = import_part.split(',').map(|s| s.trim()).collect();

                if imports.contains(&symbol_name) {
                    if let Some(eq_pos) = trimmed.find('=') {
                        let source = trimmed[eq_pos + 1..].trim();
                        return Some(ImportInfo {
                            import_line: trimmed.to_string(),
                            source: source.to_string(),
                        });
                    }
                }
            }
        }
    }

    None
}
