mod helper;
mod parse;

#[macro_use]
extern crate napi_derive;

use crate::parse::parse;

#[module_exports]
fn init(mut exports: napi::JsObject) -> napi::Result<()> {
    exports.create_named_method("parse", parse)?;
    Ok(())
}
