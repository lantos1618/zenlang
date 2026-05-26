use super::ParserPatternKeyword;

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

crate::static_spelling::impl_static_spelling_from_str!(
    ParserPatternKeyword,
    variants = ParserPatternKeyword::ALL,
    as_str = ParserPatternKeyword::as_str
);
