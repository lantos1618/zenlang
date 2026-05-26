use std::fmt;

use super::diagnostic::Severity;
use super::diagnostic_metadata::{DiagnosticCategory, DiagnosticDescriptor, DiagnosticPhase};
use super::resolver_contract_code::ResolverContractCode;
use super::CompilerDiagnosticCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    Syntax,
    Type,
    Resolution,
    Internal,
    Compiler(CompilerDiagnosticCode),
    ResolverContract(ResolverContractCode),
    #[cfg(test)]
    Test(&'static str),
}

impl DiagnosticCode {
    pub const BASE: &'static [Self] = &[Self::Syntax, Self::Type, Self::Resolution, Self::Internal];

    pub fn descriptor(self) -> DiagnosticDescriptor {
        match self {
            Self::Syntax => DiagnosticDescriptor::error(
                self,
                "E2000",
                "syntax",
                DiagnosticPhase::Parser,
                DiagnosticCategory::Syntax,
                "docs/DIAGNOSTICS.md#syntax",
                true,
            ),
            Self::Type => DiagnosticDescriptor::error(
                self,
                "E3000",
                "type",
                DiagnosticPhase::TypeChecker,
                DiagnosticCategory::Type,
                "docs/DIAGNOSTICS.md#type-checking",
                true,
            ),
            Self::Resolution => DiagnosticDescriptor::error(
                self,
                "E3500",
                "resolution",
                DiagnosticPhase::Resolver,
                DiagnosticCategory::Resolution,
                "docs/DIAGNOSTICS.md#resolution",
                true,
            ),
            Self::Internal => DiagnosticDescriptor::error(
                self,
                "E9999",
                "internal",
                DiagnosticPhase::Internal,
                DiagnosticCategory::Internal,
                "docs/DIAGNOSTICS.md#internal",
                false,
            ),
            Self::Compiler(code) => DiagnosticDescriptor {
                code: self,
                number: code.number(),
                slug: code.slug(),
                phase: code.phase(),
                category: code.category(),
                severity: Severity::Error,
                docs_path: code.docs_path().to_string(),
                user_facing: true,
            },
            Self::ResolverContract(code) => DiagnosticDescriptor {
                code: self,
                number: code.number(),
                slug: code.slug(),
                phase: DiagnosticPhase::ResolverContract,
                category: DiagnosticCategory::ResolverContract,
                severity: Severity::Error,
                docs_path: "docs/DIAGNOSTICS.md#resolver-contract".to_string(),
                user_facing: false,
            },
            #[cfg(test)]
            Self::Test(number) => DiagnosticDescriptor::error(
                self,
                number,
                "test",
                DiagnosticPhase::Unknown,
                DiagnosticCategory::Unknown,
                "",
                false,
            ),
        }
    }

    pub fn number(self) -> String {
        self.descriptor().number
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.number())
    }
}

impl From<DiagnosticCode> for String {
    fn from(code: DiagnosticCode) -> Self {
        code.number()
    }
}

impl PartialEq<&str> for DiagnosticCode {
    fn eq(&self, other: &&str) -> bool {
        self.number() == *other
    }
}

impl PartialEq<DiagnosticCode> for &str {
    fn eq(&self, other: &DiagnosticCode) -> bool {
        *self == other.number()
    }
}

impl From<CompilerDiagnosticCode> for DiagnosticCode {
    fn from(code: CompilerDiagnosticCode) -> Self {
        Self::Compiler(code)
    }
}

impl From<ResolverContractCode> for DiagnosticCode {
    fn from(code: ResolverContractCode) -> Self {
        Self::ResolverContract(code)
    }
}

#[cfg(test)]
impl From<&'static str> for DiagnosticCode {
    fn from(code: &'static str) -> Self {
        Self::Test(code)
    }
}
