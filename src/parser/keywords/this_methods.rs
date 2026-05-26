use super::ParserThisMethod;

impl ParserThisMethod {
    const ALL: &[ParserThisMethod] = &[ParserThisMethod::Defer];

    const DEFER: &'static str = "defer";

    fn as_str(self) -> &'static str {
        match self {
            Self::Defer => Self::DEFER,
        }
    }
}

crate::static_spelling::impl_static_spelling_from_str!(
    ParserThisMethod,
    variants = ParserThisMethod::ALL,
    as_str = ParserThisMethod::as_str
);
