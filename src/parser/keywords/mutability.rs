use super::ParserMutabilityKeyword;
use std::str::FromStr;

impl ParserMutabilityKeyword {
    const ALL: &[ParserMutabilityKeyword] = &[ParserMutabilityKeyword::Mut];

    const MUT: &'static str = "mut";

    fn as_str(self) -> &'static str {
        match self {
            Self::Mut => Self::MUT,
        }
    }
}

impl FromStr for ParserMutabilityKeyword {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        crate::static_spelling::parse_static_spelling(Self::ALL, value, Self::as_str)
    }
}
