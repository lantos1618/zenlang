mod scan;
mod strings;
mod tokens;

pub use tokens::Token;

use crate::error::{CompileError, FileId, Span};

// ── Lexer ─────────────────────────────────────────────────────────

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    file_id: FileId,
    /// byte_offsets[i] = byte offset of source char i.
    /// Sentinel at [source.len()] = total byte length.
    byte_offsets: Vec<u32>,
    /// Buffered tokens from string interpolation.
    pending: Vec<(Token, Span)>,
}

impl Lexer {
    pub fn new(source: &str, file_id: FileId) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let byte_offsets = Self::build_byte_offsets(source, chars.len());
        Self {
            source: chars,
            pos: 0,
            file_id,
            byte_offsets,
            pending: Vec::new(),
        }
    }

    /// Tokenise the entire source, returning tokens paired with spans.
    pub fn tokenize(&mut self) -> Result<Vec<(Token, Span)>, CompileError> {
        let mut tokens = Vec::new();
        loop {
            let (tok, span) = self.next_token()?;
            let done = tok.is_eof();
            tokens.push((tok, span));
            if done {
                break;
            }
        }
        Ok(tokens)
    }

    /// Return the next token (drains pending buffer first).
    pub fn next_token(&mut self) -> Result<(Token, Span), CompileError> {
        // Drain buffered tokens from string interpolation
        if !self.pending.is_empty() {
            return Ok(self.pending.remove(0));
        }
        self.lex_next()
    }

    // ── Character helpers ────────────────────────────────────────

    fn build_byte_offsets(source: &str, char_count: usize) -> Vec<u32> {
        let mut offsets = Vec::with_capacity(char_count + 1);
        for (byte_idx, _) in source.char_indices() {
            offsets.push(byte_idx as u32);
        }
        offsets.push(source.len() as u32); // sentinel for EOF
        offsets
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.source.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn advance_n(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.source.len());
    }

    fn byte_pos(&self) -> u32 {
        self.byte_offsets[self.pos.min(self.source.len())]
    }

    fn make_span(&self, start: u32, end: u32) -> Span {
        Span::new(self.file_id, start, end)
    }

    /// Check whether the source at `self.pos` starts with `s`.
    fn matches(&self, s: &str) -> bool {
        for (i, expected) in s.chars().enumerate() {
            match self.source.get(self.pos + i) {
                Some(&ch) if ch == expected => {}
                _ => return false,
            }
        }
        true
    }
}

// ── Convenience function ──────────────────────────────────────────

