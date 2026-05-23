use super::ParserBehaviorKeyword;

impl ParserBehaviorKeyword {
    const ALL: &[ParserBehaviorKeyword] = &[ParserBehaviorKeyword::Behavior];

    const BEHAVIOR: &'static str = "behavior";

    pub(in crate::parser) fn as_str(self) -> &'static str {
        match self {
            Self::Behavior => Self::BEHAVIOR,
        }
    }
}

crate::static_spelling::impl_static_spelling_from_str!(
    ParserBehaviorKeyword,
    variants = ParserBehaviorKeyword::ALL,
    as_str = ParserBehaviorKeyword::as_str
);
