use super::diagnostic::Severity;
use super::diagnostic_code::DiagnosticCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticPhase {
    Unknown,
    Parser,
    Resolver,
    TypeChecker,
    ResolverContract,
    Internal,
}

impl DiagnosticPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Parser => "parser",
            Self::Resolver => "resolver",
            Self::TypeChecker => "typechecker",
            Self::ResolverContract => "resolver_contract",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCategory {
    Unknown,
    Syntax,
    Resolution,
    Type,
    Pattern,
    Generic,
    Behavior,
    ResolverContract,
    Internal,
}

impl DiagnosticCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Syntax => "syntax",
            Self::Resolution => "resolution",
            Self::Type => "type",
            Self::Pattern => "pattern",
            Self::Generic => "generic",
            Self::Behavior => "behavior",
            Self::ResolverContract => "resolver_contract",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticDescriptor {
    pub code: DiagnosticCode,
    pub number: String,
    pub slug: String,
    pub phase: DiagnosticPhase,
    pub category: DiagnosticCategory,
    pub severity: Severity,
    pub docs_path: String,
    pub user_facing: bool,
}

impl DiagnosticDescriptor {
    pub(crate) fn error(
        code: DiagnosticCode,
        number: &str,
        slug: &str,
        phase: DiagnosticPhase,
        category: DiagnosticCategory,
        docs_path: &str,
        user_facing: bool,
    ) -> Self {
        Self {
            code,
            number: number.to_string(),
            slug: slug.to_string(),
            phase,
            category,
            severity: Severity::Error,
            docs_path: docs_path.to_string(),
            user_facing,
        }
    }
}
