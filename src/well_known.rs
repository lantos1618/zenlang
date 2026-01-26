//! Registry for well-known types that have special compiler semantics.
//!
//! This module centralizes the recognition of types like Option, Result, Ptr, etc.
//! that require special handling in the typechecker and codegen. By using this
//! registry instead of hardcoded string comparisons, we:
//!
//! 1. Enable future self-hosting (a Zen compiler written in Zen can use the same pattern)
//! 2. Make the codebase more maintainable (one source of truth)
//! 3. Allow LSP "Go to definition" to work on stdlib types
//! 4. Allow adding new well-known types without changing compiler code

use std::collections::HashMap;

/// Well-known types that have special compiler semantics.
///
/// IMPORTANT: Only types that REQUIRE compiler support belong here:
/// - Option/Result: Pattern exhaustiveness, ? operator, .raise()
/// - Ptr types: Pointer codegen, dereference, null checks
///
/// Regular stdlib types (Vec, HashMap, String, Range, etc.) do NOT belong here.
/// They are Layer 3 types with no special compiler handling.
/// See docs/design/SEPARATION_OF_CONCERNS.md for the three-layer architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WellKnownType {
    /// Option<T> - nullable type (pattern matching, ? operator)
    Option,
    /// Result<T, E> - error handling type (pattern matching, .raise())
    Result,
    /// Ptr<T> - immutable pointer (dereference codegen)
    Ptr,
    /// MutPtr<T> - mutable pointer (dereference codegen)
    MutPtr,
    /// RawPtr<T> - raw/unsafe pointer (FFI, unsafe codegen)
    RawPtr,
}

/// Well-known enum variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WellKnownVariant {
    /// Option::Some(T)
    Some,
    /// Option::None
    None,
    /// Result::Ok(T)
    Ok,
    /// Result::Err(E)
    Err,
}

/// Registry of well-known types and their variants
#[derive(Debug, Clone)]
pub struct WellKnownTypes {
    /// Map from type name to well-known type
    types: HashMap<String, WellKnownType>,
    /// Map from variant name to (parent type, variant)
    variants: HashMap<String, (WellKnownType, WellKnownVariant)>,
}

impl WellKnownTypes {
    /// Create a new registry with all well-known types registered.
    ///
    /// Only Layer 2 types (Option, Result, Ptr) are registered here.
    /// Layer 3 types (Vec, HashMap, String, etc.) are NOT registered -
    /// they have no special compiler handling.
    pub fn new() -> Self {
        let mut wkt = Self {
            types: HashMap::with_capacity(5),
            variants: HashMap::with_capacity(4),
        };

        // Layer 2: Types requiring special compiler support
        wkt.types.insert("Option".into(), WellKnownType::Option);
        wkt.types.insert("Result".into(), WellKnownType::Result);
        wkt.types.insert("Ptr".into(), WellKnownType::Ptr);
        wkt.types.insert("MutPtr".into(), WellKnownType::MutPtr);
        wkt.types.insert("RawPtr".into(), WellKnownType::RawPtr);

        // Well-known variants (for Option and Result pattern matching)
        wkt.variants.insert(
            "Some".into(),
            (WellKnownType::Option, WellKnownVariant::Some),
        );
        wkt.variants.insert(
            "None".into(),
            (WellKnownType::Option, WellKnownVariant::None),
        );
        wkt.variants
            .insert("Ok".into(), (WellKnownType::Result, WellKnownVariant::Ok));
        wkt.variants
            .insert("Err".into(), (WellKnownType::Result, WellKnownVariant::Err));

        wkt
    }

    // ========================================================================
    // Type checks
    // ========================================================================

    /// Get the well-known type for a name, if any
    #[inline]
    pub fn get_type(&self, name: &str) -> Option<WellKnownType> {
        self.types.get(name).copied()
    }

    /// Check if a type name is Option
    #[inline]
    pub fn is_option(&self, name: &str) -> bool {
        self.get_type(name) == Some(WellKnownType::Option)
    }

    /// Check if a type name is Result
    #[inline]
    pub fn is_result(&self, name: &str) -> bool {
        self.get_type(name) == Some(WellKnownType::Result)
    }

    /// Check if a type name is any pointer type (Ptr, MutPtr, RawPtr)
    #[inline]
    pub fn is_ptr(&self, name: &str) -> bool {
        matches!(
            self.get_type(name),
            Some(WellKnownType::Ptr | WellKnownType::MutPtr | WellKnownType::RawPtr)
        )
    }

    /// Check if a type name is an immutable pointer (Ptr)
    #[inline]
    pub fn is_immutable_ptr(&self, name: &str) -> bool {
        self.get_type(name) == Some(WellKnownType::Ptr)
    }

    /// Check if a type name is a mutable pointer (MutPtr)
    #[inline]
    pub fn is_mutable_ptr(&self, name: &str) -> bool {
        self.get_type(name) == Some(WellKnownType::MutPtr)
    }

