//! Loads various types of files.

use crate::ics_parser::{
  ics_syntax::ICSParser, lexer::IcsLexer, ICSProcessError,
};

/// [todo] elim code-duplication from test_parser
pub fn load_schedule_ics(ics_filename: &str) -> Result<(), ICSProcessError> {
  let content = std::fs::read_to_string(ics_filename)
    .expect(format!("Cannot read from `{}`", ics_filename).as_str());

  let lex = IcsLexer::new(ics_filename, &content);
  let mut parser = ICSParser::from_ics_lexer(lex);
  let parse_result = parser.parse()?;

  for ve in &parse_result.content {
    println!("{}", ve);
    if let Some(ref fr) = ve.repeat {
      let ppt = crate::time::date::parse_dateproperty_week(fr);
      println!("Parsed ppt: {:?}\n", ppt);
    }
  }

  Ok(())
}
