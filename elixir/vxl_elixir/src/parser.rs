use rustler::{Atom, Error, NifResult as Result};
use serde_json::to_string;

mod atoms {
  rustler::atoms! {
      ok,
      error,

      // errors
      parse_error,
      json_error
  }
}

// TODO: Replace JSON serialisation to proper NIF terms
#[rustler::nif(schedule = "DirtyCpu")]
fn parse(input: &str) -> Result<(Atom, String)> {
  let result = core::parse(input).map_err(|_| Error::Term(Box::new(atoms::parse_error())))?;
  let json = to_string(&result).map_err(|_| Error::Term(Box::new(atoms::json_error())))?;
  Ok((atoms::ok(), json))
}
