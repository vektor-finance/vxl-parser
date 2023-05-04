use nom_tracable::tracable_parser;

use super::{number, Result, Span};

#[tracable_parser]
pub(super) fn numeric(i: Span) -> Result {
  let (i, num) = number(i)?;
  Ok((i, num))
}
