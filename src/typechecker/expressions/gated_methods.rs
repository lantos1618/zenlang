use super::*;
use std::fmt;
use std::str::FromStr;

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

impl fmt::Display for GatedMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for GatedMethod {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        GatedMethod::ALL
            .iter()
            .copied()
            .find(|method| method.as_str() == value)
            .ok_or(())
    }
}
