use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GatedMethod {
    ResultRaise,
    EffectAwait,
}

impl GatedMethod {
    const RAISE: &'static str = "raise";
    const AWAIT: &'static str = "await";

    const ALL: &[GatedMethod] = &[Self::ResultRaise, Self::EffectAwait];

    const fn as_str(self) -> &'static str {
        match self {
            Self::ResultRaise => Self::RAISE,
            Self::EffectAwait => Self::AWAIT,
        }
    }

    pub(super) fn diagnostic(self, span: Span) -> Diagnostic {
        match self {
            Self::ResultRaise => Diagnostic::error(
                "E3054",
                "`.raise()` is gated until Result propagation typing and lowering are implemented",
                span,
            ),
            Self::EffectAwait => Diagnostic::error(
                "E3055",
                "`.await()` is gated until Sync/Async effect checking and task lowering are implemented",
                span,
            ),
        }
    }
}

crate::static_spelling::impl_static_spelling_display!(GatedMethod, as_str = GatedMethod::as_str);
crate::static_spelling::impl_static_spelling_from_str!(
    GatedMethod,
    variants = GatedMethod::ALL,
    as_str = GatedMethod::as_str
);
