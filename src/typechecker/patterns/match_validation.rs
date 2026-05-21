use crate::ast::expressions::MatchArm;
use crate::ast::typed::*;
use crate::ast::Pattern;

use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn determine_match_kind(
        &self,
        scrutinee_type: &Type,
        arms: &[MatchArm],
    ) -> MatchKind {
        let all_bool = arms.iter().all(|arm| {
            matches!(
                &arm.pattern,
                Pattern::BoolTrue { .. } | Pattern::BoolFalse { .. }
            )
        });
        if all_bool {
            if arms.len() >= 2 {
                return MatchKind::ConditionalElse;
            }
            return MatchKind::Conditional;
        }

        match scrutinee_type {
            Type::Named(name) if self.enums.contains_key(name) => MatchKind::EnumMatch,
            Type::Enum { .. } => MatchKind::EnumMatch,
            _ => MatchKind::ValueMatch,
        }
    }
}
