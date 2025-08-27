use crate::ast::AstType;

/// Check if two types are compatible (for assignment, parameter passing, etc.)
pub fn types_compatible(expected: &AstType, actual: &AstType) -> bool {
    // Exact match is always compatible
    if std::mem::discriminant(expected) == std::mem::discriminant(actual) {
        return true;
    }

    // Check for numeric compatibility with implicit conversions
    if expected.is_numeric() && actual.is_numeric() {
        // Allow widening conversions (smaller to larger)
        if let (Some(expected_size), Some(actual_size)) = (expected.bit_size(), actual.bit_size()) {
            // Allow if actual fits in expected
            if actual_size <= expected_size {
                // Check sign compatibility
                if expected.is_signed_integer() && actual.is_unsigned_integer() {
                    // Unsigned to signed is OK if there's room
                    return actual_size < expected_size;
                }
                return true;
            }
        }
    }

    // Check for pointer compatibility
    match (expected, actual) {
        (AstType::Pointer(expected_inner), AstType::Pointer(actual_inner)) => {
            types_compatible(expected_inner, actual_inner)
        }
        // Allow array to decay to pointer
        (AstType::Pointer(expected_inner), AstType::Array(actual_inner)) => {
            types_compatible(expected_inner, actual_inner)
        }
        // Allow fixed array to decay to pointer
        (AstType::Pointer(expected_inner), AstType::FixedArray { element_type, .. }) => {
            types_compatible(expected_inner, element_type)
        }
        // Check struct compatibility
        (AstType::Struct { name: expected_name, .. }, AstType::Struct { name: actual_name, .. }) => {
            expected_name == actual_name
        }
        // Check enum compatibility
        (AstType::Enum { name: expected_name, .. }, AstType::Enum { name: actual_name, .. }) => {
            expected_name == actual_name
        }
        // Check option compatibility
        (AstType::Option(expected_inner), AstType::Option(actual_inner)) => {
            types_compatible(expected_inner, actual_inner)
        }
        // Allow T to Option<T>
        (AstType::Option(expected_inner), actual) => {
            types_compatible(expected_inner, actual)
        }
        // Check result compatibility
        (
            AstType::Result { ok_type: expected_ok, err_type: expected_err },
            AstType::Result { ok_type: actual_ok, err_type: actual_err },
        ) => {
            types_compatible(expected_ok, actual_ok) && types_compatible(expected_err, actual_err)
        }
        // Check range compatibility
        (
            AstType::Range { start_type: expected_start, end_type: expected_end, .. },
            AstType::Range { start_type: actual_start, end_type: actual_end, .. },
        ) => types_compatible(expected_start, actual_start) && types_compatible(expected_end, actual_end),
        // Function and FunctionPointer compatibility
        (AstType::Function { args: expected_args, return_type: expected_ret }, 
         AstType::FunctionPointer { param_types: actual_params, return_type: actual_ret }) => {
            expected_args.len() == actual_params.len()
                && expected_args.iter().zip(actual_params.iter()).all(|(e, a)| types_compatible(e, a))
                && types_compatible(expected_ret, actual_ret)
        }
        (AstType::FunctionPointer { param_types: expected_params, return_type: expected_ret },
         AstType::Function { args: actual_args, return_type: actual_ret }) => {
            expected_params.len() == actual_args.len()
                && expected_params.iter().zip(actual_args.iter()).all(|(e, a)| types_compatible(e, a))
                && types_compatible(expected_ret, actual_ret)
        }
        // Void is only compatible with void
        (AstType::Void, AstType::Void) => true,
        // All other combinations are incompatible
        _ => false,
    }
}

/// Check if a type can be implicitly converted to another
pub fn can_implicitly_convert(from: &AstType, to: &AstType) -> bool {
    // Same type needs no conversion
    if std::mem::discriminant(from) == std::mem::discriminant(to) {
        return true;
    }

    // Numeric widening conversions
    if from.is_numeric() && to.is_numeric() {
        if let (Some(from_size), Some(to_size)) = (from.bit_size(), to.bit_size()) {
            // Allow widening
            if from_size <= to_size {
                // Check sign compatibility
                if from.is_unsigned_integer() && to.is_signed_integer() {
                    // Unsigned to signed needs extra bit for sign
                    return from_size < to_size;
                }
                return true;
            }
        }
    }

    // Array to pointer decay
    matches!(
        (from, to),
        (AstType::Array(from_elem), AstType::Pointer(to_elem))
            if types_compatible(from_elem, to_elem)
    ) || matches!(
        (from, to),
        (AstType::FixedArray { element_type: from_elem, .. }, AstType::Pointer(to_elem))
            if types_compatible(from_elem, to_elem)
    )
}

/// Check if a type requires explicit initialization
pub fn requires_initialization(type_: &AstType) -> bool {
    match type_ {
        // References must be initialized
        AstType::Ref(_) => true,
        // Immutable values should be initialized
        AstType::Struct { .. } | AstType::Enum { .. } => false,
        // Primitives can have default values
        _ => false,
    }
}

/// Check if a type can be used in a loop condition
pub fn is_valid_condition_type(type_: &AstType) -> bool {
    matches!(type_, AstType::Bool)
        || type_.is_numeric()
        || matches!(type_, AstType::Option(_))
}

/// Check if a type can be indexed
pub fn can_be_indexed(type_: &AstType) -> Option<AstType> {
    match type_ {
        AstType::Array(elem_type) => Some((**elem_type).clone()),
        AstType::FixedArray { element_type, .. } => Some((**element_type).clone()),
        AstType::Pointer(elem_type) => Some((**elem_type).clone()),
        AstType::String => Some(AstType::U8), // Indexing string gives bytes
        _ => None,
    }
}

/// Check if a type supports the dereference operation
pub fn can_be_dereferenced(type_: &AstType) -> Option<AstType> {
    match type_ {
        AstType::Pointer(inner) => Some((**inner).clone()),
        AstType::Ref(inner) => Some((**inner).clone()),
        _ => None,
    }
}