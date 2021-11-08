use std::env;
use std::fs;
use std::time::Instant;
use x_lang_ast::state::Parser;
use x_lang_codegen::compile;

fn print_help_info() {
    println!("Usage: x-lang <path/example.x> [--debug]");
}

pub fn handle_commander() {
    let mut i = 0;
    let args: Vec<String> = env::args().collect();
    let args = &args[1..];

    if args.len() == 0 {
        print_help_info();
        return;
    }

    let mut args = args.iter();
    let filename = args.next().expect("Missing filename");
    let is_debug = match args.next() {
        Some(v) => {
            if v == "--debug" {
                true
            } else {
                panic!("Invalid argument: {}", v);
            }
        }
        None => false,
    };

    let content = fs::read_to_string(filename).unwrap();

    // parse ast
    let parser = Parser::new(&content);
    let node = parser.node.unwrap();

    compile(&node, is_debug);
}
