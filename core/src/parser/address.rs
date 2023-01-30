use nom::{
  branch::alt,
  bytes::complete::{tag_no_case, take_while},
  combinator::map,
  sequence::tuple,
};

use nom_tracable::tracable_parser;

// use bs58::decode;

use super::{Node, Result, Span, Token};

fn valid_dns_char(c: char) -> bool {
  c.is_alphanumeric()
}

fn valid_address_char(c: char) -> bool {
  c.is_ascii_hexdigit()
}

// #[allow(dead_code)]
// fn valid_base58(i: Span) -> bool {
//   match decode(i.fragment()).into_vec() {
//     Ok(_) => true,
//     _ => false,
//   }
// }

#[tracable_parser]
pub fn address(i: Span) -> Result {
  map(
    alt((
      // Ethereum
      tuple((tag_no_case("0x"), take_while(valid_address_char))), // Ethereum
      tuple((take_while(valid_dns_char), tag_no_case(".eth"))),   // ENS
    )),
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
            case("0xcac725bef4f114f728cbcfd744a731c2a463c3fc", address!("0xcac725bef4f114f728cbcfd744a731c2a463c3fc")),
            case("vektor.eth", address!("vektor.eth")),
    )]
  fn test_address(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let (span, actual) = address(Span::new_extra(input, info))?;
    assert_eq!(span.fragment().len(), 0);
    assert_eq!(actual.token, expected);

    Ok(())
  }
}
