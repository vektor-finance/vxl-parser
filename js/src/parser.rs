use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(input: &str) -> Result<JsValue, JsValue> {
  // TODO: Improve error response
  let result = core::parse(input).map_err(|err| JsValue::from(err.to_string()))?;
  to_value(&result).map_err(|err| err.into())
}
