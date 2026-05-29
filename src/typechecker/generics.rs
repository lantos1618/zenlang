//! Generic specialization (monomorphization).
//!
//! On-demand, call-site driven: a generic function/struct/enum is specialized
//! to concrete types the first time a call site or type reference needs it,
//! deduplicated by a mangled key. See `docs/SEMA_DESIGN.md` for the intended
//! end-state (memoized queries replacing the hand-maintained dedup maps).
pub(crate) mod monomorphize;
pub(crate) mod monomorphize_dependencies;
pub(crate) mod monomorphize_inference;
pub(crate) mod monomorphize_inference_shapes;
pub(crate) mod monomorphize_method_self;
pub(crate) mod monomorphize_names;
pub(crate) mod monomorphize_specialized_type_names;
pub(crate) mod monomorphize_specialized_type_refs;
pub(crate) mod monomorphize_specialized_types;
pub(crate) mod monomorphize_types;
