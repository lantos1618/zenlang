pub(crate) fn static_spelling<T: Copy + PartialEq>(
    spellings: &[(T, &'static str)],
    value: T,
) -> &'static str {
    spellings
        .iter()
        .find_map(|(variant, spelling)| (*variant == value).then_some(*spelling))
        .expect("missing static spelling")
}

macro_rules! impl_static_spelling_from_str {
    ($ty:ty, table = $spellings:path) => {
        impl ::std::str::FromStr for $ty {
            type Err = ();

            fn from_str(value: &str) -> Result<Self, ()> {
                $spellings
                    .iter()
                    .find_map(|(variant, spelling)| (*spelling == value).then_some(*variant))
                    .ok_or(())
            }
        }
    };
}

pub(crate) use impl_static_spelling_from_str;

macro_rules! impl_static_spelling_display {
    ($ty:ty, table = $spellings:path) => {
        impl ::std::fmt::Display for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str($crate::static_spelling::static_spelling($spellings, *self))
            }
        }
    };
}

pub(crate) use impl_static_spelling_display;
