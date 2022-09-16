pub use self::build_info::build_info;
pub use self::parser::parse;

use error::set_panic_hook;
use wasm_bindgen::prelude::wasm_bindgen;

#[macro_use]
mod logging;
mod build_info;
mod error;
mod parser;
mod timer;

#[wasm_bindgen(start)]
pub fn init_console() {
  set_panic_hook();
}
