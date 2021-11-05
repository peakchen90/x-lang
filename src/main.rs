use x_lang_ast::state::Parser;
use std::fs;

fn main() {
    let str = fs::read_to_string("test.x").unwrap();
    let parser = Parser::new(&str);
    let node = parser.node.unwrap();

    let ast_json_str = serde_json::to_string(&node).unwrap();
    let format_json_str = tiny_json::stringify(
        &tiny_json::parse(&ast_json_str),
        2,
    );
    fs::write(".ast.json", format_json_str).unwrap();
    println!("Success: write ast at: .ast.json")
}
