use nom::{
  branch::alt,
  bytes::complete::{tag_no_case, take_while1},
  character::complete::{alpha1, char},
  combinator::{map, not, opt},
  error::ErrorKind,
  sequence::{pair, preceded, terminated, tuple},
  Err,
};
use nom_locate::position;
use nom_tracable::tracable_parser;
use rust_decimal::prelude::*;

use super::{n::N, Node, Operator, Result, Span, Token};

fn is_digit_or_underscore(c: char) -> bool {
  c.is_digit(10) || c == '_'
}

#[tracable_parser]
fn sign(i: Span) -> Result {
  let (i, start) = position(i)?;
  map(alt((char('-'), char('+'))), move |c: char| {
    let op = if c == '-' { Operator::Minus } else { Operator::Plus };
    Node::new(Token::Operator(op), &start)
  })(i)
}

#[tracable_parser]
fn exponent(i: Span) -> Result<Span, i64> {
  let (i, (maybe_sign, num)) = preceded(tag_no_case("e"), pair(opt(sign), take_while1(is_digit_or_underscore)))(i)?;

  let n: i64 = (*num.fragment().replace("_", ""))
    .parse()
    .map_err(|_| Err::Failure((i, ErrorKind::Digit)))?;

  if let Some(Node {
    token: Token::Operator(Operator::Minus),
    ..
  }) = maybe_sign
  {
    Ok((i, -n))
  } else {
    Ok((i, n))
  }
}

// TODO: refactor to multiple parsers
#[tracable_parser]
pub(super) fn number(i: Span) -> Result {
  let start = i;
  let (i, maybe_sign) = opt(sign)(i)?;

  let (i, num) = map(
    terminated(
      tuple((
        take_while1(is_digit_or_underscore),
        opt(preceded(char('.'), take_while1(is_digit_or_underscore))),
      )),
      // allows identifiers starting with numbers
      not(preceded(opt(tag_no_case("e")), alpha1)),
    ),
    |(dec, maybe_fract): (Span, Option<Span>)| {
      let mut buf = dec.fragment().replace("_", "");
      if let Some(fract) = maybe_fract {
        buf.push('.');
        buf.push_str(&fract.fragment().replace("_", ""));
      }

      let n: N = buf.parse().map_err(|_| Err::Failure((dec, ErrorKind::Float)))?;
      // FIXME: should be ternary
      let n = if maybe_sign.is_some() { n.negate() } else { n };

      Ok(n)
    },
  )(i)?;

  let num = num?;

  let (i, maybe_exp) = opt(exponent)(i)?;
  let num = maybe_exp.map_or(num, |exp| match num {
    N::Int(i) => {
      let pow = 10i64.pow(exp.unsigned_abs() as u32);
      if exp < 0 {
        let v = Decimal::from_i64(i).unwrap() * (Decimal::ONE / Decimal::from_i64(pow).unwrap());
        N::Decimal(v)
      } else {
        N::Int(i * pow)
      }
    }
    N::Decimal(d) => {
      let pow: Decimal = 10i64.pow(exp.unsigned_abs() as u32).into();
      if exp < 0 {
        N::Decimal(d * (Decimal::ONE / pow))
      } else {
        N::Int((d * pow).to_i64().unwrap())
      }
    }
  });

  let num = Node::new(Token::Number(num), &start);
  Ok((i, num))
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::test::{info, Result};

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
      case("0.333333333333333334", number!(0.333333333333333334))
    )]
  fn test_number(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let span = Span::new_extra(input, info);
    let (span, node) = number(span)?;
    assert_eq!(span.fragment().len(), 0);
    node.assert_same_token(&node!(expected));

    Ok(())
  }
}
