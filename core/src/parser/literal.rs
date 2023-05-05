use nom::{character::complete::anychar, combinator::peek, error::ErrorKind, Err};
use nom_tracable::tracable_parser;

use super::{boolean, numeric, string, Result, Span};

#[tracable_parser]
pub(super) fn literal(i: Span) -> Result {
  let (_, head): (_, char) = peek(anychar)(i)?;
  match head {
    't' | 'T' | 'f' | 'F' => boolean(i),
    '"' => string(i),
    '-' | '0'..='9' => numeric(i),
    _ => Err(Err::Error((i, ErrorKind::Char))),
  }
}
