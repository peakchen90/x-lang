use crate::shared::is_keyword_str;
use crate::state::Parser;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TokenType {
    Begin, // 初始Token
    EOF,   // 结束 Token
    Keyword,
    Identifier,
    Number,
    Boolean,
    Assign,    // =
    Plus,      // +
    Sub,       // -
    Mul,       // *
    Div,       // /
    REM,       // %
    LT,        // <
    LE,        // <=
    GT,        // >
    GE,        // >=
    EQ,        // ==
    NE,        // !=
    LogicAnd,  // &&
    LogicOr,   // ||
    LogicNot,  // !
    BitAnd,    // &
    BitOr,     // |
    BitNot,    // ~
    BitXor,    // ^
    ParenL,    // (
    ParenR,    // )
    BracketL,  // [
    BracketR,  // ]
    BraceL,    // {
    BraceR,    // }
    Comma,     // ,
    Dot,       // .
    Semi,      // ;
    Colon,     // :
    Star,      // *
    ReturnSym, // ->
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub precedence: i8,
    pub start: usize,
    pub end: usize,
}

impl Token {
    // 创建一个 Token
    pub fn new(
        p: &mut Parser,
        token_type: TokenType,
        value: &str,
        position: (usize, usize),
    ) -> Self {
        Token::create_op(p, token_type, value, -1, position)
    }

    // 创建一个运算符 Token
    pub fn create_op(
        p: &mut Parser,
        token_type: TokenType,
        value: &str,
        precedence: i8,
        position: (usize, usize),
    ) -> Self {
        match token_type {
            TokenType::Assign
            | TokenType::Plus
            | TokenType::Sub
            | TokenType::Mul
            | TokenType::Div
            | TokenType::ParenL
            | TokenType::BracketL
            | TokenType::BraceL
            | TokenType::BraceR
            | TokenType::Colon
            | TokenType::Comma
            | TokenType::Semi => p.allow_expr = true,
            _ => p.allow_expr = false,
        }

        Token {
            token_type,
            value: String::from(value),
            precedence,
            start: position.0,
            end: position.1,
        }
    }

    // 克隆 Token
    pub fn clone(&self) -> Self {
        Token {
            token_type: self.token_type.clone(),
            value: self.value.clone(),
            precedence: self.precedence,
            start: self.start,
            end: self.end,
        }
    }
}

