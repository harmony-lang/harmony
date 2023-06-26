use crate::token::{SourceLocation, Token, TokenKind};

pub struct Tokenizer {
    pub source: String,
    pub tokens: Vec<Token>,
    filename: String,
    index: usize,
    line: usize,
    column: usize,
}

impl Tokenizer {
    pub fn new(filename: &String, source: &String) -> Tokenizer {
        Tokenizer {
            source: source.clone(),
            tokens: vec![],
            filename: filename.clone(),
            index: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            let c: char = self.next().unwrap();
            match c {
                ' ' | '\r' | '\t' => {
                    // self.column += 1;
                }
                '\n' => {
                    // self.add_token(TokenKind::Newline, c.to_string());
                    // let mut spaces: usize = 0;
                    // while self.peek() == Some(' ') {
                    //     self.next();
                    //     spaces += 1;
                    //     if spaces % 4 == 0 {
                    //         self.add_token(TokenKind::Whitespace, "    ".to_string());
                    //     }
                    // }
                }
                '(' => self.add_token(TokenKind::OpenParenthesis, c.to_string()),
                ')' => self.add_token(TokenKind::CloseParenthesis, c.to_string()),
                '[' => self.add_token(TokenKind::OpenBracket, c.to_string()),
                ']' => self.add_token(TokenKind::CloseBracket, c.to_string()),
                '.' => {
                    if self.peek() == Some('.') {
                        self.next();
                        self.add_token(TokenKind::DoubleDot, "..".to_string());
                    } else {
                        self.add_token(TokenKind::Dot, c.to_string());
                    }
                }
                ',' => self.add_token(TokenKind::Comma, c.to_string()),
                ':' => {
                    if self.peek() == Some(':') {
                        self.next();
                        self.add_token(TokenKind::DoubleColon, "::".to_string());
                    } else {
                        self.add_token(TokenKind::Colon, c.to_string());
                    }
                }
                '|' => {
                    if self.peek() == Some('|') {
                        self.next();
                        self.add_token(TokenKind::Or, "||".to_string());
                    } else {
                        self.add_token(TokenKind::Pipe, c.to_string());
                    }
                }
                '&' => {
                    if self.peek() == Some('&') {
                        self.next();
                        self.add_token(TokenKind::And, "&&".to_string());
                    } else {
                        self.add_token(TokenKind::Unknown, c.to_string());
                    }
                }
                '+' => {
                    if self.peek() == Some('+') {
                        self.next();
                        self.add_token(TokenKind::PlusPlus, "++".to_string());
                    } else {
                        self.add_token(TokenKind::Plus, c.to_string());
                    }
                }
                '-' => {
                    if self.peek() == Some('>') {
                        self.next();
                        self.add_token(TokenKind::Arrow, "->".to_string());
                    } else if self.peek() == Some('-') {
                        self.next();
                        while self.peek() != Some('\n') && !self.is_at_end() {
                            self.next();
                        }
                    } else {
                        self.add_token(TokenKind::Minus, c.to_string());
                    }
                }
                '*' => self.add_token(TokenKind::Asterisk, c.to_string()),
                '/' => self.add_token(TokenKind::Slash, c.to_string()),
                '%' => self.add_token(TokenKind::Percent, c.to_string()),
                '=' => {
                    if self.peek() == Some('>') {
                        self.next();
                        self.add_token(TokenKind::FatArrow, "=>".to_string());
                    } else {
                        self.add_token(TokenKind::Equals, c.to_string());
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        self.next();
                        self.add_token(TokenKind::LessThanEquals, "<=".to_string());
                    } else {
                        self.add_token(TokenKind::LessThan, c.to_string());
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.next();
                        self.add_token(TokenKind::GreaterThanEquals, ">=".to_string());
                    } else {
                        self.add_token(TokenKind::GreaterThan, c.to_string());
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.next();
                        self.add_token(TokenKind::NotEquals, "!=".to_string());
                    } else {
                        self.add_token(TokenKind::Not, c.to_string());
                    }
                }
                '"' => {
                    let mut string = String::new();
                    while self.peek() != Some('"') && !self.is_at_end() {
                        string.push(self.next().unwrap());
                    }
                    if self.is_at_end() {
                        panic!("Unterminated string");
                    }
                    self.next();
                    self.add_token(TokenKind::StringLiteral, string);
                }
                '\'' => {
                    let mut character = String::new();
                    while self.peek() != Some('\'') && !self.is_at_end() {
                        character.push(self.next().unwrap());
                    }
                    if self.is_at_end() {
                        panic!("Unterminated character");
                    }
                    self.next();
                    self.add_token(TokenKind::CharacterLiteral, character);
                }
                _ => {
                    if c.is_digit(10) {
                        let mut number = c.to_string();
                        while self.peek().is_some() && self.peek().unwrap().is_digit(10) {
                            number.push(self.next().unwrap());
                        }
                        if self.peek() == Some('.') {
                            number.push(self.next().unwrap());
                            while self.peek().is_some() && self.peek().unwrap().is_digit(10) {
                                number.push(self.next().unwrap());
                            }
                            self.add_token(TokenKind::FloatLiteral, number);
                        } else {
                            self.add_token(TokenKind::IntegerLiteral, number);
                        }
                    } else if c.is_alphabetic() || c == '_' {
                        let mut identifier = c.to_string();
                        while self.peek().is_some()
                            && (self.peek().unwrap().is_alphanumeric()
                                || self.peek().unwrap() == '_')
                        {
                            identifier.push(self.next().unwrap());
                        }
                        match identifier.as_str() {
                            "module" => self.add_token(TokenKind::Module, identifier),
                            "import" => self.add_token(TokenKind::Import, identifier),
                            "as" => self.add_token(TokenKind::As, identifier),
                            "exposing" => self.add_token(TokenKind::Exposing, identifier),
                            "extern" => self.add_token(TokenKind::Extern, identifier),
                            "enum" => self.add_token(TokenKind::Enum, identifier),
                            "fun" => self.add_token(TokenKind::Fun, identifier),
                            "case" => self.add_token(TokenKind::Case, identifier),
                            "of" => self.add_token(TokenKind::Of, identifier),
                            "end" => self.add_token(TokenKind::End, identifier),
                            "if" => self.add_token(TokenKind::If, identifier),
                            "then" => self.add_token(TokenKind::Then, identifier),
                            "else" => self.add_token(TokenKind::Else, identifier),
                            "let" => self.add_token(TokenKind::Let, identifier),
                            "in" => self.add_token(TokenKind::In, identifier),
                            "int" => self.add_token(TokenKind::Int, identifier),
                            "float" => self.add_token(TokenKind::Float, identifier),
                            "string" => self.add_token(TokenKind::String, identifier),
                            "char" => self.add_token(TokenKind::Char, identifier),
                            "bool" => self.add_token(TokenKind::Bool, identifier),
                            "any" => self.add_token(TokenKind::Any, identifier),
                            "unit" => self.add_token(TokenKind::Unit, identifier),
                            "true" => self.add_token(TokenKind::BooleanLiteral, identifier),
                            "false" => self.add_token(TokenKind::BooleanLiteral, identifier),
                            _ => self.add_token(TokenKind::Identifier, identifier),
                        }
                    } else {
                        self.add_token(TokenKind::Unknown, c.to_string());
                    }
                }
            }
        }
        self.tokens.clone()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.source.chars().nth(self.index);
        self.index += 1;
        if c == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    fn peek(&mut self) -> Option<char> {
        self.source.chars().nth(self.index)
    }

    fn is_at_end(&self) -> bool {
        self.index >= self.source.len()
    }

    fn add_token(&mut self, kind: TokenKind, lexeme: String) {
        self.tokens.push(Token {
            kind: kind,
            lexeme: lexeme.clone(),
            location: SourceLocation {
                file: self.filename.clone(),
                line: self.line,
                column: self.column,
                length: lexeme.len(),
            },
        });
    }
}
