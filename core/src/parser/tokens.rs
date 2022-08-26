use std::{convert::TryFrom, fmt, num::ParseIntError, rc::Rc};

use super::{Node, N};
use paste::paste;
use rust_decimal::Error as DecimalError;
use serde::Serialize;
use serde_with::SerializeDisplay;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenError {
  #[error("unable to parse int: {0}")]
  ParseIntError(#[from] ParseIntError),

  #[error("unable to parse decimal: {0}")]
  ParseDecimalError(#[from] DecimalError),

  #[error("unrecognized operator: {0}")]
  OperatorError(String),
}

/// Function call node
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Function {
  pub name: Rc<Node>,
  pub subfunction: Option<Rc<Node>>,
  pub args: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ForLoop {
  Tuple {
    binds: Vec<Node>,
    expr: Rc<Node>,
    body: Rc<Node>,
    cond: Option<Rc<Node>>,
  },
  Object {
    binds: Vec<Node>,
    expr: Rc<Node>,
    body: Rc<(Node, Node)>,
    cond: Option<Rc<Node>>,
    grouping: bool,
  },
}

/// Conditional node
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Conditional {
  pub condition: Rc<Node>,
  pub if_true: Rc<Node>,
  pub if_false: Option<Rc<Node>>,
}

/// Uniary operation node
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnaryOp {
  pub operator: Rc<Node>,
  pub operand: Rc<Node>,
}

/// Binary operation node
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BinaryOp {
  pub operator: Rc<Node>,
  pub left: Rc<Node>,
  pub right: Rc<Node>,
}

/// Attribute node
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Attribute {
  pub ident: Rc<Node>,
  pub expr: Rc<Node>,
}

/// Object node
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ObjectItem {
  pub key: Rc<Node>,
  pub val: Rc<Node>,
}

/// Option node
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Opt {
  pub key: Rc<Node>,
  pub value: Rc<Node>,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, SerializeDisplay)]
pub enum Operator {
  // Arithmetic
  Plus,
  Minus,
  Multiply,
  Divide,
  Modulus,
  Exponent,

  // Logical
  And,
  Or,
  Not,

  // Comparison
  Equal,
  NotEqual,
  Greater,
  Less,
  GreaterEqual,
  LessEqual,

  // Postfix
  Elipsis,
  AttrAccess,
  IndexAccess,
  AttrSplat,
  FullSplat,

  // List,
  ListConcatenate,
  ListSubtract,
}

impl fmt::Display for Operator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use Operator::*;
    let symbol = match self {
      // Arithmetic
      Plus => "+",
      Minus => "-",
      Multiply => "*",
      Divide => "/",
      Modulus => "%",
      Exponent => "^",

      // Logical
      And => "&&",
      Or => "||",
      Not => "!",

      // Comparison
      Equal => "==",
      NotEqual => "!=",
      Greater => ">",
      Less => "<",
      GreaterEqual => ">=",
      LessEqual => "<=",

      // Postfix
      Elipsis => "...",
      AttrAccess => ".",
      AttrSplat => ".*",
      IndexAccess => "[",
      FullSplat => "[*]",

      // List
      ListConcatenate => "++",
      ListSubtract => "--",
    };
    write!(f, "{}", symbol)
  }
}

impl TryFrom<&str> for Operator {
  type Error = TokenError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      // Arithmetic
      "+" => Ok(Operator::Plus),
      "-" => Ok(Operator::Minus),
      "*" => Ok(Operator::Multiply),
      "/" => Ok(Operator::Divide),
      "%" => Ok(Operator::Modulus),
      "^" => Ok(Operator::Exponent),

      // Logical
      "&&" => Ok(Operator::And),
      "||" => Ok(Operator::Or),
      "!" => Ok(Operator::Not),

      // Comparison
      "==" => Ok(Operator::Equal),
      "!=" => Ok(Operator::NotEqual),
      ">" => Ok(Operator::Greater),
      "<" => Ok(Operator::Less),
      ">=" => Ok(Operator::GreaterEqual),
      "<=" => Ok(Operator::LessEqual),

      // Postfix
      "..." => Ok(Operator::Elipsis),
      "." => Ok(Operator::AttrAccess),
      ".*" => Ok(Operator::AttrSplat),
      "[" => Ok(Operator::IndexAccess),
      "[*]" => Ok(Operator::FullSplat),

