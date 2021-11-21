mod build_in;
mod compiler;
mod expression;
mod helper;
mod scope;
mod tests;

use crate::compiler::Compiler;
use x_lang_ast::node::Node;

pub fn compile(ast: &Node, is_debug: bool) {
    Compiler::compile(ast, is_debug);
}
