use wasm_bindgen_test::*;

use vxl_wasm::parse;

#[wasm_bindgen_test]
pub fn parses() {
  let _ = parse("function.subfunction(10, false, \"hello\")");
}
