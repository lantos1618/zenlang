use super::golden_support::{assert_stage_golden, assert_stage_source_golden, fixture};

#[path = "hir_golden/basic.rs"]
mod basic;
#[path = "hir_golden/behavior_bounds.rs"]
mod behavior_bounds;
#[path = "hir_golden/generic_enums.rs"]
mod generic_enums;
#[path = "hir_golden/generic_methods.rs"]
mod generic_methods;
#[path = "hir_golden/generic_values.rs"]
mod generic_values;

fn assert_hir_golden(source: &str, golden: &str, description: &str) {
    assert_stage_golden("hir", "HIR", &fixture(source), golden, description);
}

fn assert_hir_source_golden(source: &str, filename: &str, golden: &str, description: &str) {
    assert_stage_source_golden("hir", "HIR", source, filename, golden, description);
}
