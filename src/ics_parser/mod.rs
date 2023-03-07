use std::fs::File;

use self::{lexer::{IcsLexer}, ics_syntax::ICSParser};
use crate::error::ICSProcessError;

use std::io::Write;

pub mod lexer;
pub mod ics_syntax;
pub mod peekbuf;

pub fn test_lexer(ics_filename: &str) -> Result<(), ICSProcessError> {
  let content = std::fs::read_to_string(ics_filename)
    .expect(format!("Cannot read from `{}`", ics_filename).as_str());

  let mut lex = IcsLexer::new(&content);

  let mut out_file = File::create(format!("{}.tokens", ics_filename))
    .expect("Cannot open test lexer file");

  loop {
    match lex.token() {
      Ok(tok) => {
        write!(out_file, "{}\n", tok)
          .expect("Failed to write lexing result to out-file");
      }
      Err(ICSProcessError::EOF) => {
        break Ok(())
      }
      Err(ICSProcessError::Other(s)) => {
        break Err(ICSProcessError::Other(s))
      }
    }
  }
}

pub fn test_parser(ics_filename: &str) -> Result<(), ICSProcessError> {
  let content = std::fs::read_to_string(ics_filename)
    .expect(format!("Cannot read from `{}`", ics_filename).as_str());

  let mut parser = ICSParser::from_ics_lexer(IcsLexer::new(&content));
  let parse_result = parser.parse()?;

  let mut out_file = File::create(format!("{}.parsed", ics_filename))
    .expect("Cannot open test parser file");

  writeln!(out_file, "// Parse result\n\n{}", parse_result)
    .expect("Failed to write parsing result to out file");

  Ok(())
}
