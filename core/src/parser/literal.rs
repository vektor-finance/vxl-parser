use nom::{
  branch::alt,
  bytes::complete::{escaped, is_not, tag, tag_no_case},
  character::complete::{anychar, char, one_of},
  combinator::{map, peek},
  error::ErrorKind,
  sequence::delimited,
  Err,
};
use nom_tracable::tracable_parser;

use super::{number, Node, Result, Span, Token};

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
fn none_literal(i: Span) -> Result {
  map(tag_no_case("none"), |span: Span| Node::new(Token::None, &span))(i)
}

#[tracable_parser]
fn boolean(i: Span) -> Result {
  let (_, head): (_, char) = peek(anychar)(i)?;
  match head {
    't' | 'T' => true_literal(i),
    'f' | 'F' => false_literal(i),
    _ => Err(Err::Error((i, ErrorKind::Tag))),
  }
}

#[tracable_parser]
fn none(i: Span) -> Result {
  let (_, head): (_, char) = peek(anychar)(i)?;
  match head {
    'n' => none_literal(i),
    _ => Err(Err::Error((i, ErrorKind::Tag))),
  }
}

#[tracable_parser]
pub(super) fn single_line_string(i: Span) -> Result {
  map(
    delimited(
      char('"'),
      // empty string is allowed
      alt((escaped(is_not("\\\"\n"), '\\', one_of(r#"rnt"\"#)), tag(""))),
      char('"'),
    ),
    |span: Span| Node::new(Token::String(String::from(*span.fragment())), &span),
  )(i)
}

#[tracable_parser]
pub(super) fn string(i: Span) -> Result {
  let (_, head): (_, char) = peek(anychar)(i)?;
  match head {
    '"' => single_line_string(i),
    _ => Err(Err::Error((i, ErrorKind::Char))),
  }
}

#[tracable_parser]
pub(super) fn literal(i: Span) -> Result {
  let (_, head): (_, char) = peek(anychar)(i)?;
  match head {
    't' | 'T' | 'f' | 'F' => boolean(i),
    'n' => none(i),
    '"' => string(i),
    '-' | '0'..='9' => number(i),
    _ => Err(Err::Error((i, ErrorKind::Char))),
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::test::{info, Result};
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

  #[rstest(input, expected,
        case("none", none!()),
    )]
  fn test_none(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let input = Span::new_extra(input, info);
    let (span, node) = none(input)?;
    assert!(span.fragment().is_empty());

    assert_eq!(node.token, expected);

    Ok(())
  }

  #[rstest(input, expected,
        case(r#""""#, string!("")),
        case(r#""  ""#, string!("  ")),
        case(r#""hello there""#, string!("hello there")),
        case(r#""with numbers 1 2 3""#, string!("with numbers 1 2 3")),
        case(r#""escaped \"""#, string!("escaped \\\"")),
        case(r#""escaped \n""#, string!("escaped \\n"))
    )]
  fn test_single_line_string(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let input = Span::new_extra(input, info);
    let (span, node) = single_line_string(input)?;
    assert!(span.fragment().is_empty());

    assert_eq!(node.token, expected);

    Ok(())
  }

  #[rstest(input, expected,
        case(r#""""#, node!(string!(""))),
        case(r#""  ""#, node!(string!("  "))),
        case(r#""single line string""#, node!(string!("single line string")))
    )]
  fn test_strings(input: &'static str, expected: Node, info: TracableInfo) -> Result {
    let input = Span::new_extra(input, info);
    let (span, node) = string(input)?;
    assert!(span.fragment().is_empty());

    node.assert_same_token(&expected);

    Ok(())
  }
}
