use std::fs::File;

use crate::{time::MinInstant, util_typs::RefinementError};

use self::{
  ics_syntax::ICSParser,
  lexer::{IcsLexer, Token},
};

use std::io::Write;

pub mod ics_syntax;
pub mod lexer;
pub mod peekbuf;

#[allow(dead_code)]
#[derive(Clone)]
pub enum ICSProcessError {
  EOF,
  NaN(Token),
  CannotCastTok(Token),
  ICSTimeMalformatted(String, String),
  MalformedList(Token, Token),
  InvalidFreq(Token),
  UntilAndCountBothAppear(usize, MinInstant),
  Refinement(RefinementError),
  Msg(&'static str),
  Other(String),
}

impl std::fmt::Display for ICSProcessError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ICSProcessError::EOF => write!(f, "End of file error"),
      ICSProcessError::CannotCastTok(t) => {
        write!(f, "Cannot cast token `{}` to str", t)
      }
      ICSProcessError::NaN(tok) => write!(f, "`{}` is not a number", tok),
      ICSProcessError::ICSTimeMalformatted(s1, s2) => {
        write!(f, "Cannot parse `{}/{}` as valid time", s1, s2)
      }
      ICSProcessError::MalformedList(t1, t2) => {
        write!(f, "List malformed with elements {} {}", t1, t2)
      }
      ICSProcessError::InvalidFreq(t) => write!(f, "{} is invalid freq", t),
      ICSProcessError::UntilAndCountBothAppear(n, mi) => {
        write!(f, "count=`{}` and until=`{}` cannot both appear", n, mi)
      }
      ICSProcessError::Msg(s) => write!(f, "ICS err: {}", s),
      ICSProcessError::Other(s) => write!(f, "ICS process error: {}", s),
      ICSProcessError::Refinement(r) => write!(f, "{:?}", r),
    }
  }
}

impl std::fmt::Debug for ICSProcessError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self)
  }
}

impl<'a> From<&'a ICSProcessError> for ICSProcessError {
  fn from(value: &'a ICSProcessError) -> Self {
    value.clone()
  }
}

impl From<RefinementError> for ICSProcessError {
  fn from(value: RefinementError) -> Self {
    ICSProcessError::Refinement(value)
  }
}

pub fn test_lexer(ics_filename: &str) -> Result<(), ICSProcessError> {
  let content = std::fs::read_to_string(ics_filename)
    .expect(format!("Cannot read from `{}`", ics_filename).as_str());

  let mut lex = IcsLexer::new(ics_filename, &content);

  let mut out_file = File::create(format!("{}.tokens", ics_filename))
    .expect("Cannot open test lexer file");

  loop {
    match lex.token() {
      Ok(tok) => {
        write!(out_file, "{}\n", tok)
          .expect("Failed to write lexing result to out-file");
      }
      Err(ICSProcessError::EOF) => break Ok(()),
      Err(e) => break Err(e),
    }
  }
}

pub fn test_parser(ics_filename: &str) -> Result<(), ICSProcessError> {
  let content = std::fs::read_to_string(ics_filename)
    .expect(format!("Cannot read from `{}`", ics_filename).as_str());

  let lex = IcsLexer::new(ics_filename, &content);
  let mut parser = ICSParser::from_ics_lexer(lex);
  let parse_result = parser.parse()?;

  let mut out_file = File::create(format!("{}.parsed", ics_filename))
    .expect("Cannot open test parser file");

  writeln!(out_file, "// Parse result\n\n{}", parse_result)
    .expect("Failed to write parsing result to out file");

  Ok(())
}
