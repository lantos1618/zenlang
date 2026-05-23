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
        Self::ALL
            .iter()
            .copied()
            .find(|keyword| keyword.as_str() == value)
            .ok_or(())
    }
}
