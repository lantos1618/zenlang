//! Type inference module - broken into logical submodules for maintainability

pub mod binary_ops;
pub mod calls;
pub mod casts;
pub mod closures;
pub mod enums;
pub mod helpers;
pub mod identifiers;
pub mod member_access;
pub mod result_ops;

// Re-export all public functions for backward compatibility
pub use binary_ops::{infer_binary_op_type, promote_numeric_types, types_comparable};
pub use calls::{infer_function_call_type, infer_method_call_type};
pub use casts::infer_cast_type;
pub use closures::infer_closure_type;
pub use enums::{infer_enum_literal_type, infer_enum_variant_type};
pub use helpers::{extract_type_name, is_string_type};
pub use identifiers::infer_identifier_type;
pub use member_access::{infer_member_type, infer_struct_field_type};
pub use result_ops::infer_raise_type;
