// Symbol position finding and workspace search
use super::super::types::SymbolInfo;
use super::DocumentStore;
use crate::ast::Declaration;
use lsp_types::*;

impl DocumentStore {
    pub(super) fn find_declaration_position(
        &self,
        content: &str,
        decl: &Declaration,
        _index: usize,
    ) -> (usize, usize) {
        // Find the line number and character position where the declaration starts
        let search_str = match decl {
            Declaration::Function(f) => &f.name,
            Declaration::Struct(s) => &s.name,
            Declaration::Enum(e) => &e.name,
            Declaration::Constant { name, .. } => name,
            _ => return (0, 0),
        };

        let lines: Vec<&str> = content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            // Look for the symbol name at word boundaries followed by = or :
            if let Some(pos) = self.find_word_in_line_for_symbol(line, search_str) {
                // Check if this looks like a definition (has = or : after the name)
                let after_symbol = &line[pos + search_str.len()..].trim();
                if after_symbol.starts_with('=')
                    || after_symbol.starts_with(':')
                    || after_symbol.starts_with('(')
                {
                    return (line_num, pos);
                }
            }
        }
        (0, 0)
    }

    pub(super) fn find_word_in_line_for_symbol(&self, line: &str, symbol: &str) -> Option<usize> {
        let mut search_pos = 0;
        while let Some(pos) = line[search_pos..].find(symbol) {
            let actual_pos = search_pos + pos;

            // Check word boundaries
            let before_ok = actual_pos == 0 || {
                let before = line.chars().nth(actual_pos - 1).unwrap_or(' ');
                !before.is_alphanumeric() && before != '_'
            };
            let after_pos = actual_pos + symbol.len();
            let after_ok = after_pos >= line.len() || {
                let after = line.chars().nth(after_pos).unwrap_or(' ');
                !after.is_alphanumeric() && after != '_'
            };

            if before_ok && after_ok {
                return Some(actual_pos);
            }

            search_pos = actual_pos + 1;
        }
        None
    }

    pub fn search_workspace_for_symbol(&self, symbol_name: &str) -> Option<(Url, SymbolInfo)> {
        use std::path::Path;

        let workspace_root = self.workspace_root.as_ref()?;
        let root_path = Path::new(workspace_root.path());

        let mut files_parsed = 0;
        self.search_directory_for_symbol_bounded(root_path, symbol_name, 0, &mut files_parsed)
    }

    fn search_directory_for_symbol_bounded(
        &self,
        dir: &std::path::Path,
        symbol_name: &str,
        depth: usize,
        files_parsed: &mut usize,
    ) -> Option<(Url, SymbolInfo)> {
        use crate::lsp::search_limits::{MAX_DIRECTORY_DEPTH, MAX_FILES_TO_PARSE};
        use std::fs;

        // Prevent stack overflow from deep recursion
        if depth >= MAX_DIRECTORY_DEPTH {
            log::debug!(
                "[LSP] search_directory_for_symbol: max depth {} reached",
                depth
            );
            return None;
        }

        // Prevent OOM from parsing too many files
        if *files_parsed >= MAX_FILES_TO_PARSE {
            log::debug!(
                "[LSP] search_directory_for_symbol: max files {} reached",
                *files_parsed
            );
            return None;
        }

        if !dir.is_dir() {
            return None;
        }

        let entries = fs::read_dir(dir).ok()?;

        for entry in entries.flatten() {
            // Check file limit on each iteration
            if *files_parsed >= MAX_FILES_TO_PARSE {
                return None;
            }

            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|e| e == "zen") {
                *files_parsed += 1;
                if let Ok(content) = fs::read_to_string(&path) {
                    let symbols = self.extract_symbols(&content);

                    if let Some(symbol_info) = symbols.get(symbol_name) {
                        if let Ok(uri) = Url::from_file_path(&path) {
                            let mut symbol = symbol_info.clone();
                            symbol.definition_uri = Some(uri.clone());
                            return Some((uri, symbol));
                        }
                    }
                }
            } else if path.is_dir() {
                let file_name = path.file_name()?.to_str()?;

                if file_name.starts_with('.')
                    || file_name == "target"
                    || file_name == "node_modules"
                {
                    continue;
                }

                if let Some(result) = self.search_directory_for_symbol_bounded(
                    &path,
                    symbol_name,
                    depth + 1,
                    files_parsed,
                ) {
                    return Some(result);
                }
            }
        }

        None
    }
}
