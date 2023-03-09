//! Peek buffer datastructure.
//!
//! This code fragment is from the author's 15411 Compiler class.

use std::collections::VecDeque;

use super::lexer::{IcsLexer, Token};
use super::ICSProcessError;

/// A buffer wrapped on top of a lexer that allows arbitrary-depth `peek()`.
pub struct PeekBuffer<'a> {
  buf: VecDeque<Result<Token, ICSProcessError>>,
  lex: IcsLexer<'a>,
}

impl<'a> PeekBuffer<'a> {
  /// Creates a new `PeekBuffer` instance from a `crate::lex::Lexer`.
  pub fn from_lexer(lexer: IcsLexer<'a>) -> Self {
    Self {
      buf: VecDeque::<Result<Token, ICSProcessError>>::new(),
      lex: lexer,
    }
  }

  /// Peeks the `k`th item, where `k=0` peeks the immediate next item. This
  /// function never advances the lexer. Multiple calls return the same result.
  ///
  /// [args] `k` the index of desired item, where `0` stands for the immediate
  /// next item.
  pub fn peek(
    &mut self,
    n: usize,
  ) -> std::result::Result<&Token, &ICSProcessError> {
    while self.buf.len() <= n {
      // so that get() will return Some(_).
      self.buf.push_back(self.lex.token());
    }
    self
      .buf
      .get(n)
      .expect("peekable should have kth element")
      .as_ref()
  }

  /// Gets the next token. This function always advances the lexer by one token.
  pub fn token(&mut self) -> Result<Token, ICSProcessError> {
    let ret = self.buf.pop_front();

    // !!! This is NOT the same as unwrap_or(). This short-circuits !!!
    let ret = match ret {
      Some(r) => r,
      None => self.lex.token(),
    };

    ret
  }
}