/// Tokenise source code into a list of (Token, Span) pairs.
pub fn tokenize(source: &str, file_id: FileId) -> Result<Vec<(Token, Span)>, CompileError> {
    Lexer::new(source, file_id).tokenize()
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Tokenise and return just token variants, filtering out Newline/EOF.
    fn toks(src: &str) -> Vec<Token> {
        tokenize(src, 0)
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .filter(|t| !matches!(t, Token::Newline | Token::EOF))
            .collect()
    }

    /// Tokenise and return ALL tokens including Newline/EOF.
    fn toks_all(src: &str) -> Vec<Token> {
        tokenize(src, 0)
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect()
    }

    // ── Delimiters & separators ──────────────────────────────────

    #[test]
    fn delimiters() {
        assert_eq!(
            toks("(){}[]"),
            vec![
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::RBrace,
                Token::LBracket,
                Token::RBracket,
            ]
        );
    }

    #[test]
    fn separators() {
        assert_eq!(
            toks(",;:"),
            vec![Token::Comma, Token::Semicolon, Token::Colon]
        );
    }

    // ── Identifiers & keywords ───────────────────────────────────

    #[test]
    fn identifiers_and_pub() {
        assert_eq!(
            toks("hello pub world _foo _"),
            vec![
                Token::Identifier("hello".into()),
                Token::Pub,
                Token::Identifier("world".into()),
                Token::Identifier("_foo".into()),
                Token::Identifier("_".into()),
            ]
        );
    }

    // ── Numbers ──────────────────────────────────────────────────

    #[test]
    fn integers() {
        assert_eq!(
            toks("42 0 1_000"),
            vec![
                Token::IntLiteral(42),
                Token::IntLiteral(0),
                Token::IntLiteral(1000)
            ]
        );
    }

    #[test]
    fn floats() {
        assert_eq!(
            toks("3.14 0.0"),
            vec![Token::FloatLiteral(3.14), Token::FloatLiteral(0.0)]
        );
    }

    #[test]
    fn hex_binary_octal() {
        assert_eq!(
            toks("0xFF 0b1010 0o777 0xDE_AD"),
            vec![
                Token::IntLiteral(0xFF),
                Token::IntLiteral(0b1010),
                Token::IntLiteral(0o777),
                Token::IntLiteral(0xDEAD),
            ]
        );
    }

    // ── Strings ──────────────────────────────────────────────────

    #[test]
    fn plain_string() {
        assert_eq!(
            toks(r#""hello world""#),
            vec![Token::StringLiteral("hello world".into())]
        );
    }

    #[test]
    fn escape_sequences() {
        assert_eq!(
            toks(r#""\n\t\\\"""#),
            vec![Token::StringLiteral("\n\t\\\"".into())]
        );
    }

    #[test]
    fn hex_escape() {
        assert_eq!(toks(r#""\x41""#), vec![Token::StringLiteral("A".into())]);
    }

    #[test]
    fn null_escape() {
        assert_eq!(toks(r#""\0""#), vec![Token::StringLiteral("\0".into())]);
    }

    #[test]
    fn escaped_dollar_no_interpolation() {
        assert_eq!(
            toks(r#""\${not interpolated}""#),
            vec![Token::StringLiteral("${not interpolated}".into())]
        );
    }

    #[test]
    fn string_interpolation_simple() {
        assert_eq!(
            toks(r#""hello ${name}!""#),
            vec![
                Token::StringChunk("hello ".into()),
                Token::InterpolationStart,
                Token::Identifier("name".into()),
                Token::InterpolationEnd,
                Token::StringChunk("!".into()),
            ]
        );
    }

    #[test]
    fn string_interpolation_expr() {
        assert_eq!(
            toks(r#""result = ${a + b}""#),
            vec![
                Token::StringChunk("result = ".into()),
                Token::InterpolationStart,
                Token::Identifier("a".into()),
                Token::Plus,
                Token::Identifier("b".into()),
                Token::InterpolationEnd,
            ]
        );
    }

    #[test]
    fn string_interpolation_call() {
        assert_eq!(
            toks(r#""${f(x)}""#),
            vec![
                Token::InterpolationStart,
                Token::Identifier("f".into()),
                Token::LParen,
                Token::Identifier("x".into()),
                Token::RParen,
                Token::InterpolationEnd,
            ]
        );
    }

    #[test]
    fn string_interpolation_multiple() {
        assert_eq!(
            toks(r#""${a} and ${b}""#),
            vec![
                Token::InterpolationStart,
                Token::Identifier("a".into()),
                Token::InterpolationEnd,
                Token::StringChunk(" and ".into()),
                Token::InterpolationStart,
                Token::Identifier("b".into()),
                Token::InterpolationEnd,
            ]
        );
    }

    // ── Operators ────────────────────────────────────────────────

    #[test]
    fn arithmetic_operators() {
        assert_eq!(
            toks("+ - * / %"),
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Percent
            ]
        );
    }

    #[test]
    fn comparison_operators() {
        assert_eq!(
            toks("== != < > <= >="),
            vec![
                Token::Eq,
                Token::NotEq,
                Token::Lt,
                Token::Gt,
                Token::LtEq,
                Token::GtEq
            ]
        );
    }

    #[test]
    fn logical_operators() {
        assert_eq!(toks("&& || !"), vec![Token::And, Token::Or, Token::Not]);
    }

    #[test]
    fn bitwise_operators() {
        assert_eq!(
            toks("& ^ ~ << >>"),
            vec![
                Token::BitAnd,
                Token::BitXor,
                Token::Tilde,
                Token::ShiftLeft,
                Token::ShiftRight
            ]
        );
    }

    #[test]
    fn assignment_operators() {
        assert_eq!(
            toks("= := ::="),
            vec![Token::Assign, Token::ConstAssign, Token::DeclareAssign]
        );
    }

    // ── Punctuation ──────────────────────────────────────────────

    #[test]
    fn dot_operators() {
        assert_eq!(
            toks(". .. ..="),
            vec![Token::Dot, Token::DotDot, Token::DotDotEq]
        );
    }

    #[test]
    fn arrows() {
        assert_eq!(toks("-> =>"), vec![Token::Arrow, Token::FatArrow]);
    }

    #[test]
    fn pipe_and_question() {
        assert_eq!(
            toks("| ? ||"),
            vec![Token::Pipe, Token::Question, Token::Or]
        );
    }

    // ── @ tokens ─────────────────────────────────────────────────

    #[test]
    fn at_tokens() {
        assert_eq!(
            toks("@std @builtin @this @export"),
            vec![
                Token::AtStd,
                Token::AtBuiltin,
                Token::AtThis,
                Token::AtExport
            ]
        );
    }

    #[test]
    fn unknown_at_token() {
        assert_eq!(toks("@custom"), vec![Token::Identifier("@custom".into())]);
    }

    // ── Newlines ─────────────────────────────────────────────────

    #[test]
    fn newlines_are_tokens() {
        assert_eq!(
            toks_all("a\nb\n"),
            vec![
                Token::Identifier("a".into()),
                Token::Newline,
                Token::Identifier("b".into()),
                Token::Newline,
                Token::EOF,
            ]
        );
    }

    // ── Comments ─────────────────────────────────────────────────

    #[test]
    fn line_comment() {
        assert_eq!(
            toks_all("a // comment\nb"),
            vec![
                Token::Identifier("a".into()),
                Token::Newline,
                Token::Identifier("b".into()),
                Token::EOF,
            ]
        );
    }

    #[test]
    fn block_comment() {
        assert_eq!(
            toks("a /* comment */ b"),
            vec![Token::Identifier("a".into()), Token::Identifier("b".into())]
        );
    }

    #[test]
    fn nested_block_comments() {
        assert_eq!(
            toks("a /* outer /* inner */ still */ b"),
            vec![Token::Identifier("a".into()), Token::Identifier("b".into())]
        );
    }

    // ── Spans ────────────────────────────────────────────────────

    #[test]
    fn spans_basic() {
        let tokens = tokenize("ab cd", 1).unwrap();
        assert_eq!(tokens[0].1, Span::new(1, 0, 2)); // "ab"
        assert_eq!(tokens[1].1, Span::new(1, 3, 5)); // "cd"
    }

    #[test]
    fn spans_multichar_operator() {
        let tokens = tokenize("::=", 0).unwrap();
        assert_eq!(tokens[0].0, Token::DeclareAssign);
        assert_eq!(tokens[0].1, Span::new(0, 0, 3));
    }

    // ── Integration-style ────────────────────────────────────────

    #[test]
    fn zen_function_def() {
        assert_eq!(
            toks("add = (a: i32, b: i32) i32 {"),
            vec![
                Token::Identifier("add".into()),
                Token::Assign,
                Token::LParen,
                Token::Identifier("a".into()),
                Token::Colon,
                Token::Identifier("i32".into()),
                Token::Comma,
                Token::Identifier("b".into()),
                Token::Colon,
                Token::Identifier("i32".into()),
                Token::RParen,
                Token::Identifier("i32".into()),
                Token::LBrace,
            ]
        );
    }

    #[test]
    fn zen_import() {
        assert_eq!(
            toks("{ io } = std"),
            vec![
                Token::LBrace,
                Token::Identifier("io".into()),
                Token::RBrace,
                Token::Assign,
                Token::Identifier("std".into()),
            ]
        );
    }

    #[test]
    fn zen_declare_assign() {
        assert_eq!(
            toks("i ::= 0"),
            vec![
                Token::Identifier("i".into()),
                Token::DeclareAssign,
                Token::IntLiteral(0)
            ]
        );
    }

    #[test]
    fn zen_const_assign() {
        assert_eq!(
            toks("x := 42"),
            vec![
                Token::Identifier("x".into()),
                Token::ConstAssign,
                Token::IntLiteral(42)
            ]
        );
    }

    #[test]
    fn zen_ufc_chain() {
        assert_eq!(
            toks("5.double().add_ten()"),
            vec![
                Token::IntLiteral(5),
                Token::Dot,
                Token::Identifier("double".into()),
                Token::LParen,
                Token::RParen,
                Token::Dot,
                Token::Identifier("add_ten".into()),
                Token::LParen,
                Token::RParen,
            ]
        );
    }

    #[test]
    fn zen_pattern_match() {
        assert_eq!(
            toks("x ?\n    | true { 1 }\n    | false { 0 }"),
            vec![
                Token::Identifier("x".into()),
                Token::Question,
                Token::Pipe,
                Token::Identifier("true".into()),
                Token::LBrace,
                Token::IntLiteral(1),
                Token::RBrace,
                Token::Pipe,
                Token::Identifier("false".into()),
                Token::LBrace,
                Token::IntLiteral(0),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn float_vs_range() {
        assert_eq!(toks("3.14"), vec![Token::FloatLiteral(3.14)]);
        assert_eq!(
            toks("3..10"),
            vec![Token::IntLiteral(3), Token::DotDot, Token::IntLiteral(10)]
        );
    }

    #[test]
    fn method_call_on_int() {
        // 5.double() — int then dot then ident
        assert_eq!(
            toks("5.double()"),
            vec![
                Token::IntLiteral(5),
                Token::Dot,
                Token::Identifier("double".into()),
                Token::LParen,
                Token::RParen,
            ]
        );
    }

    #[test]
    fn empty_source() {
        assert_eq!(toks_all(""), vec![Token::EOF]);
    }

    #[test]
    fn only_whitespace() {
        assert_eq!(toks_all("   \t  "), vec![Token::EOF]);
    }

    #[test]
    fn consecutive_newlines() {
        assert_eq!(
            toks_all("\n\n"),
            vec![Token::Newline, Token::Newline, Token::EOF]
        );
    }
}
