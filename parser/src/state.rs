use crate::node::Node;
use crate::token::{Token, TokenType};

#[derive(Debug)]
pub struct Parser {
    pub input: String,
    pub chars: Vec<char>,
    pub index: usize,
    pub is_start: bool,
    pub current_char: char,
    pub current_token: Token,
    pub allow_expr: bool,
    pub node: Option<Node>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut parser = Parser {
            input: String::from(input),
            chars: input.chars().collect(),
            index: 0,
            is_start: true,
            current_char: ' ',
            current_token: Token::new(TokenType::EOF, ""),
            allow_expr: true,
            node: None,
        };
        parser.node = Some(parser.parse());
        parser
    }

    pub fn parse(&mut self) -> Node {
        let mut body: Vec<Box<Node>> = vec![];
        self.next_token();
        while self.check_valid_index() {
            let stat = self.parse_statement();
            body.push(Box::new(stat));
        }
        Node::Program { body }
    }

    pub fn check_valid_index(&self) -> bool {
        self.index < self.chars.len()
    }

    pub fn look_behind(&self) -> char {
        if self.index < self.chars.len() - 1 {
            self.chars[self.index + 1]
        } else {
            0 as char
        }
    }

    pub fn next_char(&mut self) {
        if self.is_start {
            self.is_start = false;
        } else {
            self.index += 1;
        }
        if self.index < self.chars.len() {
            self.current_char = self.chars[self.index];
        } else {
            self.current_char = 0 as char;
        }
    }

    pub fn is_token(&self, token_type: TokenType) -> bool {
        self.current_token.token_type == token_type
    }

    pub fn expect(&mut self, token_type: TokenType) {
        if !self.is_token(token_type) {
            self.unexpected();
        }
    }

    pub fn consume(&mut self, token_type: TokenType) -> bool {
        if self.is_token(token_type) {
            self.next_token();
            true
        } else {
            false
        }
    }

    pub fn consume_or_panic(&mut self, token_type: TokenType) {
        self.expect(token_type);
        self.next_token();
    }

    pub fn unexpected(&mut self) -> ! {
        let mut line = 1;
        let mut column = 1;
        for i in self.chars.iter() {
            if *i == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        panic!("Unexpected `{:?}` at ({}, {})", self.current_token, line, column)
    }
}
