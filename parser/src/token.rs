use crate::state::Parser;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Identifier,
    Number,
    EOF,
    Eq,     // =
    Plus,   // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    ParenL, // (
    ParenR, // )
    BraceL, // {
    BraceR, // }
    Comma,  // ,
    Semi,   // ;
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub precedence: i8,
}

impl Token {
    pub fn new(token_type: TokenType, value: &str) -> Self {
        Token::create(token_type, value, -1)
    }

    pub fn create(token_type: TokenType, value: &str, precedence: i8) -> Self {
        Token {
            token_type,
            value: String::from(value),
            precedence,
        }
    }
}

const KEYWORDS: [&str; 3] = ["fn", "var", "return"];

impl Parser {
    pub fn next_token(&mut self) {
        self.skip_space();
        self.skip_comment();

        let token = match self.current_char {
            'A'..='Z' | 'a'..='z' | '_' | '$' => self.read_identifier(),
            '0'..='9' => self.read_number(),
            '=' => {
                self.next_char();
                self.allow_expr = true;
                Token::new(TokenType::Eq, "=")
            }
            '+' => {
                self.next_char();
                self.allow_expr = true;
                Token::create(TokenType::Plus, "+", 13)
            }
            '-' => {
                if self.allow_expr {
                    self.read_number()
                } else {
                    self.next_char();
                    self.allow_expr = true;
                    Token::create(TokenType::Sub, "-", 13)
                }
            }
            '*' => {
                self.next_char();
                self.allow_expr = true;
                Token::create(TokenType::Mul, "*", 14)
            }
            '/' => {
                self.next_char();
                self.allow_expr = true;
                Token::create(TokenType::Div, "/", 14)
            }
            '(' => {
                self.next_char();
                self.allow_expr = true;
                Token::new(TokenType::ParenL, "(")
            }
            ')' => {
                self.next_char();
                self.allow_expr = false;
                Token::new(TokenType::ParenR, ")")
            }
            '{' => {
                self.next_char();
                self.allow_expr = true;
                Token::new(TokenType::BraceL, "{")
            }
            '}' => {
                self.next_char();
                self.allow_expr = true;
                Token::new(TokenType::BraceR, "}")
            }
            ',' => {
                self.next_char();
                self.allow_expr = true;
                Token::new(TokenType::Comma, ",")
            }
            ';' => {
                self.next_char();
                self.allow_expr = true;
                Token::new(TokenType::Semi, ";")
            }
            _ => {
                if self.index == self.chars.len() {
                    Token::new(TokenType::EOF, "EOF")
                } else {
                    self.unexpected();
                }
            }
        };
        self.current_token = token;
    }

    pub fn read_identifier(&mut self) -> Token {
        let mut value = String::new();
        while self.check_valid_index()
            && match self.current_char {
                'A'..='Z' | 'a'..='z' => true,
                _ => false,
            }
        {
            value.push(self.current_char);
            self.next_char();
        }
        for kw in KEYWORDS.iter() {
            if *kw == value {
                return Token::new(TokenType::Keyword, &value);
            }
        }

        self.allow_expr = false;
        Token::new(TokenType::Identifier, &value)
    }

    pub fn read_number(&mut self) -> Token {
        let mut value = String::new();
        if self.current_char == '-' {
            value.push('-');
            self.next_char();
        }

        while self.check_valid_index()
            && match self.current_char {
                '0'..='9' | '.' => true,
                _ => false,
            }
        {
            value.push(self.current_char);
            self.next_char();
        }

        self.allow_expr = false;
        Token::new(TokenType::Number, &value)
    }

    pub fn skip_space(&mut self) {
        while self.current_char == ' '
            || self.current_char == '\t'
            || self.current_char == '\n'
            || self.current_char == '\r'
        {
            if self.check_valid_index() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    pub fn skip_comment(&mut self) {
        while self.current_char == '/' && self.look_behind() == '/' {
            self.next_char();
            self.next_char();
            while self.check_valid_index() && self.current_char != '\n' && self.current_char != '\r'
            {
                self.next_char();
            }
            self.skip_space();
        }
    }
}
