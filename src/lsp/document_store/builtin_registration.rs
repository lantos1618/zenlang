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
        use crate::intrinsics::get_all_intrinsics;

        let intrinsics = get_all_intrinsics();

        for (name, func) in intrinsics {
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
                func.doc, func.category, detail
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
