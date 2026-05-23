use super::ParserBehaviorKeyword;
use std::str::FromStr;

impl ParserBehaviorKeyword {
    const ALL: &[ParserBehaviorKeyword] = &[ParserBehaviorKeyword::Behavior];

    const BEHAVIOR: &'static str = "behavior";

    pub(in crate::parser) fn as_str(self) -> &'static str {
        match self {
            Self::Behavior => Self::BEHAVIOR,
        }
    }
}

impl FromStr for ParserBehaviorKeyword {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        crate::static_spelling::parse_static_spelling(Self::ALL, value, Self::as_str)
    }
}
