use crate::node::Node;
use crate::token::{Token, TokenType};

#[derive(Debug)]
pub struct Parser {
    pub input: String,
    pub chars: Vec<char>, // 字符 vec
    pub index: usize, // 光标位置
    pub is_start: bool, // 光标是否在开始位置
    pub current_char: char, // 当前字符
    pub current_token: Token, // 当前 token
    pub allow_expr: bool, // 当前上下文是否允许表达式
    pub current_block_scope: usize, // 当前进入到第几层块级作用域
    pub node: Option<Node>, // 解析的 ast
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
            current_block_scope: 0,
            node: None,
        };
        parser.node = Some(parser.parse());
        parser
    }

    // 开始解析
    pub fn parse(&mut self) -> Node {
        let mut body: Vec<Box<Node>> = vec![];
        self.next_token();
        while self.check_valid_index() {
            let stat = self.parse_statement();
            body.push(Box::new(stat));
        }
        Node::Program { body }
    }

    // 检查光标是否超过最大值
    pub fn check_valid_index(&self) -> bool {
        self.index < self.chars.len()
    }

    // 向后查看 n 个字符（不移动光标）
    pub fn look_behind(&self, n: usize) -> char {
        let next = self.index + n;
        if next < self.chars.len() {
            self.chars[next]
        } else {
            0 as char
        }
    }

    // 移动并读取一个字符
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

    // 判断是否是某一类型的token
    pub fn is_token(&self, token_type: TokenType) -> bool {
        self.current_token.token_type == token_type
    }

    // 期望当前 token 类型为指定类型，否则抛错
    pub fn expect(&mut self, token_type: TokenType) {
        if !self.is_token(token_type) {
            self.unexpected();
        }
    }

    // 消费一个 token 类型，如果消费成功，返回 true 并读取下一个 token，否则返回 false
    pub fn consume(&mut self, token_type: TokenType) -> bool {
        if self.is_token(token_type) {
            self.next_token();
            true
        } else {
            false
        }
    }

    // 消费一个 token 类型，如果消费成功，读取下一个 token，否则抛错
    pub fn consume_or_panic(&mut self, token_type: TokenType) {
        self.expect(token_type);
        self.next_token();
    }

    // 抛出一个 unexpected token 错误
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
        panic!(
            "Unexpected `{:?}` at ({}, {})",
            self.current_token, line, column
        )
    }
}
