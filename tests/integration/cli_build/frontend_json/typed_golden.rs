use super::golden_support::assert_stage_golden;
use crate::support::*;

#[path = "typed_golden/behavior_bounds.rs"]
mod behavior_bounds;
#[path = "typed_golden/generic_enums.rs"]
mod generic_enums;
#[path = "typed_golden/generic_values.rs"]
mod generic_values;
#[path = "typed_golden/methods.rs"]
mod methods;

fn assert_typed_golden(source: &str, golden: &str, description: &str) {
    assert_stage_golden(
        "typed",
        "typed",
        &test_dir().join(source),
        golden,
        description,
    );
}
