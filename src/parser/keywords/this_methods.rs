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
        Self::ALL
            .iter()
            .copied()
            .find(|method| method.as_str() == value)
            .ok_or(())
    }
}
