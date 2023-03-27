use super::{expr_term, BinaryOp, Conditional, Node, Operator, Result, Span, Token, UnaryOp};
use std::rc::Rc;

use nom::{
  branch::alt,
  bytes::complete::{is_a, tag, tag_no_case},
  character::complete::{anychar, char, space0, space1, multispace1},
  combinator::{map, recognize},
  error::ErrorKind,
  sequence::{tuple, terminated},
  Err,
};
use nom_locate::position;
use nom_tracable::tracable_parser;

#[tracable_parser]
pub(super) fn sign(i: Span) -> Result {
  let (i, start) = position(i)?;
  map(alt((char('-'), char('+'))), move |c: char| {
    let op = if c == '-' { Operator::Minus } else { Operator::Plus };
    Node::new(Token::Operator(op), &start)
  })(i)
}

#[tracable_parser]
fn negation(i: Span) -> Result {
  let (i, start) = position(i)?;
  map(alt((tag("!"), terminated(tag_no_case("not"), multispace1))), move |_| {
    Node::new(Token::Operator(Operator::Not), &start)
  })(i)
}

#[tracable_parser]
fn unary_operator(i: Span) -> Result {
  alt((sign, negation))(i)
}

#[tracable_parser]
fn arithmetic_operator(i: Span) -> Result {
  let (i, start) = position(i)?;
  let (i, c) = anychar(i)?;

  let op = match c {
    '+' => Ok(Operator::Plus),
    '-' => Ok(Operator::Minus),
    '*' => Ok(Operator::Multiply),
    '/' => Ok(Operator::Divide),
    '%' => Ok(Operator::Modulus),
    '^' => Ok(Operator::Exponent),
    _ => Err(Err::Error((i, ErrorKind::Char))),
  }?;

  Ok((i, Node::new(Token::Operator(op), &start)))
}

#[tracable_parser]
fn logic_operator(i: Span) -> Result {
  map(
    alt((tag("&&"), tag_no_case("and"), tag("||"), tag_no_case("or"))),
    move |span: Span| {
      let op = if *span.fragment() == "&&" || span.fragment().to_lowercase() == "and" {
        Operator::And
      } else {
        Operator::Or
      };
      Node::new(Token::Operator(op), &span)
    },
  )(i)
}

#[tracable_parser]
fn comparison_operator(i: Span) -> Result {
  let (i, span) = is_a("=!><")(i)?;
  let op = match *span.fragment() {
    "==" => Ok(Operator::Equal),
    "!=" => Ok(Operator::NotEqual),
    "<" => Ok(Operator::Less),
    ">" => Ok(Operator::Greater),
    "<=" => Ok(Operator::LessEqual),
    ">=" => Ok(Operator::GreaterEqual),
    _ => Err(Err::Error((i, ErrorKind::IsA))),
  }?;

  Ok((i, Node::new(Token::Operator(op), &span)))
}

#[tracable_parser]
fn other_operator(i: Span) -> Result {
  let (i, span) = is_a("+-|>")(i)?;
  let op = match *span.fragment() {
    "++" => Ok(Operator::Concatenate),
    "--" => Ok(Operator::Subtract),
    "|>" => Ok(Operator::Pipe),
    _ => Err(Err::Error((i, ErrorKind::IsA))),
  }?;

  Ok((i, Node::new(Token::Operator(op), &span)))
}

#[tracable_parser]
fn membership_operator(i: Span) -> Result {
  map(
    alt((
      tag_no_case("in"),
      recognize(tuple((tag_no_case("not"), space1, tag_no_case("in")))),
    )),
    move |span: Span| {
      let op = if span.fragment().to_lowercase() == "in" {
        Operator::In
      } else {
        Operator::NotIn
      };
      Node::new(Token::Operator(op), &span)
    },
  )(i)
}

#[tracable_parser]
pub(super) fn binary_operator(i: Span) -> Result {
  alt((
    other_operator,
    membership_operator,
    arithmetic_operator,
    comparison_operator,
    logic_operator,
  ))(i)
}

pub(super) fn unary_operation(i: Span) -> Result {
  map(tuple((unary_operator, space0, expr_term)), move |(op, _, expr)| {
    let op = Rc::new(op);
    let unop = UnaryOp {
      operator: Rc::clone(&op),
      operand: Rc::new(expr),
    };

    Node::from_node(Token::UnaryOp(unop), &op)
  })(i)
}

