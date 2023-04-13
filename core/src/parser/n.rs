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
