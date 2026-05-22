use super::HostEffectResultVariant;
use std::{fmt, str::FromStr};

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

impl fmt::Display for HostEffectResultVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HostEffectResultVariant {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, <Self as FromStr>::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|variant| variant.as_str() == value)
            .ok_or(())
    }
}
