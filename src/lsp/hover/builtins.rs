// Built-in types hover information
//
// This module provides hover text by querying compiler metadata sources:
// - intrinsics.rs: Compiler intrinsics (@std.compiler.*)
// - intrinsics.rs: Option, Result, Ptr types (well-known types)
// - stdlib_types.rs: Parsed stdlib definitions (Vec, HashMap, String, etc.)
//
// Only truly primitive types and keywords are hardcoded here.

use crate::ast::AstType;
use crate::intrinsics::{get_intrinsic, Intrinsic};
use crate::intrinsics::{well_known, WellKnownType};
use crate::stdlib_types::stdlib_types;

/// Get hover text for built-in types and keywords
pub fn get_builtin_hover_text(symbol_name: &str) -> Option<String> {
    // 1. Check compiler intrinsics first (authoritative source)
    if let Some(intrinsic) = get_intrinsic(symbol_name) {
        return Some(format_intrinsic_hover(symbol_name, intrinsic));
    }

    // 2. Check well-known types (Option, Result, Ptr)
    if let Some(wkt) = well_known().get_type(symbol_name) {
        return Some(format_well_known_hover(wkt));
    }

    // 3. Check stdlib parsed types (Vec, HashMap, String, etc.)
    if let Some(struct_def) = stdlib_types().get_struct_definition(symbol_name) {
        return Some(format_stdlib_struct_hover(struct_def));
    }

    // 4. Primitives and keywords (truly static - OK to hardcode)
    get_primitive_or_keyword_hover(symbol_name)
}

/// Format hover for a compiler intrinsic from intrinsics.rs
fn format_intrinsic_hover(name: &str, intrinsic: &Intrinsic) -> String {
    let params = intrinsic
        .params
        .iter()
        .map(|(param_name, param_type)| format!("{}: {}", param_name, format_ast_type(param_type)))
        .collect::<Vec<_>>()
        .join(", ");

    let return_type = format_ast_type(&intrinsic.return_type);

    // Check if it's a generic intrinsic (sizeof, alignof, load, store)
    let generic_suffix =
        if name == "sizeof" || name == "alignof" || name == "load" || name == "store" {
            "<T>"
        } else {
            ""
        };

    // Use the doc string from intrinsics registry (single source of truth)
    let description = intrinsic.doc;

    format!(
        "```zen\n@std.compiler.{}{}({}) -> {}\n```\n\n**Compiler intrinsic** ({})\n\n{}",
        name, generic_suffix, params, return_type, intrinsic.category, description
    )
}

/// Format hover for well-known types (Option, Result, Ptr)
fn format_well_known_hover(wkt: WellKnownType) -> String {
    match wkt {
        WellKnownType::Option => "```zen\nOption<T>:\n    Some: T,\n    None\n```\n\n\
             **Optional value type**\n\n\
             - Represents a value that may or may not exist\n\
             - Use `?` for pattern matching: `opt? | Some(v) { ... } | None { ... }`\n\
             - No null in Zen - use Option instead"
            .to_string(),
        WellKnownType::Result => "```zen\nResult<T, E>:\n    Ok: T,\n    Err: E\n```\n\n\
             **Result type for error handling**\n\n\
             - Represents success (Ok) or failure (Err)\n\
             - Use `.raise()` for error propagation (like Rust's `?`)\n\
             - Pattern match with `?` operator"
            .to_string(),
        WellKnownType::Ptr => "```zen\nPtr<T>\n```\n\n\
             **Immutable pointer**\n\n\
             - Safe, non-null pointer to T\n\
             - Use `.deref()` to read the value\n\
             - Cannot be reassigned after creation"
            .to_string(),
        WellKnownType::MutPtr => "```zen\nMutPtr<T>\n```\n\n\
             **Mutable pointer**\n\n\
             - Safe, non-null pointer to T\n\
             - Use `.val` to read/write the value\n\
             - Required for mutating struct fields"
            .to_string(),
        WellKnownType::RawPtr => "```zen\nRawPtr<T>\n```\n\n\
             **Raw/unsafe pointer**\n\n\
             - Can be null - check with `compiler.is_null()`\n\
             - Used for FFI and low-level memory operations\n\
             - No safety guarantees"
            .to_string(),
    }
}

