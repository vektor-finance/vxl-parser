use std::{rc::Rc, str::FromStr};

use nom::{
  bytes::complete::tag_no_case,
  character::complete::{alpha1, char, digit1},
  combinator::{map, not, opt},
  error::ErrorKind,
  sequence::{pair, preceded, terminated, tuple},
  Err,
};
use nom_tracable::tracable_parser;
use serde::{Serialize, Serializer};

use super::operation::sign;
use super::{Node, Operator, Result, Span, Token, TokenError, UnaryOp};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum N {
  Int(i64),
  Float(f64),
}

impl Eq for N {}

impl From<i64> for N {
  fn from(i: i64) -> Self {
    N::Int(i)
  }
}

impl From<f64> for N {
  fn from(f: f64) -> Self {
    N::Float(f)
  }
}

impl N {
  pub fn is_int(&self) -> bool {
    self.as_int().is_some()
  }

  pub fn is_float(&self) -> bool {
    self.as_float().is_some()
  }

  pub fn as_int(&self) -> Option<i64> {
    if let N::Int(i) = self {
      Some(*i)
    } else {
      None
    }
  }

  pub fn as_float(&self) -> Option<f64> {
    if let N::Float(f) = self {
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

    let f = s.parse::<f64>();
    if f.is_ok() {
      return Ok(N::Float(f?));
    }

    Err(f.err().unwrap().into())
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
      N::Float(ref v) => serializer.serialize_newtype_variant("number", 1, "float", &v.to_string()),
    }
  }
}

#[tracable_parser]
fn exponent(i: Span) -> Result<Span, i64> {
  let (i, (maybe_sign, num)) = preceded(tag_no_case("e"), pair(opt(sign), digit1))(i)?;

  let n: i64 = (*num.fragment())
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

// TODO: factor this out to multiple parsers
// TODO: The negative sign is technically a uniary operation as defined by the spec
// and so should be factored out into the expression syntax parser once that is in
#[tracable_parser]
pub(super) fn number(i: Span) -> Result {
  let start = i;
  let (i, maybe_sign) = opt(sign)(i)?;

  let (i, num) = map(
    terminated(
      tuple((digit1, opt(preceded(char('.'), digit1)))),
      // allows identifiers starting with numbers
      not(preceded(opt(tag_no_case("e")), alpha1)),
    ),
    |(dec, maybe_fract): (Span, Option<Span>)| {
      let mut buf = String::from(*dec.fragment());
      if let Some(fract) = maybe_fract {
        buf.push('.');
        buf.push_str(*fract.fragment());
      }

      let n: N = buf.parse().map_err(|_| Err::Failure((dec, ErrorKind::Float)))?;

      Ok(n)
    },
  )(i)?;
  let num = num?;

  let (i, maybe_exp) = opt(exponent)(i)?;
  let num = maybe_exp.map_or(num, |exp| match num {
    N::Int(i) => {
      let pow = 10i64.pow(exp.abs() as u32);
      if exp < 0 {
        N::Float((i as f64) * (1.0 / (pow as f64)))
      } else {
        N::Int(i * pow)
      }
    }
    N::Float(f) => {
      let pow = 10i64.pow(exp.abs() as u32) as f64;
      if exp < 0 {
        N::Float(f * (1.0 / pow))
      } else {
        N::Int((f * pow) as i64)
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
  use serde_test::{assert_ser_tokens, Token as SerdeToken};

  #[rstest(input, expected,
        case("1.23", number!(1.23)),
        case("47", number!(47)),
        case("17.3809", number!(17.3809)),
        case("17892037", number!(17892037)),
        case("-38", number!(-38)),
        case("-471.399", number!(-471.399)),
        case("1.7e8", number!(170000000)),
        case("-17E10", number!(-170000000000)),
        case("8.6e-6", number!(0.0000086)),
        case("1e-4", number!(0.0001)),
        case("-1e-4", number!(-0.0001))
    )]
  fn test_number(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let span = Span::new_extra(input, info);
    let (span, node) = number(span)?;
    assert_eq!(span.fragment().len(), 0);
    node.assert_same_token(&node!(expected));

    Ok(())
  }

  #[rstest(input, expected,
        case(N::Float(1.0), ("float", "1")),
        case(N::Float(1.00), ("float", "1")),
        case(N::Float(1.23), ("float", "1.23")),
        case(N::Int(47), ("int", "47")),
        case(N::Float(17.3809), ("float", "17.3809")),
        case(N::Int(17892037), ("int", "17892037")),
        case(N::Int(-38), ("int", "-38")),
        case(N::Float(-471.399), ("float", "-471.399")),
        case(N::Int(170000000), ("int", "170000000")),
        case(N::Int(-170000000000), ("int", "-170000000000")),
        case(N::Float(0.000008599999999999999), ("float", "0.000008599999999999999")),
        case(N::Float(0.333333333333333333), ("float", "0.333333333333333333")),
        case(N::Float(-0.0000123), ("float", "-0.0000123")),
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
