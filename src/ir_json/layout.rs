use crate::ast::typed::TypedProgram;

mod context;
#[path = "layout/metrics.rs"]
mod metrics;
#[path = "layout/schema.rs"]
mod schema;

use context::LayoutContext;
use metrics::{POINTER_ALIGN, POINTER_SIZE, USIZE_SIZE};
use schema::{LayoutJsonProgram, LayoutJsonTarget};

pub(super) fn program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    let context = LayoutContext::new(program);
    let graph = LayoutJsonProgram {
        format: "zen.layout.v0",
        schema_version: 0,
        semantic_status: "checked",
        target: LayoutJsonTarget {
            pointer_size: POINTER_SIZE,
            pointer_alignment: POINTER_ALIGN,
            usize_size: USIZE_SIZE,
        },
        layouts: context.layouts(),
    };

    serde_json::to_string_pretty(&graph)
}
