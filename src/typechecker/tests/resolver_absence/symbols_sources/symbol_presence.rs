use super::*;

#[test]
fn resolver_symbol_presence_validation_formats_messages() {
    let extra = ResolverSymbolPresenceValidation {
        code: "EXTRA",
        presence: ResolverSymbolPresence::Extra,
    };
    let missing = ResolverSymbolPresenceValidation {
        code: "MISSING",
        presence: ResolverSymbolPresence::Missing,
    };

    assert_eq!(extra.code, "EXTRA");
    assert_eq!(
        extra.message("value", "main"),
        "resolver symbol table has extra value symbol 'main'"
    );
    assert_eq!(missing.code, "MISSING");
    assert_eq!(
        missing.message("local", "value"),
        "resolver symbol table missing local symbol 'value'"
    );
}

#[test]
fn resolver_symbol_presence_validation_uses_resolver_codes() {
    let missing = ResolverSymbolPresenceValidation::missing_resolver_code();
    let missing_local = ResolverSymbolPresenceValidation::missing_local_resolver_code();
    let extra_declaration = ResolverSymbolPresenceValidation::extra_declaration_resolver_code();
    let extra_local = ResolverSymbolPresenceValidation::extra_local_resolver_code();

    assert_eq!(missing.code, "E0210");
    assert!(matches!(missing.presence, ResolverSymbolPresence::Missing));
    assert_eq!(missing_local.code, "E0228");
    assert!(matches!(
        missing_local.presence,
        ResolverSymbolPresence::Missing
    ));
    assert_eq!(extra_declaration.code, "E0243");
    assert!(matches!(
        extra_declaration.presence,
        ResolverSymbolPresence::Extra
    ));
    assert_eq!(extra_local.code, "E0244");
    assert!(matches!(
        extra_local.presence,
        ResolverSymbolPresence::Extra
    ));
}

#[test]
fn resolver_symbol_presence_validation_pushes_diagnostic() {
    let mut tc = TypeChecker::new();

    tc.validate_resolver_symbol_presence(
        "value",
        "main",
        ResolverSymbolPresenceValidation {
            code: "EXTRA",
            presence: ResolverSymbolPresence::Extra,
        },
        Span::dummy(),
    );

    assert_eq!(tc.diagnostics.len(), 1);
    assert_eq!(tc.diagnostics[0].code, "EXTRA");
    assert_eq!(
        tc.diagnostics[0].message,
        "resolver symbol table has extra value symbol 'main'"
    );
}
