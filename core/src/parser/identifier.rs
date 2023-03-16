use super::{Node, Result, Span, Token};

use nom::{
  branch::alt,
  bytes::complete::{take_while, take_while1, take_while_m_n},
  combinator::map,
  sequence::tuple,
};

use nom_tracable::tracable_parser;

fn valid_ident_start_char_a(c: char) -> bool {
  c.is_alphabetic() || matches!(c, '_')
}

fn valid_ident_char_a(c: char) -> bool {
  c.is_alphanumeric() || matches!(c, '_' | '-')
}

fn valid_ident_start_char_1(c: char) -> bool {
  c.is_ascii_digit()
}

fn valid_ident_char_1(c: char) -> bool {
  c.is_alphabetic()
}

#[tracable_parser]
pub(super) fn identifier(i: Span) -> Result {
  map(
    alt((
      // starts with alphabetic char
      tuple((
        take_while_m_n(1, 1, valid_ident_start_char_a),
        take_while(valid_ident_char_a),
      )),
      // starts with a number
      tuple((
        take_while_m_n(1, 1, valid_ident_start_char_1),
        take_while1(valid_ident_char_1),
      )),
    )),
    |(first, rest): (Span, Span)| {
      let mut m = String::from(*first.fragment());
      m.push_str(rest.fragment());

      Node::new(Token::Identifier(m.to_lowercase()), &first)
    },
  )(i)
}

#[cfg(test)]
mod test {
  use super::*;
  use nom_tracable::TracableInfo;
  use rstest::{fixture, rstest};

  pub(super) type Result = std::result::Result<(), Box<dyn std::error::Error>>;

  #[fixture]
  pub(super) fn info() -> TracableInfo {
    TracableInfo::default()
  }

  #[rstest(input, expected,
    case("test", ident!("test")),
    case("TEST_LOWERCASING", ident!("test_lowercasing")),
    case("test_with_underscores", ident!("test_with_underscores")),
    case("test-with-dashes", ident!("test-with-dashes")),
    case("test-14_with_numbers", ident!("test-14_with_numbers")),
    case("1test", ident!("1test")),
    case("a", ident!("a")),
    case("a_", ident!("a_")),
    case("1a", ident!("1a")),
    case("1foo", ident!("1foo")),
    case("1foo1", ident!("1foo1")),
    case("1foo_v1", ident!("1foo_v1")),
)]
  fn test_identfier(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let (span, actual) = identifier(Span::new_extra(input, info))?;
    assert_eq!(span.fragment().len(), 0);
    assert_eq!(actual.token, expected);

    Ok(())
  }

  #[rstest(input, case("1_"), case("11abc"), case("11111a"), case("11111a"))]
  fn test_identifier_invalid(input: &'static str, info: TracableInfo) -> Result {
    assert!(identifier(Span::new_extra(input, info)).is_err());
    Ok(())
  }
}
