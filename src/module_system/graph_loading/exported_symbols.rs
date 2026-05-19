use crate::resolver::{Namespace, SymbolTable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExportedModuleSymbol {
    Public,
    Private,
    Missing,
}

pub(super) fn exported_module_symbol(symbols: &SymbolTable, name: &str) -> ExportedModuleSymbol {
    let mut found_private = false;

    for namespace in [Namespace::Value, Namespace::Type, Namespace::Behavior] {
        let Some(symbol) = symbols.lookup(namespace, name) else {
            continue;
        };
        if symbol.is_public {
            return ExportedModuleSymbol::Public;
        }
        found_private = true;
    }

    if found_private {
        ExportedModuleSymbol::Private
    } else {
        ExportedModuleSymbol::Missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer, parser, resolver::Resolver};

    fn resolve_symbols(source: &str) -> SymbolTable {
        let tokens = lexer::tokenize(source, 0).expect("lex source");
        let program = parser::parse(tokens, 0).expect("parse source");
        Resolver::new()
            .resolve_program(&program)
            .expect("resolve source")
    }

    #[test]
    fn exported_module_symbol_reads_resolver_public_visibility() {
        let symbols = resolve_symbols(
            r#"
hidden = () i32 { 1 }
pub Model: { value: i32 }
pub Json<T>: behavior {
    encode: (Self) T
}
"#,
        );

        assert_eq!(
            exported_module_symbol(&symbols, "hidden"),
            ExportedModuleSymbol::Private
        );
        assert_eq!(
            exported_module_symbol(&symbols, "Model"),
            ExportedModuleSymbol::Public
        );
        assert_eq!(
            exported_module_symbol(&symbols, "Json"),
            ExportedModuleSymbol::Public
        );
        assert_eq!(
            exported_module_symbol(&symbols, "Missing"),
            ExportedModuleSymbol::Missing
        );
    }

    #[test]
    fn exported_module_symbol_accepts_public_symbol_over_private_symbol_in_other_namespace() {
        let symbols = resolve_symbols(
            r#"
Name = () i32 { 1 }
pub Name: { value: i32 }
"#,
        );

        assert_eq!(
            exported_module_symbol(&symbols, "Name"),
            ExportedModuleSymbol::Public
        );
    }
}
