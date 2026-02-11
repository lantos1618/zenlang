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
    pub fn is_eof(&self) -> bool {
        matches!(self, Token::EOF)
    }
}
