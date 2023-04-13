use std::rc::Rc;

use nom::{
  bytes::complete::{tag_no_case, take_while1},
  character::complete::{alpha1, char},
  combinator::{map, not, opt},
  error::ErrorKind,
  sequence::{pair, preceded, terminated, tuple},
  Err,
};
use nom_tracable::tracable_parser;
use rust_decimal::prelude::*;

use super::{n::N, operation::sign, Node, Operator, Result, Span, Token, UnaryOp};

fn is_digit_or_underscore(c: char) -> bool {
  c.is_digit(10) || c == '_'
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

  if let Some(
    op @ Node {
      token: Token::Operator(Operator::Minus),
      ..
    },
  ) = maybe_sign
  {
    let op = Rc::new(op);
    let unary = UnaryOp {
      operator: Rc::clone(&op),
      operand: Rc::new(num),
    };

    Ok((i, Node::from_node(Token::UnaryOp(unary), &op)))
  } else {
    Ok((i, num))
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::test::{info, Result};
  use std::convert::TryFrom;

  use nom_tracable::TracableInfo;
  use rstest::rstest;
  use rust_decimal_macros::dec;
  use serde_test::{assert_ser_tokens, Token as SerdeToken};

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

  #[rstest(input, expected,
        case(N::Int(47), ("int", "47")),
        case(N::Int(17892037), ("int", "17892037")),
        case(N::Int(-38), ("int", "-38")),
        case(N::Int(170000000), ("int", "170000000")),
        case(N::Int(-170000000000), ("int", "-170000000000")),
        case(N::Int(1_0), ("int", "10")),
        case(N::Int(-1_0), ("int", "-10")),
        case(N::Int(-1_700_000_000_00), ("int", "-170000000000")),
        case(N::Decimal(dec!(1)), ("decimal", "1")),
        case(N::Decimal(dec!(1.0)), ("decimal", "1.0")),
        case(N::Decimal(dec!(1.00)), ("decimal", "1.00")),
        case(N::Decimal(dec!(1.23)), ("decimal", "1.23")),
        case(N::Decimal(dec!(17.3809)), ("decimal", "17.3809")),
        case(N::Decimal(dec!(-471.399)), ("decimal", "-471.399")),
        case(N::Decimal(dec!(0.000008599999999999999)), ("decimal", "0.000008599999999999999")),
        case(N::Decimal(dec!(0.3333333333333333333333333333)), ("decimal", "0.3333333333333333333333333333")),
        case(N::Decimal(dec!(-0.0000123)), ("decimal", "-0.0000123")),
        case(N::Decimal(dec!(-0.0_000_123)), ("decimal", "-0.0000123")),
        case(N::Decimal(dec!(-1_000_0.0_000_123)), ("decimal", "-10000.0000123")),
  )]
  fn test_serialize(input: N, expected: (&'static str, &'static str)) -> Result {
    let (t, v) = expected;
    assert_ser_tokens(
      &input,
      &[
        SerdeToken::NewtypeVariant {
          name: "number",
          variant: t,
        },
        SerdeToken::Str(v),
      ],
    );

    Ok(())
  }
}
