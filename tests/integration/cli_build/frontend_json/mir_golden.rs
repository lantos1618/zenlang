use super::golden_support::{assert_stage_golden, assert_stage_source_golden, fixture};

#[path = "mir_golden/basic.rs"]
mod basic;
#[path = "mir_golden/behavior_bounds.rs"]
mod behavior_bounds;
#[path = "mir_golden/generic_enums.rs"]
mod generic_enums;
#[path = "mir_golden/generic_methods.rs"]
mod generic_methods;
#[path = "mir_golden/generic_values.rs"]
mod generic_values;

fn assert_mir_golden(source: &str, golden: &str, description: &str) {
    assert_stage_golden("mir", "MIR", &fixture(source), golden, description);
}

fn assert_mir_source_golden(source: &str, filename: &str, golden: &str, description: &str) {
    assert_stage_source_golden("mir", "MIR", source, filename, golden, description);
}
