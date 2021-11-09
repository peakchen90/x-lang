use crate::compile;
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

    #[test]
    fn test1() {
        let code = r#"
            fn a() {}
        "#;
        run_test(code)
    }
}
