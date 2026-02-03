// Built-in type and compiler intrinsic registration
use super::super::utils::format_type;
use super::utilities::{dummy_range, make_symbol};
use super::DocumentStore;
use crate::ast::{AstType, PRIMITIVE_TYPE_MAP};
use crate::well_known::well_known;
use lsp_types::*;

impl DocumentStore {
    /// Register built-in primitive types that are always available
    pub(super) fn register_builtin_types(&mut self) {
        let range = dummy_range();

        // Register all primitive types using centralized definitions
        for (name, type_) in PRIMITIVE_TYPE_MAP {
            self.stdlib_symbols.insert(
                name.to_string(),
                make_symbol(
                    name.to_string(),
                    SymbolKind::TYPE_PARAMETER,
                    range,
                    Some(format!("{} - Built-in primitive type", name)),
                    Some(format!(
                        "Built-in primitive type `{}`. Always available, no import needed.",
                        name
                    )),
                    Some(type_.clone()),
                ),
            );
        }

        // Also register built-in generic types (Option, Result)
        let wk = well_known();
        for name in [wk.option_name(), wk.result_name()] {
            let type_ = AstType::Generic {
                name: name.to_string(),
                type_args: vec![],
            };
            self.stdlib_symbols.insert(
                name.to_string(),
                make_symbol(
                    name.to_string(),
                    SymbolKind::ENUM,
                    range,
                    Some(format!("{}<T> - Built-in generic type", name)),
                    Some(format!(
                        "Built-in generic type `{}`. Always available, no import needed.",
                        name
                    )),
                    Some(type_),
                ),
            );
        }

        self.register_compiler_intrinsics(&range);
    }

    fn register_compiler_intrinsics(&mut self, range: &Range) {
        use crate::intrinsics::get_intrinsic;

        // (name, description, category)
        let intrinsics: &[(&str, &str, &str)] = &[
            (
                "raw_allocate",
                "Allocates raw memory using malloc",
                "Memory",
            ),
            ("raw_deallocate", "Deallocates memory", "Memory"),
            (
                "raw_reallocate",
                "Reallocates memory to a new size",
                "Memory",
            ),
            (
                "raw_ptr_offset",
                "Offset a pointer by byte count",
                "Pointer",
            ),
            ("raw_ptr_cast", "Reinterprets a pointer type", "Pointer"),
            (
                "gep",
                "GetElementPointer - byte-level pointer arithmetic",
                "Pointer",
            ),
            ("gep_struct", "Struct field access using GEP", "Pointer"),
            ("null_ptr", "Returns a null pointer", "Pointer"),
            ("nullptr", "Alias for null_ptr", "Pointer"),
            ("sizeof", "Returns the size of a type in bytes", "Type"),
            ("alignof", "Returns the alignment of a type", "Type"),
            (
                "discriminant",
                "Reads the discriminant from an enum",
                "Enum",
            ),
            (
                "set_discriminant",
                "Sets the discriminant of an enum",
                "Enum",
            ),
            ("get_payload", "Returns pointer to enum payload", "Enum"),
            ("set_payload", "Copies payload into enum", "Enum"),
            ("load", "Load a value from a pointer", "Memory"),
            ("store", "Store a value to a pointer", "Memory"),
            ("memcpy", "Copy bytes (non-overlapping)", "Memory"),
            ("memmove", "Copy bytes (overlapping safe)", "Memory"),
            ("memset", "Set all bytes to a value", "Memory"),
            ("memcmp", "Compare bytes in memory", "Memory"),
            ("ptr_to_int", "Convert pointer to integer", "Convert"),
            ("int_to_ptr", "Convert integer to pointer", "Convert"),
            ("trunc_f64_i64", "Truncate f64 to i64", "Convert"),
            ("trunc_f32_i32", "Truncate f32 to i32", "Convert"),
            ("sitofp_i64_f64", "Convert signed i64 to f64", "Convert"),
            ("uitofp_u64_f64", "Convert unsigned u64 to f64", "Convert"),
            ("bswap16", "Byte-swap 16-bit value", "Bitwise"),
            ("bswap32", "Byte-swap 32-bit value", "Bitwise"),
            ("bswap64", "Byte-swap 64-bit value", "Bitwise"),
            ("ctlz", "Count leading zeros", "Bitwise"),
            ("cttz", "Count trailing zeros", "Bitwise"),
            ("ctpop", "Population count", "Bitwise"),
            ("atomic_load", "Atomically load a value", "Atomic"),
            ("atomic_store", "Atomically store a value", "Atomic"),
            ("atomic_add", "Atomic add", "Atomic"),
            ("atomic_sub", "Atomic subtract", "Atomic"),
            ("atomic_cas", "Compare-and-swap", "Atomic"),
            ("atomic_xchg", "Atomic exchange", "Atomic"),
            ("fence", "Memory fence", "Atomic"),
            ("add_overflow", "Add with overflow detection", "Overflow"),
            (
                "sub_overflow",
                "Subtract with overflow detection",
                "Overflow",
            ),
            (
                "mul_overflow",
                "Multiply with overflow detection",
                "Overflow",
            ),
            ("unreachable", "Mark code as unreachable", "Debug"),
            ("trap", "Trigger a trap/abort", "Debug"),
            ("debugtrap", "Trigger a debug trap", "Debug"),
            ("inline_c", "Inline C code compilation", "FFI"),
            ("load_library", "Load a dynamic library", "FFI"),
            ("get_symbol", "Get symbol from library", "FFI"),
            ("unload_library", "Unload a dynamic library", "FFI"),
            ("call_external", "Call external function", "FFI"),
        ];

        for &(name, doc, category) in intrinsics {
            if let Some(func) = get_intrinsic(name) {
                let params_str = func
                    .params
                    .iter()
                    .map(|(pname, ptype)| format!("{}: {}", pname, format_type(ptype)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let detail = format!(
                    "@std.compiler.{}({}) -> {}",
                    name,
                    params_str,
                    format_type(&func.return_type)
                );
                let full_doc = format!(
                    "{}\n\n**Category:** {}\n\n**Signature:**\n```zen\n{}\n```",
                    doc, category, detail
                );

                // Register both "compiler.name" and "@std.compiler.name" variants
                for prefix in ["compiler.", "@std.compiler."] {
                    self.stdlib_symbols.insert(
                        format!("{}{}", prefix, name),
                        make_symbol(
                            name.to_string(),
                            SymbolKind::FUNCTION,
                            *range,
                            Some(detail.clone()),
                            Some(full_doc.clone()),
                            Some(func.return_type.clone()),
                        ),
                    );
                }
            }
        }
    }
}
