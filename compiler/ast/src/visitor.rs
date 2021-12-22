use crate::node::Node;
use std::ops::Deref;

pub struct Visitor {
    is_stop: bool,
}

impl Visitor {
    pub fn walk<T: FnMut(&Node, &mut Visitor)>(node: &Node, callback: &mut T) {
        let mut visitor = Visitor { is_stop: false };
        visitor.walk_node(node, callback);
    }

    // 停止遍历
    pub fn stop(&mut self) {
        self.is_stop = true;
    }

    // 递归访问 node
    fn walk_node<T: FnMut(&Node, &mut Visitor)>(
        &mut self,
        node: &Node,
        callback: &mut T,
    ) {
        if self.is_stop {
            return;
        }
        callback(node, self);

        match node {
            Node::Program { body, .. } => {
                for stat in body.iter() {
                    if self.is_stop {
                        break;
                    }
                    self.walk_node(stat.deref(), callback);
                }
            }
            Node::ImportDeclaration { specifiers, .. } => {
                if let Some(specifiers) = specifiers {
                    for specifier in specifiers.iter() {
                        if self.is_stop {
                            break;
                        }
                        self.walk_node(specifier.deref(), callback);
                    }
                }
            }
            Node::FunctionDeclaration {
                id,
                arguments,
                body,
                ..
            } => {
                self.walk_node(id.deref(), callback);
                for arg in arguments.iter() {
                    if self.is_stop {
                        break;
                    }
                    self.walk_node(arg.deref(), callback);
                }
                self.walk_node(body.deref(), callback);
            }
            Node::VariableDeclaration { id, init, .. } => {
                self.walk_node(id.deref(), callback);
                self.walk_node(init.deref(), callback);
            }
            Node::BlockStatement { body, .. } => {
                for stat in body.iter() {
                    if self.is_stop {
                        break;
                    }
                    self.walk_node(stat.deref(), callback);
                }
            }
            Node::ReturnStatement { argument, .. } => {
                if let Some(argument) = argument {
                    self.walk_node(argument.deref(), callback);
                }
            }
            Node::ExpressionStatement { expression, .. } => {
                self.walk_node(expression.deref(), callback);
            }
            Node::IfStatement {
                condition,
                consequent,
                alternate,
                ..
            } => {
                self.walk_node(condition.deref(), callback);
                self.walk_node(consequent.deref(), callback);
                if let Some(alternate) = alternate {
                    self.walk_node(alternate.deref(), callback);
                }
            }
            Node::LoopStatement { body, .. } => {
                self.walk_node(body.deref(), callback);
            }
            Node::BreakStatement { .. } => {}
            Node::ContinueStatement { .. } => {}
            Node::ImportSpecifier { .. } => {}
            Node::CallExpression {
                callee, arguments, ..
            } => {
                self.walk_node(callee.deref(), callback);
                for arg in arguments.iter() {
                    if self.is_stop {
                        break;
                    }
                    self.walk_node(arg.deref(), callback);
                }
            }
            Node::BinaryExpression { left, right, .. } => {
                self.walk_node(left.deref(), callback);
                self.walk_node(right.deref(), callback);
            }
            Node::UnaryExpression { argument, .. } => {
                self.walk_node(argument.deref(), callback);
            }
            Node::AssignmentExpression { left, right, .. } => {
                self.walk_node(left.deref(), callback);
                self.walk_node(right.deref(), callback);
            }
            Node::Identifier { .. } => {}
            Node::NumberLiteral { .. } => {}
            Node::BooleanLiteral { .. } => {}
            Node::StringLiteral { .. } => {}
        }
    }
}
