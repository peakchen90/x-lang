use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern {
    pub fn __throw__(msg: &str);
    pub fn __logError__(msg: &str);
}
