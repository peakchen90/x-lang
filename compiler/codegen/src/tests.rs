use crate::compile;
use std::{env, fs};
use x_lang_ast::state::Parser;

fn run_test(code: &str) {
    let parser = Parser::new(&code);
    let node = parser.node.unwrap();
    compile(&node, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile;
    use std::path::PathBuf;

    #[test]
    fn test1() {
        let mut fixtures_dir =
            PathBuf::from(env::current_dir().unwrap()).join("fixtures");

        let files = fs::read_dir(fixtures_dir);
        for i in files.unwrap() {
            let code = fs::read_to_string(i.unwrap().path()).unwrap();
            run_test(&code);
        }
    }
}
