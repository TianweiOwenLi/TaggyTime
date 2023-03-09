use std::fs::File;

use self::{ics_syntax::ICSParser, lexer::IcsLexer};
use crate::error::ICSProcessError;

use std::io::Write;

pub mod ics_syntax;
pub mod lexer;
pub mod peekbuf;

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
