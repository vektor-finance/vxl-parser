use nom::{
  branch::alt,
  bytes::complete::{escaped, is_not, tag},
  character::complete::{anychar, char, one_of},
  combinator::{map, peek},
  error::ErrorKind,
  sequence::delimited,
  Err,
};
use nom_tracable::tracable_parser;

use crate::{Node, Result, Span, Token};

#[tracable_parser]
pub fn single_line_string(i: Span) -> Result {
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
pub fn string(i: Span) -> Result {
  let (_, head): (_, char) = peek(anychar)(i)?;
  match head {
    '"' => single_line_string(i),
    _ => Err(Err::Error((i, ErrorKind::Char))),
  }
}

#[cfg(test)]
mod test {
  use crate::*;
  use crate::test::{info, Result};
  use nom_tracable::TracableInfo;

  use rstest::rstest;

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
