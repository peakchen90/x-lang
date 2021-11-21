use crate::code_frame::print_error_frame;
use crate::node::Node;
use crate::shared::Kind;
use crate::token::{Token, TokenType};

#[derive(Debug)]
pub struct Parser<'a> {
    pub input: &'a str,
    pub chars: Vec<char>,           // 字符 vec
    pub index: usize,               // 光标位置
    pub is_start: bool,             // 光标是否在开始位置
    pub is_seen_newline: bool,      // 读取下一个 token 时是否遇到过换行
    pub current_char: char,         // 当前字符
    pub current_token: Token,       // 当前 token
    pub allow_expr: bool,           // 当前上下文是否允许表达式
    pub current_block_level: usize, // 当前进入到第几层块级作用域
    pub current_loop_level: usize,  // 当前进入到第几层循环块
    pub node: Option<Node>,         // 解析的 ast
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let begin_token = Token {
            token_type: TokenType::Begin,
            value: String::new(),
            precedence: -1,
            start: 0,
            end: 0,
        };
        let mut parser = Parser {
            input,
            chars: input.chars().collect(),
            index: 0,
            is_start: true,
            is_seen_newline: false,
            current_char: ' ',
            current_token: begin_token,
            allow_expr: true,
            current_block_level: 0,
            current_loop_level: 0,
            node: None,
        };
        parser.node = Some(parser.parse());
        parser
    }

    // 开始解析
    fn parse(&mut self) -> Node {
        let mut body: Vec<Box<Node>> = vec![];
        self.next_token();
        while self.check_valid_index() {
            let stat = self.parse_statement();
            body.push(Box::new(stat));
        }
        Node::Program {
            position: (0, self.chars.len()),
            body,
        }
    }

    // 检查光标是否超过最大值
    pub fn check_valid_index(&self) -> bool {
        !self.is_token(TokenType::EOF)
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

    // 向后移动 n 个位置并读取字符
    pub fn move_index(&mut self, n: usize) {
        if self.is_start {
            self.is_start = false;
        } else {
            self.index += n;
        }

        if self.index < self.chars.len() {
            self.current_char = self.chars[self.index];
        } else {
            self.current_char = 0 as char;
            self.current_token = Token::new(
                self,
                TokenType::EOF,
                "",
                (self.chars.len(), self.chars.len()),
            )
        }
    }

    // 检查下一个有效字符是否是指定的字符
    pub fn check_next_char(&mut self, char: char) -> bool {
        self.skip_space(true);
        self.skip_comment();
        self.current_char == char
    }

    // 验证是否在函数内部，否则抛错
    pub fn validate_inside_fn(&mut self) {
        if self.current_block_level == 0 {
            self.unexpected(Some("Cannot be used outside of functions"));
        }
    }

    // 检查是否在程序根层级下，否则抛错
    pub fn validate_program_root(&mut self, title: &str) {
        if self.current_block_level > 0 {
            self.unexpected(Some(&format!("{} can only be defined in the root", title)));
        }
    }

    // 是否是某个关键字
    pub fn is_keyword(&self, keyword: &str) -> bool {
        self.is_token(TokenType::Keyword) && self.current_token.value == keyword
    }

    // 判断是否是某一类型的token
    pub fn is_token(&self, token_type: TokenType) -> bool {
        self.current_token.token_type == token_type
    }

    // 当前字符是否是换行符
    pub fn is_newline_char(&self) -> bool {
        self.current_char == '\n' || self.current_char == '\r'
    }

    // 当前字符是否是空白字符
    pub fn is_space_char(&self) -> bool {
        self.current_char == ' ' || self.current_char == '\t' || self.is_newline_char()
    }

    // 生成 Identifier 节点
    pub fn gen_identifier(&self, token: Token, kind: Kind) -> Node {
        Node::Identifier {
            position: (token.start, token.end),
            name: token.value,
            kind,
        }
    }

    // 期望当前 token 类型为指定类型，否则抛错
    pub fn expect(&mut self, token_type: TokenType) {
        if !self.is_token(token_type) {
            self.unexpected(None);
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

    // 抛出一个异常信息
    pub fn unexpected_err(&mut self, pos: usize, msg: &str) -> ! {
        let mut message = msg.to_string();
        let position = print_error_frame(self.input, pos, &message);

        if let Some((line, column)) = position {
            message.push_str(&format!(" ({}:{})", line, column))
        }
        panic!("{}", message)
    }

    // 抛出一个 unexpected 错误
    pub fn unexpected_pos(&mut self, pos: usize, msg: Option<&str>) -> ! {
        let mut message = match self.chars.get(pos) {
            None => String::from("Unexpected end of file"),
            Some(ch) => format!("Unexpected token `{}`", ch),
        };
        if let Some(msg) = msg {
            message = msg.to_string();
        }

        self.unexpected_err(pos, &message)
    }

    // 抛出一个 unexpected token 错误
    pub fn unexpected(&mut self, msg: Option<&str>) -> ! {
        self.unexpected_token(self.current_token.clone(), msg)
    }

    // 抛出一个 unexpected token 错误
    pub fn unexpected_token(&mut self, token: Token, msg: Option<&str>) -> ! {
        let mut message = String::new();
        match token.token_type {
            TokenType::EOF => message.push_str("Unexpected end of file"),
            _ => message.push_str(&format!("Unexpected token `{}`", token.value)),
        };

        if let Some(msg) = msg {
            message = msg.to_string();
        }

        self.unexpected_err(token.start, &message)
    }

    // 抛出一个 unexpected kind 错误
    pub fn unexpected_kind(&mut self, token: Token) {
        let mut message = String::new();
        if token.value == "void" {
            message.push_str("Unexpected kind: ")
        } else {
            message.push_str("Invalid kind: ")
        }
        message.push_str(&format!("`{}`", token.value));
        self.unexpected_token(token, Some(&message))
    }
}
