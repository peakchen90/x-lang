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

    let mut input_content = fs::read_to_string(filename).unwrap();
    let content: Vec<char> = input_content.chars().collect();

    // remove shell env header (start with `#!`)
    if content[0] == '#' && content[1] == '!' {
        let content = input_content.lines().skip(1).collect::<Vec<&str>>();
        input_content = content.join("\n");
    }

    // parse ast
    let parser = Parser::new(&input_content);
    let node = parser.node.unwrap();

    compile(&node, is_debug);
}