impl<'a> Parser<'a> {
    // 读取下一个 token
    pub fn next_token(&mut self) {
        self.is_seen_newline = false;
        self.skip_space();
        self.skip_comment();

        let token = match self.current_char {
            'A'..='Z' | 'a'..='z' | '_' | '$' => self.read_identifier(),
            '0'..='9' => self.read_number(),
            '=' => {
                self.move_index(1);
                if self.current_char == '=' {
                    self.move_index(1);
                    Token::create_op(
                        self,
                        TokenType::EQ,
                        "==",
                        11,
                        (self.index - 2, self.index),
                    )
                } else {
                    Token::create_op(
                        self,
                        TokenType::Assign,
                        "=",
                        1,
                        (self.index - 1, self.index),
                    )
                }
            }
            '+' => {
                self.move_index(1);
                Token::create_op(
                    self,
                    TokenType::Plus,
                    "+",
                    14,
                    (self.index - 1, self.index),
                )
            }
            '-' => {
                let next_char = self.look_behind(1);
                if next_char == '>' {
                    self.move_index(2);
                    Token::new(
                        self,
                        TokenType::ReturnSym,
                        "->",
                        (self.index - 2, self.index),
                    )
                } else if self.allow_expr {
                    self.read_number()
                } else {
                    self.move_index(1);
                    Token::create_op(
                        self,
                        TokenType::Sub,
                        "-",
                        14,
                        (self.index - 1, self.index),
                    )
                }
            }
            '*' => {
                self.move_index(1);
                if self.allow_expr {
                    Token::new(self, TokenType::Star, "*", (self.index - 1, self.index))
                } else {
                    Token::create_op(
                        self,
                        TokenType::Mul,
                        "*",
                        15,
                        (self.index - 1, self.index),
                    )
                }
            }
            '/' => {
                self.move_index(1);
                Token::create_op(
                    self,
                    TokenType::Div,
                    "/",
                    15,
                    (self.index - 1, self.index),
                )
            }
            '%' => {
                self.move_index(1);
                Token::create_op(
                    self,
                    TokenType::REM,
                    "%",
                    15,
                    (self.index - 1, self.index),
                )
            }
            '<' => {
                self.move_index(1);
                if self.current_char == '=' {
                    self.move_index(1);
                    Token::create_op(
                        self,
                        TokenType::LE,
                        "<=",
                        12,
                        (self.index - 2, self.index),
                    )
                } else {
                    Token::create_op(
                        self,
                        TokenType::LT,
                        "<",
                        12,
                        (self.index - 1, self.index),
                    )
                }
            }
            '>' => {
                self.move_index(1);
                if self.current_char == '=' {
                    self.move_index(1);
                    Token::create_op(
                        self,
                        TokenType::GE,
                        ">=",
                        12,
                        (self.index - 2, self.index),
                    )
                } else {
                    Token::create_op(
                        self,
                        TokenType::GT,
                        ">",
                        12,
                        (self.index - 1, self.index),
                    )
                }
            }
            '&' => {
                self.move_index(1);
                if self.current_char == '&' {
                    self.move_index(1);
                    Token::create_op(
                        self,
                        TokenType::LogicAnd,
                        "&&",
                        7,
                        (self.index - 2, self.index),
                    )
                } else {
                    Token::create_op(
                        self,
                        TokenType::BitAnd,
                        "&",
                        10,
                        (self.index - 1, self.index),
                    )
                }
            }
            '|' => {
                self.move_index(1);
                if self.current_char == '|' {
                    self.move_index(1);
                    Token::create_op(
                        self,
                        TokenType::LogicOr,
                        "||",
                        6,
                        (self.index - 2, self.index),
                    )
                } else {
                    Token::create_op(
                        self,
                        TokenType::BitAnd,
                        "|",
                        8,
                        (self.index - 1, self.index),
                    )
                }
            }
            '!' => {
                self.move_index(1);
                if self.current_char == '=' {
                    self.move_index(1);
                    Token::create_op(
                        self,
                        TokenType::NE,
                        "!=",
                        11,
                        (self.index - 2, self.index),
                    )
                } else {
                    Token::create_op(
                        self,
                        TokenType::LogicNot,
                        "!",
                        17,
                        (self.index - 1, self.index),
                    )
                }
            }
            '~' => {
                self.move_index(1);
                Token::create_op(
                    self,
                    TokenType::BitNot,
                    "~",
                    17,
                    (self.index - 1, self.index),
                )
            }
            '^' => {
                self.move_index(1);
                Token::create_op(
                    self,
                    TokenType::BitNot,
                    "^",
                    9,
                    (self.index - 1, self.index),
                )
            }
            '(' => {
                self.move_index(1);
                Token::new(self, TokenType::ParenL, "(", (self.index - 1, self.index))
            }
            ')' => {
                self.move_index(1);
                Token::new(self, TokenType::ParenR, ")", (self.index - 1, self.index))
            }
            '[' => {
                self.move_index(1);
                Token::new(self, TokenType::BracketL, "[", (self.index - 1, self.index))
            }
            ']' => {
                self.move_index(1);
                Token::new(self, TokenType::BracketR, "]", (self.index - 1, self.index))
            }
            '{' => {
                self.move_index(1);
                Token::new(self, TokenType::BraceL, "{", (self.index - 1, self.index))
            }
            '}' => {
                self.move_index(1);
                Token::new(self, TokenType::BraceR, "}", (self.index - 1, self.index))
            }
            ',' => {
                self.move_index(1);
                Token::new(self, TokenType::Comma, ",", (self.index - 1, self.index))
            }
            '.' => {
                self.move_index(1);
                Token::new(self, TokenType::Dot, ".", (self.index - 1, self.index))
            }
            ';' => {
                self.move_index(1);
                Token::new(self, TokenType::Semi, ";", (self.index - 1, self.index))
            }
            ':' => {
                self.move_index(1);
                Token::new(self, TokenType::Colon, ":", (self.index - 1, self.index))
            }
            _ => {
                if self.index == self.chars.len() {
                    Token::new(self, TokenType::EOF, "EOF", (self.index, self.index))
                } else {
                    self.unexpected_pos(self.index, None);
                }
            }
        };
        self.current_token = token;
    }

    // 读取一个标识符 token
    pub fn read_identifier(&mut self) -> Token {
        let start = self.index;
        let mut value = String::new();
        while self.check_valid_index()
            && match self.current_char {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '$' => true,
                _ => false,
            }
        {
            value.push(self.current_char);
            self.move_index(1);
        }

        // keyword
        if is_keyword_str(&value) {
            if value == "true" || value == "false" {
                return Token::new(self, TokenType::Boolean, &value, (start, self.index));
            }
            return Token::new(self, TokenType::Keyword, &value, (start, self.index));
        }

        Token::new(self, TokenType::Identifier, &value, (start, self.index))
    }

    // 读取一个数字 token
    pub fn read_number(&mut self) -> Token {
        let start = self.index;
        let mut value = String::new();
        if self.current_char == '-' {
            value.push('-');
            self.move_index(1);
        }

        while self.check_valid_index()
            && match self.current_char {
                '0'..='9' | '.' => true,
                _ => false,
            }
        {
            value.push(self.current_char);
            self.move_index(1);
        }

        Token::new(self, TokenType::Number, &value, (start, self.index))
    }

    // 跳过空白字符
    pub fn skip_space(&mut self) {
        while self.is_space_char() {
            // 标记已经换行过
            if self.is_newline_char() {
                self.is_seen_newline = true;
            }

            if self.check_valid_index() {
                self.move_index(1);
            } else {
                break;
            }
        }
    }

    // 跳过注释
    pub fn skip_comment(&mut self) {
        while self.current_char == '/' && self.look_behind(1) == '/' {
            self.move_index(2);
            while self.check_valid_index() && !self.is_newline_char() {
                self.move_index(1);
            }
            self.skip_space();
        }
    }
}
