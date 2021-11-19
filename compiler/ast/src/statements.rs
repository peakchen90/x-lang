use crate::node::Node;
use crate::shared::{Kind, KindName};
use crate::state::Parser;
use crate::token::TokenType;

impl<'a> Parser<'a> {
    // 解析一条语句
    pub fn parse_statement(&mut self) -> Node {
        // 结尾分号是否可以省略
        let mut omit_tailing_semi = false;

        let statement = match self.current_token.token_type {
            TokenType::Keyword => {
                let value = &self.current_token.value;
                match value.as_bytes() {
                    b"fn" => {
                        if self.current_block_level > 0 {
                            panic!("Functions can only be defined up to the top level")
                        }
                        omit_tailing_semi = true;
                        self.parse_function_declaration()
                    }
                    b"var" => {
                        self.validate_inside_fn();
                        self.parse_variable_declaration()
                    }
                    b"return" => {
                        self.validate_inside_fn();
                        self.parse_return_statement()
                    }
                    b"if" => {
                        self.validate_inside_fn();
                        omit_tailing_semi = true;
                        self.parse_if_statement()
                    }
                    b"loop" => {
                        self.validate_inside_fn();
                        omit_tailing_semi = true;
                        self.parse_loop_statement(None)
                    }
                    b"break" => {
                        self.validate_inside_fn();
                        if self.current_loop_level == 0 {
                            panic!("The `break` can only be use in loop statements")
                        }
                        self.parse_break_statement()
                    }
                    b"continue" => {
                        self.validate_inside_fn();
                        if self.current_loop_level == 0 {
                            panic!("The `continue` can only be use in loop statements")
                        }
                        self.parse_continue_statement()
                    }
                    _ => self.unexpected(),
                }
            }
            TokenType::Identifier => {
                self.validate_inside_fn();

                // 可能是 label
                let maybe_label = self.current_token.value.to_string();
                if self.check_next_char(':') {
                    self.next_token();
                    self.consume_or_panic(TokenType::Colon);
                    self.expect(TokenType::Keyword);
                    match self.current_token.value.as_bytes() {
                        b"loop" => {
                            omit_tailing_semi = true;
                            self.parse_loop_statement(Some(maybe_label))
                        }
                        _ => self.unexpected(),
                    }
                } else {
                    let expression = Box::new(self.parse_expression().unwrap());
                    Node::ExpressionStatement { expression }
                }
            }
            TokenType::Number | TokenType::ParenL => {
                self.validate_inside_fn();
                let expression = Box::new(self.parse_expression().unwrap());
                Node::ExpressionStatement { expression }
            }
            TokenType::BraceL => {
                self.validate_inside_fn();
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
        if !omit_tailing_semi
            && tail_semi_count == 0
            && !self.is_seen_newline
            && !self.is_token(TokenType::EOF)
            && !self.is_token(TokenType::BraceR)
        {
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
            let kind = KindName::from(&kind_str, false).into();

            arguments.push(Box::new(Node::Identifier { name, kind }));
            self.next_token();

            // maybe has next argument
            self.consume(TokenType::Comma);
        }
        self.consume_or_panic(TokenType::ParenR);

        // maybe return kind
        let mut return_kind = Kind::None;
        if self.consume(TokenType::ReturnSym) {
            let kind_str = self.current_token.value.to_string();
            return_kind = KindName::from(&kind_str, true).into();
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
            kind = KindName::from(&kind_str, true).into();
            self.next_token();
        }

        let id = Box::new(Node::Identifier {
            name: id_name,
            kind,
        });

        // init
        self.consume_or_panic(TokenType::Assign);
        let init = Box::new(self.parse_expression().unwrap());

        Node::VariableDeclaration { id, init }
    }

    // 解析 return 语句
    pub fn parse_return_statement(&mut self) -> Node {
        if !self.allow_return {
            panic!("Return can only be use in functions")
        }
        self.next_token();

        let argument = self.parse_expression();
        Node::ReturnStatement {
            argument: match argument {
                Some(v) => Some(Box::new(v)),
                None => None,
            },
        }
    }

    // 解析 if 语句
    pub fn parse_if_statement(&mut self) -> Node {
        // 递归解析时，如果不是 else-if，只需解析块语句就行了
        if !self.is_keyword("if") {
            return self.parse_block_statement();
        }

        self.next_token();

        // condition
        let has_paren = self.consume(TokenType::ParenL);
        let condition = self.parse_expression().unwrap();
        if has_paren {
            self.consume_or_panic(TokenType::ParenR);
        }

        // consequent
        let consequent = self.parse_block_statement();

        // alternate
        let alternate = if self.is_keyword("else") {
            self.next_token();
            Some(Box::new(self.parse_if_statement()))
        } else {
            None
        };

        Node::IfStatement {
            condition: Box::new(condition),
            consequent: Box::new(consequent),
            alternate,
        }
    }

    // 解析 loop 循环语句
    pub fn parse_loop_statement(&mut self, label: Option<String>) -> Node {
        self.current_loop_level += 1;

        self.next_token();
        let body = Box::new(self.parse_block_statement());

        self.current_loop_level -= 1;
        Node::LoopStatement { label, body }
    }

    // 解析 break 语句
    pub fn parse_break_statement(&mut self) -> Node {
        self.next_token();
        let label = if self.is_token(TokenType::Identifier) {
            let label = Some(self.current_token.value.to_string());
            self.next_token();
            label
        } else {
            None
        };
        Node::BreakStatement { label }
    }

    // 解析 continue 语句
    pub fn parse_continue_statement(&mut self) -> Node {
        self.next_token();
        let label = if self.is_token(TokenType::Identifier) {
            Some(self.current_token.value.to_string())
        } else {
            None
        };
        Node::ContinueStatement { label }
    }
}
