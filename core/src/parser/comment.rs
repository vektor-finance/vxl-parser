use nom::{bytes::complete::is_not, character::complete::char, combinator::map, sequence::pair};

use nom_tracable::tracable_parser;

use super::{Node, Result, Span, Token};

#[tracable_parser]
pub fn line_comment(i: Span) -> Result {
  map(pair(char('#'), is_not("\n\r")), |(_, span): (char, Span)| {
    Node::new(Token::LineComment(String::from(*span.fragment())), &span)
  })(i)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::test::{info, Result};
  use nom_tracable::TracableInfo;

  use rstest::rstest;

  #[rstest(input, expected,
    case("# this is a comment", line_comment!(" this is a comment")),
    case("#this is a comment   ", line_comment!("this is a comment   ")),
    case("# this is a ✅ comment with 🚀 emoji ", line_comment!(" this is a ✅ comment with 🚀 emoji ")),
    case("#  FOO.BAR(1) ", line_comment!("  FOO.BAR(1) ")),
  )]
  fn test_line_comment(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let input = Span::new_extra(input, info);
    let (span, node) = line_comment(input)?;
    assert!(span.fragment().is_empty());

    assert_eq!(node.token, expected);

    Ok(())
  }
}
