// Utility functions for code actions

/// Convert a UTF-16 character offset to a byte offset in a UTF-8 string.
/// LSP positions use UTF-16 offsets, but Rust strings are UTF-8.
pub fn utf16_offset_to_byte_offset(line: &str, utf16_offset: usize) -> usize {
    let mut utf16_count = 0;
    for (byte_idx, ch) in line.char_indices() {
        if utf16_count >= utf16_offset {
            return byte_idx;
        }
        utf16_count += ch.len_utf16();
    }
    // If we've consumed all characters, return byte length (clamp to end)
    line.len()
}

/// Extract the error code string from a diagnostic's code field.
/// Returns an empty string if no code is set.
pub fn diagnostic_code(diagnostic: &lsp_types::Diagnostic) -> String {
    match &diagnostic.code {
        Some(lsp_types::NumberOrString::String(s)) => s.clone(),
        Some(lsp_types::NumberOrString::Number(n)) => n.to_string(),
        None => String::new(),
    }
}

/// Extract the symbol name from a diagnostic message like "undeclared identifier 'foo'"
pub fn extract_symbol_from_diagnostic(message: &str) -> String {
    // Try to find quoted symbol: 'symbol' or "symbol" or `symbol`
    for delim in ['\'', '"', '`'] {
        if let Some(start) = message.find(delim) {
            if let Some(end) = message[start + 1..].find(delim) {
                return message[start + 1..start + 1 + end].to_string();
            }
        }
    }

    // Try to find symbol after common patterns
    for pattern in ["identifier ", "variable ", "function ", "type ", "symbol "] {
        if let Some(pos) = message.find(pattern) {
            let after_pattern = &message[pos + pattern.len()..];
            let symbol: String = after_pattern
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !symbol.is_empty() {
                return symbol;
            }
        }
    }

    String::new()
}

/// Simple Levenshtein distance calculation
pub fn levenshtein_distance(a: &str, b: &str) -> u32 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n as u32;
    }
    if n == 0 {
        return m as u32;
    }

    let mut dp = vec![vec![0u32; n + 1]; m + 1];

    #[allow(clippy::needless_range_loop)]
    for i in 0..=m {
        dp[i][0] = i as u32;
    }
    for j in 0..=n {
        dp[0][j] = j as u32;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_symbol_single_quotes() {
        assert_eq!(
            extract_symbol_from_diagnostic("undeclared identifier 'foo'"),
            "foo"
        );
    }

    #[test]
    fn test_extract_symbol_double_quotes() {
        assert_eq!(
            extract_symbol_from_diagnostic("undefined variable \"bar\""),
            "bar"
        );
    }

    #[test]
    fn test_extract_symbol_pattern() {
        assert_eq!(
            extract_symbol_from_diagnostic("identifier baz not found"),
            "baz"
        );
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", "test"), 4);
        assert_eq!(levenshtein_distance("test", ""), 4);
    }
}
