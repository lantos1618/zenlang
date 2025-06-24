#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Integer(String),
    Float(String),
    StringLiteral(String),
    Keyword(String),
    Symbol(char),
    Operator(String),
    Eof,
}

pub struct Lexer<'a> {
    pub input: &'a str,
    pub position: usize,
    pub read_position: usize,
    pub current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            current_char: None,
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
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        match self.current_char {
            Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '@' => {
                let ident = self.read_identifier();
                if self.is_keyword(&ident) {
                    Token::Keyword(ident)
                } else {
                    Token::Identifier(ident)
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
                        return Token::Operator(":=".to_string());
                    } else if next == ':' {
                        self.read_char(); // consume ':'
                        if let Some(next2) = self.peek_char() {
                            if next2 == '=' {
                                self.read_char(); // consume second ':'
                                self.read_char(); // consume '='
                                return Token::Operator("::=".to_string());
                            }
                        }
                        self.read_char(); // consume second ':'
                        return Token::Operator("::".to_string());
                    }
                }
                self.read_char();
                Token::Symbol(':')
            }
            Some('|') => {
                // Only treat as operator if part of '||'
                if let Some(next) = self.peek_char() {
                    if next == '|' {
                        self.read_char(); // consume '|'
                        self.read_char(); // consume second '|'
                        return Token::Operator("||".to_string());
                    }
                }
                self.read_char();
                Token::Symbol('|')
            }
            Some(c) if self.is_operator_start(c) => {
                let op = self.read_operator();
                Token::Operator(op)
            }
            Some(c) if self.is_symbol(c) => {
                self.read_char();
                Token::Symbol(c)
            }
            None => Token::Eof,
            _ => {
                self.read_char();
                self.next_token()
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
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
        while let Some(c) = self.current_char {
            if c.is_ascii_digit() || c == '.' {
                self.read_char();
            } else {
                break;
            }
        }
        self.input[start..self.position].to_string()
    }

    fn is_keyword(&self, ident: &str) -> bool {
        matches!(ident, "loop" | "in" | "comptime" | "async" | "await" | "behavior" | "impl")
    }

    fn is_symbol(&self, c: char) -> bool {
        matches!(c, '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',' | '|' | '&' | '!' | '.')
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
        let start = self.position;
        let first_char = self.current_char.unwrap();
        self.read_char();
        
        // Handle three-character operators first
        if let Some(second_char) = self.current_char {
            if let Some(third_char) = self.peek_char() {
                let three_char_op = format!("{}{}{}", first_char, second_char, third_char);
                if matches!(three_char_op.as_str(), "::=") {
                    self.read_char(); // consume second char
                    self.read_char(); // consume third char
                    return three_char_op;
                }
            }
        }
        
        // Handle two-character operators
        if let Some(second_char) = self.current_char {
            let two_char_op = format!("{}{}", first_char, second_char);
            if matches!(two_char_op.as_str(), "==" | "!=" | "<=" | ">=" | "&&" | "||" | "->" | "=>" | ":=" | "::") {
                self.read_char();
                return two_char_op;
            }
        }
        
        // Single character operator
        self.input[start..self.position].to_string()
    }
    
    fn peek_char(&self) -> Option<char> {
        if self.read_position >= self.input.len() {
            None
        } else {
            self.input[self.read_position..].chars().next()
        }
    }
} 