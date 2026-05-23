use super::ParserPatternKeyword;
use std::str::FromStr;

impl ParserPatternKeyword {
    const ALL: &[ParserPatternKeyword] = &[
        ParserPatternKeyword::True,
        ParserPatternKeyword::False,
        ParserPatternKeyword::Wildcard,
    ];

    const TRUE: &'static str = "true";
    const FALSE: &'static str = "false";
    const WILDCARD: &'static str = "_";

    fn as_str(self) -> &'static str {
        match self {
            Self::True => Self::TRUE,
            Self::False => Self::FALSE,
            Self::Wildcard => Self::WILDCARD,
        }
    }
}

impl FromStr for ParserPatternKeyword {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|keyword| keyword.as_str() == value)
            .ok_or(())
    }
}
