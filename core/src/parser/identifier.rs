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
