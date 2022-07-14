use std::{error::Error, rc::Rc};

#[macro_use]
mod tokens;
mod address;
mod collection;
mod comment;
mod literal;
mod node;
mod number;
mod operation;

use crate::tracer::get_tracer;
use address::address;
use collection::collection;
use comment::line_comment;
use literal::literal;
use number::number;
use operation::operation;

pub use node::{Node, Tree};
pub use number::N;
pub use tokens::*;

use nom::{
  branch::alt,
  bytes::complete::{tag, tag_no_case, take, take_while, take_while1, take_while_m_n},
  character::complete::{char, line_ending, multispace0, newline, space0, space1},
  combinator::{all_consuming, complete, eof, map, opt, recognize},
  error::ErrorKind,
  multi::{fold_many0, fold_many1, many0, many1, separated_list0},
  sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
  Err,
};
use nom_locate::{position, LocatedSpan};
use nom_tracable::{tracable_parser, TracableInfo};

pub type Span<'a> = LocatedSpan<&'a str, TracableInfo>;

pub type SResult<O, E> = std::result::Result<O, E>;
pub type Result<'a, I = Span<'a>, O = Node, E = (I, ErrorKind)> = SResult<(I, O), nom::Err<E>>;
pub type OResult<'a> = SResult<Tree, Box<dyn Error + 'a>>;

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
fn identifier(i: Span) -> Result {
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
      m.push_str(*rest.fragment());

      Node::new(Token::Identifier(m.to_lowercase()), &first)
    },
  )(i)
}

#[tracable_parser]
fn elipsis(i: Span) -> Result {
  map(tag("..."), |span| Node::new(Token::Operator(Operator::Elipsis), &span))(i)
}

#[tracable_parser]
fn function_arg(i: Span) -> Result {
  alt((option, operation, expr_term))(i)
}

#[tracable_parser]
fn function(i: Span) -> Result {
  map(
    tuple((
      identifier,
      opt(preceded(char('.'), identifier)),
      delimited(
        char('('),
        opt(tuple((
          preceded(multispace0, function_arg),
          many0(preceded(
            pair(
              preceded(multispace0, char(',')),
              preceded(multispace0, opt(line_ending)),
            ),
            function_arg,
          )),
          opt(elipsis),
        ))),
        preceded(multispace0, char(')')),
      ),
    )),
    |(name, subfunction, args)| {
      let name = Rc::new(name);
      let subfunction = subfunction.map(Rc::new);

      let mut f = Function {
        name: Rc::clone(&name),
        subfunction,
        args: Vec::new(),
      };

      if let Some((first, mut tail, maybe_elips)) = args {
        f.args.push(first);
        f.args.append(&mut tail);

        if let Some(elips) = maybe_elips {
          let last = Rc::new(f.args.pop().unwrap());
          let op = UnaryOp {
            operator: Rc::new(elips),
            operand: Rc::clone(&last),
          };
          let last = Node::from_node(Token::UnaryOp(op), &last);
          f.args.push(last);
        }
      }

      Node::from_node(Token::Function(f), &name)
    },
  )(i)
}

#[tracable_parser]
#[allow(dead_code)]
fn variable(i: Span) -> Result {
  identifier(i)
  //   map(identifier, |node| {
  //       Node::from_node(
  //           Token::Variable(node.token.as_identifier().unwrap().to_string()),
  //           &node,
  //       )
  //   })(i)
}

#[tracable_parser]
fn attr_access(i: Span) -> Result {
  let (i, span) = position(i)?;
  map(preceded(char('.'), identifier), move |node| {
    let unop = UnaryOp {
      operator: Rc::new(Node::new(Token::Operator(Operator::AttrAccess), &span)),
      operand: Rc::new(node),
    };
    Node::new(Token::UnaryOp(unop), &span)
  })(i)
}

#[tracable_parser]
fn index_access(i: Span) -> Result {
  let (i, span) = position(i)?;
  map(delimited(char('['), expression, char(']')), move |node| {
    let unop = UnaryOp {
      operator: Rc::new(Node::new(Token::Operator(Operator::IndexAccess), &span)),
      operand: Rc::new(node),
    };
    Node::new(Token::UnaryOp(unop), &span)
  })(i)
}