/// Format hover for stdlib struct from parsed .zen files
fn format_stdlib_struct_hover(struct_def: &crate::ast::StructDefinition) -> String {
    let fields = struct_def
        .fields
        .iter()
        .map(|f| format!("    {}: {}", f.name, format_ast_type(&f.type_)))
        .collect::<Vec<_>>()
        .join(",\n");

    let generic_params = if struct_def.type_params.is_empty() {
        String::new()
    } else {
        let params: Vec<_> = struct_def
            .type_params
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        format!("<{}>", params.join(", "))
    };

    format!(
        "```zen\n{}{}: {{\n{}\n}}\n```\n\n**Stdlib type**",
        struct_def.name, generic_params, fields
    )
}

/// Format AstType for display
fn format_ast_type(ty: &AstType) -> String {
    // Use the Display impl which handles all cases correctly
    format!("{}", ty)
}

/// Get hover for truly primitive types and keywords (OK to hardcode)
fn get_primitive_or_keyword_hover(symbol_name: &str) -> Option<String> {
    let text = match symbol_name {
        // Primitive integer types
        "i8" => "```zen\ni8\n```\n\n**Signed 8-bit integer**\n- Range: -128 to 127\n- Size: 1 byte",
        "i16" => "```zen\ni16\n```\n\n**Signed 16-bit integer**\n- Range: -32,768 to 32,767\n- Size: 2 bytes",
        "i32" => "```zen\ni32\n```\n\n**Signed 32-bit integer**\n- Range: -2,147,483,648 to 2,147,483,647\n- Size: 4 bytes",
        "i64" => "```zen\ni64\n```\n\n**Signed 64-bit integer**\n- Range: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807\n- Size: 8 bytes",
        "u8" => "```zen\nu8\n```\n\n**Unsigned 8-bit integer**\n- Range: 0 to 255\n- Size: 1 byte",
        "u16" => "```zen\nu16\n```\n\n**Unsigned 16-bit integer**\n- Range: 0 to 65,535\n- Size: 2 bytes",
        "u32" => "```zen\nu32\n```\n\n**Unsigned 32-bit integer**\n- Range: 0 to 4,294,967,295\n- Size: 4 bytes",
        "u64" => "```zen\nu64\n```\n\n**Unsigned 64-bit integer**\n- Range: 0 to 18,446,744,073,709,551,615\n- Size: 8 bytes",
        "usize" => "```zen\nusize\n```\n\n**Pointer-sized unsigned integer**\n- Size: Platform dependent (4 or 8 bytes)\n- Used for array indexing and memory offsets",

        // Floating point types
        "f32" => "```zen\nf32\n```\n\n**32-bit floating point**\n- Precision: ~7 decimal digits\n- Size: 4 bytes\n- IEEE 754 single precision",
        "f64" => "```zen\nf64\n```\n\n**64-bit floating point**\n- Precision: ~15 decimal digits\n- Size: 8 bytes\n- IEEE 754 double precision",

        // Boolean
        "bool" => "```zen\nbool\n```\n\n**Boolean type**\n- Values: `true` or `false`\n- Size: 1 byte",

        // Void
        "void" => "```zen\nvoid\n```\n\n**Void type**\n- Represents the absence of a value\n- Used as return type for functions with no return",

        // Keywords
        "loop" => "```zen\nloop { ... }\nloop (i in range) { ... }\n```\n\n**Loop construct**\n- Infinite loop or iterator-based\n- Use `break` to exit, `continue` to skip",
        "return" => "```zen\nreturn expr\n```\n\n**Return statement**\n- Returns a value from a function\n- Type must match function return type",
        "break" => "```zen\nbreak\n```\n\n**Break statement**\n- Exits the current loop immediately",
        "continue" => "```zen\ncontinue\n```\n\n**Continue statement**\n- Skips to the next loop iteration",

        // Error handling keyword
        "raise" => "```zen\nexpr.raise()\n```\n\n**Error propagation**\n- Unwraps Result<T, E> or returns Err early\n- Equivalent to Rust's `?` operator",

        _ => return None,
    };

    Some(text.to_string())
}
