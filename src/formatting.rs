//! Shared formatting utilities for Zen language
//! Used by both the LSP and zen-format CLI

use crate::lexer::{Lexer, Token};

/// Information about braces and pattern matching on a single line
#[derive(Debug, Default)]
pub struct LineTokenInfo {
    pub open_braces: usize,
    pub close_braces: usize,
    pub open_brackets: usize,
    pub close_brackets: usize,
    pub ends_with_question: bool,
    pub starts_with_pipe: bool,
    pub first_token_is_close_brace: bool,
    pub first_token_is_close_bracket: bool,
}

/// Analyze a single line using the lexer to get accurate token information.
/// This properly handles strings, comments, and other contexts where
/// braces/brackets/operators might appear but shouldn't be counted.
pub fn analyze_line_tokens(line: &str) -> LineTokenInfo {
    let mut info = LineTokenInfo::default();
    let mut lexer = Lexer::new(line);
    let mut is_first_token = true;
    let mut last_token: Option<Token> = None;

    loop {
        let token_with_span = lexer.next_token_with_span();
        let token = token_with_span.token.clone();

        if token == Token::Eof {
            break;
        }

        // Track first non-whitespace token
        if is_first_token {
            match &token {
                Token::Symbol('}') => info.first_token_is_close_brace = true,
                Token::Symbol(']') => info.first_token_is_close_bracket = true,
                Token::Pipe => info.starts_with_pipe = true,
                _ => {}
            }
            is_first_token = false;
        }

        // Count braces and brackets
        match &token {
            Token::Symbol('{') => info.open_braces += 1,
            Token::Symbol('}') => info.close_braces += 1,
            Token::Symbol('[') => info.open_brackets += 1,
            Token::Symbol(']') => info.close_brackets += 1,
            _ => {}
        }

        last_token = Some(token);
    }

    // Check if line ends with '?'
    if let Some(Token::Question) = last_token {
        info.ends_with_question = true;
    }

    info
}

/// Check if a line is a pattern match arm (starts with `|` token).
pub fn is_pattern_arm_line(line: &str) -> bool {
    analyze_line_tokens(line.trim()).starts_with_pipe
}

/// Format enum variants to be on separate lines with proper indentation
/// Handles both inline enums and multi-line enums missing indentation.
pub fn format_enum_variants(content: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let leading_whitespace = &line[..line.len() - line.trim_start().len()];

        // Check for enum definition patterns
        if let Some(colon_pos) = trimmed.find(':') {
            let before_colon = trimmed[..colon_pos].trim();
            let after_colon = trimmed[colon_pos + 1..].trim();

            // Skip if not a valid enum name or contains assignment/struct markers
            // Use is_valid_enum_name to properly handle generic types like Option<T>
            if is_valid_enum_name(before_colon)
                && !trimmed.contains('=')
                && !trimmed.contains('{')
                && !after_colon.starts_with(':')
            {
                // Case 1: Inline enum (Name: Variant1, Variant2, ...)
                if after_colon.contains(',') {
                    let variants: Vec<&str> = after_colon
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|v| !v.is_empty())
                        .collect();

                    let all_valid = variants.iter().all(|v| is_valid_variant(v));

                    if all_valid && variants.len() > 1 {
                        result.push_str(leading_whitespace);
                        result.push_str(before_colon);
                        result.push_str(":\n");

                        for (idx, variant) in variants.iter().enumerate() {
                            result.push_str(leading_whitespace);
                            result.push_str("    ");
                            result.push_str(variant);
                            if idx < variants.len() - 1 {
                                result.push(',');
                            }
                            result.push('\n');
                        }
                        i += 1;
                        continue;
                    }
                }

                // Case 2: Multi-line enum (Name: on its own, variants on following lines)
                if after_colon.is_empty() {
                    let mut variants = Vec::new();
                    let mut j = i + 1;

                    while j < lines.len() {
                        let next_line = lines[j];
                        let next_trimmed = next_line.trim();

                        if next_trimmed.is_empty() || next_trimmed.starts_with("//") {
                            break;
                        }

                        let variant_name = next_trimmed.trim_end_matches(',');
                        if is_valid_variant(variant_name) {
                            variants.push(variant_name);
                            j += 1;
                        } else {
                            break;
                        }
                    }

                    if !variants.is_empty() {
                        result.push_str(leading_whitespace);
                        result.push_str(before_colon);
                        result.push_str(":\n");

                        for (idx, variant) in variants.iter().enumerate() {
                            result.push_str(leading_whitespace);
                            result.push_str("    ");
                            result.push_str(variant);
                            if idx < variants.len() - 1 {
                                result.push(',');
                            }
                            result.push('\n');
                        }

                        i = j;
                        continue;
                    }
                }
            }
        }

        result.push_str(line);
        result.push('\n');
        i += 1;
    }

    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Check if a string is a valid Zen identifier
