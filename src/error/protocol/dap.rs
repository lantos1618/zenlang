use serde::Serialize;

use super::agent::{diagnostics_for_ai, AgentDiagnostic};
use crate::error::{Diagnostic, FileTable};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DapLaunchFailure {
    pub format: &'static str,
    pub event: &'static str,
    pub body: DapOutputBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DapOutputBody {
    pub category: &'static str,
    pub output: String,
    pub data: DapDiagnosticBundle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DapDiagnosticBundle {
    pub format: &'static str,
    pub diagnostics: Vec<AgentDiagnostic>,
}

pub fn dap_launch_failure(diagnostics: &[Diagnostic], files: &FileTable) -> DapLaunchFailure {
    DapLaunchFailure {
        format: "zen.dap.diagnostics.v1",
        event: "output",
        body: DapOutputBody {
            category: "stderr",
            output: diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
            data: DapDiagnosticBundle {
                format: "zen.diagnostics.v1",
                diagnostics: diagnostics_for_ai(diagnostics, files),
            },
        },
    }
}
