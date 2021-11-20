use std::io::repeat;
use std::ops::Deref;
use x_lang_ast::node::Node;
use x_lang_ast::state::Parser;

pub fn format(code: &str) -> String {
    let parser = Parser::new(code);
    let node = parser.node.unwrap();
    Formatter::format(&node, 4)
}

struct Formatter {
    current_indent: usize,
    config_indent: usize,
}

impl Formatter {
    pub fn format(node: &Node, config_indent: usize) -> String {
        let mut formatter = Formatter {
            current_indent: 0,
            config_indent,
        };
        formatter.format_node(node)
    }

    fn push_indent(&mut self) {
        self.current_indent += self.config_indent;
    }

    fn pop_indent(&mut self) {
        self.current_indent -= self.config_indent;
    }

    fn get_indent_str(&self) -> String {
        let mut str = String::new();
        for i in 0..self.current_indent {
            str.push(' ');
        }
        str
    }

    fn format_node(&mut self, node: &Node) -> String {
        let mut code = String::new();
        match node {
            Node::Program { body } => {
                for i in body.iter() {
                    code.push_str(self.format_node(i.deref()).as_str());
                }
            }
            Node::ImportDeclaration {
                source,
                is_std_source,
                specifiers,
            } => {
                code.push_str("import ");
                if *is_std_source {
                    code.push_str("<")
                }
                code.push_str(source);
                if *is_std_source {
                    code.push_str(">")
                }
                if let Some(specifiers) = specifiers {
                    code.push_str(".{");
                    for (i, specifier) in specifiers.iter().enumerate() {
                        code.push_str(&self.format_node(specifier.deref()));
                        if i < specifiers.len() - 1 {
                            code.push_str(", ");
                        }
                    }
                    code.push_str("}");
                }
                code.push_str("\n");
            }
            Node::FunctionDeclaration {
                id,
                arguments,
                body,
                return_kind,
                is_pub,
            } => {
                code.push_str("\n");
                if *is_pub {
                    code.push_str("pub ");
                }
                code.push_str("fn");
                let (name, ..) = id.deref().read_identifier();
                code.push_str(" ");
                code.push_str(name);
                code.push_str("(");

                for (i, arg) in arguments.iter().enumerate() {
                    let (arg_name, arg_kind) = arg.deref().read_identifier();
                    code.push_str(arg_name);
                    code.push_str(" :");
                    code.push_str(&arg_kind.to_string());
                    if i < arguments.len() - 1 {
                        code.push_str(", ");
                    }
                }

                code.push_str(")");
                if return_kind.is_exact() {
                    code.push_str(" -> ");
                    code.push_str(&return_kind.to_string());
                }
                code.push_str(" ");
                code.push_str(&self.format_node(body));
                code.push_str("\n");
            }
            Node::VariableDeclaration { id, init } => {
                code.push_str("var ");
                let (name, kind) = id.deref().read_identifier();
                code.push_str(name);
                code.push_str(" = ");
                code.push_str(&self.format_node(init.deref()));
                code.push_str(";\n");
            }
            Node::BlockStatement { body } => {
                code.push_str(&self.format_block(body, true))
            }
            Node::ReturnStatement { argument } => {
                code.push_str("return");
                if let Some(v) = argument {
                    code.push_str(" ");
                    code.push_str(&self.format_node(v.deref()))
                }
                code.push_str(";\n");
            }
            Node::ExpressionStatement { expression } => {
                code.push_str(&self.format_node(expression.deref()));
                code.push_str(";\n")
            }
            Node::IfStatement {
                condition,
                consequent,
                alternate,
            } => {
                code.push_str("if (");
                code.push_str(&self.format_node(condition.deref()));
                code.push_str(") ");
                code.push_str(
                    &self.format_block(consequent.deref().read_block_body(), false),
                );
                if let Some(v) = alternate {
                    code.push_str(" else ");
                    code.push_str(&self.format_node(v.deref()));
                } else {
                    code.push_str("\n");
                }
            }
            Node::LoopStatement { label, body } => {
                if let Some(v) = label {
                    code.push_str(v);
                    code.push_str(": ")
                }
                code.push_str("loop ");
                code.push_str(&self.format_node(body.deref()));
            }
            Node::BreakStatement { label } => {
                code.push_str("break");
                if let Some(v) = label {
                    code.push_str(" ");
                    code.push_str(v);
                }
                code.push_str(";\n")
            }
            Node::ContinueStatement { label } => {
                code.push_str("continue");
                if let Some(v) = label {
                    code.push_str(" ");
                    code.push_str(v);
                }
                code.push_str(";\n")
            }
            Node::ImportSpecifier { imported, local } => {
                code.push_str(imported);
                if let Some(local) = local {
                    code.push_str(" as ");
                    code.push_str(local);
                }
            }
            Node::CallExpression { callee, arguments } => {
                let (callee_str, ..) = callee.deref().read_identifier();
                code.push_str(callee_str);
                code.push_str("(");
                for (i, arg) in arguments.iter().enumerate() {
                    code.push_str(&self.format_node(arg.deref()));
                    if i < arguments.len() - 1 {
                        code.push_str(", ");
                    }
                }
                code.push_str(")");
            }
            Node::BinaryExpression {
                left,
                right,
                operator,
            } => {
                code.push_str(&self.format_node(left.deref()));
                code.push_str(" ");
                code.push_str(operator);
                code.push_str(" ");
                code.push_str(&self.format_node(right.deref()));
            }
            Node::UnaryExpression { argument, operator } => {
                code.push_str(operator);
                code.push_str(&self.format_node(argument.deref()));
            }
            Node::AssignmentExpression {
                left,
                right,
                operator,
            } => {
                code.push_str(&self.format_node(left.deref()));
                code.push_str(" ");
                code.push_str(operator);
                code.push_str(" ");
                code.push_str(&self.format_node(right.deref()));
            }
            Node::Identifier { name, .. } => {
                code.push_str(name);
            }
            Node::NumberLiteral { value } => {
                code.push_str(&value.to_string());
            }
            Node::BooleanLiteral { value } => {
                code.push_str(&value.to_string());
            }
        };
        code
    }

    fn format_block(
        &mut self,
        statements: &Vec<Box<Node>>,
        tail_newline: bool,
    ) -> String {
        let mut code = String::new();
        code.push_str("{\n");
        self.push_indent();
        for i in statements.iter() {
            code.push_str(&self.get_indent_str());
            code.push_str(&self.format_node(i.deref()));
        }
        self.pop_indent();
        code.push_str(&self.get_indent_str());
        code.push_str("}");
        if tail_newline {
            code.push_str("\n");
        }
        code
    }
}
