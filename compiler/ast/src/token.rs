use crate::shared::is_keyword_str;
use crate::state::Parser;

#[derive(Debug, PartialEq, Eq)]
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
}

impl Token {
    // 创建一个 Token
    pub fn new(p: &mut Parser, token_type: TokenType, value: &str) -> Self {
        Token::create_op(p, token_type, value, -1)
    }

    // 创建一个运算符 Token
    pub fn create_op(
        p: &mut Parser,
        token_type: TokenType,
        value: &str,
        precedence: i8,
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
                    Token::create_op(self, TokenType::EQ, "==", 11)
                } else {
                    Token::create_op(self, TokenType::Assign, "=", 1)
                }
            }
            '+' => {
                self.move_index(1);
                Token::create_op(self, TokenType::Plus, "+", 14)
            }
            '-' => {
                let next_char = self.look_behind(1);
                if next_char == '>' {
                    self.move_index(2);
                    Token::new(self, TokenType::ReturnSym, "->")
                } else if self.allow_expr {
                    self.read_number()
                } else {
                    self.move_index(1);
                    Token::create_op(self, TokenType::Sub, "-", 14)
                }
            }
            '*' => {
                self.move_index(1);
                if self.allow_expr {
                    Token::new(self, TokenType::Star, "*")
                } else {
                    Token::create_op(self, TokenType::Mul, "*", 15)
                }
            }
            '/' => {
                self.move_index(1);
                Token::create_op(self, TokenType::Div, "/", 15)
            }
            '%' => {
                self.move_index(1);
                Token::create_op(self, TokenType::REM, "%", 15)
            }
            '<' => {
                self.move_index(1);
                if self.current_char == '=' {
                    self.move_index(1);
                    Token::create_op(self, TokenType::LE, "<=", 12)
                } else {
                    Token::create_op(self, TokenType::LT, "<", 12)
                }
            }
            '>' => {
                self.move_index(1);
                if self.current_char == '=' {
                    self.move_index(1);
                    Token::create_op(self, TokenType::GE, ">=", 12)
                } else {
                    Token::create_op(self, TokenType::GT, ">", 12)
                }
            }
            '&' => {
                self.move_index(1);
                if self.current_char == '&' {
                    self.move_index(1);
                    Token::create_op(self, TokenType::LogicAnd, "&&", 7)
                } else {
                    Token::create_op(self, TokenType::BitAnd, "&", 10)
                }
            }
            '|' => {
                self.move_index(1);
                if self.current_char == '|' {
                    self.move_index(1);
                    Token::create_op(self, TokenType::LogicOr, "||", 6)
                } else {
                    Token::create_op(self, TokenType::BitAnd, "|", 8)
                }
            }
            '!' => {
                self.move_index(1);
                if self.current_char == '=' {
                    self.move_index(1);
                    Token::create_op(self, TokenType::NE, "!=", 11)
                } else {
                    Token::create_op(self, TokenType::LogicNot, "!", 17)
                }
            }
            '~' => {
                self.move_index(1);
                Token::create_op(self, TokenType::BitNot, "~", 17)
            }
            '^' => {
                self.move_index(1);
                Token::create_op(self, TokenType::BitNot, "^", 9)
            }
            '(' => {
                self.move_index(1);
                Token::new(self, TokenType::ParenL, "(")
            }
            ')' => {
                self.move_index(1);
                Token::new(self, TokenType::ParenR, ")")
            }
            '[' => {
                self.move_index(1);
                Token::new(self, TokenType::BracketL, "[")
            }
            ']' => {
                self.move_index(1);
                Token::new(self, TokenType::BracketR, "]")
            }
            '{' => {
                self.move_index(1);
                Token::new(self, TokenType::BraceL, "{")
            }
            '}' => {
                self.move_index(1);
                Token::new(self, TokenType::BraceR, "}")
            }
            ',' => {
                self.move_index(1);
                Token::new(self, TokenType::Comma, ",")
            }
            '.' => {
                self.move_index(1);
                Token::new(self, TokenType::Dot, ".")
            }
            ';' => {
                self.move_index(1);
                Token::new(self, TokenType::Semi, ";")
            }
            ':' => {
                self.move_index(1);
                Token::new(self, TokenType::Colon, ":")
            }
            _ => {
                if self.index == self.chars.len() {
                    Token::new(self, TokenType::EOF, "EOF")
                } else {
                    self.unexpected();
                }
            }
        };
        self.current_token = token;
    }

    // 读取一个标识符 token
    pub fn read_identifier(&mut self) -> Token {
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
                return Token::new(self, TokenType::Boolean, &value);
            }
            return Token::new(self, TokenType::Keyword, &value);
        }

        Token::new(self, TokenType::Identifier, &value)
    }

    // 读取一个数字 token
    pub fn read_number(&mut self) -> Token {
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

        Token::new(self, TokenType::Number, &value)
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
