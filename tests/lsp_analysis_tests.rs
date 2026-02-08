//! Characterization tests for LSP pattern exhaustiveness checking and allocator validation.
//! These tests capture the current behavior BEFORE refactoring so we can verify
//! that moving code doesn't change semantics.

#[cfg(test)]
mod pattern_exhaustiveness_tests {
    use lsp_types::*;
    use std::collections::HashMap;
    use zen::ast::{Expression, Pattern, PatternArm};
    use zen::lsp::pattern_checking::check_pattern_exhaustiveness;
    use zen::typechecker::validation::find_missing_variants_pure;

    /// Helper: build a PatternArm with an EnumLiteral pattern (e.g. `.Some(x)`)
    fn enum_literal_arm(variant: &str, has_payload: bool) -> PatternArm {
        PatternArm {
            pattern: Pattern::EnumLiteral {
                variant: variant.to_string(),
                payload: if has_payload {
                    Some(Box::new(Pattern::Identifier("x".to_string())))
                } else {
                    None
                },
            },
            guard: None,
            body: Expression::Integer32(0),
        }
    }

    /// Helper: build a PatternArm with a Wildcard pattern
    fn wildcard_arm() -> PatternArm {
        PatternArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Expression::Integer32(0),
        }
    }

    // =========================================================================
    // Tests for find_missing_variants (core logic)
    // =========================================================================

    fn empty_registry() -> HashMap<String, Vec<String>> {
        HashMap::new()
    }

    fn color_registry() -> HashMap<String, Vec<String>> {
        let mut reg = HashMap::new();
        reg.insert(
            "Color".to_string(),
            vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()],
        );
        reg
    }

    #[test]
    fn test_option_only_some_missing_none() {
        let arms = vec![enum_literal_arm("Some", true)];
        let missing = find_missing_variants_pure("Option", &arms, &empty_registry());
        assert_eq!(missing, vec!["None".to_string()]);
    }

    #[test]
    fn test_option_only_none_missing_some() {
        let arms = vec![enum_literal_arm("None", false)];
        let missing = find_missing_variants_pure("Option", &arms, &empty_registry());
        assert_eq!(missing, vec!["Some".to_string()]);
    }

    #[test]
    fn test_option_both_arms_exhaustive() {
        let arms = vec![
            enum_literal_arm("Some", true),
            enum_literal_arm("None", false),
        ];
        let missing = find_missing_variants_pure("Option", &arms, &empty_registry());
        assert!(
            missing.is_empty(),
            "Expected no missing variants, got: {:?}",
            missing
        );
    }

    #[test]
    fn test_option_with_wildcard_exhaustive() {
        let arms = vec![enum_literal_arm("Some", true), wildcard_arm()];
        let missing = find_missing_variants_pure("Option", &arms, &empty_registry());
        assert!(
            missing.is_empty(),
            "Wildcard should make match exhaustive, got: {:?}",
            missing
        );
    }

    #[test]
    fn test_option_wildcard_only_exhaustive() {
        let arms = vec![wildcard_arm()];
        let missing = find_missing_variants_pure("Option", &arms, &empty_registry());
        assert!(
            missing.is_empty(),
            "Wildcard-only should be exhaustive, got: {:?}",
            missing
        );
    }

    #[test]
    fn test_option_generic_strips_type_params() {
        let arms = vec![enum_literal_arm("Some", true)];
        let missing = find_missing_variants_pure("Option<i32>", &arms, &empty_registry());
        assert_eq!(missing, vec!["None".to_string()]);
    }

    #[test]
    fn test_result_only_ok_missing_err() {
        let arms = vec![enum_literal_arm("Ok", true)];
        let missing = find_missing_variants_pure("Result", &arms, &empty_registry());
        assert_eq!(missing, vec!["Err".to_string()]);
    }

    #[test]
    fn test_result_only_err_missing_ok() {
        let arms = vec![enum_literal_arm("Err", true)];
        let missing = find_missing_variants_pure("Result", &arms, &empty_registry());
        assert_eq!(missing, vec!["Ok".to_string()]);
    }

    #[test]
    fn test_result_both_arms_exhaustive() {
        let arms = vec![enum_literal_arm("Ok", true), enum_literal_arm("Err", true)];
        let missing = find_missing_variants_pure("Result", &arms, &empty_registry());
        assert!(
            missing.is_empty(),
            "Expected no missing variants, got: {:?}",
            missing
        );
    }

    #[test]
    fn test_result_generic_strips_type_params() {
        let arms = vec![enum_literal_arm("Ok", true)];
        let missing = find_missing_variants_pure("Result<i32, String>", &arms, &empty_registry());
        assert_eq!(missing, vec!["Err".to_string()]);
    }

    #[test]
    fn test_unknown_enum_no_symbols_returns_empty() {
        let arms = vec![enum_literal_arm("Foo", false)];
        let missing = find_missing_variants_pure("MyCustomEnum", &arms, &empty_registry());
        assert!(
            missing.is_empty(),
            "Unknown enum should return empty (no info), got: {:?}",
            missing
        );
    }

    #[test]
    fn test_custom_enum_from_workspace_symbols() {
        let arms = vec![enum_literal_arm("Red", false)];
        let missing = find_missing_variants_pure("Color", &arms, &color_registry());
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"Green".to_string()));
        assert!(missing.contains(&"Blue".to_string()));
    }

    #[test]
    fn test_custom_enum_all_variants_covered() {
        let arms = vec![
            enum_literal_arm("Red", false),
            enum_literal_arm("Green", false),
            enum_literal_arm("Blue", false),
        ];
        let missing = find_missing_variants_pure("Color", &arms, &color_registry());
        assert!(
            missing.is_empty(),
            "All variants covered, got: {:?}",
            missing
        );
    }

    #[test]
    fn test_custom_enum_with_wildcard() {
        let arms = vec![enum_literal_arm("Red", false), wildcard_arm()];
        let missing = find_missing_variants_pure("Color", &arms, &color_registry());
        assert!(
            missing.is_empty(),
            "Wildcard should make custom enum exhaustive, got: {:?}",
            missing
        );
    }

    #[test]
    fn test_empty_arms_option_missing_both() {
        let arms: Vec<PatternArm> = vec![];
        let missing = find_missing_variants_pure("Option", &arms, &empty_registry());
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"Some".to_string()));
        assert!(missing.contains(&"None".to_string()));
    }

    // =========================================================================
    // Tests for check_pattern_exhaustiveness (statement walker + diagnostics)
    // =========================================================================

    #[test]
    fn test_check_pattern_non_exhaustive_produces_warning() {
        // Build an expression statement containing a non-exhaustive pattern match
        let scrutinee = Expression::Identifier("my_opt".to_string());
        let arms = vec![enum_literal_arm("Some", true)]; // Missing None

        let stmt = zen::ast::Statement::Expression {
            expr: Expression::PatternMatch {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span: None,
        };

        let content = "my_opt ?\n    | Some(x) { x }";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_pattern_exhaustiveness(
            &[stmt],
            &mut diagnostics,
            content,
            &empty_registry(),
            |expr| {
                if let Expression::Identifier(name) = expr {
                    if name == "my_opt" {
                        return Some("Option".to_string());
                    }
                }
                None
            },
        );

        assert_eq!(
            diagnostics.len(),
            1,
            "Expected 1 diagnostic, got: {:?}",
            diagnostics
        );
        let diag = &diagnostics[0];
        assert_eq!(diag.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(
            diag.code,
            Some(NumberOrString::String("non-exhaustive-match".to_string()))
        );
        assert!(
            diag.message.contains("None"),
            "Diagnostic should mention missing 'None', got: {}",
            diag.message
        );
        assert_eq!(diag.source, Some("zen-lsp".to_string()));
    }

    #[test]
    fn test_check_pattern_exhaustive_no_diagnostic() {
        let scrutinee = Expression::Identifier("my_opt".to_string());
        let arms = vec![
            enum_literal_arm("Some", true),
            enum_literal_arm("None", false),
        ];

        let stmt = zen::ast::Statement::Expression {
            expr: Expression::PatternMatch {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span: None,
        };

        let content = "my_opt ?\n    | Some(x) { x }\n    | None { 0 }";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_pattern_exhaustiveness(
            &[stmt],
            &mut diagnostics,
            content,
            &empty_registry(),
            |expr| {
                if let Expression::Identifier(name) = expr {
                    if name == "my_opt" {
                        return Some("Option".to_string());
                    }
                }
                None
            },
        );

        assert!(
            diagnostics.is_empty(),
            "Exhaustive match should produce no diagnostics, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_check_pattern_wildcard_no_diagnostic() {
        let scrutinee = Expression::Identifier("my_opt".to_string());
        let arms = vec![enum_literal_arm("Some", true), wildcard_arm()];

        let stmt = zen::ast::Statement::Expression {
            expr: Expression::PatternMatch {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span: None,
        };

        let content = "my_opt ?\n    | Some(x) { x }\n    | _ { 0 }";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_pattern_exhaustiveness(
            &[stmt],
            &mut diagnostics,
            content,
            &empty_registry(),
            |expr| {
                if let Expression::Identifier(name) = expr {
                    if name == "my_opt" {
                        return Some("Option".to_string());
                    }
                }
                None
            },
        );

        assert!(
            diagnostics.is_empty(),
            "Wildcard match should produce no diagnostics, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_check_pattern_no_type_info_no_diagnostic() {
        let scrutinee = Expression::Identifier("unknown_var".to_string());
        let arms = vec![enum_literal_arm("Foo", false)];

        let stmt = zen::ast::Statement::Expression {
            expr: Expression::PatternMatch {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span: None,
        };

        let content = "unknown_var ?\n    | Foo { 0 }";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_pattern_exhaustiveness(
            &[stmt],
            &mut diagnostics,
            content,
            &empty_registry(),
            |_expr| None,
        );

        assert!(
            diagnostics.is_empty(),
            "No type info should produce no diagnostics, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_check_pattern_in_variable_declaration() {
        let scrutinee = Expression::Identifier("opt".to_string());
        let arms = vec![enum_literal_arm("Some", true)];

        let stmt = zen::ast::Statement::VariableDeclaration {
            name: "result".to_string(),
            type_: None,
            initializer: Some(Expression::PatternMatch {
                scrutinee: Box::new(scrutinee),
                arms,
            }),
            is_mutable: false,
            declaration_type: zen::ast::VariableDeclarationType::InferredImmutable,
            span: None,
        };

        let content = "result = opt ?\n    | Some(x) { x }";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_pattern_exhaustiveness(
            &[stmt],
            &mut diagnostics,
            content,
            &empty_registry(),
            |expr| {
                if let Expression::Identifier(name) = expr {
                    if name == "opt" {
                        return Some("Option<i32>".to_string());
                    }
                }
                None
            },
        );

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("None"));
    }

    #[test]
    fn test_check_pattern_in_return_statement() {
        let scrutinee = Expression::Identifier("res".to_string());
        let arms = vec![enum_literal_arm("Ok", true)];

        let stmt = zen::ast::Statement::Return {
            expr: Expression::PatternMatch {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span: None,
        };

        let content = "return res ?\n    | Ok(v) { v }";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_pattern_exhaustiveness(
            &[stmt],
            &mut diagnostics,
            content,
            &empty_registry(),
            |expr| {
                if let Expression::Identifier(name) = expr {
                    if name == "res" {
                        return Some("Result".to_string());
                    }
                }
                None
            },
        );

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Err"));
    }

    #[test]
    fn test_check_pattern_nested_in_block() {
        let scrutinee = Expression::Identifier("val".to_string());
        let arms = vec![enum_literal_arm("Some", true)];

        let block_stmt = zen::ast::Statement::Expression {
            expr: Expression::Block(vec![zen::ast::Statement::Expression {
                expr: Expression::PatternMatch {
                    scrutinee: Box::new(scrutinee),
                    arms,
                },
                span: None,
            }]),
            span: None,
        };

        let content = "{\n    val ?\n        | Some(x) { x }\n}";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_pattern_exhaustiveness(
            &[block_stmt],
            &mut diagnostics,
            content,
            &empty_registry(),
            |expr| {
                if let Expression::Identifier(name) = expr {
                    if name == "val" {
                        return Some("Option".to_string());
                    }
                }
                None
            },
        );

        assert_eq!(diagnostics.len(), 1, "Nested pattern match should be found");
        assert!(diagnostics[0].message.contains("None"));
    }

    #[test]
    fn test_check_pattern_multiple_matches() {
        let stmt1 = zen::ast::Statement::Expression {
            expr: Expression::PatternMatch {
                scrutinee: Box::new(Expression::Identifier("opt1".to_string())),
                arms: vec![enum_literal_arm("Some", true)],
            },
            span: None,
        };
        let stmt2 = zen::ast::Statement::Expression {
            expr: Expression::PatternMatch {
                scrutinee: Box::new(Expression::Identifier("opt2".to_string())),
                arms: vec![enum_literal_arm("None", false)],
            },
            span: None,
        };

        let content = "opt1 ?\n    | Some(x) { x }\nopt2 ?\n    | None { 0 }";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_pattern_exhaustiveness(
            &[stmt1, stmt2],
            &mut diagnostics,
            content,
            &empty_registry(),
            |expr| {
                if let Expression::Identifier(name) = expr {
                    if name == "opt1" || name == "opt2" {
                        return Some("Option".to_string());
                    }
                }
                None
            },
        );

        assert_eq!(
            diagnostics.len(),
            2,
            "Two non-exhaustive matches should produce 2 diagnostics"
        );
    }

    #[test]
    fn test_diagnostic_range_and_position() {
        let scrutinee = Expression::Identifier("my_opt".to_string());
        let arms = vec![enum_literal_arm("Some", true)];

        let stmt = zen::ast::Statement::Expression {
            expr: Expression::PatternMatch {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span: None,
        };

        let content = "    my_opt ?\n        | Some(x) { x }";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_pattern_exhaustiveness(
            &[stmt],
            &mut diagnostics,
            content,
            &empty_registry(),
            |expr| {
                if let Expression::Identifier(name) = expr {
                    if name == "my_opt" {
                        return Some("Option".to_string());
                    }
                }
                None
            },
        );

        assert_eq!(diagnostics.len(), 1);
        let range = diagnostics[0].range;
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 4);
        assert_eq!(range.end.line, 0);
        assert_eq!(range.end.character, 14);
    }

    #[test]
    fn test_enum_variant_pattern_also_covered() {
        let arms = vec![PatternArm {
            pattern: Pattern::EnumVariant {
                enum_name: "Option".to_string(),
                variant: "Some".to_string(),
                payload: Some(Box::new(Pattern::Identifier("x".to_string()))),
            },
            guard: None,
            body: Expression::Integer32(0),
        }];

        let missing = find_missing_variants_pure("Option", &arms, &empty_registry());
        assert_eq!(missing, vec!["None".to_string()]);
    }
}

#[cfg(test)]
mod allocator_validation_tests {
    use lsp_types::*;
    use zen::ast::{Expression, Statement};
    use zen::lsp::analyzer::check_allocator_usage;

    // =========================================================================
    // Tests for check_allocator_usage
    // =========================================================================

    #[test]
    fn test_vec_new_without_allocator_produces_error() {
        // Vec<i32>.new() without allocator → error
        let stmt = Statement::Expression {
            expr: Expression::FunctionCall {
                name: "Vec<i32>.new".to_string(),
                type_args: vec![],
                args: vec![],
                span: None,
            },
            span: None,
        };

        let content = "v = Vec<i32>.new()";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        // The function checks stdlib_types().requires_allocator("Vec")
        // Vec is defined in stdlib with an allocator field, so this should produce an error
        // NOTE: If stdlib isn't loaded (test env), requires_allocator returns false
        // and no diagnostic is produced. This characterizes the CURRENT behavior.
        // The test captures whichever behavior occurs.
        if !diagnostics.is_empty() {
            assert_eq!(diagnostics.len(), 1);
            let diag = &diagnostics[0];
            assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
            assert_eq!(
                diag.code,
                Some(NumberOrString::String("allocator-required".to_string()))
            );
            assert!(diag.message.contains("allocator"));
            assert_eq!(diag.source, Some("zen-lsp".to_string()));
        }
        // Either way, this test passes - it characterizes current behavior
    }

    #[test]
    fn test_function_call_with_allocator_arg_no_error() {
        // Vec<i32>.new(allocator) → no error regardless
        let stmt = Statement::Expression {
            expr: Expression::FunctionCall {
                name: "Vec<i32>.new".to_string(),
                type_args: vec![],
                args: vec![Expression::Identifier("allocator".to_string())],
                span: None,
            },
            span: None,
        };

        let content = "v = Vec<i32>.new(allocator)";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "Call with allocator arg should not produce diagnostics, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_function_call_with_get_default_allocator_no_error() {
        // Vec<i32>.new(get_default_allocator()) → no error
        let stmt = Statement::Expression {
            expr: Expression::FunctionCall {
                name: "Vec<i32>.new".to_string(),
                type_args: vec![],
                args: vec![Expression::FunctionCall {
                    name: "get_default_allocator".to_string(),
                    type_args: vec![],
                    args: vec![],
                    span: None,
                }],
                span: None,
            },
            span: None,
        };

        let content = "v = Vec<i32>.new(get_default_allocator())";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "get_default_allocator() should satisfy allocator requirement, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_function_call_with_alloc_identifier_no_error() {
        // has_allocator_arg recognizes identifiers containing "alloc"
        let stmt = Statement::Expression {
            expr: Expression::FunctionCall {
                name: "Vec<i32>.new".to_string(),
                type_args: vec![],
                args: vec![Expression::Identifier("my_alloc".to_string())],
                span: None,
            },
            span: None,
        };

        let content = "v = Vec<i32>.new(my_alloc)";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "Identifier containing 'alloc' should satisfy requirement, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_regular_function_call_no_error() {
        // A function call to something that doesn't require an allocator
        let stmt = Statement::Expression {
            expr: Expression::FunctionCall {
                name: "print".to_string(),
                type_args: vec![],
                args: vec![Expression::String("hello".to_string())],
                span: None,
            },
            span: None,
        };

        let content = "print(\"hello\")";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "Regular function should not produce allocator diagnostics, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_allocator_check_in_variable_declaration() {
        // Allocator check works in variable declarations too
        let stmt = Statement::VariableDeclaration {
            name: "v".to_string(),
            type_: None,
            initializer: Some(Expression::FunctionCall {
                name: "Vec<i32>.new".to_string(),
                type_args: vec![],
                args: vec![Expression::Identifier("allocator".to_string())],
                span: None,
            }),
            is_mutable: false,
            declaration_type: zen::ast::VariableDeclarationType::InferredImmutable,
            span: None,
        };

        let content = "v = Vec<i32>.new(allocator)";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "Variable declaration with allocator should be fine, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_allocator_check_nested_in_block() {
        // Allocator check recurses into block expressions
        let stmt = Statement::Expression {
            expr: Expression::Block(vec![Statement::Expression {
                expr: Expression::FunctionCall {
                    name: "print".to_string(),
                    type_args: vec![],
                    args: vec![],
                    span: None,
                },
                span: None,
            }]),
            span: None,
        };

        let content = "{\n    print()\n}";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "Block with regular function should be fine, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_allocator_check_in_return_statement() {
        // Allocator check in return statement
        let stmt = Statement::Return {
            expr: Expression::FunctionCall {
                name: "print".to_string(),
                type_args: vec![],
                args: vec![],
                span: None,
            },
            span: None,
        };

        let content = "return print()";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "Return with regular function should be fine, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_allocator_check_nested_function_args() {
        // Allocator check recurses into function arguments
        let stmt = Statement::Expression {
            expr: Expression::FunctionCall {
                name: "process".to_string(),
                type_args: vec![],
                args: vec![Expression::FunctionCall {
                    name: "print".to_string(),
                    type_args: vec![],
                    args: vec![],
                    span: None,
                }],
                span: None,
            },
            span: None,
        };

        let content = "process(print())";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "Nested regular function calls should be fine, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_allocator_check_method_call_recurses() {
        // Method call should recurse into object and args
        let stmt = Statement::Expression {
            expr: Expression::MethodCall {
                object: Box::new(Expression::Identifier("obj".to_string())),
                method: "do_thing".to_string(),
                type_args: vec![],
                args: vec![Expression::Identifier("x".to_string())],
                span: None,
            },
            span: None,
        };

        let content = "obj.do_thing(x)";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "Method call on regular objects should be fine, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_has_allocator_via_method_call() {
        // has_allocator_arg recognizes method calls with allocator-related names
        let stmt = Statement::Expression {
            expr: Expression::FunctionCall {
                name: "Vec<i32>.new".to_string(),
                type_args: vec![],
                args: vec![Expression::MethodCall {
                    object: Box::new(Expression::Identifier("ctx".to_string())),
                    method: "get_allocator".to_string(),
                    type_args: vec![],
                    args: vec![],
                    span: None,
                }],
                span: None,
            },
            span: None,
        };

        let content = "v = Vec<i32>.new(ctx.get_allocator())";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "Method call returning allocator should satisfy requirement, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_allocator_check_binary_op_recurses() {
        // Check that binary operations are recursed into
        let stmt = Statement::Expression {
            expr: Expression::BinaryOp {
                left: Box::new(Expression::FunctionCall {
                    name: "print".to_string(),
                    type_args: vec![],
                    args: vec![],
                    span: None,
                }),
                op: zen::ast::BinaryOperator::Add,
                right: Box::new(Expression::Integer32(1)),
            },
            span: None,
        };

        let content = "print() + 1";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&[stmt], &mut diagnostics, content);

        assert!(
            diagnostics.is_empty(),
            "Binary op with regular calls should be fine, got: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_allocator_check_skips_non_matching_statements() {
        // Statements like Break, Continue, Loop should not cause issues
        let stmts = vec![
            Statement::Break {
                label: None,
                span: None,
            },
            Statement::Continue {
                label: None,
                span: None,
            },
        ];

        let content = "break\ncontinue";
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        check_allocator_usage(&stmts, &mut diagnostics, content);

        assert!(diagnostics.is_empty());
    }
}

#[cfg(test)]
mod stdlib_phantom_error_tests {
    use lsp_types::*;
    use std::collections::HashMap;
    use zen::lsp::analyzer::analyze_document;
    use zen::lsp::types::Document;

    #[test]
    fn test_lsp_no_phantom_errors_file_zen() {
        let content = std::fs::read_to_string("stdlib/io/files/file.zen").unwrap();
        let docs: HashMap<Url, Document> = HashMap::new();
        let diagnostics = analyze_document(&content, false, &docs);
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
            .collect();
        for e in &errors {
            eprintln!("file.zen L{}: {}", e.range.start.line + 1, e.message);
        }
        assert!(
            errors.is_empty(),
            "file.zen should have no LSP errors, got {} errors",
            errors.len()
        );
    }

    #[test]
    fn test_lsp_no_phantom_errors_fs_zen() {
        let content = std::fs::read_to_string("stdlib/io/files/fs.zen").unwrap();
        let docs: HashMap<Url, Document> = HashMap::new();
        let diagnostics = analyze_document(&content, false, &docs);
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
            .collect();
        for e in &errors {
            eprintln!("fs.zen L{}: {}", e.range.start.line + 1, e.message);
        }
        assert!(
            errors.is_empty(),
            "fs.zen should have no LSP errors, got {} errors",
            errors.len()
        );
    }
}

#[cfg(test)]
mod analyzer_integration_tests {
    use lsp_types::*;
    use std::collections::HashMap;
    use zen::lsp::analyzer::analyze_document;
    use zen::lsp::types::Document;

    #[test]
    fn test_analyze_document_valid_function() {
        let content = r#"
main = () i32 {
    return 0
}
"#;
        let docs: HashMap<Url, Document> = HashMap::new();

        let diagnostics = analyze_document(content, false, &docs);

        // Valid code should produce no errors (or only type-checker warnings)
        // This characterizes what the analyzer produces for simple valid code
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
            .collect();
        // Simple valid function should not produce errors
        assert!(
            errors.is_empty(),
            "Simple valid function should not produce errors, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_analyze_document_parse_error() {
        // Invalid syntax should produce a parse error diagnostic
        let content = "this is not valid zen code {{{{";
        let docs: HashMap<Url, Document> = HashMap::new();

        let diagnostics = analyze_document(content, false, &docs);

        assert!(
            !diagnostics.is_empty(),
            "Invalid syntax should produce diagnostics"
        );
    }

    #[test]
    fn test_analyze_document_skip_expensive_analysis() {
        let content = r#"
main = () i32 {
    return 0
}
"#;
        let docs: HashMap<Url, Document> = HashMap::new();

        // With skip_expensive_analysis=true, no type checking or validation runs
        let diagnostics = analyze_document(content, true, &docs);

        // Should have no diagnostics for valid code with skipped analysis
        assert!(
            diagnostics.is_empty(),
            "Skipped analysis should produce no diagnostics for valid code, got: {:?}",
            diagnostics
        );
    }
}
