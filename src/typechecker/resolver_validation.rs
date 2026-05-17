//! Resolver-symbol validation against resolver metadata.
#![allow(clippy::result_large_err)]

#[allow(unused_imports)]
use super::*;

include!("resolver_validation/entry_symbols.rs");
include!("resolver_validation/replay_tasks.rs");
include!("resolver_validation/imports_modules.rs");
include!("resolver_validation/imports_dependencies.rs");
include!("resolver_validation/imports_behavior_dependencies.rs");
include!("resolver_validation/imports_source_dependencies.rs");
include!("resolver_validation/symbols_locals.rs");
include!("resolver_validation/local_traversal.rs");
include!("resolver_validation/metadata_core.rs");
include!("resolver_validation/metadata_diagnostics.rs");
include!("resolver_validation/metadata_types.rs");
include!("resolver_validation/metadata_behavior_refs.rs");
include!("resolver_validation/metadata_values.rs");
