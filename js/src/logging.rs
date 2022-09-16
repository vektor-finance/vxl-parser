// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[allow(unused_macros)]
macro_rules! console_log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use crate::build_info::build_info;

#[allow(dead_code)]
pub fn console_log_build_info() {
  let info = build_info();
  // #[cfg(debug_assertions)]
  console_log!("VXL {:?}", info);
}
