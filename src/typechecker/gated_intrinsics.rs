#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GatedIntrinsic {
    TypeMatch,
}

impl GatedIntrinsic {
    pub(super) const INTRINSIC_MODULE: &'static str = "@builtin";
    pub(super) const TYPE_MATCH: &'static str = "type_match";
    const ALL: &[GatedIntrinsic] = &[Self::TypeMatch];

    pub(super) fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|intrinsic| intrinsic.as_str() == name)
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::TypeMatch => Self::TYPE_MATCH,
        }
    }

    pub(super) const fn gate_message(self) -> &'static str {
        match self {
            Self::TypeMatch => {
                "comptime type matching is gated until typed metadata and derive lowering are implemented"
            }
        }
    }
}
