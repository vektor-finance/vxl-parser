use nom::{
  character::complete::{anychar, char, multispace0},
  combinator::{map, not, opt, peek, recognize},
  error::ErrorKind,
  multi::many0,
  sequence::{pair, preceded, terminated, tuple},
  Err,
};
use nom_tracable::tracable_parser;

use super::{expression, Node, Result, Span, Token};

#[tracable_parser]
fn array_end(i: Span) -> Result {
  map(tuple((opt(char(',')), multispace0, char(']'))), |(_, _, _)| {
    Node::default()
  })(i)
}

// NOTE: technically a "tuple" in the spec, but that name is reserved here
#[tracable_parser]
fn array(i: Span) -> Result {
  let (i, start) = recognize(pair(char('['), multispace0))(i)?;
  // short-circuit empty array
  if let Ok((i, _)) = array_end(i) {
    return Ok((i, Node::new(Token::List(Vec::new()), &start)));
  }

  map(
    terminated(
      tuple((
        opt(expression),
        many0(pair(not(array_end), preceded(pair(char(','), multispace0), expression))),
      )),
      array_end,
    ),
    move |(first, tail): (Option<Node>, Vec<((), Node)>)| {
      let items = first.map_or(Vec::new(), |head| {
        let mut v = vec![head];
        let mut tail = tail.into_iter().map(|(_, node)| node).collect();
        v.append(&mut tail);
        v
      });
      Node::new(Token::List(items), &start)
    },
  )(i)
}

#[tracable_parser]
pub(super) fn collection(i: Span) -> Result {
  let (_, head): (_, char) = peek(anychar)(i)?;
  match head {
    '[' => array(i),
    _ => Err(Err::Error((i, ErrorKind::Char))),
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::{
    test::{info, Result},
    BinaryOp, Operator,
  };
  use std::convert::TryFrom;

  use nom_tracable::TracableInfo;
  use rstest::rstest;

  #[rstest(input, expected,
        case("[true, false]", list![boolean!(true), boolean!(false)]),
        case("[]", list![]),
        case(
            r#"[
            1,
            2,
            3,
           ]"#,
            list![number!(1), number!(2), number!(3)],
        ),
        case(
            r#"[1, ["nested list"]]"#,
            list![number!(1), list![string!("nested list")]]),
        case(
            "[true, [false, [[1]]]]",
            list!(boolean!(true),
                list!(boolean!(false),
                    list!(
                        list!(number!(1))
                    )
                )
            )
        ),
        case(
            r#"[
                "test string",
                "another string",
                false,
                17.38
               ]"#,
            list![
                string!("test string"),
                string!("another string"),
                boolean!(false),
                number!(17.38)
            ],
        ),
        case(
            "[false, foo == bar]",
            list![boolean!(false), binary_op!(ident!("foo"), "==", ident!("bar"))],
        )
    )]
  fn test_tuple(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let span = Span::new_extra(input, info);
    let (span, node) = array(span)?;
    assert_eq!(span.fragment().len(), 0);

    let expected = expected.as_list().ok_or("expected was not a list")?;
    let items = node.token.as_list().ok_or("node.token was not a list")?;
    for (i, item) in items.iter().enumerate() {
      item.assert_same_token(&expected[i])
    }

    Ok(())
  }
}