#[tracable_parser]
fn attr_splat(i: Span) -> Result {
  let (i, span) = position(i)?;
  let node = Node::new(Token::Operator(Operator::AttrSplat), &span);
  preceded(
    tag(".*"),
    fold_many0(attr_access, node, |node, attr| {
      let attr_op = attr.token.as_unary_op().cloned().unwrap();
      let node = Rc::new(node);
      let op = BinaryOp {
        left: Rc::clone(&node),
        operator: attr_op.operator,
        right: attr_op.operand,
      };

      Node::from_node(Token::BinaryOp(op), &node)
    }),
  )(i)
}

#[tracable_parser]
fn full_splat(i: Span) -> Result {
  let (i, span) = position(i)?;
  let node = Node::new(Token::Operator(Operator::FullSplat), &span);
  preceded(
    tag("[*]"),
    fold_many0(alt((attr_access, index_access)), node, |node, access| {
      let access_op = access.token.as_unary_op().cloned().unwrap();
      let node = Rc::new(node);
      let op = BinaryOp {
        left: Rc::clone(&node),
        operator: access_op.operator,
        right: access_op.operand,
      };

      Node::from_node(Token::BinaryOp(op), &node)
    }),
  )(i)
}

#[tracable_parser]
fn expr_postfix(i: Span) -> Result {
  let (_, head): (_, Span) = take(2usize)(i)?;
  let mut head = head.fragment().chars();
  match head.next() {
    Some('.') => {
      if let Some('*') = head.next() {
        attr_splat(i)
      } else {
        attr_access(i)
      }
    }
    Some('[') => {
      if let Some('*') = head.next() {
        full_splat(i)
      } else {
        index_access(i)
      }
    }
    _ => Err(Err::Error((i, ErrorKind::Alt))),
  }
}

#[tracable_parser]
fn sub_expression(i: Span) -> Result {
  delimited(
    tuple((char('('), multispace0)),
    expression,
    tuple((multispace0, char(')'))),
  )(i)
}

#[tracable_parser]
fn expr_term(i: Span) -> Result {
  let (rest, term) = alt((
    address,
    literal,
    for_loop,
    collection,
    if_statement,
    function,
    identifier,
    sub_expression,
  ))(i)?;
  fold_many0(expr_postfix, term, |node, postfix| {
    let node = Rc::new(node);
    let postfix = Rc::new(postfix);

    let op = match &postfix.token {
      Token::BinaryOp(op) => BinaryOp {
        left: Rc::clone(&node),
        operator: op.operator.clone(),
        right: Rc::clone(&postfix),
      },
      Token::UnaryOp(op) => BinaryOp {
        left: Rc::clone(&node),
        operator: op.operator.clone(),
        right: op.operand.clone(),
      },
      _ => panic!("wrong type"),
    };
    Node::from_node(Token::BinaryOp(op), &node)
  })(rest)
}

#[tracable_parser]
fn if_statement(i: Span) -> Result {
  map(
    tuple((
      tag_no_case("if"),
      delimited(
        char('('),
        tuple((
          preceded(multispace0, expression),
          preceded(pair(char(','), multispace0), expression),
          opt(preceded(pair(char(','), multispace0), expression)),
        )),
        preceded(multispace0, char(')')),
      ),
    )),
    |(_, (cond, left, right))| {
      let cond = Rc::new(cond);
      let right = right.map(Rc::new);
      let c = Conditional {
        condition: Rc::clone(&cond),
        if_true: Rc::new(left),
        if_false: right,
      };
      Node::from_node(Token::Conditional(c), &cond)
    },
  )(i)
}

#[tracable_parser]
fn for_intro(i: Span) -> Result<Span, (Vec<Node>, Node)> {
  let kw_for = recognize(tuple((space0, tag_no_case("for"), space1)));
  let kw_in = recognize(tuple((space1, tag_no_case("in"), space1)));
  let colon = recognize(tuple((space1, char(':'), space1)));

  map(
    tuple((
      kw_for,
      separated_list0(tuple((char(','), space0)), expression),
      kw_in,
      expression,
      colon,
    )),
    |(_, binds, _, expr, _)| (binds, expr),
  )(i)
}

#[tracable_parser]
fn for_cond(i: Span) -> Result {
  let kw_in = recognize(tuple((space1, tag_no_case("if"), space1)));
  preceded(kw_in, expression)(i)
}

