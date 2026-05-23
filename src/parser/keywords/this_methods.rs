use super::ParserThisMethod;
use std::str::FromStr;

impl ParserThisMethod {
    const ALL: &[ParserThisMethod] = &[ParserThisMethod::Defer];

    const DEFER: &'static str = "defer";

    fn as_str(self) -> &'static str {
        match self {
            Self::Defer => Self::DEFER,
        }
    }
}

impl FromStr for ParserThisMethod {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        crate::static_spelling::parse_static_spelling(Self::ALL, value, Self::as_str)
    }
}
