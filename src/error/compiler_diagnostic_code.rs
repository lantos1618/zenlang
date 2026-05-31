use super::{DiagnosticCategory, DiagnosticPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum CompilerDiagnosticCode {
    E0200 = 200,
    E0201 = 201,
    E0202 = 202,
    E0203 = 203,
    E0204 = 204,
    E0205 = 205,
    E0206 = 206,
    E0207 = 207,
    E0208 = 208,
    E0209 = 209,
    E0210 = 210,
    E0211 = 211,
    E0212 = 212,
    E0213 = 213,
    E0214 = 214,
    E0215 = 215,
    E0216 = 216,
    E0217 = 217,
    E0232 = 232,
    E2000 = 2000,
    E3010 = 3010,
    E3011 = 3011,
    E3012 = 3012,
    E3013 = 3013,
    E3020 = 3020,
    E3021 = 3021,
    E3022 = 3022,
    E3023 = 3023,
    E3030 = 3030,
    E3031 = 3031,
    E3034 = 3034,
    E3035 = 3035,
    E3036 = 3036,
    E3037 = 3037,
    E3040 = 3040,
    E3043 = 3043,
    E3051 = 3051,
    E3052 = 3052,
    E3054 = 3054,
    E3055 = 3055,
    E3056 = 3056,
    E3060 = 3060,
    E3061 = 3061,
    E3062 = 3062,
    E3063 = 3063,
    E3064 = 3064,
    E3070 = 3070,
    E3071 = 3071,
    E3072 = 3072,
    E3073 = 3073,
    E3080 = 3080,
    E3081 = 3081,
    E3082 = 3082,
    E3500 = 3500,
    E4000 = 4000,
    E4001 = 4001,
    E4002 = 4002,
    E4003 = 4003,
    E4004 = 4004,
    E4005 = 4005,
    E4006 = 4006,
    E5000 = 5000,
    E5001 = 5001,
    E5002 = 5002,
    E6001 = 6001,
    E6002 = 6002,
    E6003 = 6003,
    E6004 = 6004,
    E6005 = 6005,
    E6006 = 6006,
    E6007 = 6007,
    E6008 = 6008,
    E6009 = 6009,
    E6010 = 6010,
    E6011 = 6011,
    E6013 = 6013,
    E9999 = 9999,
}

impl CompilerDiagnosticCode {
    pub(crate) fn slug(self) -> String {
        match self {
            Self::E2000 => "syntax".to_string(),
            Self::E3500 => "resolution".to_string(),
            Self::E9999 => "internal".to_string(),
            _ => format!("{}_e{:04}", self.category().as_str(), self as u16),
        }
    }

    pub(crate) fn phase(self) -> DiagnosticPhase {
        match self as u16 {
            2000 => DiagnosticPhase::Parser,
            200..=299 | 3500 => DiagnosticPhase::Resolver,
            9999 => DiagnosticPhase::Internal,
            _ => DiagnosticPhase::TypeChecker,
        }
    }

    pub(crate) fn category(self) -> DiagnosticCategory {
        match self as u16 {
            2000 => DiagnosticCategory::Syntax,
            200..=299 | 3500 => DiagnosticCategory::Resolution,
            4000..=4999 => DiagnosticCategory::Pattern,
            5000..=5999 => DiagnosticCategory::Generic,
            6000..=6999 => DiagnosticCategory::Behavior,
            9999 => DiagnosticCategory::Internal,
            _ => DiagnosticCategory::Type,
        }
    }

    pub(crate) fn docs_path(self) -> &'static str {
        match self.category() {
            DiagnosticCategory::Syntax => "docs/DIAGNOSTICS.md#syntax",
            DiagnosticCategory::Resolution => "docs/DIAGNOSTICS.md#resolution",
            DiagnosticCategory::Pattern => "docs/DIAGNOSTICS.md#type-checking",
            DiagnosticCategory::Generic => "docs/DIAGNOSTICS.md#json-stable-codes",
            DiagnosticCategory::Behavior => "docs/DIAGNOSTICS.md#json-stable-codes",
            DiagnosticCategory::Type => "docs/DIAGNOSTICS.md#type-checking",
            DiagnosticCategory::Internal => "docs/DIAGNOSTICS.md#internal",
        }
    }
}
