// Shared utility functions for navigation module

use lsp_types::*;

/// Find the symbol (identifier) at the given position in the document
pub fn find_symbol_at_position(content: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;

    // Find word boundaries around the cursor position
    let chars: Vec<char> = line.chars().collect();

    // Check if position is valid
    if chars.is_empty() || char_pos > chars.len() {
        return None;
    }

    let mut start = char_pos.min(chars.len());
    let mut end = char_pos.min(chars.len());

    // Move start backwards to find word beginning (allow @ for @std and . for module paths)
    while start > 0 && start <= chars.len() {
        let ch = chars[start - 1];
        if ch.is_alphanumeric() || ch == '_' || ch == '.' || (start == 1 && ch == '@') {
            start -= 1;
            // If we hit @, stop (don't go before it)
            if start == 0 && chars.first() == Some(&'@') {
                break;
            }
        } else {
            break;
        }
    }

    // Move end forward to find word end (allow . for module paths like @std.text)
    while end < chars.len() {
        let ch = chars[end];
        if ch.is_alphanumeric() || ch == '_' || ch == '.' {
            end += 1;
        } else {
            break;
        }
    }

    if start < end {
        let symbol: String = chars[start..end].iter().collect();
        // Don't return empty or just punctuation
        if !symbol.is_empty() && symbol.chars().any(|c| c.is_alphanumeric()) {
            Some(symbol)
        } else {
            None
        }
    } else {
        None
    }
}

/// Check if a byte position is a word boundary character
#[inline]
pub fn is_word_boundary_char(line: &str, byte_pos: usize) -> bool {
    if byte_pos >= line.len() {
        return true;
    }
    // For ASCII characters (most common case), use direct byte access
    if let Some(&b) = line.as_bytes().get(byte_pos) {
        // Only valid for ASCII (0-127)
        if b < 128 {
            let ch = b as char;
            return !ch.is_alphanumeric() && ch != '_';
        }
    }
    // For non-ASCII, need to find the char at this byte position
    let mut char_count = 0;
    for (idx, _) in line.char_indices() {
        if idx >= byte_pos {
            break;
        }
        char_count += 1;
    }
    line.chars()
        .nth(char_count)
        .map(|c| !c.is_alphanumeric() && c != '_')
        .unwrap_or(true)
}

/// Find a word in a line, respecting word boundaries
pub fn find_word_in_line(line: &str, word: &str) -> Option<usize> {
    let mut search_pos = 0;
    while let Some(pos) = line[search_pos..].find(word) {
        let actual_pos = search_pos + pos;

        // Check word boundaries
        let before_ok =
            actual_pos == 0 || is_word_boundary_char(line, actual_pos.saturating_sub(1));
        let end_pos = actual_pos + word.len();
        let after_ok = end_pos >= line.len() || is_word_boundary_char(line, end_pos);

        if before_ok && after_ok {
            return Some(actual_pos);
        }

        search_pos = actual_pos + 1;
    }
    None
}

/// Check if a position in a line is inside a string or comment
pub fn is_in_string_or_comment(line: &str, col: usize) -> bool {
    let mut in_string = false;
    let mut in_comment = false;
    let mut prev_char = ' ';

    for (i, ch) in line.chars().enumerate() {
        if i >= col {
            break;
        }

        if in_comment {
            continue;
        }

        if ch == '"' && prev_char != '\\' {
            in_string = !in_string;
        } else if !in_string && ch == '/' && prev_char == '/' {
            in_comment = true;
        }

        prev_char = ch;
    }

    in_string || in_comment
}

/// Find function range - prefers symbol info, falls back to content parsing
pub fn find_function_range_from_doc(
    doc: &super::super::types::Document,
    func_name: &str,
) -> Option<Range> {
    // Try to get range from symbol info first (most accurate)
    if let Some(symbol) = doc.symbols.get(func_name) {
        if matches!(
            symbol.kind,
            lsp_types::SymbolKind::FUNCTION | lsp_types::SymbolKind::METHOD
        ) {
            return Some(symbol.range);
        }
    }
    // Fallback to content parsing
    find_function_range(&doc.content, func_name)
}

/// Find the range of a function in the document content
pub fn find_function_range(content: &str, func_name: &str) -> Option<Range> {
    let lines: Vec<&str> = content.lines().collect();
    let mut start_line = None;

    for (line_num, line) in lines.iter().enumerate() {
        if line.contains(&format!("{} =", func_name)) {
            start_line = Some(line_num);
            break;
        }
    }

    if let Some(start) = start_line {
        let mut brace_depth = 0;
        let mut found_opening = false;

        for (line_num, line) in lines.iter().enumerate().skip(start) {
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth += 1;
                    found_opening = true;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if found_opening && brace_depth == 0 {
                        return Some(Range {
                            start: Position {
                                line: start as u32,
                                character: 0,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: line.len() as u32,
                            },
                        });
                    }
                }
            }
        }
    }
    None
}

/// Find symbol definition in content using text search
pub fn find_symbol_definition_in_content(content: &str, symbol_name: &str) -> Option<Range> {
    let lines: Vec<&str> = content.lines().collect();

    // First pass: look for actual definitions (function, variable, etc.)
    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Look for symbol at word boundaries
        if let Some(pos) = find_word_in_line(line, symbol_name) {
            let is_word_boundary_start =
                pos == 0 || is_word_boundary_char(line, pos.saturating_sub(1));
            let end_pos = pos + symbol_name.len();
            let is_word_boundary_end =
                end_pos >= line.len() || is_word_boundary_char(line, end_pos);

            if is_word_boundary_start && is_word_boundary_end {
                // Check if this looks like a definition
                let after_symbol = &line[pos + symbol_name.len()..].trim();
                let is_definition = after_symbol.starts_with('=')
                    || after_symbol.starts_with(':')
                    || after_symbol.starts_with('(')
                    || (line[..pos].trim().is_empty()
                        && (after_symbol.starts_with('=') || after_symbol.starts_with(':')));

                if is_definition {
                    return Some(Range {
                        start: Position {
                            line: line_idx as u32,
                            character: pos as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end_pos as u32,
                        },
                    });
                }
            }
        }
    }

    None
}
