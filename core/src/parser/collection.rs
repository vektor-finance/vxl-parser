use nom::{character::complete::anychar, combinator::peek, error::ErrorKind, Err};
use nom_tracable::tracable_parser;

use crate::{list, Result, Span};

#[tracable_parser]
pub fn collection(i: Span) -> Result {
  let (_, head): (_, char) = peek(anychar)(i)?;
  match head {
    '[' => list(i),
    _ => Err(Err::Error((i, ErrorKind::Char))),
  }
}
