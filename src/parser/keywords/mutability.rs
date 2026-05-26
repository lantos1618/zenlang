use super::ParserMutabilityKeyword;

impl ParserMutabilityKeyword {
    const ALL: &[ParserMutabilityKeyword] = &[ParserMutabilityKeyword::Mut];

    const MUT: &'static str = "mut";

    fn as_str(self) -> &'static str {
        match self {
            Self::Mut => Self::MUT,
        }
    }
}

crate::static_spelling::impl_static_spelling_from_str!(
    ParserMutabilityKeyword,
    variants = ParserMutabilityKeyword::ALL,
    as_str = ParserMutabilityKeyword::as_str
);
