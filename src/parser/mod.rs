use crate::ast::declarations::{Declaration, EnumVariant, StructField, TypeParam};
use crate::ast::expressions::{
    BinaryOp, Expression, LoopControlAction, MatchArm, StringPart, UnaryOp,
};
use crate::ast::patterns::Pattern;
use crate::ast::statements::Statement;
use crate::ast::types::{AstType, Param};
use crate::ast::Program;
use crate::error::{CompileError, FileId, Span};
use crate::lexer::Token;

mod atoms;
mod behavior_declarations;
mod block_helpers;
mod declarations;
mod expressions;
mod patterns;
mod statements;
mod types;

#[cfg(test)]
mod tests;

// ── Public API ────────────────────────────────────────────────────

/// Parse a token stream into a Program (list of declarations).
pub fn parse(tokens: Vec<(Token, Span)>, file_id: FileId) -> Result<Program, Vec<CompileError>> {
    let mut parser = Parser::new(tokens, file_id);
    let decls = parser.parse_program();
    if parser.errors.is_empty() {
        Ok(Program {
            declarations: decls,
            file_id,
        })
    } else {
        Err(parser.errors)
    }
}

// ── Parser ────────────────────────────────────────────────────────

struct Parser {
    tokens: Vec<(Token, Span)>,
    pos: usize,
    #[allow(dead_code)]
    file_id: FileId,
    errors: Vec<CompileError>,
    loop_controls: Vec<(String, String)>,
    next_loop_control_id: usize,
}

impl Parser {
    fn new(tokens: Vec<(Token, Span)>, file_id: FileId) -> Self {
        Self {
            tokens,
            pos: 0,
            file_id,
            errors: Vec::new(),
            loop_controls: Vec::new(),
            next_loop_control_id: 0,
        }
    }

