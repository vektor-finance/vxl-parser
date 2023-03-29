use nom::{
  bytes::complete::{tag_no_case, take_while_m_n},
  combinator::map,
  sequence::tuple,
};

use nom_tracable::tracable_parser;

// use bs58::decode;

use super::{Node, Result, Span, Token};

fn valid_address_char(c: char) -> bool {
  c.is_ascii_hexdigit()
}

#[tracable_parser]
pub fn address(i: Span) -> Result {
  map(
    tuple((tag_no_case("0x"), take_while_m_n(40, 40, valid_address_char))), // Ethereum EOA: ^(0x)?[0-9a-fA-F]{40}$
    |(first, rest): (Span, Span)| {
      let mut a = String::from(*first.fragment());
      a.push_str(rest.fragment());

      Node::new(Token::Address(a), &first)
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
            case("0xcac725bef4f114f728cbcfd744a731c2a463c3fc", address!("0xcac725bef4f114f728cbcfd744a731c2a463c3fc"))
    )]
  fn test_address_valid(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let (span, actual) = address(Span::new_extra(input, info))?;
    assert_eq!(span.fragment().len(), 0);
    assert_eq!(actual.token, expected);

    Ok(())
  }

  #[rstest(input, case(""), case("0x"), case("0X"), case("0xcac725bef4f114f7a463c3fc"))]
  fn test_address_invalid(input: &'static str, info: TracableInfo) -> Result {
    assert!(address(Span::new_extra(input, info)).is_err());
    Ok(())
  }
}
