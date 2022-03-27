mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn format(code: &str) -> String {
    return x_lang_format_tool::format(code);
}

#[wasm_bindgen]
pub fn parse(input: &str) -> String {
    let ast = x_lang_ast::state::Parser::new(input).parse();
    serde_json::to_string(&ast).unwrap()
}