    /// Check if a type name is a raw pointer (RawPtr)
    #[inline]
    pub fn is_raw_ptr(&self, name: &str) -> bool {
        self.get_type(name) == Some(WellKnownType::RawPtr)
    }

    /// Check if a type name is Option or Result (types with success/failure variants)
    #[inline]
    #[allow(dead_code)] // Utility for future use
    pub fn is_option_or_result(&self, name: &str) -> bool {
        matches!(
            self.get_type(name),
            Some(WellKnownType::Option | WellKnownType::Result)
        )
    }

    // ========================================================================
    // Variant checks
    // ========================================================================

    /// Get the well-known variant info for a name, if any
    #[inline]
    pub fn get_variant(&self, name: &str) -> Option<(WellKnownType, WellKnownVariant)> {
        self.variants.get(name).copied()
    }

    /// Check if a variant name belongs to Option (Some or None)
    #[inline]
    pub fn is_option_variant(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((WellKnownType::Option, _)))
    }

    /// Check if a variant name belongs to Result (Ok or Err)
    #[inline]
    pub fn is_result_variant(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((WellKnownType::Result, _)))
    }

    /// Check if a variant name is Some
    #[inline]
    pub fn is_some(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((_, WellKnownVariant::Some)))
    }

    /// Check if a variant name is None
    #[inline]
    pub fn is_none(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((_, WellKnownVariant::None)))
    }

    /// Check if a variant name is Ok
    #[inline]
    pub fn is_ok(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((_, WellKnownVariant::Ok)))
    }

    /// Check if a variant name is Err
    #[inline]
    pub fn is_err(&self, name: &str) -> bool {
        matches!(self.get_variant(name), Some((_, WellKnownVariant::Err)))
    }

    /// Get the parent type for a variant
    #[inline]
    pub fn get_variant_parent(&self, variant_name: &str) -> Option<WellKnownType> {
        self.get_variant(variant_name).map(|(parent, _)| parent)
    }

    /// Get the canonical type name for a variant's parent
    #[inline]
    pub fn get_variant_parent_name(&self, variant_name: &str) -> Option<&'static str> {
        self.get_variant_parent(variant_name).map(|t| match t {
            WellKnownType::Option => "Option",
            WellKnownType::Result => "Result",
            WellKnownType::Ptr => "Ptr",
            WellKnownType::MutPtr => "MutPtr",
            WellKnownType::RawPtr => "RawPtr",
        })
    }

    // ========================================================================
    // Canonical name getters (for type construction)
    // ========================================================================

    /// Get the canonical name for Option type
    #[inline]
    pub fn option_name(&self) -> &'static str {
        "Option"
    }

    /// Get the canonical name for Result type
    #[inline]
    pub fn result_name(&self) -> &'static str {
        "Result"
    }

    /// Get the canonical name for Ptr type
    #[inline]
    pub fn ptr_name(&self) -> &'static str {
        "Ptr"
    }

    /// Get the canonical name for MutPtr type
    #[inline]
    pub fn mut_ptr_name(&self) -> &'static str {
        "MutPtr"
    }

    /// Get the canonical name for RawPtr type
    #[inline]
    pub fn raw_ptr_name(&self) -> &'static str {
        "RawPtr"
    }

    // ========================================================================
    // Variant name getters (for passing to get_variant_parent_name etc)
    // ========================================================================

    /// Get the canonical name for Some variant
    #[inline]
    pub fn some_name(&self) -> &'static str {
        "Some"
    }

    /// Get the canonical name for None variant
    #[inline]
    pub fn none_name(&self) -> &'static str {
        "None"
    }

    /// Get the canonical name for Ok variant
    #[inline]
    pub fn ok_name(&self) -> &'static str {
        "Ok"
    }

    /// Get the canonical name for Err variant
    #[inline]
    pub fn err_name(&self) -> &'static str {
        "Err"
    }

    /// Get discriminant tag for a variant (for codegen)
    /// Returns 0 for success variants (Some, Ok), 1 for failure variants (None, Err)
    #[inline]
    #[allow(dead_code)] // Utility for future codegen use
    pub fn get_variant_tag(&self, variant_name: &str) -> Option<u64> {
        match self.get_variant(variant_name) {
            Some((_, WellKnownVariant::Some)) => Some(0),
            Some((_, WellKnownVariant::Ok)) => Some(0),
            Some((_, WellKnownVariant::None)) => Some(1),
            Some((_, WellKnownVariant::Err)) => Some(1),
            None => None,
        }
    }
}

impl Default for WellKnownTypes {
    fn default() -> Self {
        Self::new()
    }
}

/// Global static instance for use in parser and other contexts where
/// threading through parameters is impractical
pub fn well_known() -> &'static WellKnownTypes {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<WellKnownTypes> = OnceLock::new();
    INSTANCE.get_or_init(WellKnownTypes::new)
}
