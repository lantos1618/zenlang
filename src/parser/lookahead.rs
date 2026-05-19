use super::*;

impl Parser {
    /// Check if current `{` starts an import: `{ name, ... } = module`.
    pub(super) fn is_import(&self) -> bool {
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
    pub(super) fn is_struct_def(&self) -> bool {
        let mut i = self.pos + 1;
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::LBrace))
    }

    /// After seeing `Name:`, check if this is an enum def (next significant token is an identifier).
    pub(super) fn is_enum_def(&self) -> bool {
        let mut i = self.pos + 1;
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        matches!(
            self.tokens.get(i).map(|(t, _)| t),
            Some(Token::Identifier(_))
        )
    }

    pub(super) fn colon_is_followed_by_identifier(&self, expected: &str) -> bool {
        let mut i = self.pos + 1;
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        matches!(
            self.tokens.get(i).map(|(t, _)| t),
            Some(Token::Identifier(name)) if name == expected
        )
    }

    /// Check if current `{` starts a struct destructuring pattern (not a block body).
    pub(super) fn is_struct_pattern(&self) -> bool {
        let mut i = self.pos + 1;
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
}
