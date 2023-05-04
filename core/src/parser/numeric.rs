use std::rc::Rc;

use nom::{
  combinator::{opt},
};
use nom_tracable::tracable_parser;

use super::{number, operation::sign, Node, Operator, Result, Span, Token, UnaryOp};

#[tracable_parser]
pub(super) fn numeric(i: Span) -> Result {
  let (i, maybe_sign) = opt(sign)(i)?;
  let (i, num) = number(i)?;

  // Wrap unary op if negative
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

// #[cfg(test)]
// mod test {
//   use super::*;
//   use crate::parser::test::{info, Result};
//   use std::convert::TryFrom;

//   use nom_tracable::TracableInfo;
//   use rstest::rstest;


//   #[rstest(input, expected,
//         case("1.23", number!(1.23)),
//         case("47", number!(47)),
//         case("17.3809", number!(17.3809)),
//         case("17892037", number!(17892037)),
//         case("1_0", number!(1_0)),
//         case("1_000_000_00", number!(1_000_000_00)),
//         case("1_000.0_100_001", number!(1_000.0_100_001)),
//         case("-38", unary_op!("-", number!(38))),
//         case("-471.399", unary_op!("-", number!(471.399))),
//         case("1.7e8", number!(170000000)),
//         case("-17E10", node!(unary_op!("-", number!(-170000000000)))),
//         case("8.6e-6", number!(0.0000086)),
//         case("1e-4", number!(1e-4)),
//         case("-1e-4", unary_op!("-", number!(-1e-4))),
//         case("-1_000e-4", unary_op!("-", number!(-1_000e-4))),
//         case("-1e0_1", unary_op!("-", number!(-10))),
//         case("0.333333333333333334", number!(0.333333333333333334))
//     )]
//   fn test_numeric(input: &'static str, expected: Token, info: TracableInfo) -> Result {
//     let span = Span::new_extra(input, info);
//     let (span, node) = numeric(span)?;
//     assert_eq!(span.fragment().len(), 0);
//     node.assert_same_token(&node!(expected));

//     Ok(())
//   }
// }
