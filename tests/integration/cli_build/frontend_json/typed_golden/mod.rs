use super::golden_support::{assert_stage_golden, stage_golden_path};
use crate::support::*;
mod behavior_bounds;
mod generic_enums;
mod generic_values;
mod methods;

fn assert_typed_golden(source: &str, golden_stem: &str, description: &str) {
    let golden = stage_golden_path("typed", golden_stem);
    assert_stage_golden(
        "typed",
        "typed",
        &test_dir().join(source),
        &golden,
        description,
    );
}
