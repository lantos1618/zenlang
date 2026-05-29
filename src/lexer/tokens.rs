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

    Pub,    // pub
    Extern, // extern (C FFI function declaration)

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

    pub(in crate::lexer) fn from_keyword(word: &str) -> Option<Self> {
        match word {
            "pub" => Some(Self::Pub),
            "extern" => Some(Self::Extern),
            _ => None,
        }
    }

    pub(in crate::lexer) fn from_at_name(word: &str) -> Option<Self> {
        match word {
            crate::root_spelling::STD_ROOT => Some(Self::AtStd),
            crate::root_spelling::BUILTIN_ROOT_NAME => Some(Self::AtBuiltin),
            "this" => Some(Self::AtThis),
            "export" => Some(Self::AtExport),
            _ => None,
        }
    }
}
