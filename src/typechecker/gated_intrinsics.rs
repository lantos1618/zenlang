#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GatedIntrinsic {
    AsyncEnqueue,
    AsyncYield,
    TypeMatch,
}

impl GatedIntrinsic {
    pub(super) const INTRINSIC_MODULE: &'static str = "@builtin";
    pub(super) const ASYNC_ENQUEUE: &'static str = "async_enqueue";
    pub(super) const ASYNC_YIELD: &'static str = "async_yield";
    pub(super) const TYPE_MATCH: &'static str = "type_match";
    const ALL: &[GatedIntrinsic] = &[Self::AsyncEnqueue, Self::AsyncYield, Self::TypeMatch];

    pub(super) fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|intrinsic| intrinsic.as_str() == name)
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::AsyncEnqueue => Self::ASYNC_ENQUEUE,
            Self::AsyncYield => Self::ASYNC_YIELD,
            Self::TypeMatch => Self::TYPE_MATCH,
        }
    }

    pub(super) const fn gate_message(self) -> &'static str {
        match self {
            Self::AsyncEnqueue => {
                "async task enqueue is gated until Sync/Async effect checking and task lowering are implemented"
            }
            Self::AsyncYield => {
                "async yield is gated until Sync/Async effect checking and task lowering are implemented"
            }
            Self::TypeMatch => {
                "comptime type matching is gated until typed metadata and derive lowering are implemented"
            }
        }
    }
}
