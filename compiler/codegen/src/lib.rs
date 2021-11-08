mod build_in;
mod compiler;
mod expression;
mod helper;
mod scope;

use crate::compiler::Compiler;
use x_lang_ast::shared::Node;

pub fn compile(ast: &Node, is_debug: bool) {
    Compiler::compile(ast, is_debug)
}
