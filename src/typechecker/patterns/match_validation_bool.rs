use crate::ast::expressions::MatchArm;
use crate::ast::Pattern;
use crate::error::{
    Diagnostic, Span, SuggestedFix, TextEdit, MISSING_BOOL_MATCH_ARM_FIX_KIND,
    MISSING_BOOL_MATCH_ARM_FIX_TITLE,
};

use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_bool_match_patterns(
        &mut self,
        arms: &[MatchArm],
        require_exhaustive: bool,
        span: Span,
    ) {
        let mut true_seen = false;
        let mut false_seen = false;
        let mut wildcard_seen = false;

        for arm in arms {
            match &arm.pattern {
                Pattern::BoolTrue { span } => {
                    if true_seen || wildcard_seen {
                        self.diagnostics.push(Diagnostic::error(
                            "E4005",
                            "duplicate match arm for `true`",
                            *span,
                        ));
                    }
                    true_seen = true;
                }
                Pattern::BoolFalse { span } => {
                    if false_seen || wildcard_seen {
                        self.diagnostics.push(Diagnostic::error(
                            "E4005",
                            "duplicate match arm for `false`",
                            *span,
                        ));
                    }
                    false_seen = true;
                }
                Pattern::Wildcard { span } => {
                    if wildcard_seen || (true_seen && false_seen) {
                        self.diagnostics.push(Diagnostic::error(
                            "E4005",
                            "redundant wildcard match arm",
                            *span,
                        ));
                    }
                    wildcard_seen = true;
                }
                _ => {}
            }
        }

        if require_exhaustive && !wildcard_seen && !(true_seen && false_seen) {
            let missing_values = Self::missing_bool_match_values(true_seen, false_seen);
            let missing = missing_values
                .iter()
                .map(|value| format!("`{value}`"))
                .collect::<Vec<_>>()
                .join(", ");
            let replacement = Self::missing_bool_match_fix_replacement(&missing_values);
            let insertion = Span::new(span.file_id, span.end, span.end);
            self.diagnostics.push(
                Diagnostic::error(
                    "E4006",
                    format!("non-exhaustive bool match: missing {missing}"),
                    span,
                )
                .with_suggested_fix(SuggestedFix::new(
                    MISSING_BOOL_MATCH_ARM_FIX_KIND,
                    MISSING_BOOL_MATCH_ARM_FIX_TITLE,
                    vec![TextEdit::new(insertion, format!("\n{replacement}"))],
                )),
            );
        }
    }

    fn missing_bool_match_values(true_seen: bool, false_seen: bool) -> Vec<&'static str> {
        match (true_seen, false_seen) {
            (true, false) => vec!["false"],
            (false, true) => vec!["true"],
            _ => vec!["true", "false"],
        }
    }

    fn missing_bool_match_fix_replacement(missing_values: &[&str]) -> String {
        missing_values
            .iter()
            .map(|value| format!("        | {value} {{ <expression> }}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