pub fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Check if a string is a valid enum name (identifier optionally followed by generic params)
/// Uses the Parser to properly parse `Name` or `Name<T>` or `Name<T, E>` patterns
pub fn is_valid_enum_name(s: &str) -> bool {
    use crate::parser::Parser;

    if s.is_empty() {
        return false;
    }

    // Quick check for unclosed generics
    if s.contains('<') && !s.contains('>') {
        return false;
    }

    let lexer = Lexer::new(s);
    let mut parser = Parser::new(lexer);

    // Use the parser's existing method to parse type name with generics
    match parser.parse_type_name_with_generics() {
        Ok(_) => {
            // After parsing, we should be at EOF (consumed entire string)
            parser.current_token == Token::Eof
        }
        Err(_) => false,
    }
}

/// Check if a string looks like a valid enum variant
/// Can be: `VariantName` or `VariantName(Type)` or `VariantName: Type`
pub fn is_valid_variant(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    // Variant with type annotation: `Ok: i32` or `Err: StaticString`
    if let Some(colon_pos) = s.find(':') {
        let name = s[..colon_pos].trim();
        let type_part = s[colon_pos + 1..].trim();
        return is_valid_identifier(name) && !type_part.is_empty();
    }

    // Variant with associated data in parens: `Some(i32)`
    if let Some(paren_pos) = s.find('(') {
        let name = &s[..paren_pos];
        return is_valid_identifier(name) && s.ends_with(')');
    }

    // Simple variant name: `Active`
    is_valid_identifier(s)
}

