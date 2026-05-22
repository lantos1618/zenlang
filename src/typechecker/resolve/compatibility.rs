use crate::ast::typed::Type;

use super::TypeChecker;

impl TypeChecker {
    /// Check if two types are compatible (for assignment/return contexts).
    /// Returns true if the types are clearly compatible or if either is ambiguous.
    /// Returns false only for clear mismatches between concrete primitive types.
    pub(crate) fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }
        // Unknown types are always compatible (error recovery)
        if *expected == Type::Unknown || *actual == Type::Unknown {
            return true;
        }
        // Named/nominal types match only by explicit identity.
        match (expected, actual) {
            (Type::Named(a), Type::Named(b)) if a == b => return true,
            (Type::Struct { name: a, .. }, Type::Struct { name: b, .. }) if a == b => return true,
            (Type::Struct { name, .. }, Type::Named(n))
            | (Type::Named(n), Type::Struct { name, .. })
                if name == n =>
            {
                return true;
            }
            (Type::Enum { name: a, .. }, Type::Enum { name: b, .. }) if a == b => return true,
            (Type::Enum { name, .. }, Type::Named(n))
            | (Type::Named(n), Type::Enum { name, .. })
                if name == n =>
            {
                return true;
            }
            _ => {}
        }
        // Never type is compatible with anything (diverging expression)
        if *expected == Type::Never || *actual == Type::Never {
            return true;
        }
        // Numeric width/sign conversions require explicit casts. Literal
        // coercion is handled before this check at declaration sites.
        false
    }
}
