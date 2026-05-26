// ── Token ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),

    // String interpolation: "hello ${name} world" produces
    //   [StringChunk("hello "), InterpolationStart, Identifier("name"),
    //    InterpolationEnd, StringChunk(" world")]
    StringChunk(String),
    InterpolationStart, // ${
    InterpolationEnd,   // } closing interpolation

    // Operators
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

    // Assignment
    Assign,        // =
    DeclareAssign, // ::=
    ConstAssign,   // :=

    // Punctuation
    Dot,      // .
    DotDot,   // ..
    DotDotEq, // ..=
    Arrow,    // ->
    FatArrow, // =>
    Question, // ?
    Pipe,     // | (pattern / bitwise-or — parser disambiguates)

    // Delimiters
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    // Separators
    Comma,
    Colon,
    Semicolon,

    // Special @ tokens
    AtStd,     // @std
    AtBuiltin, // @builtin
    AtThis,    // @this
    AtExport,  // @export

    // Keyword
    Pub, // pub

    // Control
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

    pub(in crate::lexer) const KEYWORDS: &'static [(&'static str, Self)] = &[("pub", Self::Pub)];

    pub(in crate::lexer) const AT_TOKENS: &'static [(&'static str, Self)] = &[
        (crate::root_spelling::STD_ROOT, Self::AtStd),
        (crate::root_spelling::BUILTIN_ROOT_NAME, Self::AtBuiltin),
        ("this", Self::AtThis),
        ("export", Self::AtExport),
    ];

    pub fn is_eof(&self) -> bool {
        matches!(self, Token::EOF)
    }

    pub(in crate::lexer) fn from_single_char(ch: char) -> Option<Self> {
        Self::SINGLE_CHAR_TOKENS
            .iter()
            .find(|(spelling, _)| *spelling == ch)
            .map(|(_, token)| token.clone())
    }

    pub(in crate::lexer) fn from_keyword(word: &str) -> Option<Self> {
        Self::KEYWORDS
            .iter()
            .find(|(spelling, _)| *spelling == word)
            .map(|(_, token)| token.clone())
    }

    pub(in crate::lexer) fn from_at_name(word: &str) -> Option<Self> {
        Self::AT_TOKENS
            .iter()
            .find(|(spelling, _)| *spelling == word)
            .map(|(_, token)| token.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::Token;

    #[test]
    fn single_char_tokens_round_trip_through_owned_table() {
        for (spelling, token) in Token::SINGLE_CHAR_TOKENS {
            assert_eq!(Token::from_single_char(*spelling), Some(token.clone()));
        }
        assert_eq!(Token::from_single_char('@'), None);
        assert_eq!(Token::from_single_char('\n'), None);
    }

    #[test]
    fn keyword_and_at_tokens_round_trip_through_owned_tables() {
        for (spelling, token) in Token::KEYWORDS {
            assert_eq!(Token::from_keyword(spelling), Some(token.clone()));
        }
        for (spelling, token) in Token::AT_TOKENS {
            assert_eq!(Token::from_at_name(spelling), Some(token.clone()));
        }
        assert_eq!(Token::from_keyword("main"), None);
        assert_eq!(Token::from_at_name("custom"), None);
    }
}