      // List
      "++" => Ok(Operator::ListConcatenate),
      "--" => Ok(Operator::ListSubtract),

      _ => Err(TokenError::OperatorError(value.into())),
    }
  }
}

/// Tokens are the parsed elements of a VXL
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Token {
  /// Unknown is used to be able to implement Default for Node, it should
  /// never actually be encountered outside this lib's internals and should
  /// be treated as a bug if it is.
  Unknown,

  Identifier(String),
  Variable(String),
  Option(Opt),
  Address(String),

  // Basic literal types
  Boolean(bool),
  Number(N),
  String(String),
  None,

  // Expression terms
  Function(Function),
  Conditional(Conditional),
  Operator(Operator),
  BinaryOp(BinaryOp),
  UnaryOp(UnaryOp),

  // Containers
  List(Vec<Node>),
  Object(Vec<Node>),
  ObjectItem(ObjectItem),
  ForLoop(ForLoop),

  // Body elements
  LineComment(String),
  BlockComment(String),
  Attribute(Attribute),

  // Body
  Body(Vec<Node>),
}

macro_rules! gen_as {
  ($n:ident, $t:pat) => {
    paste! {
        pub fn [<as_ $n>](&self) -> Option<()> {
            if matches!(self, $t) {
                Some(())
            } else {
                None
            }
        }
    }
  };
  ($n:ident, $t:pat, $ot:ty, $o:tt) => {
    paste! {
        pub fn [<as_ $n>](&self) -> Option<$ot> {
            if let $t = self {
                Some($o)
            } else {
                None
            }
        }
    }
  };
  ($n:ident, $t:pat, $ot:ty, clone $o:tt) => {
    paste! {
        pub fn [<as_ $n>](&self) -> Option<$ot> {
            if let $t = self {
                Some(Rc::clone($o))
            } else {
                None
            }
        }
    }
  };
}

impl Token {
  gen_as!(identifier, Token::Identifier(s), &str, s);
  gen_as!(variable, Token::Variable(v), &str, v);

  gen_as!(true, Token::Boolean(true));
  gen_as!(false, Token::Boolean(false));
  gen_as!(string, Token::String(s), &str, s);
  gen_as!(number, Token::Number(n), &N, n);
  gen_as!(address, Token::Address(a), &str, a);
  gen_as!(none, Token::None);

  gen_as!(function, Token::Function(f), &Function, f);
  gen_as!(conditional, Token::Conditional(c), &Conditional, c);
  gen_as!(operator, Token::Operator(o), &Operator, o);
  gen_as!(binary_op, Token::BinaryOp(o), &BinaryOp, o);
  gen_as!(unary_op, Token::UnaryOp(u), &UnaryOp, u);

  gen_as!(list, Token::List(l), &Vec<Node>, l);
  gen_as!(object, Token::Object(o), &Vec<Node>, o);
  gen_as!(object_item, Token::ObjectItem(oi), &ObjectItem, oi);
  gen_as!(for_loop, Token::ForLoop(f), &ForLoop, f);

  gen_as!(line_comment, Token::LineComment(s), &str, s);
  gen_as!(block_comment, Token::BlockComment(s), &str, s);
  gen_as!(attribute, Token::Attribute(a), &Attribute, a);
  gen_as!(option, Token::Option(o), &Opt, o);
  gen_as!(body, Token::Body(b), &Vec<Node>, b);

  pub fn as_boolean(&self) -> Option<bool> {
    match self {
      Token::Boolean(true) => Some(true),
      Token::Boolean(false) => Some(false),
      _ => None,
    }
  }
}

#[macro_export]
macro_rules! some {
  ($s:expr) => {
    Some($s)
  };

  (rc $s:expr) => {
    std::rc::Rc::new(Some($s))
  };
}

#[macro_export]
macro_rules! ident {
  ($s:expr) => {
    Token::Identifier(String::from($s))
  };
}

#[macro_export]
macro_rules! string {
  ($s:expr) => {
    Token::String(String::from($s))
  };
}

#[macro_export]
macro_rules! operator {
  ($s:expr) => {
    Token::Operator(Operator::try_from($s).unwrap())
  };
}

