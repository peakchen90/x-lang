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
            Node::Program { body } => {
                for stat in body.iter() {
                    if self.is_stop {
                        break;
                    }
                    self.walk_node(stat.deref(), callback);
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
            Node::VariableDeclaration { id, init } => {
                self.walk_node(id.deref(), callback);
                self.walk_node(init.deref(), callback);
            }
            Node::BlockStatement { body } => {
                for stat in body.iter() {
                    if self.is_stop {
                        break;
                    }
                    self.walk_node(stat.deref(), callback);
                }
            }
            Node::ReturnStatement { argument } => {
                if argument.is_some() {
                    self.walk_node(argument.as_ref().unwrap().deref(), callback);
                }
            }
            Node::ExpressionStatement { expression } => {
                self.walk_node(expression.deref(), callback);
            }
            Node::CallExpression { callee, arguments } => {
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
            Node::AssignmentExpression { left, right, .. } => {
                self.walk_node(left.deref(), callback);
                self.walk_node(right.deref(), callback);
            }
            _ => {}
        }
    }
}
