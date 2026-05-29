use std::fmt;
use std::str::FromStr;

mod spelling;

pub(super) use spelling::CIntrinsic;

impl fmt::Display for CIntrinsic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CIntrinsic {
    type Err = ();

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        spelling::SPELLINGS
            .iter()
            .find(|(_, spelling)| *spelling == name)
            .map(|(intrinsic, _)| *intrinsic)
            .ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_c_backend_intrinsic_exists_in_the_registry() {
        // The registry in `crate::intrinsics` is the single source of truth.
        // Every spelling the C backend can emit must name a real intrinsic
        // there, or the backend table has drifted from the registry.
        for (_, spelling) in spelling::SPELLINGS {
            assert!(
                crate::intrinsics::get_intrinsic(spelling).is_some(),
                "C-backend intrinsic `{spelling}` is not in the intrinsic registry (crate::intrinsics)"
            );
        }
    }

    #[test]
    fn intrinsic_spellings_round_trip_through_single_table() {
        let mut seen = HashSet::new();

        for (intrinsic, spelling) in spelling::SPELLINGS {
            assert!(
                seen.insert(*spelling),
                "duplicate intrinsic spelling: {spelling}"
            );
            assert_eq!(intrinsic.as_str(), *spelling);
            assert_eq!(intrinsic.to_string(), *spelling);
            assert_eq!(spelling.parse::<CIntrinsic>(), Ok(*intrinsic));
        }
    }
}
