mod spelling;

pub(super) use spelling::CIntrinsic;

crate::static_spelling::impl_static_spelling_display!(CIntrinsic, as_str = CIntrinsic::as_str);
crate::static_spelling::impl_static_spelling_from_str!(CIntrinsic, table = spelling::SPELLINGS);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intrinsic_spellings_round_trip_through_single_table() {
        crate::static_spelling::assert_static_spelling_table_round_trip(
            spelling::SPELLINGS,
            "intrinsic",
        );
    }
}
