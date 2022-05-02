use nom_locate::position;
use nom::sequence::preceded;
use nom::multi::many1;
use nom::character::complete::space0;
use nom::sequence::tuple;
use crate::parser::literal::string;

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

use super::{Node, Result, Span, Token};

#[tracable_parser]
fn line_comment(i: Span) -> Result {
  let (i, span) = position(i)?;
  map(preceded(tag_no_case("#"), string), |node| {
    Node::new(Token::LineComment(node.fragment()), &span)
  })(i)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::test::{info, Result};
  use nom_tracable::TracableInfo;

  use rstest::rstest;

  #[rstest(input, expected,
        case("# this is a comment", line_comment!("this is a comment")),
        case("#this is a comment", line_comment!("this is a comment")),
    )]
  fn test_line_comment(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let input = Span::new_extra(input, info);
    let (span, node) = line_comment(input)?;
    assert!(span.fragment().is_empty());

    assert_eq!(node.token, expected);

    Ok(())
  }
}