/// Remove trailing whitespace from all lines
pub fn remove_trailing_whitespace(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Normalize variable declaration syntax
/// - `i: i32 ::= 0` → `i :: i32 = 0` (mutable with explicit type)
/// - `i: i32 = 0` stays as `i: i32 = 0` (immutable with explicit type)
/// - `i ::= 0` stays as `i ::= 0` (mutable with inferred type)
/// - `i = 0` stays as `i = 0` (immutable with inferred type)
pub fn normalize_variable_declarations(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            let leading_whitespace = &line[..line.len() - line.trim_start().len()];

            // Check for the problematic pattern: `name: Type ::= value`
            // This should become `name :: Type = value`
            if let Some(double_colon_eq) = trimmed.find("::=") {
                let before_dce = &trimmed[..double_colon_eq];
                let after_dce = &trimmed[double_colon_eq + 3..];

                // Check if there's a single colon before ::=
                if let Some(colon_pos) = before_dce.find(':') {
                    // Make sure it's not :: (double colon)
                    let after_first_colon = &before_dce[colon_pos..];
                    if !after_first_colon.starts_with("::") {
                        let name = before_dce[..colon_pos].trim();
                        let type_part = before_dce[colon_pos + 1..].trim();

                        // Only transform if we have a valid name and type
                        if !name.is_empty() && !type_part.is_empty() && is_valid_identifier(name) {
                            // Transform: `name: Type ::= value` → `name :: Type = value`
                            return format!(
                                "{}{} :: {} ={}",
                                leading_whitespace, name, type_part, after_dce
                            );
                        }
                    }
                }
            }

            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert tabs to 4 spaces
pub fn fix_indentation(content: &str) -> String {
    content.replace('\t', "    ")
}

/// Format based on braces and pattern matching.
/// Uses the lexer to correctly handle braces/brackets inside strings and comments.
pub fn format_braces(content: &str) -> String {
    let mut formatted = String::new();
    let mut indent_level: usize = 0;
    let mut pattern_match_stack: Vec<usize> = Vec::new(); // stack of indent levels
    let indent_str = "    "; // 4 spaces

    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            formatted.push('\n');
            continue;
        }

        // Analyze the line using the lexer for accurate token detection
        let token_info = analyze_line_tokens(trimmed);
        let is_closing_brace =
            token_info.first_token_is_close_brace || token_info.first_token_is_close_bracket;
        let is_pattern_arm = token_info.starts_with_pipe;

        // Exit pattern matches before handling closing braces
        // We need to check this BEFORE decrementing for closing braces
        while let Some(&pm_indent) = pattern_match_stack.last() {
            // Exit pattern match if:
            // 1. Current line is not a pattern arm
            // 2. We're at the pattern match's indent level + 1 (inside the match)
            // 3. No more arms are coming
            let at_pattern_indent = indent_level == pm_indent + 1;

            if !is_pattern_arm
                && (at_pattern_indent || (is_closing_brace && indent_level == pm_indent + 2))
            {
                // Check if there are more arms coming (using lexer-based check)
                let more_arms = lines
                    .iter()
                    .skip(i + 1)
                    .find(|l| !l.trim().is_empty())
                    .map(|l| is_pattern_arm_line(l))
                    .unwrap_or(false);

                if !more_arms {
                    pattern_match_stack.pop();
                    indent_level = indent_level.saturating_sub(1);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Decrease indent for lines starting with closing brace (after pattern match handling)
        if is_closing_brace {
            indent_level = indent_level.saturating_sub(1);
        }

        // Add indentation
        for _ in 0..indent_level {
            formatted.push_str(indent_str);
        }

        formatted.push_str(trimmed);
        formatted.push('\n');

        // Count braces for indent change using lexer-based counts
        let opens = token_info.open_braces + token_info.open_brackets;
        let mut closes = token_info.close_braces + token_info.close_brackets;

        // Don't double-count leading close brace
        if token_info.first_token_is_close_brace {
            closes = closes.saturating_sub(1);
        }
        if token_info.first_token_is_close_bracket {
            closes = closes.saturating_sub(1);
        }

        // Update indent
        if opens > closes {
            indent_level += opens - closes;
        } else if closes > opens {
            indent_level = indent_level.saturating_sub(closes - opens);
        }

        // Start pattern match (using lexer-based check)
        if token_info.ends_with_question {
            let has_arms = lines
                .iter()
                .skip(i + 1)
                .find(|l| !l.trim().is_empty())
                .map(|l| is_pattern_arm_line(l))
                .unwrap_or(false);

            if has_arms {
                pattern_match_stack.push(indent_level);
                indent_level += 1;
            }
        }
    }

    formatted
}

#[cfg(test)]
mod tests {
    use crate::formatting::{
        analyze_line_tokens, format_braces, format_enum_variants, is_pattern_arm_line,
        is_valid_enum_name,
    };

    #[test]
    fn test_analyze_line_braces_in_string() {
        // Braces inside strings should NOT be counted
        let info = analyze_line_tokens(r#"msg = "{ braces }""#);
        assert_eq!(
            info.open_braces, 0,
            "braces in string should not be counted"
        );
        assert_eq!(info.close_braces, 0);
    }

    #[test]
    fn test_analyze_line_real_braces() {
        // Real braces should be counted
        let info = analyze_line_tokens("foo = () {");
        assert_eq!(info.open_braces, 1);
        assert_eq!(info.close_braces, 0);

        let info = analyze_line_tokens("}");
        assert_eq!(info.open_braces, 0);
        assert_eq!(info.close_braces, 1);
        assert!(info.first_token_is_close_brace);
    }

    #[test]
    fn test_analyze_line_question_in_string() {
        // Question mark in string should NOT trigger pattern match
        let info = analyze_line_tokens(r#"msg = "What?""#);
        assert!(
            !info.ends_with_question,
            "? in string should not trigger pattern match"
        );
    }

    #[test]
    fn test_analyze_line_real_question() {
        // Real question mark should trigger pattern match
        let info = analyze_line_tokens("result?");
        assert!(
            info.ends_with_question,
            "real ? should trigger pattern match"
        );

        let info = analyze_line_tokens("foo.bar?");
        assert!(info.ends_with_question);
    }

    #[test]
    fn test_analyze_line_pipe_in_string() {
        // Pipe in string should NOT be detected as pattern arm
        let info = analyze_line_tokens(r#"regex = "a|b|c""#);
        assert!(
            !info.starts_with_pipe,
            "| in string should not be pattern arm"
        );
    }

    #[test]
    fn test_analyze_line_real_pipe() {
        // Real pipe at start should be detected as pattern arm
        let info = analyze_line_tokens("| Ok(val) { return val }");
        assert!(info.starts_with_pipe, "real | should be pattern arm");

        let info = analyze_line_tokens("| true { }");
        assert!(info.starts_with_pipe);
    }

    #[test]
    fn test_is_pattern_arm_line() {
        assert!(is_pattern_arm_line("| Ok(x) { }"));
        assert!(is_pattern_arm_line("  | true { }"));
        assert!(!is_pattern_arm_line(r#"regex = "a|b""#));
        assert!(!is_pattern_arm_line("foo | bar")); // bitwise or in middle, not at start
    }

    #[test]
    fn test_format_braces_with_strings() {
        let input = r#"test = () {
msg = "{ braces }"
return msg
}"#;
        let formatted = format_braces(input);
        // Should properly indent despite braces in string
        assert!(formatted.contains("    msg = \"{ braces }\""));
        assert!(formatted.contains("    return msg"));
    }

    #[test]
    fn test_format_braces_pattern_match() {
        let input = r#"test = () {
result?
| Ok(x) { return x }
| Err(e) { return 0 }
}"#;
        let formatted = format_braces(input);
        // Pattern arms should be indented
        assert!(formatted.contains("    result?"));
        assert!(formatted.contains("        | Ok(x)"));
        assert!(formatted.contains("        | Err(e)"));
    }

    #[test]
    fn test_format_braces_question_in_string() {
        // Question mark in string should NOT trigger pattern match indentation
        let input = r#"test = () {
msg = "What?"
return msg
}"#;
        let formatted = format_braces(input);
        let lines: Vec<&str> = formatted.lines().collect();
        // Line 0: test = () {
        // Line 1:     msg = "What?"
        // Line 2:     return msg
        // Line 3: }
        assert!(
            lines[1].starts_with("    msg"),
            "line 1 should be indented msg"
        );
        assert!(
            lines[2].starts_with("    return"),
            "line 2 should be indented return"
        );
    }

    #[test]
    fn test_is_valid_enum_name_simple() {
        assert!(is_valid_enum_name("Option"));
        assert!(is_valid_enum_name("Result"));
        assert!(is_valid_enum_name("MyEnum"));
        assert!(!is_valid_enum_name(""));
        assert!(!is_valid_enum_name("123"));
    }

    #[test]
    fn test_is_valid_enum_name_generic() {
        assert!(is_valid_enum_name("Option<T>"));
        assert!(is_valid_enum_name("Result<T, E>"));
        assert!(is_valid_enum_name("Map<K, V>"));
        assert!(!is_valid_enum_name("Option<")); // unclosed
        assert!(!is_valid_enum_name("Option<T> extra")); // extra content
    }

    #[test]
    fn test_format_enum_variants_generic() {
        // Generic enum with variants not indented should be fixed
        let input = "Option<T>:\nSome: T,\nNone";
        let formatted = format_enum_variants(input);
        let lines: Vec<&str> = formatted.lines().collect();
        assert_eq!(lines[0], "Option<T>:");
        assert_eq!(lines[1], "    Some: T,");
        assert_eq!(lines[2], "    None");
    }

    #[test]
    fn test_format_enum_variants_generic_result() {
        let input = "Result<T, E>:\nOk: T,\nErr: E";
        let formatted = format_enum_variants(input);
        let lines: Vec<&str> = formatted.lines().collect();
        assert_eq!(lines[0], "Result<T, E>:");
        assert_eq!(lines[1], "    Ok: T,");
        assert_eq!(lines[2], "    Err: E");
    }

    #[test]
    fn test_format_enum_variants_simple() {
        // Non-generic enum should still work
        let input = "Status:\nActive,\nInactive";
        let formatted = format_enum_variants(input);
        let lines: Vec<&str> = formatted.lines().collect();
        assert_eq!(lines[0], "Status:");
        assert_eq!(lines[1], "    Active,");
        assert_eq!(lines[2], "    Inactive");
    }
}
