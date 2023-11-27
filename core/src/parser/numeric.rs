use nom::{branch::alt, character::complete::char, combinator::map, sequence::terminated};
use nom_tracable::tracable_parser;

use crate::{n, Node, Result, Span, Token};

#[tracable_parser]
pub fn number(i: Span) -> Result {
  map(n, |num| Node::new(Token::Number(num), &i))(i)
}

#[tracable_parser]
pub fn percentage(i: Span) -> Result {
  map(terminated(n, char('%')), |pct| Node::new(Token::Percentage(pct), &i))(i)
}

#[tracable_parser]
pub fn numeric(i: Span) -> Result {
  alt((percentage, number))(i)
}

#[cfg(test)]
mod test {
  use crate::*;
  use crate::test::{info, Result};

  use nom_tracable::TracableInfo;
  use rstest::rstest;

  #[rstest(input, expected,
      case("1.23", number!(1.23)),
      case("47", number!(47)),
      case("17.3809", number!(17.3809)),
      case("17892037", number!(17892037)),
      case("1_0", number!(1_0)),
      case("1_000_000_00", number!(1_000_000_00)),
      case("1_000.0_100_001", number!(1_000.0_100_001)),
      case("-38", number!(-38)),
      case("-471.399", number!(-471.399)),
      case("1.7e8", number!(170000000)),
      case("-17E10", number!(-170000000000)),
      case("8.6e-6", number!(0.0000086)),
      case("1e-4", number!(1e-4)),
      case("-1e-4", number!(-1e-4)),
      case("-1_000e-4", number!(-1_000e-4)),
      case("-1e0_1", number!(-10)),
      case("0.333333333333333334", number!(0.333333333333333334)),
      case("1.23%", percentage!(1.23)),
      case("47%", percentage!(47)),
      case("17.3809%", percentage!(17.3809)),
      case("17892037%", percentage!(17892037)),
      case("1_0%", percentage!(1_0)),
      case("1_000_000_00%", percentage!(1_000_000_00)),
      case("1_000.0_100_001%", percentage!(1_000.0_100_001)),
      case("-38%", percentage!(-38)),
      case("-471.399%", percentage!(-471.399)),
      case("1.7e8%", percentage!(170000000)),
      case("-17E10%", percentage!(-170000000000)),
      case("8.6e-6%", percentage!(0.0000086)),
      case("1e-4%", percentage!(1e-4)),
      case("-1e-4%", percentage!(-1e-4)),
      case("-1_000e-4%", percentage!(-1_000e-4)),
      case("-1e0_1%", percentage!(-10)),
      case("0.3333333333333333%", percentage!(0.333333333333333334)),
    )]
  fn test_percentage(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let span = Span::new_extra(input, info);
    let (span, node) = numeric(span)?;
    assert_eq!(span.fragment().len(), 0);
    node.assert_same_token(&node!(expected));

    Ok(())
  }
}
