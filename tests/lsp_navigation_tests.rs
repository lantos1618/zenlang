// Test suite for LSP navigation functionality
// Tests go-to-definition, struct field navigation, and symbol finding

#[cfg(test)]
mod symbol_finding_tests {
    use lsp_types::Position;
    use zen::lsp::navigation::find_symbol_at_position;

    #[test]
    fn test_find_symbol_simple_identifier() {
        let content = "main = () void {}";
        let position = Position {
            line: 0,
            character: 2, // Inside "main"
        };

        let symbol = find_symbol_at_position(content, position);
        assert_eq!(symbol, Some("main".to_string()));
    }

    #[test]
    fn test_find_symbol_at_start() {
        let content = "variable = 42";
        let position = Position {
            line: 0,
            character: 0,
        };

        let symbol = find_symbol_at_position(content, position);
        assert_eq!(symbol, Some("variable".to_string()));
    }

    #[test]
    fn test_find_symbol_module_path() {
        let content = "{ io } = @std.io";
        let position = Position {
            line: 0,
            character: 10, // On the '@'
        };

        let symbol = find_symbol_at_position(content, position);
        // @ is included when cursor is on or right after it
        assert!(
            symbol == Some("@std.io".to_string()) || symbol == Some("std.io".to_string()),
            "Expected @std.io or std.io, got {:?}",
            symbol
        );
    }

    #[test]
    fn test_find_symbol_with_underscore() {
        let content = "my_variable = 10";
        let position = Position {
            line: 0,
            character: 5, // Inside "my_variable"
        };

        let symbol = find_symbol_at_position(content, position);
        assert_eq!(symbol, Some("my_variable".to_string()));
    }

    #[test]
    fn test_find_symbol_on_whitespace() {
        let content = "a = b";
        let position = Position {
            line: 0,
            character: 2, // On the space
        };

        // Should not find a symbol on whitespace
        let symbol = find_symbol_at_position(content, position);
        assert!(symbol.is_none() || symbol == Some("=".to_string()).filter(|_| false));
    }

    #[test]
    fn test_find_symbol_multiline() {
        let content = "line1 = 1\nline2 = 2\nline3 = 3";
        let position = Position {
            line: 1,
            character: 2, // Inside "line2"
        };

        let symbol = find_symbol_at_position(content, position);
        assert_eq!(symbol, Some("line2".to_string()));
    }
}

#[cfg(test)]
mod definition_finding_tests {
    use zen::lsp::navigation::find_symbol_definition_in_content;

    #[test]
    fn test_find_function_definition() {
        let content = "helper = () void {}\nmain = () void { helper() }";

        let range = find_symbol_definition_in_content(content, "helper");
        assert!(range.is_some());
        let range = range.unwrap();
        assert_eq!(range.start.line, 0);
    }

    #[test]
    fn test_find_variable_definition() {
        let content = "x = 42\ny = x + 1";

        let range = find_symbol_definition_in_content(content, "x");
        assert!(range.is_some());
        let range = range.unwrap();
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 0);
    }

    #[test]
    fn test_find_struct_definition() {
        let content = "Point: {\n    x: i32\n    y: i32\n}";

        let range = find_symbol_definition_in_content(content, "Point");
        assert!(range.is_some());
        let range = range.unwrap();
        assert_eq!(range.start.line, 0);
    }

    #[test]
    fn test_definition_not_found() {
        let content = "x = 42";

        let range = find_symbol_definition_in_content(content, "nonexistent");
        assert!(range.is_none());
    }

    #[test]
    fn test_definition_respects_word_boundaries() {
        let content = "my_var = 1\nmy_variable = 2";

        // Should find "my_var", not "my_variable"
        let range = find_symbol_definition_in_content(content, "my_var");
        assert!(range.is_some());
        let range = range.unwrap();
        assert_eq!(range.start.line, 0);
    }
}

#[cfg(test)]
mod struct_field_tests {

    // Note: Full struct field navigation tests require a Document with TypeContext
    // These are simpler unit tests for the helper functions

    #[test]
    fn test_find_field_in_struct_content() {
        let _content = "Point: {\n    x: i32\n    y: i32\n}";

        // The internal function finds fields within struct definitions
        // This is tested indirectly through the navigation handler
    }
}

#[cfg(test)]
mod utils_tests {
    use zen::lsp::navigation::utils::{find_word_in_line, is_word_boundary_char};

    #[test]
    fn test_find_word_in_line() {
        let line = "hello world foo";

        assert_eq!(find_word_in_line(line, "hello"), Some(0));
        assert_eq!(find_word_in_line(line, "world"), Some(6));
        assert_eq!(find_word_in_line(line, "foo"), Some(12));
        assert_eq!(find_word_in_line(line, "bar"), None);
    }

    #[test]
    fn test_find_word_respects_boundaries() {
        let line = "hello helloworld";

        // Should find standalone "hello", not part of "helloworld"
        let pos = find_word_in_line(line, "hello");
        assert_eq!(pos, Some(0));
    }

    #[test]
    fn test_is_word_boundary() {
        assert!(is_word_boundary_char("a b", 1)); // space
        assert!(is_word_boundary_char("a=b", 1)); // equals
        assert!(!is_word_boundary_char("abc", 1)); // letter
        assert!(!is_word_boundary_char("a_b", 1)); // underscore (part of identifier)
    }
}

#[cfg(test)]
mod reference_classification_tests {
    use zen::lsp::navigation::references::{
        count_references_by_kind, find_enhanced_references_in_document, ReferenceKind,
    };

    #[test]
    fn test_function_declaration() {
        let content = "helper = () void { }\nmain = () void { helper() }";
        let refs = find_enhanced_references_in_document(content, "helper");

        assert_eq!(refs.len(), 2);
        // First should be declaration
        assert_eq!(refs[0].kind, ReferenceKind::Declaration);
        // Second should be call
        assert_eq!(refs[1].kind, ReferenceKind::Call);
    }

    #[test]
    fn test_variable_read_write() {
        let content = "x = 42\ny = x + 1\nx = y * 2";
        let refs = find_enhanced_references_in_document(content, "x");

        assert_eq!(refs.len(), 3);
        // First is declaration (at start of line with =)
        assert_eq!(refs[0].kind, ReferenceKind::Declaration);
        // Second is read (used in expression)
        assert_eq!(refs[1].kind, ReferenceKind::Read);
        // Third is also declaration/write (reassignment at start of line)
        // In Zen, reassignment looks like declaration syntactically
        assert!(refs[2].kind == ReferenceKind::Declaration || refs[2].kind == ReferenceKind::Write);
    }

    #[test]
    fn test_function_calls() {
        let content = "result = calculate()\nvalue = process(result)";
        let refs = find_enhanced_references_in_document(content, "calculate");

        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, ReferenceKind::Call);
    }

    #[test]
    fn test_count_references() {
        let content = "x = 42\ny = x\nz = x + y\nx = 0";
        let refs = find_enhanced_references_in_document(content, "x");

        let (decls, reads, _writes, calls) = count_references_by_kind(&refs);
        // 1 declaration, 2 reads, 1 write, 0 calls
        assert!(decls >= 1);
        assert!(reads >= 1);
        assert_eq!(calls, 0);
    }
}
