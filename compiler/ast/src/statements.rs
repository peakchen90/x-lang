use crate::node::Node;
use crate::shared::{Kind, KindName};
use crate::state::Parser;
use crate::token::TokenType;
use std::collections::HashSet;

impl<'a> Parser<'a> {
    // 解析一条语句
    pub fn parse_statement(&mut self) -> Node {
        // 结尾分号是否可以省略
        let mut omit_tailing_semi = false;

        let statement = match self.current_token.token_type {
            TokenType::Keyword => {
                let value = &self.current_token.value;
                match value.as_bytes() {
                    b"pub" => {
                        self.next_token();
                        if self.is_token(TokenType::Keyword) {
                            match self.current_token.value.as_bytes() {
                                b"fn" => {
                                    omit_tailing_semi = true;
                                    self.parse_function_declaration(true)
                                }
                                _ => self.unexpected(None),
                            }
                        } else {
                            self.unexpected(None);
                        }
                    }
                    b"import" => self.parse_import_declaration(),
                    b"fn" => {
                        omit_tailing_semi = true;
                        self.parse_function_declaration(false)
                    }
                    b"var" => self.parse_variable_declaration(),
                    b"return" => self.parse_return_statement(),
                    b"if" => {
                        omit_tailing_semi = true;
                        self.parse_if_statement()
                    }
                    b"loop" => {
                        omit_tailing_semi = true;
                        self.parse_loop_statement(None)
                    }
                    b"break" => self.parse_break_statement(),
                    b"continue" => self.parse_continue_statement(),
                    _ => self.unexpected(None),
                }
            }
            TokenType::Identifier => {
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
                        _ => self.unexpected(None),
                    }
                } else {
                    self.parse_expression_statement()
                }
            }
            TokenType::Number | TokenType::ParenL => self.parse_expression_statement(),
            TokenType::BraceL => {
                omit_tailing_semi = true;
                self.parse_block_statement(false)
            }
            _ => self.parse_expression_statement(),
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
            self.unexpected(None);
        }

        statement
    }

    // 解析 import 语句
    pub fn parse_import_declaration(&mut self) -> Node {
        self.validate_program_root("Import declaration");

        self.skip_space();
        self.skip_comment();
        let mut source = String::new();
        let mut is_std_source = false;
        let mut specifiers = None;

        let mut has_std_ending = false;

        if self.current_char == '<' {
            is_std_source = true;
            self.move_index(1);
        }

        // parse import source
        while self.check_valid_index() {
            if is_std_source && self.current_char == '>' {
                has_std_ending = true;
                self.move_index(1);
                if self.current_char == '.' && self.look_behind(1) == '{' {
                    specifiers = Some(vec![]);
                    self.move_index(1);
                }
                break;
            }
            if self.current_char == '.' && self.look_behind(1) == '{' {
                specifiers = Some(vec![]);
                self.move_index(1);
                break;
            }
            if self.is_space_char() {
                break;
            }
            source.push(self.current_char);
            self.move_index(1);
        }
        if is_std_source && !has_std_ending {
            panic!("Invalid import source")
        }
        if source.is_empty() {
            panic!("Invalid import declaration, missing source")
        }

        // parse import specifiers
        if let Some(ref mut specifiers) = specifiers {
            self.next_token();
            self.consume_or_panic(TokenType::BraceL);
            let mut specifier_set = HashSet::new();
            while self.check_valid_index() && !self.is_token(TokenType::BraceR) {
                let imported = self.current_token.value.to_string();
                let mut local = None;

                if self.is_token(TokenType::Star) {
                    self.next_token();
                } else {
                    self.expect(TokenType::Identifier);
                    self.next_token();
                    if self.is_keyword("as") {
                        self.next_token();
                        self.expect(TokenType::Identifier);
                        local = Some(self.current_token.value.to_string());
                        self.next_token();
                    }
                }

                // 重复校验
                let spec = match &local {
                    None => &imported,
                    Some(v) => v,
                };
                let spec = spec.clone();
                if specifier_set.get(&spec).is_some() {
                    panic!("The import specifier `{}` already exists", spec);
                }
                specifier_set.insert(spec);

                let specifier = Node::ImportSpecifier { imported, local };
                specifiers.push(Box::new(specifier));

                // maybe has next specifier
                if !self.consume(TokenType::Comma) {
                    break;
                }
            }
            self.consume_or_panic(TokenType::BraceR);
        } else {
            self.next_token();
        }

        Node::ImportDeclaration {
            source,
            is_std_source,
            specifiers,
        }
    }

    // 解析函数定义语句
    pub fn parse_function_declaration(&mut self, is_pub: bool) -> Node {
        self.validate_program_root("Function declaration");

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
            let kind_name = KindName::from(&kind_str, false);
            if kind_name.is_none() {
                self.unexpected_kind(self.current_token.clone());
            }

            arguments.push(Box::new(Node::Identifier {
                name,
                kind: kind_name.unwrap().into(),
            }));
            self.next_token();

            // maybe has next argument
            self.consume(TokenType::Comma);
        }
        self.consume_or_panic(TokenType::ParenR);

        // maybe return kind
        let mut return_kind = Kind::None;
        if self.consume(TokenType::ReturnSym) {
            self.expect(TokenType::Identifier);
            let kind_str = self.current_token.value.to_string();
            let kind_name = KindName::from(&kind_str, true);
            if kind_name.is_none() {
                self.unexpected_kind(self.current_token.clone());
            }
            return_kind = kind_name.unwrap().into();
            self.next_token();
        }

        // body
        let body = Box::new(self.parse_block_statement(true));
        self.allow_return = false;

        Node::FunctionDeclaration {
            id,
            arguments,
            body,
            return_kind,
            is_pub,
        }
    }

    // 解析表达式语句
    pub fn parse_expression_statement(&mut self) -> Node {
        self.validate_inside_fn();
        let expression = self.parse_expression();
        if expression.is_none() {
            self.unexpected(Some("Invalid expression"));
        }
        let expression = Box::new(expression.unwrap());
        Node::ExpressionStatement { expression }
    }

    // 解析块级语句
    pub fn parse_block_statement(&mut self, with_fn: bool) -> Node {
        if !with_fn {
            self.validate_inside_fn();
        }

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
        self.validate_inside_fn();
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
            let kind_name = KindName::from(&kind_str, false);
            if kind_name.is_none() {
                self.unexpected_kind(self.current_token.clone());
            }
            kind = kind_name.unwrap().into();
            self.next_token();
        }

        let id = Box::new(Node::Identifier {
            name: id_name,
            kind,
        });

        // init
        let op_token = self.current_token.clone();
        self.consume_or_panic(TokenType::Assign);
        let init = self.parse_expression();
        if init.is_none() {
            self.unexpected_token(op_token, Some("Missing initial value"));
        }
        let init = Box::new(init.unwrap());

        Node::VariableDeclaration { id, init }
    }

    // 解析 return 语句
    pub fn parse_return_statement(&mut self) -> Node {
        self.validate_inside_fn();
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
        self.validate_inside_fn();

        // 递归解析时，如果不是 else-if，只需解析块语句就行了
        if !self.is_keyword("if") {
            return self.parse_block_statement(false);
        }

        let if_token = self.current_token.clone();
        self.next_token();

        // condition
        let has_paren = self.consume(TokenType::ParenL);
        let condition = self.parse_expression();
        if condition.is_none() {
            self.unexpected_err(if_token.end, "Missing condition");
        }

        if has_paren {
            self.consume_or_panic(TokenType::ParenR);
        }

        // consequent
        let consequent = self.parse_block_statement(false);

        // alternate
        let alternate = if self.is_keyword("else") {
            self.next_token();
            Some(Box::new(self.parse_if_statement()))
        } else {
            None
        };

        Node::IfStatement {
            condition: Box::new(condition.unwrap()),
            consequent: Box::new(consequent),
            alternate,
        }
    }

    // 解析 loop 循环语句
    pub fn parse_loop_statement(&mut self, label: Option<String>) -> Node {
        self.validate_inside_fn();
        self.current_loop_level += 1;

        self.next_token();
        let body = Box::new(self.parse_block_statement(false));

        self.current_loop_level -= 1;
        Node::LoopStatement { label, body }
    }

    // 解析 break 语句
    pub fn parse_break_statement(&mut self) -> Node {
        self.validate_inside_fn();
        if self.current_loop_level == 0 {
            panic!("The `break` can only be use in loop statements")
        }

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
        self.validate_inside_fn();
        if self.current_loop_level == 0 {
            panic!("The `continue` can only be use in loop statements")
        }

        self.next_token();
        let label = if self.is_token(TokenType::Identifier) {
            Some(self.current_token.value.to_string())
        } else {
            None
        };
        Node::ContinueStatement { label }
    }
}
