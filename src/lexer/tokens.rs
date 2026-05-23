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

    pub fn is_eof(&self) -> bool {
        matches!(self, Token::EOF)
    }

    pub(in crate::lexer) fn from_single_char(ch: char) -> Option<Self> {
        match ch {
            '+' => Some(Self::Plus),
            '-' => Some(Self::Minus),
            '*' => Some(Self::Star),
            '/' => Some(Self::Slash),
            '%' => Some(Self::Percent),
            '<' => Some(Self::Lt),
            '>' => Some(Self::Gt),
            '!' => Some(Self::Not),
            '&' => Some(Self::BitAnd),
            '|' => Some(Self::Pipe),
            '^' => Some(Self::BitXor),
            '~' => Some(Self::Tilde),
            '=' => Some(Self::Assign),
            '.' => Some(Self::Dot),
            '?' => Some(Self::Question),
            '(' => Some(Self::LParen),
            ')' => Some(Self::RParen),
            '{' => Some(Self::LBrace),
            '}' => Some(Self::RBrace),
            '[' => Some(Self::LBracket),
            ']' => Some(Self::RBracket),
            ',' => Some(Self::Comma),
            ':' => Some(Self::Colon),
            ';' => Some(Self::Semicolon),
            _ => None,
        }
    }
}
