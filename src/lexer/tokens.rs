#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    StringChunk(String),
    InterpolationStart, // ${
    InterpolationEnd,   // } closing interpolation

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,    // ==
    NotEq, // !=
    Lt,
    Gt,
    LtEq,       // <=
    GtEq,       // >=
    And,        // &&
    Or,         // ||
    Not,        // !
    BitAnd,     // & (single)
    BitOr,      // (reserved — lexer emits Pipe for single |)
    BitXor,     // ^
    Tilde,      // ~
    ShiftLeft,  // <<
    ShiftRight, // >>

    Assign,        // =
    DeclareAssign, // ::=
    ConstAssign,   // :=

    Dot,      // .
    DotDot,   // ..
    DotDotEq, // ..=
    Arrow,    // ->
    FatArrow, // =>
    Question, // ?
    Pipe,     // | (pattern / bitwise-or — parser disambiguates)

    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    Comma,
    Colon,
    Semicolon,

    AtStd,     // @std
    AtBuiltin, // @builtin
    AtThis,    // @this
    AtExport,  // @export
    AtExtern,  // @extern (C FFI function declaration)

    Newline,
    EOF,
}

impl Token {
    pub(in crate::lexer) const MULTI_CHAR_OPERATORS: &'static [(&'static str, Self)] = &[
        ("::=", Self::DeclareAssign),
        (":=", Self::ConstAssign),
        ("..=", Self::DotDotEq),
        ("..", Self::DotDot),
        ("=>", Self::FatArrow),
        ("->", Self::Arrow),
        ("==", Self::Eq),
        ("!=", Self::NotEq),
        ("<=", Self::LtEq),
        (">=", Self::GtEq),
        ("&&", Self::And),
        ("||", Self::Or),
        ("<<", Self::ShiftLeft),
        (">>", Self::ShiftRight),
    ];

    pub(in crate::lexer) const SINGLE_CHAR_TOKENS: &'static [(char, Self)] = &[
        ('+', Self::Plus),
        ('-', Self::Minus),
        ('*', Self::Star),
        ('/', Self::Slash),
        ('%', Self::Percent),
        ('<', Self::Lt),
        ('>', Self::Gt),
        ('!', Self::Not),
        ('&', Self::BitAnd),
        ('|', Self::Pipe),
        ('^', Self::BitXor),
        ('~', Self::Tilde),
        ('=', Self::Assign),
        ('.', Self::Dot),
        ('?', Self::Question),
        ('(', Self::LParen),
        (')', Self::RParen),
        ('{', Self::LBrace),
        ('}', Self::RBrace),
        ('[', Self::LBracket),
        (']', Self::RBracket),
        (',', Self::Comma),
        (':', Self::Colon),
        (';', Self::Semicolon),
    ];

    pub fn is_eof(&self) -> bool {
        matches!(self, Token::EOF)
    }

    pub(in crate::lexer) fn from_single_char(ch: char) -> Option<Self> {
        Self::SINGLE_CHAR_TOKENS
            .iter()
            .find_map(|(spelling, token)| (*spelling == ch).then(|| token.clone()))
    }

    /// Zen has no hard keywords — every "magic" word is an `@`-directive
    /// (`@std`/`@builtin`/`@this`/`@export`/`@extern`) or a sigil, so a bare
    /// word is always an identifier. Kept as the single guard point for that
    /// invariant (see the keyword-free test).
    pub(in crate::lexer) fn from_keyword(_word: &str) -> Option<Self> {
        None
    }

    pub(in crate::lexer) fn from_at_name(word: &str) -> Option<Self> {
        match word {
            crate::root_spelling::STD_ROOT => Some(Self::AtStd),
            crate::root_spelling::BUILTIN_ROOT_NAME => Some(Self::AtBuiltin),
            "this" => Some(Self::AtThis),
            "export" => Some(Self::AtExport),
            "extern" => Some(Self::AtExtern),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Token;

    /// Zen invariant: there are NO hard keywords. Every reserved/"magic" word is
    /// an `@`-directive or a sigil, so any bare word lexes as an identifier.
    /// `from_keyword` must return `None` for everything — including words that
    /// are keywords in other languages and the ones Zen itself once reserved
    /// (`pub`, `extern`, now `@export`/`@extern`).
    #[test]
    fn zen_has_no_hard_keywords() {
        for word in [
            "pub", "extern", "fn", "let", "const", "mut", "if", "else", "match",
            "loop", "return", "struct", "enum", "behavior", "impl", "type",
            "import", "export", "use", "mod", "pub_", "true", "false", "self",
            "this", "and", "or", "not", "while", "for", "in", "defer", "cast",
            "async", "await",
        ] {
            assert_eq!(
                Token::from_keyword(word),
                None,
                "`{word}` must lex as an identifier — Zen is keyword-free",
            );
        }
    }
}
