use super::ParserPrefixKeyword;

impl ParserPrefixKeyword {
    const ALL: &[ParserPrefixKeyword] = &[
        ParserPrefixKeyword::True,
        ParserPrefixKeyword::False,
        ParserPrefixKeyword::Return,
        ParserPrefixKeyword::As,
        ParserPrefixKeyword::Break,
        ParserPrefixKeyword::Continue,
        ParserPrefixKeyword::Loop,
        ParserPrefixKeyword::Cast,
    ];

    const TRUE: &'static str = "true";
    const FALSE: &'static str = "false";
    const RETURN: &'static str = "return";
    const AS: &'static str = "as";
    const BREAK: &'static str = "break";
    const CONTINUE: &'static str = "continue";
    const LOOP: &'static str = "loop";
    const CAST: &'static str = "cast";

    pub(in crate::parser) fn as_str(self) -> &'static str {
        match self {
            Self::True => Self::TRUE,
            Self::False => Self::FALSE,
            Self::Return => Self::RETURN,
            Self::As => Self::AS,
            Self::Break => Self::BREAK,
            Self::Continue => Self::CONTINUE,
            Self::Loop => Self::LOOP,
            Self::Cast => Self::CAST,
        }
    }
}

crate::static_spelling::impl_static_spelling_from_str!(
    ParserPrefixKeyword,
    variants = ParserPrefixKeyword::ALL,
    as_str = ParserPrefixKeyword::as_str
);
