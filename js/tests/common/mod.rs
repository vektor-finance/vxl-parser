use wasm_bindgen_test::*;

use vxl_wasm::parse;

#[wasm_bindgen_test]
pub fn parses() {
  let _ = parse("COMMAND(10, ARG1, ARG2, ARG3, FALSE)");
}
