use super::Span;

#[derive(Debug, Clone)]
pub struct RelatedDiagnostic {
    pub span: Span,
    pub message: String,
}

impl RelatedDiagnostic {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticFact {
    pub key: String,
    pub value: String,
}

impl DiagnosticFact {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

pub(super) fn raw_slug(code: &str) -> String {
    let mut slug = String::with_capacity("raw_".len() + code.len());
    slug.push_str("raw_");
    slug.extend(code.chars().map(|ch| ch.to_ascii_lowercase()));
    slug
}
