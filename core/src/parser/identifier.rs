use super::{Node, Result, Span, Token};

use nom::{bytes::complete::take_while1, combinator::map};

use nom_tracable::tracable_parser;

fn valid_ident_char_a(c: char) -> bool {
  c.is_alphanumeric() || matches!(c, '_')
}

#[tracable_parser]
pub(super) fn identifier(i: Span) -> Result {
  map(take_while1(valid_ident_char_a), |span: Span| {
    let s = String::from(*span.fragment());

    Node::new(Token::Identifier(s.to_lowercase()), &span)
  })(i)
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

  #[rstest(input, case(""))]
  fn test_identifier_invalid(input: &'static str, info: TracableInfo) -> Result {
    assert!(identifier(Span::new_extra(input, info)).is_err());
    Ok(())
  }
}
