use crate::ast::expressions::MatchArm;
use crate::ast::Pattern;
use crate::error::{CompilerDiagnosticCode::*, Diagnostic, Span};

use super::super::{quoted_list, TypeChecker};

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
            let (value, seen, span) = match &arm.pattern {
                Pattern::BoolTrue { span } => ("true", &mut true_seen, *span),
                Pattern::BoolFalse { span } => ("false", &mut false_seen, *span),
                Pattern::Wildcard { span } => {
                    if wildcard_seen || (true_seen && false_seen) {
                        self.push_error(E4005, "redundant wildcard match arm", *span);
                    }
                    wildcard_seen = true;
                    continue;
                }
                _ => continue,
            };
            if *seen || wildcard_seen {
                self.push_error(E4005, format!("duplicate match arm for `{value}`"), span);
            }
            *seen = true;
        }

        if require_exhaustive && !wildcard_seen && !(true_seen && false_seen) {
            let missing_values: &[&str] = match (true_seen, false_seen) {
                (true, false) => &["false"],
                (false, true) => &["true"],
                _ => &["true", "false"],
            };
            let missing = quoted_list(missing_values);
            let replacement = missing_values
                .iter()
                .map(|value| format!("        | {value} {{ <expression> }}"))
                .collect::<Vec<_>>()
                .join("\n");
            let insertion = Span::new(span.file_id, span.end, span.end);
            self.diagnostics.push(
                Diagnostic::error_code(
                    E4006,
                    format!("non-exhaustive bool match: missing {missing}"),
                    span,
                )
                .with_fix(
                    "add_missing_bool_match_arm",
                    "Add missing bool match arm",
                    insertion,
                    format!("\n{replacement}"),
                ),
            );
        }
    }
}
