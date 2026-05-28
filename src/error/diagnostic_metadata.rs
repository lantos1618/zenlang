#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticPhase {
    Parser,
    Resolver,
    TypeChecker,
    Internal,
}

impl DiagnosticPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Parser => "parser",
            Self::Resolver => "resolver",
            Self::TypeChecker => "typechecker",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCategory {
    Syntax,
    Resolution,
    Type,
    Pattern,
    Generic,
    Behavior,
    Internal,
}

impl DiagnosticCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Syntax => "syntax",
            Self::Resolution => "resolution",
            Self::Type => "type",
            Self::Pattern => "pattern",
            Self::Generic => "generic",
            Self::Behavior => "behavior",
            Self::Internal => "internal",
        }
    }
}
