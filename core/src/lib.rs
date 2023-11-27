use std::{error::Error, rc::Rc};

pub mod tracer;
use tracer::get_tracer;

#[macro_use]
mod address;
mod boolean;
mod collection;
mod comment;
mod identifier;
mod list;
mod literal;
mod n;
mod node;
mod numeric;
mod operation;
mod result;
mod string;
mod tokens;

pub use address::*;
pub use boolean::*;
pub use collection::*;
pub use comment::*;
pub use identifier::*;
pub use list::*;
pub use literal::*;
pub use n::*;
pub use node::*;
pub use numeric::*;
pub use operation::*;
pub use result::*;
pub use string::*;
pub use tokens::*;

use nom::{
  branch::alt,
  bytes::complete::{tag, tag_no_case, take},
  character::complete::{char, line_ending, multispace0, newline, space0},
  combinator::{all_consuming, complete, eof, map, opt, recognize},
  error::ErrorKind,
  multi::{fold_many0, fold_many1, many0, many1},
  sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
  Err,
};

use nom_locate::position;
use nom_tracable::tracable_parser;

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

      if let Some((first, mut tail)) = args {
        f.args.push(first);
        f.args.append(&mut tail);
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
fn expr_postfix(i: Span) -> Result {
  let (_, head): (_, Span) = take(2usize)(i)?;
  let mut head = head.fragment().chars();
  match head.next() {
    Some('.') => attr_access(i),
    Some('[') => index_access(i),
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
  use crate::*;
  use nom_tracable::TracableInfo;
  use rstest::{fixture, rstest};
  use std::convert::TryFrom;

  pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

  #[fixture]
  pub fn info() -> TracableInfo {
    TracableInfo::default()
  }

  #[rstest(input, expected,
            case("test_1 = true", attr!("test_1", boolean!(true))),
            case("TEST_1 = true", attr!("test_1", boolean!(true))),
            case(
                "another_test = -193.5\n",
                attr!("another_test", number!(-193.5)),
            ),
            case(
                "testing = var.foo\n",
                attr!("testing", binary_op!(ident!("var"), ".", ident!("foo")))
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
            case(
                "another_test=-193.5",
                opt!("another_test", number!(-193.5)),
            ),
            case(
                "testing=var.foo",
                opt!("testing", binary_op!(ident!("var"), ".", ident!("foo")))
            ),
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
        case("fun(1foo_v1)", function!("fun", none, ident!("1foo_v1"))),
        case(
          "fun(1, 2%, false, none, 1dent)",
          function!("fun", none, number!(1), percentage!(2), boolean!(false), ident!("none"), ident!("1dent"))
        ),
        case("fun.sub(1, 2, 3)", function!("fun", "sub", number!(1), number!(2), number!(3))),
        case("fun.sub( 1 , 2 , 3 )", function!("fun", "sub", number!(1), number!(2), number!(3))),
        case("_fun.sub(1, 2%, 3)", function!("_fun", "sub", number!(1), percentage!(2), number!(3))),
        case("foo(false)", function!("foo", none, boolean!(false))),
        case("foo(!false)", function!("foo", none, unary_op!("!", boolean!(false)))),
        case("foo(-bar)", function!("foo", none, unary_op!("-", ident!("bar")))),
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
        case("fun.sub(123, foo=321, bar=false, baz=\"a test string\", faz=test, foz=-1_000.12%)",
          function!(
            "fun",
            "sub",
            number!(123),
            opt!("foo", number!(321)),
            opt!("bar", boolean!(false)),
            opt!("baz", string!("a test string")),
            opt!("faz", ident!("test")),
            opt!("foz", percentage!(-1_000.12))
          )
        ),
        case("fun.sub(123, 0xcac725bef4f114f728cbcfd744a731c2a463c3fc, 0x)",
          function!(
            "fun",
            "sub",
            number!(123),
            address!("0xcac725bef4f114f728cbcfd744a731c2a463c3fc"),
            ident!("0x")
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
        case(r#"fun(not_or_my_label)"#, function!("fun", none, ident!("not_or_my_label"))),
        case(r#"fun(not_and_or_my_label)"#, function!("fun", none, ident!("not_and_or_my_label"))),
        case(r#"fun(and_label)"#, function!("fun", none, ident!("and_label"))),
        case(r#"fun(or_my_label)"#, function!("fun", none, ident!("or_my_label"))),
        case(r#"fun(in_my_label)"#, function!("fun", none, ident!("in_my_label"))),
        case(r#"fun(not(not_my_label))"#, function!("fun", none, function!("not", none, ident!("not_my_label")))),
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
            r"if(true, foo())",
            conditional!(boolean!(true), function!("foo"), none)
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
            conditional!(boolean!(true), function!("foo"), ident!("none"))
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
            "[1, 2.0, 3%] ++ [1, 2.0, 3%]",
             vec![node!(binary_op!(list!(number!(1), number!(2.0), percentage!(3)), "++", list!(number!(1), number!(2.0), percentage!(3))))]
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

          1 + 3_000.0_0_01 # comment 2

          # comment 3

          if(2 >= 1, fun2(), fun3(opt=1))#comment 4"#,
          vec![
            node!(function!("fun", "sub", number!(1), boolean!(true))),
            node!(ident!("1dentifier")),
            node!(binary_op!(number!(1), "+", number!(3_000.0_0_01))),
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
            true,
            1foo_v1
          )

          1 + 3%

          if(
            2 >= 1,
            fun2(),
            fun3(opt=1)
          )"#,
          vec![
            node!(function!("fun", "sub", number!(1), boolean!(true), ident!("1foo_v1"))),
            node!(binary_op!(number!(1), "+", percentage!(3))),
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