#[tracable_parser]
fn tuple_for_loop(i: Span) -> Result {
  let (_, span): (_, Span) = position(i)?;
  map(
    delimited(char('['), tuple((for_intro, expression, opt(for_cond))), char(']')),
    move |((binds, expr), body, cond)| {
      let for_loop = ForLoop::Tuple {
        binds,
        expr: Rc::new(expr),
        body: Rc::new(body),
        cond: cond.map(Rc::new),
      };

      Node::new(Token::ForLoop(for_loop), &span)
    },
  )(i)
}

#[tracable_parser]
fn object_for_loop(i: Span) -> Result {
  let (_, span): (_, Span) = position(i)?;
  let arrow = recognize(tuple((space0, tag("=>"), space0)));
  map(
    delimited(
      char('{'),
      tuple((for_intro, expression, arrow, expression, opt(tag("...")), opt(for_cond))),
      char('}'),
    ),
    move |((binds, expr), key, _, val, elipsis, cond)| {
      let for_loop = ForLoop::Object {
        binds,
        expr: Rc::new(expr),
        body: Rc::new((key, val)),
        grouping: elipsis.is_some(),
        cond: cond.map(Rc::new),
      };

      Node::new(Token::ForLoop(for_loop), &span)
    },
  )(i)
}

#[tracable_parser]
fn for_loop(i: Span) -> Result {
  alt((tuple_for_loop, object_for_loop))(i)
}

#[tracable_parser]
fn expression(i: Span) -> Result {
  alt((operation, expr_term))(i)
}

#[tracable_parser]
#[allow(dead_code)]
fn attribute(i: Span) -> Result {
  map(
    terminated(
      separated_pair(identifier, tuple((space0, char('='), space0)), expr_term),
      opt(newline),
    ),
    |(ident, value): (Node, Node)| {
      let ident = Rc::new(ident);
      let attr = Attribute {
        ident: Rc::clone(&ident),
        expr: Rc::new(value),
      };
      Node::from_node(Token::Attribute(attr), &ident)
    },
  )(i)
}

#[tracable_parser]
fn option(i: Span) -> Result {
  map(
    separated_pair(identifier, tuple((space0, char('='), space0)), expr_term),
    |(ident, value): (Node, Node)| {
      let ident = Rc::new(ident);
      let opt = Opt {
        key: Rc::clone(&ident),
        value: Rc::new(value),
      };
      Node::from_node(Token::Option(opt), &ident)
    },
  )(i)
}

fn file(i: Span) -> OResult {
  let (_, tree) = all_consuming(complete(fold_many1(
    delimited(
      multispace0,
      alt((expression, line_comment)),
      tuple((
        space0,
        alt((
          tag(";"),
          recognize(tuple((opt(tag(";")), space0, line_comment, opt(line_ending)))),
          eof,
          recognize(many1(line_ending)),
        )),
      )),
    ),
    Tree::new(),
    |mut tree, node| {
      match node.token {
        // Filter out top-level line comments
        // TODO: Better strip at parse time
        // OR make all comments top-level and then strip
        Token::LineComment(_) => {}
        _ => tree.push(node),
      }
      tree
    },
  )))(i)?;

  Ok(tree)
}

pub fn parse(i: &str) -> OResult {
  let span = Span::new_extra(i, get_tracer());
  file(span)
}

#[cfg(test)]
mod test {
  use super::*;
  use rstest::{fixture, rstest};
  use std::convert::TryFrom;

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
            case("1inch", ident!("1inch")),
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

