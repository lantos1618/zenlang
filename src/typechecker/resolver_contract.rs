//! Resolver contract checks between semantic analysis and resolver metadata.
//!
//! This is not the core expression/type checking pass. It replays the resolver's
//! symbol, local, import, and metadata expectations against the typechecker view
//! so the two semantic front-end phases stay aligned.
#![allow(clippy::result_large_err)]

#[allow(unused_imports)]
use super::*;

include!("resolver_contract/entry_symbols.rs");
include!("resolver_contract/entry_behavior_associations.rs");
include!("resolver_contract/entry_locals.rs");
include!("resolver_contract/post_pass.rs");
include!("resolver_contract/replay_collection.rs");
include!("resolver_contract/replay_tasks.rs");
include!("resolver_contract/replay_task_association_lists.rs");
include!("resolver_contract/imports_modules.rs");
include!("resolver_contract/imports_graph_seeding.rs");
include!("resolver_contract/imports_dependencies.rs");
include!("resolver_contract/imports_behavior_extends.rs");
include!("resolver_contract/imports_behavior_dependencies.rs");
include!("resolver_contract/imported_generic_behavior_impls.rs");
include!("resolver_contract/imports_source_dependencies.rs");
include!("resolver_contract/imports_source_dependency_types.rs");
include!("resolver_contract/imports_source_dependency_callables.rs");
include!("resolver_contract/imported_method_seeding.rs");
include!("resolver_contract/symbols_locals.rs");
include!("resolver_contract/local_scope_helpers.rs");
include!("resolver_contract/local_traversal.rs");
include!("resolver_contract/pattern_locals.rs");
include!("resolver_contract/metadata_core.rs");
include!("resolver_contract/metadata_absence.rs");
include!("resolver_contract/metadata_diagnostics.rs");
include!("resolver_contract/metadata_types.rs");
include!("resolver_contract/metadata_variants.rs");
include!("resolver_contract/metadata_behavior_refs.rs");
include!("resolver_contract/metadata_values.rs");
