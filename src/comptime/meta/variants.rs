// AST variant name constants for Zen compile-time scope.
// Exposed as meta.Expression, meta.Statement, etc. so Zen code
// can write `meta.Expression.BinaryOp` instead of `"BinaryOp"`.

use crate::comptime::values::ComptimeValue;

/// Helper: build a struct of variant name constants (each field maps name -> name string)
fn variant_constants(name: &str, variants: &[&str]) -> ComptimeValue {
    ComptimeValue::Struct {
        name: name.to_string(),
        fields: variants
            .iter()
            .map(|v| (v.to_string(), ComptimeValue::String(v.to_string())))
            .collect(),
    }
}

pub fn expression_variants() -> ComptimeValue {
    variant_constants(
        "Expression",
        &[
            "Integer8",
            "Integer16",
            "Integer32",
            "Integer64",
            "Unsigned8",
            "Unsigned16",
            "Unsigned32",
            "Unsigned64",
            "Float32",
            "Float64",
            "Boolean",
            "String",
            "Identifier",
            "Unit",
            "BinaryOp",
            "FunctionCall",
            "QuestionMatch",
            "Conditional",
            "AddressOf",
            "Dereference",
            "PointerOffset",
            "StructLiteral",
            "StructField",
            "ArrayLiteral",
            "ArrayIndex",
            "EnumVariant",
            "EnumLiteral",
            "MemberAccess",
            "PointerDereference",
            "PointerAddress",
            "CreateReference",
            "CreateMutableReference",
            "StringLength",
            "Some",
            "None",
            "StringInterpolation",
            "Comptime",
            "Range",
            "PatternMatch",
            "StdReference",
            "BuiltinReference",
            "ThisReference",
            "MethodCall",
            "Loop",
            "CollectionLoop",
            "Closure",
            "Block",
            "Return",
            "Raise",
            "Defer",
            "Break",
            "Continue",
            "VecConstructor",
            "DynVecConstructor",
            "ArrayConstructor",
        ],
    )
}

pub fn statement_variants() -> ComptimeValue {
    variant_constants(
        "Statement",
        &[
            "Expression",
            "Return",
            "VariableDeclaration",
            "VariableAssignment",
            "PointerAssignment",
            "Loop",
            "Break",
            "Continue",
            "ComptimeBlock",
            "ModuleImport",
            "Defer",
            "ThisDefer",
            "DestructuringImport",
            "Block",
        ],
    )
}

pub fn declaration_variants() -> ComptimeValue {
    variant_constants(
        "Declaration",
        &[
            "Function",
            "ExternalFunction",
            "Struct",
            "Enum",
            "Behavior",
            "Trait",
            "TraitImplementation",
            "TraitRequirement",
            "ImplBlock",
            "ComptimeBlock",
            "Constant",
            "ModuleImport",
            "Export",
            "TypeAlias",
        ],
    )
}

pub fn type_variants() -> ComptimeValue {
    variant_constants(
        "AstType",
        &[
            "I8",
            "I16",
            "I32",
            "I64",
            "U8",
            "U16",
            "U32",
            "U64",
            "Usize",
            "F32",
            "F64",
            "Bool",
            "StaticLiteral",
            "StaticString",
            "Void",
            "Slice",
            "FixedArray",
            "Function",
            "FunctionPointer",
            "Struct",
            "Enum",
            "Ref",
            "Range",
            "Generic",
            "EnumType",
            "StdModule",
        ],
    )
}

pub fn pattern_variants() -> ComptimeValue {
    variant_constants(
        "Pattern",
        &[
            "Literal",
            "Identifier",
            "Struct",
            "EnumVariant",
            "Wildcard",
            "EnumLiteral",
            "Or",
            "Tuple",
            "Range",
            "Binding",
            "Type",
            "Guard",
        ],
    )
}
