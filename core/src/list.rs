use nom::{
  character::complete::{char, multispace0},
  combinator::{map, not, opt, recognize},
  multi::many0,
  sequence::{pair, preceded, terminated, tuple},
};

use nom_tracable::tracable_parser;

use crate::{expression, Node, Result, Span, Token};

#[tracable_parser]
fn list_end(i: Span) -> Result {
  map(tuple((opt(char(',')), multispace0, char(']'))), |(_, _, _)| {
    Node::default()
  })(i)
}

#[tracable_parser]
pub fn list(i: Span) -> Result {
  let (i, start) = recognize(pair(char('['), multispace0))(i)?;
  // short-circuit empty list
  if let Ok((i, _)) = list_end(i) {
    return Ok((i, Node::new(Token::List(Vec::new()), &start)));
  }

  map(
    terminated(
      tuple((
        opt(expression),
        many0(pair(not(list_end), preceded(pair(char(','), multispace0), expression))),
      )),
      list_end,
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

#[cfg(test)]
mod test {
  use crate::*;
  use crate::{
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
  fn test_list(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let span = Span::new_extra(input, info);
    let (span, node) = list(span)?;
    assert_eq!(span.fragment().len(), 0);

    let expected = expected.as_list().ok_or("expected was not a list")?;
    let items = node.token.as_list().ok_or("node.token was not a list")?;
    for (i, item) in items.iter().enumerate() {
      item.assert_same_token(&expected[i])
    }

    Ok(())
  }
}
