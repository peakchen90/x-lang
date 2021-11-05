use crate::node::Node;
use crate::state::Parser;
use crate::token::TokenType;

impl Parser {
    // 解析表达式
    pub fn parse_expression(&mut self) -> Node {
        self.parse_maybe_binary_expression(-1)
    }

    // 解析一个可能的二元表达式
    pub fn parse_maybe_binary_expression(
        &mut self,
        current_precedence: i8,
    ) -> Node {
        if self.current_token.token_type == TokenType::ParenL {
            self.next_token();
            let left = self.parse_expression();
            self.consume_or_panic(TokenType::ParenR);
            self.parse_binary_expression_precedence(left, current_precedence)
        } else {
            let left = self.parse_atom_expression();
            self.parse_binary_expression_precedence(left, current_precedence)
        }
    }

    // 解析二元表达式优先级
    pub fn parse_binary_expression_precedence(
        &mut self,
        left: Node,
        current_precedence: i8,
    ) -> Node {
        let precedence = self.current_token.precedence;

        // 如果当前二元运算符优先级比当前上下文优先级高，优先组合（只有二元运算符优先级会大于0）
        if precedence > 0 && precedence > current_precedence {
            let operator = self.current_token.value.to_string();
            if operator == "=" {
                self.unexpected();
            }

            // 解析可能更高优先级的右侧表达式，如: `1 + 2 * 3` 将解析 `2 * 3` 作为右值
            self.next_token();
            let maybe_higher_precedence_expr =
                self.parse_maybe_binary_expression(precedence);
            let right = self.parse_binary_expression_precedence(
                maybe_higher_precedence_expr,
                precedence,
            );

            let node = Node::BinaryExpression {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            };

            // 将已经解析的二元表达式作为左值，然后递归解析后面可能的同等优先级或低优先级的表达式作为右值
            // 如: `1 + 2 + 3`, 当前已经解析 `1 + 2`, 然后将该节点作为左值递归解析表达式优先级
            self.parse_binary_expression_precedence(node, current_precedence)
        } else {
            left
        }
    }

    // 解析一个原子表达式，如: `foo()`, `3.14`, `var1`, `var2 = expr`
    pub fn parse_atom_expression(&mut self) -> Node {
        let value = self.current_token.value.to_string();
        match self.current_token.token_type {
            TokenType::Identifier => {
                self.next_token();
                let next_token = &self.current_token;
                let next_value = next_token.value.to_string();

                match next_token.token_type {
                    TokenType::ParenL => self.parse_call_expression(&value),
                    // 赋值表达式
                    TokenType::Eq => {
                        let left = Box::new(Node::Identifier { name: value });
                        self.next_token();
                        let right = Box::new(self.parse_expression());
                        Node::AssignmentExpression {
                            left,
                            right,
                            operator: next_value,
                        }
                    }
                    _ => Node::Identifier { name: value },
                }
            }
            TokenType::Number => {
                self.next_token();
                Node::NumberLiteral {
                    value: value.parse().unwrap(),
                }
            }
            _ => self.unexpected(),
        }
    }

    // 解析函数调用
    pub fn parse_call_expression(&mut self, callee_name: &str) -> Node {
        let callee = Box::new(Node::Identifier {
            name: callee_name.to_string(),
        });

        // arguments
        let mut arguments = vec![];
        self.consume_or_panic(TokenType::ParenL);
        while self.check_valid_index() && !self.is_token(TokenType::ParenR) {
            let arg = self.parse_expression();
            arguments.push(Box::new(arg));
            self.consume(TokenType::Comma);
        }
        self.consume_or_panic(TokenType::ParenR);

        Node::CallExpression { callee, arguments }
    }
}
