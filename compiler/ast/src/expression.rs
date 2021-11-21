use crate::node::Node;
use crate::shared::Kind;
use crate::state::Parser;
use crate::token::{Token, TokenType};

impl<'a> Parser<'a> {
    // 解析表达式
    pub fn parse_expression(&mut self) -> Option<Node> {
        self.parse_maybe_binary_expression(-1)
    }

    // 解析一个二元表达式（可能）
    pub fn parse_maybe_binary_expression(
        &mut self,
        current_precedence: i8,
    ) -> Option<Node> {
        if self.is_token(TokenType::ParenL) {
            self.next_token();
            let expr = self.parse_expression();
            if expr.is_none() {
                self.unexpected(Some("Missing expression"));
            }
            self.consume_or_panic(TokenType::ParenR);
            self.parse_binary_expression_precedence(expr.unwrap(), current_precedence)
        } else {
            let left = self.parse_maybe_unary_expression(current_precedence)?;
            self.parse_binary_expression_precedence(left, current_precedence)
        }
    }

    // 解析一个一元表达式（可能）
    pub fn parse_maybe_unary_expression(
        &mut self,
        current_precedence: i8,
    ) -> Option<Node> {
        if self.current_token.precedence <= current_precedence {
            return self.parse_atom_expression();
        }

        match self.current_token.token_type {
            TokenType::Sub | TokenType::Plus => {
                panic!("TODO: No implement")
            }
            TokenType::LogicNot | TokenType::BitNot => {
                let operator = self.current_token.value.to_string();
                let start = self.current_token.start;
                self.next_token();
                let argument = self.parse_expression();
                if argument.is_none() {
                    self.unexpected_err(start, "Incomplete unary expression");
                }
                let argument = argument.unwrap();
                Some(Node::UnaryExpression {
                    position: (start, argument.read_position().1),
                    operator,
                    argument: Box::new(argument),
                })
            }
            _ => self.parse_atom_expression(),
        }
    }

    // 解析二元表达式优先级
    pub fn parse_binary_expression_precedence(
        &mut self,
        left: Node,
        current_precedence: i8,
    ) -> Option<Node> {
        let precedence = self.current_token.precedence;

        // 如果当前二元运算符优先级比当前上下文优先级高，优先组合（只有二元运算符优先级会大于0）
        if precedence > 0 && precedence > current_precedence {
            let operator = self.current_token.value.to_string();
            if operator == "=" {
                self.unexpected(None);
            }
            let mark_pos = self.current_token.start;
            self.next_token();

            // 解析可能更高优先级的右侧表达式，如: `1 + 2 * 3` 将解析 `2 * 3` 作为右值
            let maybe_higher_precedence_expr =
                self.parse_maybe_binary_expression(precedence);
            if maybe_higher_precedence_expr.is_none() {
                self.unexpected_err(mark_pos, "Incomplete binary expression");
            }
            let right = self.parse_binary_expression_precedence(
                maybe_higher_precedence_expr.unwrap(),
                precedence,
            );
            if right.is_none() {
                self.unexpected_err(mark_pos, "Incomplete binary expression");
            }
            let right = right.unwrap();
            let node = Node::BinaryExpression {
                position: (left.read_position().0, right.read_position().1),
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };

            // 将已经解析的二元表达式作为左值，然后递归解析后面可能的同等优先级或低优先级的表达式作为右值
            // 如: `1 + 2 + 3`, 当前已经解析 `1 + 2`, 然后将该节点作为左值递归解析表达式优先级
            self.parse_binary_expression_precedence(node, current_precedence)
        } else {
            Some(left)
        }
    }

    // 解析一个原子表达式，如: `foo()`, `3.14`, `var1`, `var2 = expr`, `true`
    pub fn parse_atom_expression(&mut self) -> Option<Node> {
        let token = self.current_token.clone();
        match self.current_token.token_type {
            TokenType::Identifier => {
                self.next_token();
                let next_token = &self.current_token;
                let next_value = next_token.value.to_string();

                match next_token.token_type {
                    TokenType::ParenL => self.parse_call_expression(token),
                    // 赋值表达式
                    TokenType::Assign => {
                        let left = self.gen_identifier(token, Kind::Infer);
                        let mark_pos = self.current_token.start;
                        self.next_token();

                        let right = self.parse_expression();
                        if right.is_none() {
                            self.unexpected_err(mark_pos, "Missing initial value");
                        }
                        let right = right.unwrap();
                        Some(Node::AssignmentExpression {
                            position: (left.read_position().0, right.read_position().1),
                            operator: next_value,
                            left: Box::new(left),
                            right: Box::new(right),
                        })
                    }
                    _ => Some(self.gen_identifier(token, Kind::Infer)),
                }
            }
            TokenType::Number => {
                self.next_token();
                Some(Node::NumberLiteral {
                    position: (token.start, token.end),
                    value: token.value.parse().unwrap(),
                })
            }
            TokenType::Boolean => {
                self.next_token();
                Some(Node::BooleanLiteral {
                    position: (token.start, token.end),
                    value: token.value == "true",
                })
            }
            _ => None,
        }
    }

    // 解析函数调用
    pub fn parse_call_expression(&mut self, callee_token: Token) -> Option<Node> {
        let start = callee_token.start;
        let callee = Box::new(self.gen_identifier(callee_token, Kind::None));

        // arguments
        let mut arguments = vec![];
        self.consume_or_panic(TokenType::ParenL);
        while self.check_valid_index() && !self.is_token(TokenType::ParenR) {
            let arg = self.parse_expression();
            if arg.is_none() {
                self.unexpected(Some("Invalid argument"))
            }
            arguments.push(Box::new(arg.unwrap()));
            self.consume(TokenType::Comma);
        }
        let end = self.current_token.end;
        self.consume_or_panic(TokenType::ParenR);

        Some(Node::CallExpression {
            position: (start, end),
            callee,
            arguments,
        })
    }
}
