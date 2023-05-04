use std::str::FromStr;

use rust_decimal::prelude::*;
use serde::{Serialize, Serializer};

use super::TokenError;

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

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::test::Result;

  use rstest::rstest;
  use rust_decimal_macros::dec;
  use serde_test::{assert_ser_tokens, Token as SerdeToken};

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
