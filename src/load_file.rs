//! Loads various types of files.

use crate::{ics_parser::{
  ics_syntax::ICSParser, lexer::IcsLexer, ICSProcessError,
}, calendar::cal_event::Event};

/// [todo] elim code-duplication from test_parser
pub fn load_schedule_ics(ics_filename: &str) -> Result<Vec<Event>, ICSProcessError> {
  let content = std::fs::read_to_string(ics_filename)
    .expect(format!("Cannot read from `{}`", ics_filename).as_str());

  let lex = IcsLexer::new(ics_filename, &content);
  let mut parser = ICSParser::from_ics_lexer(lex);
  let parse_result = parser.parse()?;

  parse_result.content.into_iter().map(Event::try_from).collect()
}
