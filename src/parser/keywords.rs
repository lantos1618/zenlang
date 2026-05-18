use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserPrefixKeyword {
    True,
    False,
    Return,
    Break,
    Continue,
    Loop,
    Cast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserPatternKeyword {
    True,
    False,
    Wildcard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserThisMethod {
    Defer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserModuleRoot {
    AtBuiltin,
    AtStd,
}

impl ParserPrefixKeyword {
    const ALL: &[ParserPrefixKeyword] = &[
        ParserPrefixKeyword::True,
        ParserPrefixKeyword::False,
        ParserPrefixKeyword::Return,
        ParserPrefixKeyword::Break,
        ParserPrefixKeyword::Continue,
        ParserPrefixKeyword::Loop,
        ParserPrefixKeyword::Cast,
    ];

    const TRUE: &'static str = "true";
    const FALSE: &'static str = "false";
    const RETURN: &'static str = "return";
    const BREAK: &'static str = "break";
    const CONTINUE: &'static str = "continue";
    const LOOP: &'static str = "loop";
    const CAST: &'static str = "cast";

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::True => Self::TRUE,
            Self::False => Self::FALSE,
            Self::Return => Self::RETURN,
            Self::Break => Self::BREAK,
            Self::Continue => Self::CONTINUE,
            Self::Loop => Self::LOOP,
            Self::Cast => Self::CAST,
        }
    }
}

impl FromStr for ParserPrefixKeyword {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|keyword| keyword.as_str() == value)
            .ok_or(())
    }
}

impl ParserPatternKeyword {
    const ALL: &[ParserPatternKeyword] = &[
        ParserPatternKeyword::True,
        ParserPatternKeyword::False,
        ParserPatternKeyword::Wildcard,
    ];

    const TRUE: &'static str = "true";
    const FALSE: &'static str = "false";
    const WILDCARD: &'static str = "_";

    pub(super) fn as_str(self) -> &'static str {
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

impl ParserThisMethod {
    const ALL: &[ParserThisMethod] = &[ParserThisMethod::Defer];

    const DEFER: &'static str = "defer";

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Defer => Self::DEFER,
        }
    }
}

impl FromStr for ParserThisMethod {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|method| method.as_str() == value)
            .ok_or(())
    }
}

impl ParserModuleRoot {
    const ALL: &[ParserModuleRoot] = &[ParserModuleRoot::AtBuiltin, ParserModuleRoot::AtStd];

    const AT_BUILTIN: &'static str = "@builtin";
    const AT_STD: &'static str = "@std";

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::AtBuiltin => Self::AT_BUILTIN,
            Self::AtStd => Self::AT_STD,
        }
    }

    pub(super) fn join_module_parts(self, parts: &[String]) -> String {
        if parts.is_empty() {
            self.as_str().to_string()
        } else {
            format!("{}.{}", self.as_str(), parts.join("."))
        }
    }
}

impl FromStr for ParserModuleRoot {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|root| root.as_str() == value)
            .ok_or(())
    }
}
