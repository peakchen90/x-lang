mod build_in;
mod compiler;
mod expression;
mod helper;
mod scope;

use x_lang_ast::node::Node;
use crate::compiler::Compiler;

pub fn compile(ast: &Node, is_debug: bool) {
    Compiler::compile(ast, is_debug)
}
