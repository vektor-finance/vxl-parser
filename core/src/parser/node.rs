use serde::Serialize;
use std::fmt::Debug;

use super::{Span, Token};

pub type Tree = Vec<Node>;

/// A single node in the AST containing a span and parsed token
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Node {
  pub offset: usize,
  pub line: u32,
  pub column: u32,
  pub token: Token,
}

impl Default for Node {
  fn default() -> Self {
    Node {
      offset: 0,
      line: 0,
      column: 0,
      token: Token::Unknown,
    }
  }
}

impl Node {
  pub fn new(token: Token, span: &Span) -> Self {
    Node {
      token,
      offset: span.location_offset(),
      line: span.location_line(),
      column: span.get_utf8_column() as u32,
    }
  }

  pub fn from_node(token: Token, node: &Node) -> Self {
    Node {
      token,
      offset: node.offset,
      line: node.line,
      column: node.column,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::parser::N;
  use rust_decimal::Decimal;
  use std::rc::Rc;

  impl Node {
    pub(in crate::parser) fn assert_same_token_if_some(&self, other: &Option<Rc<Node>>) {
      if let Some(other) = other {
        self.assert_same_token(&other)
      } else {
        panic!("expected {:#?}, got None", self.token);
      }
    }

    pub(in crate::parser) fn assert_same_token(&self, other: &Node) {
      match &self.token {
        Token::Function(func) => {
          if let Some(other_func) = other.token.as_function() {
            func.name.assert_same_token(&other_func.name);
            if let Some(subfunction) = &func.subfunction {
              subfunction.assert_same_token_if_some(&other_func.subfunction);
            } else {
              assert!(other_func.subfunction.is_none())
            }
            for (i, arg) in func.args.iter().enumerate() {
              arg.assert_same_token(&other_func.args[i]);
            }
          } else {
            panic!("expected Function, got {:#?}; self is {:#?}", other.token, self.token);
          }
        }
        Token::BinaryOp(token) => {
          if let Some(op) = other.token.as_binary_op() {
            token.left.assert_same_token(&op.left);
            token.right.assert_same_token(&op.right);
            token.operator.assert_same_token(&op.operator);
          } else {
            panic!("expected BinaryOp, got {:#?}; self is {:#?}", other.token, self.token);
          }
        }
        Token::UnaryOp(token) => {
          if let Some(op) = other.token.as_unary_op() {
            token.operand.assert_same_token(&op.operand);
            token.operator.assert_same_token(&op.operator);
          } else {
            panic!("expected UnaryOp, got {:#?}; self is {:#?}", other.token, self.token);
          }
        }
        Token::Number(N::Decimal(f1)) => {
          if let Some(N::Decimal(f2)) = other.token.as_number() {
            assert!((f1 - f2).abs() < Decimal::MAX);
          } else {
            panic!("expected N, got {:#?}; self is {:#?}", other.token, self.token)
          }
        }
        Token::List(items) => {
          let other_items = other.token.as_list().unwrap_or_else(|| {
            panic!(
              "expected list or object, got {:#?}; self is {:#?}",
              other.token, self.token
            )
          });
          for (i, item) in items.iter().enumerate() {
            item.assert_same_token(&other_items[i]);
          }
        }
        Token::Conditional(token) => {
          if let Some(cond) = other.token.as_conditional() {
            token.condition.assert_same_token(&cond.condition);
            token.if_true.assert_same_token(&cond.if_true);
            if let Some(if_false) = &token.if_false {
              if_false.assert_same_token_if_some(&cond.if_false);
            } else {
              assert!(cond.if_false.is_none())
            }
          } else {
            panic!(
              "expected Conditional, got {:#?}; self is {:#?}",
              other.token, self.token
            );
          }
        }
        Token::Option(token) => {
          if let Some(cond) = other.token.as_option() {
            token.key.assert_same_token(&cond.key);
            token.value.assert_same_token(&cond.value);
          } else {
            panic!("expected Option, got {:#?}; self is {:#?}", other.token, self.token);
          }
        }
        token => assert_eq!(token, &other.token),
      }
    }
  }
}
