pub use self::build_info::build_info;
pub use self::parser::parse;

use logging::console_log_build_info;

use error::set_panic_hook;
use wasm_bindgen::prelude::wasm_bindgen;

#[macro_use]
mod logging;
mod build_info;
mod error;
mod parser;
mod timer;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn init_console() {
  set_panic_hook();
  console_log_build_info();
}
