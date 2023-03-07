use std::fs::File;

use self::lexer::{IcsLexer, LexerError};

use std::io::Write;

pub mod lexer;

pub fn test_lexer(ics_filename: &str) -> Result<(), String> {
  let content = std::fs::read_to_string(ics_filename)
    .expect(format!("Cannot read from `{}`", ics_filename).as_str());

  let mut lex = IcsLexer::new(&content);

  let mut out_file = File::create(format!("{}.tokens", ics_filename))
    .expect("Cannot open file");

  loop {
    match lex.token() {
      Ok(tok) => {
        write!(out_file, "{}\n", tok)
          .expect("Failed to write to out-file");
      }
      Err(LexerError::EOF) => {
        break Ok(())
      }
      Err(LexerError::Other(s)) => {
        break Err(s);
      }
    }
  }
}
