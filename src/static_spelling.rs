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
