mod build_in;
mod compiler;
mod expression;
mod helper;
mod scope;
mod string;
mod utils;

mod tests;

use crate::compiler::Compiler;
use x_lang_ast::node::Node;

pub fn compile(source: &str, is_debug: bool) {
    Compiler::compile(source, is_debug);
}
