mod cli;

use std::fs;
use std::process::Command;
use x_lang_ast::state::Parser;
use x_lang_codegen::compile;
use x_lang_format_tool::format;

fn ast_test() {
    let str = fs::read_to_string("test.x").unwrap();
    let parser = Parser::new(&str);
    let node = parser.node.unwrap();

    let ast_json_str = serde_json::to_string(&node).unwrap();
    let format_json_str =
        tiny_json::stringify(&tiny_json::parse(&ast_json_str), 2);
    fs::write(".debug_ast.json", format_json_str).unwrap();

    // format code
    let format_code = format(&str);
    fs::write("test.x", format_code);

    compile(&node, true);
}

fn main() {
    ast_test();
    // cli::handle_commander();
}