    // ── Token navigation ──────────────────────────────────────

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|(t, _)| t)
            .unwrap_or(&Token::EOF)
    }

    fn loop_control_label(&self, name: &str) -> Option<String> {
        self.loop_controls
            .iter()
            .rev()
            .find(|(control_name, _)| control_name == name)
            .map(|(_, label)| label.clone())
    }

    fn fresh_loop_control_label(&mut self) -> String {
        let id = self.next_loop_control_id;
        self.next_loop_control_id += 1;
        format!("__zen_loop_{}", id)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|(_, s)| *s)
            .unwrap_or_else(Span::dummy)
    }

    /// Peek at the next non-newline token without consuming.
    fn peek_skip_newlines(&self) -> &Token {
        let mut i = self.pos;
        loop {
            match self.tokens.get(i) {
                Some((Token::Newline, _)) => i += 1,
                Some((t, _)) => return t,
                None => return &Token::EOF,
            }
        }
    }

    /// Look ahead n significant (non-newline) tokens.
    fn peek_ahead(&self, n: usize) -> &Token {
        let mut count = 0;
        let mut i = self.pos;
        loop {
            match self.tokens.get(i) {
                Some((Token::Newline, _)) => i += 1,
                Some((t, _)) => {
                    if count == n {
                        return t;
                    }
                    count += 1;
                    i += 1;
                }
                None => return &Token::EOF,
            }
        }
    }

    fn advance(&mut self) -> (Token, Span) {
        let entry = self
            .tokens
            .get(self.pos)
            .cloned()
            .unwrap_or((Token::EOF, Span::dummy()));
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        entry
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<Span, CompileError> {
        self.skip_newlines();
        let (tok, span) = self.advance();
        if std::mem::discriminant(&tok) == std::mem::discriminant(expected) {
            Ok(span)
        } else {
            Err(CompileError::Syntax(
                format!("expected {:?}, found {:?}", expected, tok),
                Some(span),
            ))
        }
    }

    /// Expect a `>` token, splitting `>>` (ShiftRight) if needed for nested generics.
    fn expect_gt(&mut self) -> Result<Span, CompileError> {
        self.skip_newlines();
        let (tok, span) = self
            .tokens
            .get(self.pos)
            .cloned()
            .unwrap_or((Token::EOF, Span::dummy()));
        match tok {
            Token::Gt => {
                self.pos += 1;
                Ok(span)
            }
            Token::ShiftRight => {
                // Split `>>` into `>` + `>`: consume first `>`, leave second in stream
                let first_span = Span::new(span.file_id, span.start, span.start + 1);
                let second_span = Span::new(span.file_id, span.start + 1, span.end);
                self.tokens[self.pos] = (Token::Gt, second_span);
                Ok(first_span)
            }
            _ => Err(CompileError::Syntax(
                format!("expected `>`, found {:?}", tok),
                Some(span),
            )),
        }
    }

    fn expect_identifier(&mut self) -> Result<(String, Span), CompileError> {
        self.skip_newlines();
        let (tok, span) = self.advance();
        match tok {
            Token::Identifier(name) => Ok((name, span)),
            _ => Err(CompileError::Syntax(
                format!("expected identifier, found {:?}", tok),
                Some(span),
            )),
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Token::EOF)
    }

    /// Skip tokens until we find something that looks like a new declaration.
    fn synchronize(&mut self) {
        loop {
            match self.peek() {
                Token::EOF => return,
                Token::Newline => {
                    self.advance();
                    // After newlines, check if next token starts a declaration
                    match self.peek() {
                        Token::Identifier(_) | Token::Pub | Token::LBrace | Token::EOF => return,
                        _ => {}
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    // ── Top-level: Program ────────────────────────────────────

    fn parse_program(&mut self) -> Vec<Declaration> {
        let mut decls = Vec::new();
        loop {
            self.skip_newlines();
            if self.at_eof() {
                break;
            }
            match self.parse_declaration() {
                Ok(decl) => decls.push(decl),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }
        decls
    }

    // ── Lookahead helpers ─────────────────────────────────────

    /// Check if current `{` starts an import: `{ name, ... } = module`
    fn is_import(&self) -> bool {
        // Walk past `{`, look for `}` then `=`
        let mut i = self.pos + 1;
        let mut depth = 1u32;
        loop {
            match self.tokens.get(i).map(|(t, _)| t) {
                Some(Token::LBrace) => {
                    depth += 1;
                    i += 1;
                }
                Some(Token::RBrace) => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        // Skip newlines
                        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
                            i += 1;
                        }
                        return matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Assign));
                    }
                    i += 1;
                }
                Some(Token::EOF) | None => return false,
                _ => i += 1,
            }
        }
    }

    /// After seeing `Name:`, check if this is a struct def (next significant token is `{`).
    fn is_struct_def(&self) -> bool {
        let mut i = self.pos + 1; // skip `:`
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::LBrace))
    }

    /// After seeing `Name:`, check if this is an enum def (next significant token is an identifier).
    fn is_enum_def(&self) -> bool {
        let mut i = self.pos + 1; // skip `:`
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        matches!(
            self.tokens.get(i).map(|(t, _)| t),
            Some(Token::Identifier(_))
        )
    }

    fn colon_is_followed_by_identifier(&self, expected: &str) -> bool {
        let mut i = self.pos + 1; // skip `:`
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        matches!(
            self.tokens.get(i).map(|(t, _)| t),
            Some(Token::Identifier(name)) if name == expected
        )
    }

    /// Check if current `{` starts a struct destructuring pattern (not a block body).
    fn is_struct_pattern(&self) -> bool {
        let mut i = self.pos + 1; // skip `{`
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        match self.tokens.get(i).map(|(t, _)| t) {
            Some(Token::Identifier(_)) => {
                i += 1;
                while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
                    i += 1;
                }
                matches!(
                    self.tokens.get(i).map(|(t, _)| t),
                    Some(Token::Comma) | Some(Token::Colon)
                )
            }
            _ => false,
        }
    }

    // ── Helpers ───────────────────────────────────────────────

    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].1
        } else {
            Span::dummy()
        }
    }

    /// Skip newlines only if the next non-newline token is a continuation
    /// (operator, dot, etc.) — prevents consuming newlines that separate statements.
    fn skip_newlines_if_continuation(&mut self) {
        if !matches!(self.peek(), Token::Newline) {
            return;
        }
        // Peek at what comes after newlines
        let next = self.peek_skip_newlines();
        match next {
            // Binary operators that continue an expression
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Percent
            | Token::Eq
            | Token::NotEq
            | Token::Lt
            | Token::Gt
            | Token::LtEq
            | Token::GtEq
            | Token::And
            | Token::Or
            | Token::BitAnd
            | Token::BitXor
            | Token::ShiftLeft
            | Token::ShiftRight
            | Token::Dot
            | Token::Question
            | Token::Pipe
            | Token::DotDot
            | Token::DotDotEq
            | Token::LBracket => {
                self.skip_newlines();
            }
            _ => {} // Don't skip — let caller handle newline as statement separator
        }
    }
}

// ── Statement/Expression disambiguation ───────────────────────

enum StmtOrExpr {
    Stmt(Statement),
    Expr(Expression),
}

// ── Binding power tables (Pratt parser) ───────────────────────

/// Infix operator binding powers (left, right).
fn infix_bp(token: &Token) -> Option<(u8, u8)> {
    match token {
        Token::Or => Some((1, 2)),
        Token::And => Some((3, 4)),
        Token::Pipe => Some((5, 6)), // bitwise or
        Token::BitXor => Some((7, 8)),
        Token::BitAnd => Some((9, 10)),
        Token::Eq | Token::NotEq => Some((11, 12)),
        Token::Lt | Token::Gt | Token::LtEq | Token::GtEq => Some((13, 14)),
        Token::ShiftLeft | Token::ShiftRight => Some((15, 16)),
        Token::Plus | Token::Minus => Some((17, 18)),
        Token::Star | Token::Slash | Token::Percent => Some((19, 20)),
        _ => None,
    }
}

/// Postfix binding power for `.`, `[`, `(`.
fn postfix_bp() -> (u8, u8) {
    (23, 24)
}

/// Prefix binding power for `-`, `!`, `~`.
fn prefix_bp() -> u8 {
    21
}

fn first_char_is_upper(s: &str) -> bool {
    s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
}
