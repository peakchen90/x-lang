use crate::shared::{Kind, KindName, Node, TokenType};
use crate::state::Parser;

impl<'a> Parser<'a> {
    // 解析一条语句
    pub fn parse_statement(&mut self) -> Node {
        // 结尾分号是否可以省略
        let mut omit_tailing_semi = false;

        let statement = match self.current_token.token_type {
            TokenType::Keyword => {
                let value = &self.current_token.value;
                if value == "fn" {
                    if self.current_block_level > 0 {
                        panic!("The function can only be defined at the root")
                    }
                    omit_tailing_semi = true;
                    self.parse_function_declaration()
                } else if value == "var" {
                    self.parse_variable_declaration()
                } else if value == "return" {
                    self.parse_return_statement()
                } else {
                    self.unexpected()
                }
            }
            TokenType::Identifier | TokenType::Number | TokenType::ParenL => {
                let expression = Box::new(self.parse_expression());
                Node::ExpressionStatement { expression }
            }
            TokenType::BraceL => {
                omit_tailing_semi = true;
                self.parse_block_statement()
            }
            _ => self.unexpected(),
        };

        let mut tail_semi_count = 0;
        while self.consume(TokenType::Semi) {
            tail_semi_count += 1;
            if self.is_token(TokenType::EOF) {
                break;
            }
        }
        if !omit_tailing_semi && tail_semi_count == 0 {
            self.unexpected();
        }
        statement
    }

    // 解析函数定义语句
    pub fn parse_function_declaration(&mut self) -> Node {
        self.allow_return = true;
        self.next_token();

        // id
        self.expect(TokenType::Identifier);
        let id = Box::new(Node::Identifier {
            name: self.current_token.value.to_string(),
            kind: Kind::None,
        });
        self.next_token();

        // arguments
        let mut arguments = vec![];
        self.consume_or_panic(TokenType::ParenL);
        while self.check_valid_index() && self.is_token(TokenType::Identifier) {
            let name = self.current_token.value.to_string();
            self.next_token();

            // argument kind
            self.consume_or_panic(TokenType::Colon);
            self.expect(TokenType::Identifier);
            let kind_str = self.current_token.value.to_string();
            let kind = Kind::Some(KindName::get(&kind_str, false));

            arguments.push(Box::new(Node::Identifier { name, kind }));
            self.next_token();

            // maybe has next argument
            self.consume(TokenType::Comma);
        }
        self.consume_or_panic(TokenType::ParenR);

        // maybe return kind
        let mut return_kind = Kind::Some(KindName::Void);
        if self.consume(TokenType::ReturnSym) {
            let kind_str = self.current_token.value.to_string();
            return_kind = Kind::Some(KindName::get(&kind_str, true));
            self.next_token();
        }

        // body
        let body = Box::new(self.parse_block_statement());
        self.allow_return = false;

        Node::FunctionDeclaration {
            id,
            arguments,
            body,
            return_kind,
        }
    }

    // 解析块级语句
    pub fn parse_block_statement(&mut self) -> Node {
        // 块级作用域层级 +1
        self.current_block_level += 1;

        let mut body = vec![];
        self.consume_or_panic(TokenType::BraceL);
        while self.check_valid_index() && !self.is_token(TokenType::BraceR) {
            body.push(Box::new(self.parse_statement()));
        }
        self.consume_or_panic(TokenType::BraceR);

        // 块级作用域层级 -1
        self.current_block_level -= 1;

        Node::BlockStatement { body }
    }

    // 解析变量定义语句
    pub fn parse_variable_declaration(&mut self) -> Node {
        self.next_token();

        // id
        self.expect(TokenType::Identifier);
        let id_name = self.current_token.value.to_string();
        self.next_token();

        // maybe variable kind
        let mut kind = Kind::Infer;
        if self.consume(TokenType::Colon) {
            self.expect(TokenType::Identifier);
            let kind_str = self.current_token.value.to_string();
            kind = Kind::Some(KindName::get(&kind_str, true));
            self.next_token();
        }

        let id = Box::new(Node::Identifier {
            name: id_name,
            kind,
        });

        // init
        self.consume_or_panic(TokenType::Assign);
        let init = Box::new(self.parse_expression());

        Node::VariableDeclaration { id, init }
    }

    // 解析 return 语句
    pub fn parse_return_statement(&mut self) -> Node {
        if !self.allow_return {
            panic!("Return can only be use in functions")
        }
        self.next_token();
        let argument = Box::new(self.parse_expression());
        Node::ReturnStatement { argument }
    }
}
