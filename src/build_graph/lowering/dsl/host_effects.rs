use super::HostEffectResultVariant;

impl HostEffectResultVariant {
    const ALL: &[HostEffectResultVariant] =
        &[HostEffectResultVariant::Ok, HostEffectResultVariant::Err];
    const OK: &'static str = "Ok";
    const ERR: &'static str = "Err";

    pub(in crate::build_graph) fn as_str(self) -> &'static str {
        match self {
            Self::Ok => Self::OK,
            Self::Err => Self::ERR,
        }
    }
}

crate::static_spelling::impl_static_spelling_display!(
    HostEffectResultVariant,
    as_str = HostEffectResultVariant::as_str
);
crate::static_spelling::impl_static_spelling_from_str!(
    HostEffectResultVariant,
    variants = HostEffectResultVariant::ALL,
    as_str = HostEffectResultVariant::as_str
);
