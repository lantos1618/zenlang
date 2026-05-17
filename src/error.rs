use std::fmt;

use serde::Serialize;

mod file_table;

pub use file_table::{FileId, FileTable};

// ── Span ───────────────────────────────────────────────────────────

/// Byte range within a specific file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    pub file_id: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(file_id: FileId, start: u32, end: u32) -> Self {
        Self {
            file_id,
            start,
            end,
        }
    }

    /// A zero-width span at file 0 — useful for tests and synthetic nodes.
    pub fn dummy() -> Self {
        Self {
            file_id: 0,
            start: 0,
            end: 0,
        }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Merge two spans (must be in the same file). Takes the min start and max end.
    pub fn merge(&self, other: Span) -> Span {
        debug_assert_eq!(self.file_id, other.file_id);
        Span {
            file_id: self.file_id,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

// ── Severity ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Hint,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Hint => write!(f, "hint"),
            Severity::Info => write!(f, "info"),
        }
    }
}

// ── Label ──────────────────────────────────────────────────────────

/// A secondary annotation pointing at a span with a message.
#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

impl Label {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }
}

// ── Context Frames ────────────────────────────────────────────────

/// What kind of context led to this diagnostic.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextKind {
    InFunction,
    InModule,
    InGenericInstantiation,
    InTraitImpl,
    InImport,
    InMacroExpansion,
}

/// A frame in the context stack showing how we reached the error.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextFrame {
    pub span: Span,
    pub kind: ContextKind,
    pub message: String,
}

// ── Diagnostic ─────────────────────────────────────────────────────

/// A single compiler diagnostic — shared across all phases.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub span: Option<Span>,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub context: Vec<ContextFrame>,
}

impl Diagnostic {
    /// Create an error diagnostic.
    pub fn error(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            span: Some(span),
            labels: Vec::new(),
            notes: Vec::new(),
            context: Vec::new(),
        }
    }

    /// Create a warning diagnostic.
    pub fn warning(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.into(),
            message: message.into(),
            span: Some(span),
            labels: Vec::new(),
            notes: Vec::new(),
            context: Vec::new(),
        }
    }

    /// Add a secondary label.
    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label::new(span, message));
        self
    }

    /// Add a help/note string.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a context frame showing how we reached this diagnostic.
    pub fn with_context(mut self, frame: ContextFrame) -> Self {
        self.context.push(frame);
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]: {}", self.severity, self.code, self.message)?;
        for frame in &self.context {
            write!(f, "\n   = {}", frame.message)?;
        }
        Ok(())
    }
}

// ── CompileError ───────────────────────────────────────────────────

