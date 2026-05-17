impl TypeChecker {
    fn validate_resolver_behavior_parent_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains_for_role(
            BehaviorRefRole::Parent,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_parent_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list_for_role(
            BehaviorRefRole::Parent,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_impl_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains_for_role(
            BehaviorRefRole::Impl,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_impl_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list_for_role(
            BehaviorRefRole::Impl,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_required_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains_for_role(
            BehaviorRefRole::Required,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_required_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list_for_role(
            BehaviorRefRole::Required,
            symbol,
            name,
            expected,
            span,
        );
    }

    pub(super) fn validate_resolver_behavior_ref_contains_for_role(
        &mut self,
        role: BehaviorRefRole,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains(
            BehaviorRefValidation::for_role(role, BehaviorRefCheck::Contains),
            name,
            BehaviorRefActual::for_role(symbol, role),
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_ref_list_for_role(
        &mut self,
        role: BehaviorRefRole,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list(
            BehaviorRefValidation::for_role(role, BehaviorRefCheck::List),
            name,
            BehaviorRefActual::for_role(symbol, role),
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_ref_contains(
        &mut self,
        validation: BehaviorRefValidation,
        name: &str,
        actual: BehaviorRefActual<'_>,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        if !actual.contains_display(&expected.display) {
            let actual = format_behavior_ref_names(actual.names);
            self.diagnostics.push(Diagnostic::error(
                validation.name_code,
                validation.contains_name_message(name, &actual, &expected.display),
                span,
            ));
        }
        if !actual.contains_metadata(&expected.metadata) {
            let actual = format_behavior_refs(actual.refs);
            let expected_ref =
                behavior_ref_display(&expected.metadata.name, &expected.metadata.type_args);
            self.diagnostics.push(Diagnostic::error(
                validation.ref_code,
                validation.contains_ref_message(name, &actual, &expected_ref),
                span,
            ));
        }
    }

    fn validate_resolver_behavior_ref_list(
        &mut self,
        validation: BehaviorRefValidation,
        name: &str,
        actual: BehaviorRefActual<'_>,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        let expected = ExpectedBehaviorEdgeMetadata::from_edges(expected);
        if !actual.names_match(&expected.names) {
            let actual = format_behavior_ref_names(actual.names);
            let expected_names = format_behavior_ref_names(Some(&expected.names));
            self.diagnostics.push(Diagnostic::error(
                validation.name_code,
                validation.list_name_message(name, &actual, &expected_names),
                span,
            ));
        }
        if !actual.refs_match(&expected.refs) {
            let actual = format_behavior_refs(actual.refs);
            let expected_refs = format_behavior_refs(Some(&expected.refs));
            self.diagnostics.push(Diagnostic::error(
                validation.ref_code,
                validation.list_ref_message(name, &actual, &expected_refs),
                span,
            ));
        }
    }
}
