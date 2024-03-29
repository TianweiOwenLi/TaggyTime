use std::{fs::File, path::Path};

use crate::{
  time::{timezone::ZoneOffset, MinInstant, TimeError},
  util::path2string,
  util_typs::RefinementError,
};

use self::{
  ics_syntax::{ICSParser, ICalendar},
  lexer::{IcsLexer, Token},
};

use std::io::Write;

pub mod ics_syntax;
pub mod lexer;
pub mod peekbuf;

#[allow(dead_code)]
#[derive(Clone)]
pub enum ICSProcessError {
  NotIcsFile(String),
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
      ICSProcessError::NotIcsFile(s) => write!(f, "`{}` is not an ics file", s),
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

/// Performs lexing plus parsing for the given `.ics` file.
pub fn lex_and_parse<P: AsRef<Path>>(
  path: P,
  default_tz: ZoneOffset,
) -> Result<ICalendar, ICSProcessError> {
  let content = std::fs::read_to_string(&path)
    .expect(format!("Cannot read from `{}`", path2string(&path)).as_str());

  let lex = IcsLexer::new(&path, &content);
  ICSParser::from_ics_lexer(lex).parse(default_tz)
}

#[allow(dead_code)]
pub fn test_lexer<P: AsRef<Path>>(path: P) -> Result<(), ICSProcessError> {
  let content = std::fs::read_to_string(&path)
    .expect(format!("Cannot read from `{}`", path2string(&path)).as_str());

  let mut lex = IcsLexer::new(&path, &content);

  let mut out_file = File::create(format!("{}.tokens", path2string(&path)))
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

#[allow(dead_code)]
pub fn test_parser(ics_filename: &str) -> Result<(), TimeError> {
  let parse_result = lex_and_parse(ics_filename, ZoneOffset::new(-240)?)?;

  let mut out_file = File::create(format!("{}.parsed", ics_filename))
    .expect("Cannot open test parser file");

  writeln!(out_file, "// Parse result\n\n{}", parse_result)
    .expect("Failed to write parsing result to out file");

  Ok(())
}