#[tracable_parser]
pub(super) fn binary_operation(i: Span) -> Result {
  map(
    tuple((expr_term, space0, binary_operator, space0, expr_term)),
    move |(left, _, op, _, right)| {
      let left = Rc::new(left);
      let binop = BinaryOp {
        operator: Rc::new(op),
        left: Rc::clone(&left),
        right: Rc::new(right),
      };

      Node::from_node(Token::BinaryOp(binop), &left)
    },
  )(i)
}

#[tracable_parser]
fn ternary_operator(i: Span) -> Result {
  let qm = recognize(tuple((space0, char('?'), space0)));
  let colon = recognize(tuple((space0, char(':'), space0)));
  map(
    tuple((expr_term, qm, expr_term, colon, expr_term)),
    |(cond, _, left, _, right)| {
      let cond = Rc::new(cond);
      let c = Conditional {
        condition: Rc::clone(&cond),
        if_true: Rc::new(left),
        if_false: Some(Rc::new(right)),
      };
      Node::from_node(Token::Conditional(c), &cond)
    },
  )(i)
}

#[tracable_parser]
pub(super) fn operation(i: Span) -> Result {
  alt((unary_operation, binary_operation, ternary_operator))(i)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::{
    test::{info, Result},
    Function,
  };
  use std::convert::TryFrom;

  use nom_tracable::TracableInfo;
  use rstest::rstest;

  #[rstest(input, expected,
        case("!foo", node!(unary_op!("!", ident!("foo")))),
        case("not foo", node!(unary_op!("!", ident!("foo")))),
        case("-test", node!(unary_op!("-", ident!("test")))),
        case("!test_func(13)", node!(unary_op!("!", function!("test_func", none, number!(13))))),
        case("not test_func(13)", node!(unary_op!("!", function!("test_func", none, number!(13))))),
        case("- 14", node!(unary_op!("-", number!(14)))),
        case("!true", node!(unary_op!("!", boolean!(true)))),
        case("not true", node!(unary_op!("!", boolean!(true)))),
        case("![1, true, false]", node!(unary_op!("!", list!(number!(1), boolean!(true), boolean!(false))))),
        case("not   [1, true, false]", node!(unary_op!("!", list!(number!(1), boolean!(true), boolean!(false)))))
    )]
  fn test_unary_op(input: &'static str, expected: Node, info: TracableInfo) -> Result {
    let span = Span::new_extra(input, info);
    let (span, node) = unary_operation(span)?;
    assert!(span.fragment().is_empty());

    node.assert_same_token(&expected);

    Ok(())
  }

  #[rstest(input, expected,
        case("1 > 2", node!(binary_op!(number!(1), ">", number!(2)))),
        case("2 < false", node!(binary_op!(number!(2), "<", boolean!(false)))),
        case("foo == bar", node!(binary_op!(ident!("foo"), "==", ident!("bar")))),
        case("baz != \"hello\"", node!(binary_op!(ident!("baz"), "!=", string!("hello")))),
        case("14.3 % 5", node!(binary_op!(number!(14.3), "%", number!(5)))),
        case("test * 7", node!(binary_op!(ident!("test"), "*", number!(7)))),
        case("17 - 73", node!(binary_op!(number!(17), "-", number!(73)))),
        case("bar + 18", node!(binary_op!(ident!("bar"), "+", number!(18)))),
        case("1 ^ 10", node!(binary_op!(number!(1), "^", number!(10)))),
        case("foo ^ 10", node!(binary_op!(ident!("foo"), "^", number!(10)))),
        case("1 ^ bar", node!(binary_op!(number!(1), "^", ident!("bar")))),
        case("foo ^ bar", node!(binary_op!(ident!("foo"), "^", ident!("bar")))),
        case("foo && bar", node!(binary_op!(ident!("foo"), "&&", ident!("bar")))),
        case("foo and bar", node!(binary_op!(ident!("foo"), "&&", ident!("bar")))),
        case("true and false", node!(binary_op!(boolean!(true), "&&", boolean!(false)))),
        case("TRUE and FALSE", node!(binary_op!(boolean!(true), "&&", boolean!(false)))),
        case("1 and 0", node!(binary_op!(number!(1), "&&", number!(0)))),
        case("foo || bar", node!(binary_op!(ident!("foo"), "||", ident!("bar")))),
        case("foo or bar", node!(binary_op!(ident!("foo"), "||", ident!("bar")))),
        case("true or false", node!(binary_op!(boolean!(true), "||", boolean!(false)))),
        case("TRUE or FALSE", node!(binary_op!(boolean!(true), "||", boolean!(false)))),
        case("1 or 0", node!(binary_op!(number!(1), "||", number!(0)))),
        case("foo && (bar || bar)", node!(
            binary_op!(
                ident!("foo"),
                "&&",
                 binary_op!(
                     ident!("bar"),
                     "||",
                     ident!("bar"))))),
        case(
            "var.foo == var.bar",
            node!(binary_op!(
                binary_op!(
                    ident!("var"),
                    ".",
                    ident!("foo")
                ),
                "==",
                binary_op!(
                    ident!("var"),
                    ".",
                    ident!("bar")
                )
            ))
        ),
        case(
            r#"var.foo[3] + var.bar["test"]"#,
            node!(binary_op!(
                binary_op!(
                    binary_op!(
                        ident!("var"),
                        ".",
                        ident!("foo")
                    ),
                    "[",
                    number!(3)
                ),
                "+",
                binary_op!(
                    binary_op!(
                        ident!("var"),
                        ".",
                        ident!("bar")
                    ),
                    "[",
                    string!("test")
                )
            ))
        ),
          case("[1, 2, 3] ++ [1, 2, 3]", node!(binary_op!(list!(number!(1), number!(2), number!(3)), "++", list!(number!(1), number!(2), number!(3))))),
          case("[1, 2, 3] -- [1, 2, 3]", node!(binary_op!(list!(number!(1), number!(2), number!(3)), "--", list!(number!(1), number!(2), number!(3))))),
          case("add(1, 2) |> add(3)", node!(binary_op!(function!("add", none, number!(1), number!(2)), "|>", function!("add", none, number!(3))))),
          case("[1,2,3] |> sum()", node!(binary_op!(list!(number!(1), number!(2), number!(3)), "|>", function!("sum")))),
          case("1 in [1,2,3]", node!(binary_op!(number!(1), "in", list!(number!(1), number!(2), number!(3))))),
          case("-1.1 in [-1.1,2,3]", node!(binary_op!(number!(-1.1), "in", list!(number!(-1.1), number!(2), number!(3))))),
          case("1 in foo()", node!(binary_op!(number!(1), "in", function!("foo")))),
          case("1 not in [1,2,3]", node!(binary_op!(number!(1), "not in", list!(number!(1), number!(2), number!(3))))),
          case("1 not    in [1,2,3]", node!(binary_op!(number!(1), "not in", list!(number!(1), number!(2), number!(3))))),
          case("-1.1 not in [-1.1,2,3]", node!(binary_op!(number!(-1.1), "not in", list!(number!(-1.1), number!(2), number!(3))))),
          case("1 not in foo()", node!(binary_op!(number!(1), "not in", function!("foo")))),
          case(
            "(1 in foo()) or (2 not in bar)",
            node!(
              binary_op!(
                binary_op!(number!(1), "in", function!("foo")),
                "||",
                binary_op!(number!(2), "not in", ident!("bar"))
              )
            )
          ),
    )]
  fn test_binary_op(input: &'static str, expected: Node, info: TracableInfo) -> Result {
    let span = Span::new_extra(input, info);
    let (span, node) = binary_operation(span)?;
    assert!(
      span.fragment().is_empty(),
      "expected fragment to be empty; got {}",
      span.fragment()
    );

    println!("expected: {:#?}", expected);
    println!("node: {:#?}", node);

    node.assert_same_token(&expected);

    Ok(())
  }

  #[rstest(input, expected,
        case(r#""string" ? true : false"#, conditional!(string!("string"), boolean!(true), boolean!(false))),
        case(
            r#"func("input") ? [1] : [2]"#,
            conditional!(function!("func", none, string!("input")), list!(number!(1)), list!(number!(2)))
        ),
    )]
  fn test_ternary_op(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let input = Span::new_extra(input, info);
    let (span, node) = ternary_operator(input)?;
    assert!(span.fragment().is_empty());

    let cond = node.token.as_conditional().ok_or("node.token was not a conditional")?;
    let expected = expected.as_conditional().ok_or("expected was not a conditional")?;

    cond.condition.assert_same_token(&expected.condition);
    cond.if_true.assert_same_token(&expected.if_true);
    // FIXME: cond.if_false.assert_same_token(&expected.if_false);

    Ok(())
  }
}
