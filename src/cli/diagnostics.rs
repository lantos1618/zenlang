use zen::error::{CompileError, Diagnostic, FileTable, Severity};

pub(super) fn print_errors(errs: &[CompileError], files: &FileTable) {
    for err in errs {
        let diag: Diagnostic = err.clone().into();
        print_diagnostic(&diag, files);
    }
}

pub(super) fn print_diagnostic(diag: &Diagnostic, files: &FileTable) {
    let severity = match diag.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Hint => "hint",
    };

    if let Some(span) = diag.span {
        let path = files.get_path(span.file_id).unwrap_or("<unknown>");
        if let Some((line, col)) = files.line_col(span.file_id, span.start) {
            eprintln!(
                "{}:{}:{}: {}: {}",
                path,
                line + 1,
                col + 1,
                severity,
                diag.message
            );
        } else {
            eprintln!("{}: {}: {}", path, severity, diag.message);
        }
    } else {
        eprintln!("{}: {}", severity, diag.message);
    }

    for label in &diag.labels {
        let path = files.get_path(label.span.file_id).unwrap_or("<unknown>");
        if let Some((line, col)) = files.line_col(label.span.file_id, label.span.start) {
            eprintln!("  --> {}:{}:{}: {}", path, line + 1, col + 1, label.message);
        }
    }
}
