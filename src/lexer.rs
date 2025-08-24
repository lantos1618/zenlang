use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Loop,
    Comptime,
    Async,
    Await,
    Behavior,
    Impl,
    Extern,
    Break,
    Continue,
    Return,
    Match,
    Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Integer(String),
    Float(String),
    StringLiteral(String),
    Keyword(Keyword),
    Symbol(char),
    Operator(String),
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Span,
}

#[derive(Clone)]
pub struct Lexer<'a> {
    pub input: &'a str,
    pub position: usize,
    pub read_position: usize,
    pub current_char: Option<char>,
    pub line: usize,
    pub column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            current_char: None,
            line: 1,
            column: 1,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.current_char = None;
        } else {
            self.current_char = self.input[self.read_position..].chars().next();
        }
        self.position = self.read_position;
        self.read_position += 1;
        
        // Update line and column tracking
        if let Some(c) = self.current_char {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.next_token_with_span().token
    }

    pub fn next_token_with_span(&mut self) -> TokenWithSpan {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        self.skip_whitespace_and_comments();
        
        let token = match self.current_char {
            Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '@' => {
                let ident = self.read_identifier();
                match self.str_to_keyword(&ident) {
                    Some(keyword) => Token::Keyword(keyword),
                    None => Token::Identifier(ident),
                }
            }
            Some(c) if c.is_ascii_digit() => {
                let number = self.read_number();
                if number.contains('.') {
                    Token::Float(number)
                } else {
                    Token::Integer(number)
                }
            }
            Some('"') => {
                let string = self.read_string();
                Token::StringLiteral(string)
            }
            Some(':') => {
                // Check for multi-character operators (:=, ::, ::=)
                if let Some(next) = self.peek_char() {
                    if next == '=' {
                        self.read_char(); // consume ':'
                        self.read_char(); // consume '='
                        return TokenWithSpan {
                            token: Token::Operator(":=".to_string()),
                            span: Span { start: start_pos, end: self.position, line: start_line, column: start_column },
                        };
                    } else if next == ':' {
                        self.read_char(); // consume ':'
                        if let Some(next2) = self.peek_char() {
                            if next2 == '=' {
                                self.read_char(); // consume second ':'
                                self.read_char(); // consume '='
                                return TokenWithSpan {
                                    token: Token::Operator("::=".to_string()),
                                    span: Span { start: start_pos, end: self.position, line: start_line, column: start_column },
                                };
                            }
                        }
                        self.read_char(); // consume second ':'
                        return TokenWithSpan {
                            token: Token::Operator("::".to_string()),
                            span: Span { start: start_pos, end: self.position, line: start_line, column: start_column },
                        };
                    }
                }
                self.read_char();
                Token::Symbol(':')
            }
            Some('.') => {
                // Check for range operators (.., ..=) and varargs (...)
                if let Some(next) = self.peek_char() {
                    if next == '.' {
                        self.read_char(); // consume first '.'
                        if let Some(next2) = self.peek_char() {
                            if next2 == '.' {
                                self.read_char(); // consume second '.'
                                self.read_char(); // consume third '.'
                                return TokenWithSpan {
                                    token: Token::Operator("...".to_string()),
                                    span: Span { start: start_pos, end: self.position, line: start_line, column: start_column },
                                };
                            } else if next2 == '=' {
                                self.read_char(); // consume second '.'
                                self.read_char(); // consume '='
                                return TokenWithSpan {
                                    token: Token::Operator("..=".to_string()),
                                    span: Span { start: start_pos, end: self.position, line: start_line, column: start_column },
                                };
                            }
                        }
                        self.read_char(); // consume second '.'
                        return TokenWithSpan {
                            token: Token::Operator("..".to_string()),
                            span: Span { start: start_pos, end: self.position, line: start_line, column: start_column },
                        };
                    }
                }
                self.read_char();
                Token::Symbol('.')
            }
            Some('?') => {
                self.read_char();
                Token::Symbol('?')
            }
            Some('|') => {
                // Only treat as operator if part of '||'
                if let Some(next) = self.peek_char() {
                    if next == '|' {
                        self.read_char(); // consume '|'
                        self.read_char(); // consume second '|'
                        return TokenWithSpan {
                            token: Token::Operator("||".to_string()),
                            span: Span { start: start_pos, end: self.position, line: start_line, column: start_column },
                        };
                    }
                }
                self.read_char();
                Token::Symbol('|')
            }
            Some('<') => {
                self.read_char();
                if self.current_char == Some('=') {
                    self.read_char();
                    Token::Operator("<=".to_string())
                } else {
                    Token::Operator("<".to_string())
                }
            }
            Some('>') => {
                self.read_char();
                if self.current_char == Some('=') {
                    self.read_char();
                    Token::Operator(">=".to_string())
                } else {
                    Token::Operator(">".to_string())
                }
            }
            Some('!') => {
                self.read_char();
                if self.current_char == Some('=') {
                    self.read_char();
                    Token::Operator("!=".to_string())
                } else {
                    Token::Symbol('!')
                }
            }
            Some(c) if self.is_symbol(c) => {
                self.read_char();
                Token::Symbol(c)
            }
            Some(c) if self.is_operator_start(c) => {
                let op = self.read_operator();
                Token::Operator(op)
            }
            None => Token::Eof,
            _ => {
                self.read_char();
                return self.next_token_with_span();
            }
        };
        
        TokenWithSpan {
            token,
            span: Span { start: start_pos, end: self.position, line: start_line, column: start_column },
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(c) = self.current_char {
                if c.is_whitespace() {
                    self.read_char();
                } else {
                    break;
                }
            }
            // Skip single-line comments
            if self.current_char == Some('/') && self.peek_char() == Some('/') {
                // Skip '//'
                self.read_char();
                self.read_char();
                // Skip until end of line or input
                while let Some(c) = self.current_char {
                    if c == '\n' {
                        break;
                    }
                    self.read_char();
                }
                continue;
            }
            // Skip multi-line comments
            if self.current_char == Some('/') && self.peek_char() == Some('*') {
                // Skip '/*'
                self.read_char();
                self.read_char();
                // Skip until '*/' or end of input
                while let Some(c) = self.current_char {
                    if c == '*' && self.peek_char() == Some('/') {
                        self.read_char(); // skip '*'
                        self.read_char(); // skip '/'
                        break;
                    }
                    self.read_char();
                }
                continue;
            }
            break;
        }
    }

    fn read_identifier(&mut self) -> String {
        let start = self.position;
        while let Some(c) = self.current_char {
            if c.is_ascii_alphanumeric() || c == '_' || c == '@' {
                self.read_char();
            } else {
                break;
            }
        }
        self.input[start..self.position].to_string()
    }

    fn read_number(&mut self) -> String {
        let start = self.position;
        let mut has_dot = false;
        
        while let Some(c) = self.current_char {
            if c.is_ascii_digit() {
                self.read_char();
            } else if c == '.' && !has_dot {
                // Only consume '.' if it's followed by a digit (for floats like 3.14)
                // This prevents consuming '..' as part of a number
                if let Some(next_c) = self.peek_char() {
                    if next_c.is_ascii_digit() {
                        has_dot = true;
                        self.read_char();
                    } else {
                        // Don't consume '.' if not followed by digit
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        self.input[start..self.position].to_string()
    }

    fn str_to_keyword(&self, ident: &str) -> Option<Keyword> {
        match ident {
            "loop" => Some(Keyword::Loop),
            "comptime" => Some(Keyword::Comptime),
            "async" => Some(Keyword::Async),
            "await" => Some(Keyword::Await),
            "behavior" => Some(Keyword::Behavior),
            "impl" => Some(Keyword::Impl),
            "extern" => Some(Keyword::Extern),
            "break" => Some(Keyword::Break),
            "continue" => Some(Keyword::Continue),
            "return" => Some(Keyword::Return),
            "match" => Some(Keyword::Match),
            "type" => Some(Keyword::Type),
            _ => None,
        }
    }

    fn is_symbol(&self, c: char) -> bool {
        matches!(c, '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',' | '|' | '&' | '.' | '?')
    }

    fn is_operator_start(&self, c: char) -> bool {
        matches!(c, '+' | '-' | '*' | '/' | '=' | '!' | '<' | '>' | '&' | '|' | ':')
    }

    fn read_string(&mut self) -> String {
        self.read_char(); // consume opening quote
        let start = self.position;
        while let Some(c) = self.current_char {
            if c == '"' {
                break;
            }
            self.read_char();
        }
        let result = self.input[start..self.position].to_string();
        self.read_char(); // consume closing quote
        result
    }

    fn read_operator(&mut self) -> String {
        let _start = self.position;
        let first_char = self.current_char.unwrap();
        self.read_char();
        
        // Handle three-character operators first
        if let Some(second_char) = self.current_char {
            if let Some(third_char) = self.peek_char() {
                let three_char_op = format!("{}{}{}", first_char, second_char, third_char);
                if matches!(three_char_op.as_str(), "::=" | "..=") {
                    self.read_char(); // consume second char
                    self.read_char(); // consume third char
                    return three_char_op;
                }
            }
        }
        
        // Handle two-character operators
        if let Some(second_char) = self.current_char {
            let two_char_op = format!("{}{}", first_char, second_char);
            if matches!(two_char_op.as_str(), "==" | "!=" | "<=" | ">=" | "&&" | "||" | "->" | "=>" | ":=" | "::" | ".." | "..=") {
                self.read_char();
                return two_char_op;
            }
        }
        
        // Single character operator
        first_char.to_string()
    }

    fn peek_char(&self) -> Option<char> {
        if self.read_position >= self.input.len() {
            None
        } else {
            self.input[self.read_position..].chars().next()
        }
    }
} 