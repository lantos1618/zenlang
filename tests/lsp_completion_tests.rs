// Test suite for LSP completion functionality
// Tests struct field completions, context detection, and completion ranking

#[cfg(test)]
mod completion_context_tests {
    use lsp_types::Position;
    use zen::lsp::completion::get_completion_context;
    use zen::lsp::types::ZenCompletionContext;

    // Helper to create a mock DocumentStore for testing
    fn create_test_store() -> zen::lsp::document_store::DocumentStore {
        zen::lsp::document_store::DocumentStore::new()
    }

    #[test]
    fn test_struct_literal_context_detected() {
        let content = "point = Point { ";
        let position = Position {
            line: 0,
            character: 16,
        };
        let _store = create_test_store();

        let context = get_completion_context(content, position, None);
        assert!(matches!(
            context,
            Some(ZenCompletionContext::StructLiteral { struct_name }) if struct_name == "Point"
        ));
    }

    #[test]
    fn test_struct_literal_after_comma() {
        let content = "point = Point { x: 1, ";
        let position = Position {
            line: 0,
            character: 22,
        };
        let _store = create_test_store();

        let context = get_completion_context(content, position, None);
        assert!(matches!(
            context,
            Some(ZenCompletionContext::StructLiteral { struct_name }) if struct_name == "Point"
        ));
    }

    #[test]
    fn test_not_struct_literal_in_value_position() {
        // After colon, we're in value position - not field name position
        let content = "point = Point { x: ";
        let position = Position {
            line: 0,
            character: 19,
        };
        let _store = create_test_store();

        let context = get_completion_context(content, position, None);
        // Should be General, not StructLiteral (we're typing a value, not a field name)
        assert!(matches!(context, Some(ZenCompletionContext::General)));
    }

    #[test]
    fn test_ufc_method_context_after_dot() {
        let content = "result = my_string.";
        let position = Position {
            line: 0,
            character: 19,
        };
        let _store = create_test_store();

        let context = get_completion_context(content, position, None);
        assert!(matches!(
            context,
            Some(ZenCompletionContext::UfcMethod { .. })
        ));
    }

    #[test]
    fn test_module_path_context() {
        let content = "{ io } = @std.";
        let position = Position {
            line: 0,
            character: 14,
        };
        let _store = create_test_store();

        let context = get_completion_context(content, position, None);
        assert!(matches!(
            context,
            Some(ZenCompletionContext::ModulePath { base }) if base == "@std"
        ));
    }

    #[test]
    fn test_general_context_at_line_start() {
        let content = "main = () void {\n    ";
        let position = Position {
            line: 1,
            character: 4,
        };
        let _store = create_test_store();

        let context = get_completion_context(content, position, None);
        assert!(matches!(context, Some(ZenCompletionContext::General)));
    }
}

#[cfg(test)]
mod struct_literal_completion_tests {
    use lsp_types::Position;
    use zen::lsp::completion::context::get_assigned_fields;

    #[test]
    fn test_get_assigned_fields_single() {
        let content = "point = Point { x: 1, ";
        let position = Position {
            line: 0,
            character: 22,
        };

        let assigned = get_assigned_fields(content, position);
        assert!(assigned.contains("x"));
        assert!(!assigned.contains("y"));
    }

    #[test]
    fn test_get_assigned_fields_multiple() {
        let content = "point = Point { x: 1, y: 2, ";
        let position = Position {
            line: 0,
            character: 28,
        };

        let assigned = get_assigned_fields(content, position);
        assert!(assigned.contains("x"));
        assert!(assigned.contains("y"));
        assert_eq!(assigned.len(), 2);
    }

    #[test]
    fn test_get_assigned_fields_none() {
        let content = "point = Point { ";
        let position = Position {
            line: 0,
            character: 16,
        };

        let assigned = get_assigned_fields(content, position);
        assert!(assigned.is_empty());
    }
}

#[cfg(test)]
mod pattern_match_completion_tests {
    use lsp_types::Position;
    use zen::lsp::completion::get_completion_context;
    use zen::lsp::types::ZenCompletionContext;

    fn create_test_store() -> zen::lsp::document_store::DocumentStore {
        zen::lsp::document_store::DocumentStore::new()
    }

    #[test]
    fn test_pattern_match_context_after_pipe() {
        let content = "result = value ? |";
        let position = Position {
            line: 0,
            character: 18,
        };
        let _store = create_test_store();

        let context = get_completion_context(content, position, None);
        assert!(matches!(
            context,
            Some(ZenCompletionContext::PatternMatch { matched_type }) if matched_type == "value"
        ));
    }

    #[test]
    fn test_pattern_match_context_with_space() {
        // After a space following `|`, we're still in pattern match context
        // because the user is still typing the pattern
        let content = "result = value ? | ";
        let position = Position {
            line: 0,
            character: 19,
        };
        let _store = create_test_store();

        let context = get_completion_context(content, position, None);
        // Still pattern match context - user is typing the pattern
        assert!(matches!(
            context,
            Some(ZenCompletionContext::PatternMatch { matched_type }) if matched_type == "value"
        ));
    }

    #[test]
    fn test_pattern_match_context_multiarm() {
        // After first arm, in second arm
        let content = "result = value ? | true { 1 } |";
        let position = Position {
            line: 0,
            character: 31,
        };
        let _store = create_test_store();

        let context = get_completion_context(content, position, None);
        assert!(matches!(
            context,
            Some(ZenCompletionContext::PatternMatch { .. })
        ));
    }

    #[test]
    fn test_not_pattern_match_in_general_context() {
        let content = "x = 42";
        let position = Position {
            line: 0,
            character: 6,
        };
        let _store = create_test_store();

        let context = get_completion_context(content, position, None);
        assert!(matches!(context, Some(ZenCompletionContext::General)));
    }
}

#[cfg(test)]
mod auto_import_tests {
    use zen::lsp::completion::auto_import::get_module_path_from_uri;

    #[test]
    fn test_get_module_path_from_uri() {
        use lsp_types::Url;

        let uri = Url::parse("file:///home/user/project/stdlib/io/io.zen").unwrap();
        let path = get_module_path_from_uri(&uri);
        assert_eq!(path, Some("@std.io.io".to_string()));
    }
}
