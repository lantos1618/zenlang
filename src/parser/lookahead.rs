use super::*;

impl Parser {
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
                        i = self.next_non_newline(i + 1);
                        return matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Assign));
                    }
                    i += 1;
                }
                Some(Token::EOF) | None => return false,
                _ => i += 1,
            }
        }
    }

    pub(super) fn is_struct_def(&self) -> bool {
        let i = self.next_non_newline(self.pos + 1);
        matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::LBrace))
    }

    pub(super) fn is_enum_def(&self) -> bool {
        let i = self.next_non_newline(self.pos + 1);
        matches!(
            self.tokens.get(i).map(|(t, _)| t),
            Some(Token::Identifier(_))
        )
    }

    pub(super) fn colon_is_followed_by_identifier(&self, expected: &str) -> bool {
        let i = self.next_non_newline(self.pos + 1);
        matches!(
            self.tokens.get(i).map(|(t, _)| t),
            Some(Token::Identifier(name)) if name == expected
        )
    }

    pub(super) fn is_struct_pattern(&self) -> bool {
        let mut i = self.next_non_newline(self.pos + 1);
        match self.tokens.get(i).map(|(t, _)| t) {
            Some(Token::Identifier(_)) => {
                i = self.next_non_newline(i + 1);
                matches!(
                    self.tokens.get(i).map(|(t, _)| t),
                    Some(Token::Comma) | Some(Token::Colon)
                )
            }
            _ => false,
        }
    }

    fn next_non_newline(&self, mut i: usize) -> usize {
        while matches!(self.tokens.get(i).map(|(t, _)| t), Some(Token::Newline)) {
            i += 1;
        }
        i
    }
}