#[macro_export]
macro_rules! function {
    ($n:expr) => {
        Token::Function(Function{
            name: node!(rc ident!($n)),
            subfunction: node!(none),
            args: vec![],
        })
    };

    ($n:expr, $s:expr) => {
        Token::Function(Function{
            name: node!(rc ident!($n)),
            subfunction: some!(node!(rc ident!($s))),
            args: vec![],
        })
    };

    ($n:expr, none, $($a:expr),*) => {
        Token::Function(Function{
            name: node!(rc ident!($n)),
            subfunction: node!(none),
            args: vec![$(node!($a),)*],
        })
    };

    ($n:expr, $s:expr, $($a:expr),*) => {
        Token::Function(Function{
            name: node!(rc ident!($n)),
            subfunction: some!(node!(rc ident!($s))),
            args: vec![$(node!($a),)*],
        })
    };
}

#[macro_export]
macro_rules! number {
    (-$s:expr) => {
        Token::UnaryOp(UnaryOp{
            operator: node!(rc operator!("-")),
            operand: node!(rc number!($s)),
        })
    };
    ($s:expr) => {
        Token::Number($s.into())
    };
}

#[macro_export]
macro_rules! boolean {
  (true) => {
    Token::Boolean(true)
  };
  (false) => {
    Token::Boolean(false)
  };
}

#[macro_export]
macro_rules! address {
  ($s:expr) => {
    Token::Address(String::from($s))
  };
}

#[macro_export]
macro_rules! line_comment {
  ($s:expr) => {
    Token::LineComment(String::from($s))
  };
}

#[macro_export]
macro_rules! none {
  () => {
    Token::None
  };
}

#[macro_export]
macro_rules! node {
  (none) => {
    None
  };

  (rc $t:expr) => {
    std::rc::Rc::new(node!($t))
  };

  (rc some $t:expr) => {
    std::rc::Rc::new(node!($t))
  };

  ($t:expr) => {
    Node {
      token: $t,
      ..Default::default()
    }
  };

  (some $t:expr) => {
    Some(Node {
      token: $t,
      ..Default::default()
    })
  };
}

#[macro_export]
macro_rules! attr {
    ($i:expr, $e:expr) => {
        Token::Attribute(Attribute {
            ident: node!(rc ident!($i)),
            expr: node!(rc $e),
        })
    };
}

#[macro_export]
macro_rules! opt {
    ($i:expr, $e:expr) => {
        Token::Option(Opt {
            key: node!(rc ident!($i)),
            value: node!(rc $e),
        })
    };
}

#[macro_export]
macro_rules! list {
    ($($i:expr),*) => {
        Token::List(vec![$(node!($i),)*])
    };
}

#[macro_export]
macro_rules! object_item {
    ($k:expr => $v:expr) => {
        Token::ObjectItem(ObjectItem{key: node!(rc $k), val: node!(rc $v)})
    };
}

#[macro_export]
macro_rules! object {
    ($($k:expr => $v:expr),*) => {
        Token::Object(vec![$(node!(object_item!($k => $v)),)*])
    }
}

#[macro_export]
macro_rules! unary_op {
    ($o:expr, $i:expr) => {
        Token::UnaryOp(UnaryOp {
            operator: node!(rc operator!($o)),
            operand: node!(rc $i),
        })
    };
}

#[macro_export]
macro_rules! binary_op {
    ($l:expr, $o:expr, $r:expr) => {
        Token::BinaryOp(BinaryOp{
            left: node!(rc $l),
            operator: node!(rc operator!($o)),
            right: node!(rc $r),
        })
    };
}

#[macro_export]
macro_rules! conditional {
    ($t:expr, $l:expr) => {
        Token::Conditional(Conditional{
            condition: node!(rc $t),
            if_true: node!(rc $l),
            if_false: node!(none),
        })
    };

    ($t:expr, $l:expr, none) => {
        Token::Conditional(Conditional{
            condition: node!(rc $t),
            if_true: node!(rc $l),
            if_false: node!(none),
        })
    };

    ($t:expr, $l:expr, $r:expr) => {
        Token::Conditional(Conditional{
            condition: node!(rc $t),
            if_true: node!(rc $l),
            if_false: some!(node!(rc $r)),
        })
    };
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn as_identifier() {
    assert_eq!(ident!("test_ident").as_identifier(), Some("test_ident"));
  }

  #[test]
  fn as_boolean() {
    let t = boolean!(true);
    assert_eq!(t.as_boolean(), Some(true));
    assert_eq!(t.as_true(), Some(()));

    let f = boolean!(false);
    assert_eq!(f.as_boolean(), Some(false));
    assert_eq!(f.as_false(), Some(()));
  }
}