  #[rstest(input, expected,
            case("test_1 = true", attr!("test_1", boolean!(true))),
            case("TEST_1 = true", attr!("test_1", boolean!(true))),
            case("TEST_1 = none", attr!("test_1", none!())),
            case(
                r#"test-2 = "a test string""#,
                attr!("test-2", string!("a test string")),
            ),
            case(
                "another_test = -193.5\n",
                attr!("another_test", number!(-193.5)),
            ),
            case(
                "testing = var.foo\n",
                attr!("testing", binary_op!(ident!("var"), ".", ident!("foo")))
            ),
            case(
                "testing = foo.*.bar.baz",
                attr!(
                    "testing",
                    binary_op!(
                        ident!("foo"),
                        ".",
                        binary_op!(
                            binary_op!(
                                Token::Operator(Operator::AttrSplat),
                                ".",
                                ident!("bar")
                            ),
                            ".",
                            ident!("baz")
                        )
                    )
                )
            ),
            case (
                "testing = foo[*].bar.baz",
                attr!(
                    "testing",
                    binary_op!(
                        ident!("foo"),
                        ".",
                        binary_op!(
                            binary_op!(
                                Token::Operator(Operator::FullSplat),
                                ".",
                                ident!("bar")
                            ),
                            ".",
                            ident!("baz")
                        )
                    )
                )
            )
    )]
  fn test_attribute(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let span = Span::new_extra(input, info);
    let (span, node) = attribute(span)?;
    assert_eq!(span.fragment().len(), 0);

    let attr = node.token.as_attribute().ok_or("node.token was not an attribute")?;
    let expected = expected.as_attribute().ok_or("expected was not an attribute")?;

    // compare just the tokens; the expected node location fields
    // are dummied and won't match
    assert_eq!(attr.ident.token, expected.ident.token);
    attr.expr.assert_same_token(&expected.expr);

    Ok(())
  }

  #[rstest(input, expected,
            case("test_1=true", opt!("test_1", boolean!(true))),
            case("TEST_1=true", opt!("test_1", boolean!(true))),
            case("TEST_1=none", opt!("test_1", none!())),
            case(
                r#"test-2= "a test string""#,
                opt!("test-2", string!("a test string")),
            ),
            case(
                "another_test=-193.5",
                opt!("another_test", number!(-193.5)),
            ),
            case(
                "testing=var.foo",
                opt!("testing", binary_op!(ident!("var"), ".", ident!("foo")))
            ),
            case(
                "testing=foo.*.bar.baz",
                opt!(
                    "testing",
                    binary_op!(
                        ident!("foo"),
                        ".",
                        binary_op!(
                            binary_op!(
                                Token::Operator(Operator::AttrSplat),
                                ".",
                                ident!("bar")
                            ),
                            ".",
                            ident!("baz")
                        )
                    )
                )
            ),
            case (
                "testing=foo[*].bar.baz",
                opt!(
                    "testing",
                    binary_op!(
                        ident!("foo"),
                        ".",
                        binary_op!(
                            binary_op!(
                                Token::Operator(Operator::FullSplat),
                                ".",
                                ident!("bar")
                            ),
                            ".",
                            ident!("baz")
                        )
                    )
                )
            )
    )]
  fn test_option(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let span = Span::new_extra(input, info);
    let (span, node) = option(span)?;
    assert_eq!(span.fragment().len(), 0);

    let option = node.token.as_option().ok_or("node.token was not an option")?;
    let expected = expected.as_option().ok_or("expected was not an option")?;

    // compare just the tokens; the expected node location fields
    // are dummied and won't match
    assert_eq!(option.key.token, expected.key.token);
    option.value.assert_same_token(&expected.value);

    Ok(())
  }

  #[rstest(input, expected,
        case("fun()", function!("fun")),
        case("_fun()", function!("_fun")),
        case("fun.sub()", function!("fun", "sub")),
        case("FuN.sUB()", function!("fun", "sub")),
        case(
          "fun(1, 2, false, none, 1dent)",
          function!("fun", none, number!(1), number!(2), boolean!(false), none!(), ident!("1dent"))
        ),
        case("fun.sub(1, 2, 3)", function!("fun", "sub", number!(1), number!(2), number!(3))),
        case("fun.sub( 1 , 2 , 3 )", function!("fun", "sub", number!(1), number!(2), number!(3))),
        case("_fun.sub(1, 2, 3)", function!("_fun", "sub", number!(1), number!(2), number!(3))),
        case("foo(false)", function!("foo", none, boolean!(false))),
        case("foo(!false)", function!("foo", none, unary_op!("!", boolean!(false)))),
        case(
          "bar([1, 2, 3]...)",
          function!("bar", none, unary_op!("...", list!(number!(1), number!(2), number!(3))))
        ),
        case("fun(1, foo=123)", function!("fun", none, number!(1), opt!("foo", number!(123)))),
        case("fun.sub(1.0, foo=fun2.sub(\"thing\", foo2=fun3(false)))",
          function!(
              "fun",
              "sub",
              number!(1.0),
              opt!(
                  "foo",
                  function!(
                      "fun2",
                      "sub",
                      string!("thing"),
                      opt!(
                        "foo2",
                        function!(
                            "fun3",
                            none,
                            boolean!(false)
                        )
                      )
                  )
              )
          )),
        case("fun.sub(123, foo=321, bar=false, baz=\"a test string\", faz=test)",
          function!(
            "fun",
            "sub",
            number!(123),
            opt!("foo", number!(321)),
            opt!("bar", boolean!(false)),
            opt!("baz", string!("a test string")),
            opt!("faz", ident!("test"))
          )
        ),
        case("fun.sub(123, vektor.eth, 0xcac725bef4f114f728cbcfd744a731c2a463c3fc)",
          function!(
            "fun",
            "sub",
            number!(123),
            address!("vektor.eth"),
            address!("0xcac725bef4f114f728cbcfd744a731c2a463c3fc")
          )
        ),
        case("fun((1 + 1))", function!("fun", none, binary_op!(number!(1), "+", number!(1)))),
        case("fun(1 + 1)", function!("fun", none, binary_op!(number!(1), "+", number!(1)))),
        case("fun(1 < 2)", function!("fun", none, binary_op!(number!(1), "<", number!(2)))),
        case("fun.sub(1 * 100.0)", function!("fun", "sub", binary_op!(number!(1), "*", number!(100.0)))),
        case("fun.sub(fun2.sub(1 * 100.01) > 100.0)",
          function!("fun", "sub",
              binary_op!(
                    function!("fun2", "sub", binary_op!(number!(1), "*", number!(100.01))),
                    ">",
                    number!(100.0))
                  )
              ),
        case("fun.sub(if(foo(), true, false))",
          function!("fun", "sub", conditional!(function!("foo"), boolean!(true), boolean!(false)))
        ),
        case("fun.sub(foo() ? true : false)",
          function!("fun", "sub", conditional!(function!("foo"), boolean!(true), boolean!(false)))
        ),
        case(r#"fun.sub(
          123
        )"#,
          function!("fun",  "sub", number!(123))
        ),
        case("fun.sub(\n\t123,\n\tfalse,\n\t\"vektor\",\n\tfoo() ? true : false)",
          function!(
            "fun",
            "sub",
            number!(123),
            boolean!(false),
            string!("vektor"),
            conditional!(function!("foo"), boolean!(true), boolean!(false))
          )
        ),
    )]
  fn test_function(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let input = Span::new_extra(input, info);

    let (span, node) = function(input)?;
    assert_eq!(span.fragment().len(), 0);

    let f = node.token.as_function().ok_or("node.token was not a function")?;
    let expected = expected.as_function().ok_or("expected was not a function")?;

    f.name.assert_same_token(&expected.name);
    if let Some(subfunction) = &expected.subfunction {
      subfunction.assert_same_token_if_some(&f.subfunction);
    } else {
      assert!(f.subfunction.is_none())
    }

    // FIXME: expected location fields are dummied so can't test args fully
    for (i, n) in expected.args.iter().enumerate() {
      f.args[i].assert_same_token(n);
    }

    Ok(())
  }

  #[rstest(input, expected,
        case(
            r"if(fun(), foo(), bar())",
            conditional!(function!("fun"), function!("foo"), function!("bar"))
        ),
        case(
            r"If(fun(), foo())",
            conditional!(function!("fun"), function!("foo"))
        ),
        case(
            r"if(true, foo(), bar())",
            conditional!(boolean!(true), function!("foo"), function!("bar"))
        ),
        case(
            r"if(foo(), false)",
            conditional!(function!("foo"), boolean!(false))
        ),
        case(
            r"if(true, foo(), none)",
            conditional!(boolean!(true), function!("foo"), none!())
        ),
        case(
          r"if(((1 + 1) >= 2), foo(123))",
          conditional!(
            binary_op!(
              binary_op!(number!(1), "+", number!(1)),
              ">=",
              number!(2)
            ),
            function!("foo", none, number!(123))
          )
        ),
        case(
            r#"if(true,
              1,
                2
            )"#,
            conditional!(boolean!(true), number!(1), number!(2))
        ),
        case(
            "if(\n\ttrue,\n\tfoo(),\n\tnone)",
            conditional!(boolean!(true), function!("foo"), none!())
        ),
    )]
  fn test_if_statement(input: &'static str, expected: Token, info: TracableInfo) -> Result {
    let input = Span::new_extra(input, info);
    let (span, node) = if_statement(input)?;
    assert!(span.fragment().is_empty());

    let cond = node.token.as_conditional().ok_or("node.token was not a conditional")?;
    let expected = expected.as_conditional().ok_or("expected was not a conditional")?;

    cond.condition.assert_same_token(&expected.condition);
    cond.if_true.assert_same_token(&expected.if_true);
    if let Some(if_false) = &expected.if_false {
      if_false.assert_same_token_if_some(&cond.if_false);
    } else {
      assert!(cond.if_false.is_none())
    }

    Ok(())
  }

  #[rstest(input, expected,
        case(
            "false",
            vec![node!(boolean!(false))]
        ),
        case(
            "! false",
            vec![node!(unary_op!("!", boolean!(false)))]
        ),
        case(
            "true or false",
            vec![node!(binary_op!(boolean!(true), "||", boolean!(false)))]
        ),
        case(
            "1 + 2",
            vec![node!(binary_op!(number!(1), "+", number!(2)))]
        ),
        case(
            "1e-4",
            vec![node!(number!(0.0001))]
        ),
        case(
            "fun()",
            vec![node!(function!("fun"))]
        ),
        case(
            "fun() # comment",
            vec![node!(function!("fun"))]
        ),
        case(
            "fun()
            # comment",
            vec![node!(function!("fun"))]
        ),
        case(
            "fun()    ",
            vec![node!(function!("fun"))]
        ),
        case(
            "fun(); # comment",
            vec![node!(function!("fun"))]
        ),
        case(
            "fun();#comment",
            vec![node!(function!("fun"))]
        ),
        case(
            "fun();
            #comment",
            vec![node!(function!("fun"))]
        ),
        case(
            "fun()    ;",
            vec![node!(function!("fun"))]
        ),
        case(
            "\n\n fun() \n\n",
            vec![node!(function!("fun"))]
        ),
        case(
            "fun(); fun2();",
            vec![node!(function!("fun")), node!(function!("fun2"))]
        ),
        case(
            "fun()\nfun2()",
            vec![node!(function!("fun")), node!(function!("fun2"))]
        ),
        case(
            "fun();\nfun2();",
            vec![node!(function!("fun")), node!(function!("fun2"))]
        ),
        case(
            "fun();\nfun2()",
            vec![node!(function!("fun")), node!(function!("fun2"))]
        ),
        case(
          r#"fun.sub(1, true) # comment 1

          1dentifier

          1 + 3 # comment 2

          # comment 3

          if(2 >= 1, fun2(), fun3(opt=1))#comment 4"#,
          vec![
            node!(function!("fun", "sub", number!(1), boolean!(true))),
            node!(ident!("1dentifier")),
            node!(binary_op!(number!(1), "+", number!(3))),
            node!(
              conditional!(
                binary_op!(
                    number!(2),
                    ">=",
                    number!(1)
                ),
                function!("fun2"),
                function!("fun3", none, opt!("opt", number!(1)))
              )
            ),
          ]
        ),
        case(
          r#"fun.sub(
            1,
            true
          )

          1 + 3

          if(
            2 >= 1,
            fun2(),
            fun3(opt=1)
          )"#,
          vec![
            node!(function!("fun", "sub", number!(1), boolean!(true))),
            node!(binary_op!(number!(1), "+", number!(3))),
            node!(
              conditional!(
                binary_op!(
                    number!(2),
                    ">=",
                    number!(1)
                ),
                function!("fun2"),
                function!("fun3", none, opt!("opt", number!(1)))
              )
            ),
          ]
      ),
    )]
  fn test_file(input: &'static str, expected: Tree, info: TracableInfo) -> Result {
    let input = Span::new_extra(input, info);
    let tree = file(input)?;

    assert_eq!(tree.is_empty(), false);
    assert_eq!(tree.len(), expected.len());

    for (i, n) in expected.iter().enumerate() {
      tree[i].assert_same_token(n);
    }

    Ok(())
  }

  #[rstest(input, case("fun() fun2()"))]
  fn test_file_invalid(input: &'static str, info: TracableInfo) {
    let input = Span::new_extra(input, info);
    assert!(file(input).is_err());
  }
}
