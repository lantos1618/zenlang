use super::golden_support::{
    assert_stage_golden, assert_stage_source_golden, fixture, stage_golden_path,
};
mod basic;
mod behavior_bounds;
mod generic_enums;
mod generic_methods;
mod generic_values;

fn assert_mir_golden(source: &str, golden_stem: &str, description: &str) {
    let golden = stage_golden_path("mir", golden_stem);
    assert_stage_golden("mir", "MIR", &fixture(source), &golden, description);
}

fn assert_mir_source_golden(source: &str, filename: &str, golden_stem: &str, description: &str) {
    let golden = stage_golden_path("mir", golden_stem);
    assert_stage_source_golden("mir", "MIR", source, filename, &golden, description);
}
