pub(crate) fn parse_static_spelling<T: Copy>(
    variants: &[T],
    value: &str,
    as_str: impl Fn(T) -> &'static str,
) -> Result<T, ()> {
    variants
        .iter()
        .copied()
        .find(|variant| as_str(*variant) == value)
        .ok_or(())
}

pub(crate) fn parse_static_spelling_table<T: Copy>(
    spellings: &[(T, &'static str)],
    value: &str,
) -> Result<T, ()> {
    spellings
        .iter()
        .find(|(_, spelling)| *spelling == value)
        .map(|(variant, _)| *variant)
        .ok_or(())
}

macro_rules! impl_static_spelling_from_str {
    ($ty:ty, variants = $variants:path, as_str = $as_str:path) => {
        impl ::std::str::FromStr for $ty {
            type Err = ();

            fn from_str(value: &str) -> Result<Self, ()> {
                $crate::static_spelling::parse_static_spelling($variants, value, $as_str)
            }
        }
    };
    ($ty:ty, table = $spellings:path) => {
        impl ::std::str::FromStr for $ty {
            type Err = ();

            fn from_str(value: &str) -> Result<Self, ()> {
                $crate::static_spelling::parse_static_spelling_table($spellings, value)
            }
        }
    };
}

pub(crate) use impl_static_spelling_from_str;

macro_rules! impl_static_spelling_display {
    ($ty:ty, as_str = $as_str:path) => {
        impl ::std::fmt::Display for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str($as_str(*self))
            }
        }
    };
}

pub(crate) use impl_static_spelling_display;

#[cfg(test)]
pub(crate) fn assert_static_spelling_table_round_trip<T>(
    spellings: &[(T, &'static str)],
    duplicate_label: &str,
) where
    T: Copy + Eq + std::fmt::Debug + std::fmt::Display + std::str::FromStr<Err = ()>,
{
    let mut seen = std::collections::HashSet::new();

    for (variant, spelling) in spellings {
        assert!(
            seen.insert(*spelling),
            "duplicate {duplicate_label} spelling: {spelling}"
        );
        assert_eq!(variant.to_string(), *spelling);
        assert_eq!(spelling.parse::<T>(), Ok(*variant));
    }
}
