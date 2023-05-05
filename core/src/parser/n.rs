use std::str::FromStr;

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
use serde::{Serialize, Serializer};

use super::{operation::sign, Node, Operator, Result, Span, Token, TokenError};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum N {
  Int(i64),
  Decimal(Decimal),
}

impl Eq for N {}

impl From<i64> for N {
  fn from(i: i64) -> Self {
    N::Int(i)
  }
}

impl From<f64> for N {
  fn from(f: f64) -> Self {
    N::Decimal(Decimal::from_f64(f).unwrap())
  }
}

impl From<Decimal> for N {
  fn from(d: Decimal) -> Self {
    N::Decimal(d)
  }
}

impl N {
  pub fn is_int(&self) -> bool {
    self.as_int().is_some()
  }

  pub fn is_decimal(&self) -> bool {
    self.as_decimal().is_some()
  }

  pub fn as_int(&self) -> Option<i64> {
    if let N::Int(i) = self {
      Some(*i)
    } else {
      None
    }
  }

  pub fn as_decimal(&self) -> Option<Decimal> {
    if let N::Decimal(f) = self {
      Some(*f)
    } else {
      None
    }
  }

  pub fn negate(&self) -> Self {
    match self {
      N::Int(i) => N::Int(i * -1),
      N::Decimal(d) => N::Decimal(d * Decimal::NEGATIVE_ONE),
    }
  }
}

impl FromStr for N {
  type Err = TokenError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    let i = s.parse::<i64>();
    if i.is_ok() {
      return Ok(N::Int(i?));
    }

    let d = Decimal::from_str(s);
    if d.is_ok() {
      return Ok(N::Decimal(d?));
    }

    Err(d.err().unwrap().into())
  }
}

/// Convert numbers to strings to keep precision in JSON
impl Serialize for N {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match *self {
      N::Int(ref v) => serializer.serialize_newtype_variant("number", 0, "int", &v.to_string()),
      N::Decimal(ref v) => serializer.serialize_newtype_variant("number", 1, "decimal", &v.to_string()),
    }
  }
}

fn is_digit_or_underscore(c: char) -> bool {
  c.is_digit(10) || c == '_'
}

#[tracable_parser]
fn exponent(i: Span) -> Result<Span, i64> {
  let (i, (maybe_sign, num)) = preceded(tag_no_case("e"), pair(opt(sign), take_while1(is_digit_or_underscore)))(i)?;

  let n: i64 = (*num.fragment().replace('_', ""))
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

#[tracable_parser]
pub(super) fn n(i: Span) -> Result<Span, N> {
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
      let mut buf = dec.fragment().replace('_', "");
      if let Some(fract) = maybe_fract {
        buf.push('.');
        buf.push_str(&fract.fragment().replace('_', ""));
      }

      let n: N = buf.parse().map_err(|_| Err::Failure((dec, ErrorKind::Float)))?;
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

  Ok((i, num))
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::test::{info, Result};

  use nom_tracable::TracableInfo;
  use rstest::rstest;
  use rust_decimal_macros::dec;
  use serde_test::{assert_ser_tokens, Token as SerdeToken};

  #[rstest(input, expected,
      case("1.23", N::Decimal(dec!(1.23))),
      case("47", N::Int(47)),
      case("17.3809", N::Decimal(dec!(17.3809))),
      case("17892037", N::Int(17892037)),
      case("1_0", N::Int(1_0)),
      case("1_000_000_00", N::Int(1_000_000_00)),
      case("1_000.0_100_001", N::Decimal(dec!(1_000.0_100_001))),
      case("-38", N::Int(-38)),
      case("-471.399", N::Decimal(dec!(-471.399))),
      case("1.7e8", N::Int(170000000)),
      case("-17E10", N::Int(-170000000000)),
      case("8.6e-6", N::Decimal(dec!(0.0000086))),
      case("1e-4", N::Decimal(dec!(1e-4))),
      case("-1e-4", N::Decimal(dec!(-1e-4))),
      case("-1_000e-4", N::Decimal(dec!(-1_000e-4))),
      case("-1e0_1", N::Int(-10)),
      case("0.333333333333333334", N::Decimal(dec!(0.333333333333333334))),
    )]
  fn test_n(input: &'static str, expected: N, info: TracableInfo) -> Result {
    let span = Span::new_extra(input, info);
    let (span, n) = n(span)?;
    assert_eq!(span.fragment().len(), 0);
    assert_eq!(n, expected);

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