/// Simple error enum for use in `Result<T, CompileError>`.
/// For richer reporting, convert to `Diagnostic`.
#[derive(Debug, Clone)]
pub enum CompileError {
    Syntax(String, Option<Span>),
    Type(String, Option<Span>),
    Resolution(String, Option<Span>),
    Internal(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Syntax(msg, _) => write!(f, "syntax error: {msg}"),
            CompileError::Type(msg, _) => write!(f, "type error: {msg}"),
            CompileError::Resolution(msg, _) => write!(f, "resolution error: {msg}"),
            CompileError::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<CompileError> for Diagnostic {
    fn from(err: CompileError) -> Self {
        match err {
            CompileError::Syntax(msg, span) => Diagnostic {
                severity: Severity::Error,
                code: "E2000".to_string(),
                message: msg,
                span,
                labels: Vec::new(),
                notes: Vec::new(),
                context: Vec::new(),
            },
            CompileError::Type(msg, span) => Diagnostic {
                severity: Severity::Error,
                code: "E3000".to_string(),
                message: msg,
                span,
                labels: Vec::new(),
                notes: Vec::new(),
                context: Vec::new(),
            },
            CompileError::Resolution(msg, span) => Diagnostic {
                severity: Severity::Error,
                code: "E3500".to_string(),
                message: msg,
                span,
                labels: Vec::new(),
                notes: Vec::new(),
                context: Vec::new(),
            },
            CompileError::Internal(msg) => Diagnostic {
                severity: Severity::Error,
                code: "E9999".to_string(),
                message: msg,
                span: None,
                labels: Vec::new(),
                notes: Vec::new(),
                context: Vec::new(),
            },
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_table_add_and_lookup() {
        let mut ft = FileTable::new();
        let id = ft.add_file("test.zen".into(), "hello\nworld\n".into());
        assert_eq!(id, 0);
        assert_eq!(ft.get_source(id), Some("hello\nworld\n"));
        assert_eq!(ft.get_path(id), Some("test.zen"));
    }

    #[test]
    fn file_table_multiple_files() {
        let mut ft = FileTable::new();
        let a = ft.add_file("a.zen".into(), "aaa".into());
        let b = ft.add_file("b.zen".into(), "bbb".into());
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(ft.get_source(a), Some("aaa"));
        assert_eq!(ft.get_source(b), Some("bbb"));
    }

    #[test]
    fn file_table_invalid_id() {
        let ft = FileTable::new();
        assert_eq!(ft.get_source(99), None);
        assert_eq!(ft.get_path(99), None);
    }

    #[test]
    fn line_col_simple() {
        let mut ft = FileTable::new();
        // "hello\nworld\n"
        //  01234 5 678...
        let id = ft.add_file("t.zen".into(), "hello\nworld\n".into());
        assert_eq!(ft.line_col(id, 0), Some((0, 0))); // 'h'
        assert_eq!(ft.line_col(id, 4), Some((0, 4))); // 'o'
        assert_eq!(ft.line_col(id, 6), Some((1, 0))); // 'w'
        assert_eq!(ft.line_col(id, 10), Some((1, 4))); // 'd'
    }

    #[test]
    fn span_dummy() {
        let s = Span::dummy();
        assert_eq!(s.file_id, 0);
        assert_eq!(s.start, 0);
        assert_eq!(s.end, 0);
        assert!(s.is_empty());
    }

    #[test]
    fn span_merge() {
        let a = Span::new(0, 5, 10);
        let b = Span::new(0, 8, 15);
        let merged = a.merge(b);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn diagnostic_error_constructor() {
        let d = Diagnostic::error("E1001", "unterminated string", Span::new(0, 5, 10));
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.code, "E1001");
        assert!(d.is_error());
    }

    #[test]
    fn diagnostic_warning_constructor() {
        let d = Diagnostic::warning("W3001", "unused variable", Span::new(0, 0, 3));
        assert_eq!(d.severity, Severity::Warning);
        assert!(!d.is_error());
    }

    #[test]
    fn diagnostic_builder_chain() {
        let d = Diagnostic::error("E3001", "type mismatch", Span::new(0, 10, 20))
            .with_label(Span::new(0, 30, 40), "expected i32 here")
            .with_note("try casting with `as i32`");
        assert_eq!(d.labels.len(), 1);
        assert_eq!(d.notes.len(), 1);
        assert_eq!(d.labels[0].message, "expected i32 here");
    }

    #[test]
    fn diagnostic_display() {
        let d = Diagnostic::error("E1001", "bad token", Span::dummy());
        assert_eq!(format!("{d}"), "error[E1001]: bad token");
    }

    #[test]
    fn compile_error_display() {
        let e = CompileError::Syntax("unexpected token".into(), None);
        assert_eq!(format!("{e}"), "syntax error: unexpected token");

        let e = CompileError::Internal("oops".into());
        assert_eq!(format!("{e}"), "internal error: oops");
    }

    #[test]
    fn compile_error_to_diagnostic() {
        let e = CompileError::Type("mismatch".into(), Some(Span::new(1, 5, 10)));
        let d: Diagnostic = e.into();
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.code, "E3000");
        assert_eq!(d.span, Some(Span::new(1, 5, 10)));
    }
}
