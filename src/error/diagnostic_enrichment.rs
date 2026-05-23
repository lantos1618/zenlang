use super::{
    ContextFrame, ContextKind, Diagnostic, SuggestedFix, TextEdit,
    GATED_GENERATED_BEHAVIOR_DERIVE_CONTEXT, GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE,
    GATED_GENERATED_BEHAVIOR_DERIVE_NOTE, GATED_GENERIC_ASSOCIATION_TARGET_CONTEXT,
    GATED_GENERIC_ASSOCIATION_TARGET_MESSAGE_PREFIX, GATED_GENERIC_ASSOCIATION_TARGET_NOTE,
    REMOVED_INFIX_AS_CAST_FIX_KIND, REMOVED_INFIX_AS_CAST_FIX_TITLE, REMOVED_INFIX_AS_CAST_MESSAGE,
    REMOVED_INFIX_AS_CAST_REPLACEMENT, REMOVED_RETURN_FIX_KIND, REMOVED_RETURN_FIX_TITLE,
    REMOVED_RETURN_KEYWORD_MESSAGE,
};

impl Diagnostic {
    pub(super) fn with_removed_return_fix(self) -> Self {
        self.with_fix_for_exact_message(
            REMOVED_RETURN_KEYWORD_MESSAGE,
            REMOVED_RETURN_FIX_KIND,
            REMOVED_RETURN_FIX_TITLE,
            "",
        )
    }

    pub(super) fn with_removed_infix_as_cast_fix(self) -> Self {
        self.with_fix_for_exact_message(
            REMOVED_INFIX_AS_CAST_MESSAGE,
            REMOVED_INFIX_AS_CAST_FIX_KIND,
            REMOVED_INFIX_AS_CAST_FIX_TITLE,
            REMOVED_INFIX_AS_CAST_REPLACEMENT,
        )
    }

    fn with_fix_for_exact_message(
        self,
        message: &'static str,
        fix_kind: &'static str,
        fix_title: &'static str,
        replacement: &'static str,
    ) -> Self {
        if self.message != message {
            return self;
        }

        let Some(span) = self.span else {
            return self;
        };

        self.with_suggested_fix(SuggestedFix::new(
            fix_kind,
            fix_title,
            vec![TextEdit::new(span, replacement)],
        ))
    }

    pub fn with_generated_behavior_derive_gate_context(self) -> Self {
        if self.message != GATED_GENERATED_BEHAVIOR_DERIVE_MESSAGE {
            return self;
        }

        let Some(span) = self.span else {
            return self;
        };

        self.with_note(GATED_GENERATED_BEHAVIOR_DERIVE_NOTE)
            .with_context(ContextFrame {
                span,
                kind: ContextKind::FeatureGate,
                message: GATED_GENERATED_BEHAVIOR_DERIVE_CONTEXT.to_string(),
            })
    }

    pub fn with_generic_association_target_gate_context(self) -> Self {
        if !self
            .message
            .starts_with(GATED_GENERIC_ASSOCIATION_TARGET_MESSAGE_PREFIX)
        {
            return self;
        }

        let Some(span) = self.span else {
            return self;
        };

        self.with_note(GATED_GENERIC_ASSOCIATION_TARGET_NOTE)
            .with_context(ContextFrame {
                span,
                kind: ContextKind::FeatureGate,
                message: GATED_GENERIC_ASSOCIATION_TARGET_CONTEXT.to_string(),
            })
    }
}
