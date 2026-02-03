// Test suite for LSP code action functionality
// Tests quick-fix suggestions, refactoring actions, and auto-imports

#[cfg(test)]
mod levenshtein_tests {
    // Test the Levenshtein distance function used for "did you mean" suggestions

    fn levenshtein_distance(a: &str, b: &str) -> u32 {
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

    #[test]
    fn test_identical_strings() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_single_insertion() {
        assert_eq!(levenshtein_distance("helo", "hello"), 1);
    }

    #[test]
    fn test_single_deletion() {
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
    }

    #[test]
    fn test_single_substitution() {
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
    }

    #[test]
    fn test_completely_different() {
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
    }

    #[test]
    fn test_empty_strings() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "xyz"), 3);
    }

    #[test]
    fn test_typo_scenarios() {
        // Common typos
        assert_eq!(levenshtein_distance("prinnt", "print"), 1);
        assert_eq!(levenshtein_distance("pirnt", "print"), 2);
        assert_eq!(levenshtein_distance("prnt", "print"), 1);
    }
}

#[cfg(test)]
mod symbol_extraction_tests {
    fn extract_symbol_from_diagnostic(message: &str) -> String {
        // Try to find quoted symbol
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

    #[test]
    fn test_single_quoted_symbol() {
        let msg = "undeclared identifier 'foo'";
        assert_eq!(extract_symbol_from_diagnostic(msg), "foo");
    }

    #[test]
    fn test_double_quoted_symbol() {
        let msg = "variable \"bar\" not found";
        assert_eq!(extract_symbol_from_diagnostic(msg), "bar");
    }

    #[test]
    fn test_backtick_symbol() {
        let msg = "undefined function `baz`";
        assert_eq!(extract_symbol_from_diagnostic(msg), "baz");
    }

    #[test]
    fn test_unquoted_identifier() {
        let msg = "identifier my_var is not declared";
        assert_eq!(extract_symbol_from_diagnostic(msg), "my_var");
    }

    #[test]
    fn test_no_symbol() {
        let msg = "generic error message";
        assert_eq!(extract_symbol_from_diagnostic(msg), "");
    }
}

#[cfg(test)]
mod import_position_tests {
    use lsp_types::Position;

    fn find_import_insert_position(content: &str) -> Position {
        let lines: Vec<&str> = content.lines().collect();
        let mut last_import_line = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('{') && trimmed.contains("} =") && trimmed.contains('@') {
                last_import_line = i + 1;
            }
            if trimmed.starts_with("//") && last_import_line == 0 {
                last_import_line = i + 1;
            }
        }

        Position {
            line: last_import_line as u32,
            character: 0,
        }
    }

    #[test]
    fn test_no_imports() {
        let content = "main = () void { }";
        let pos = find_import_insert_position(content);
        assert_eq!(pos.line, 0);
    }

    #[test]
    fn test_after_single_import() {
        let content = "{ io } = @std\n\nmain = () void { }";
        let pos = find_import_insert_position(content);
        assert_eq!(pos.line, 1);
    }

    #[test]
    fn test_after_multiple_imports() {
        let content = "{ io } = @std\n{ HashMap } = @std.collections\n\nmain = () void { }";
        let pos = find_import_insert_position(content);
        assert_eq!(pos.line, 2);
    }

    #[test]
    fn test_after_comments() {
        // Only the first contiguous block of comments is skipped
        let content = "// File description\n// Another comment\nmain = () void { }";
        let pos = find_import_insert_position(content);
        // After first comment line (logic only tracks last_import_line when it's 0)
        assert_eq!(pos.line, 1);
    }

    #[test]
    fn test_comments_and_imports() {
        let content = "// Header\n{ io } = @std\n\nmain = () void { }";
        let pos = find_import_insert_position(content);
        assert_eq!(pos.line, 2);
    }
}

#[cfg(test)]
mod variable_name_generation_tests {
    fn generate_variable_name(expression: &str) -> String {
        let expr_trimmed = expression.trim();

        // If it's a method call, use the method name
        if let Some(dot_pos) = expr_trimmed.rfind('.') {
            if let Some(method_end) = expr_trimmed[dot_pos + 1..].find('(') {
                let method_name = &expr_trimmed[dot_pos + 1..dot_pos + 1 + method_end];
                return format!("{}_result", method_name);
            }
        }

        // If it's a function call, use the function name
        if let Some(paren_pos) = expr_trimmed.find('(') {
            let func_name = expr_trimmed[..paren_pos].trim();
            if !func_name.is_empty() && func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return format!("{}_result", func_name);
            }
        }

        // If it's a binary operation
        for op in ["==", "!=", "<=", ">=", "<", ">", "+", "-", "*", "/", "%"] {
            if expr_trimmed.contains(op) {
                return "result".to_string();
            }
        }

        "extracted_value".to_string()
    }

    #[test]
    fn test_method_call() {
        assert_eq!(generate_variable_name("list.len()"), "len_result");
    }

    #[test]
    fn test_function_call() {
        assert_eq!(
            generate_variable_name("calculate_sum(a, b)"),
            "calculate_sum_result"
        );
    }

    #[test]
    fn test_binary_operation() {
        assert_eq!(generate_variable_name("a + b"), "result");
        assert_eq!(generate_variable_name("x == y"), "result");
    }

    #[test]
    fn test_simple_value() {
        assert_eq!(generate_variable_name("42"), "extracted_value");
    }
}
