use nom::{
  bytes::complete::tag_no_case,
  character::complete::anychar,
  combinator::{map, peek},
  error::ErrorKind,
  Err,
};
use nom_tracable::tracable_parser;

use crate::{Node, Result, Span, Token};

#[tracable_parser]
fn true_literal(i: Span) -> Result {
  map(tag_no_case("true"), |span: Span| Node::new(Token::Boolean(true), &span))(i)
}

#[tracable_parser]
fn false_literal(i: Span) -> Result {
  map(tag_no_case("false"), |span: Span| {
    Node::new(Token::Boolean(false), &span)
  })(i)
}

#[tracable_parser]
pub fn boolean(i: Span) -> Result {
  let (_, head): (_, char) = peek(anychar)(i)?;
  match head {
    't' | 'T' => true_literal(i),
    'f' | 'F' => false_literal(i),
    _ => Err(Err::Error((i, ErrorKind::Tag))),
  }
}

#[cfg(test)]
mod test {
  use crate::*;
  use crate::test::{info, Result};
  use nom_tracable::TracableInfo;

  use rstest::rstest;

  #[rstest(input, expected,
        case("true", boolean!(true)),
        case("false", boolean!(false))
    )]
  fn test_boolean(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let i = Span::new_extra(input, info);
    let (span, node) = boolean(i)?;

    assert!(span.fragment().is_empty());
    assert_eq!(node.token, expected);

    let parsed: bool = input.to_lowercase().parse()?;
    assert_eq!(node.token.as_boolean(), Some(parsed));

    Ok(())
  }
}
