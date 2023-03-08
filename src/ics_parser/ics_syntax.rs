//! ICalendar syntax and parsing.
//! 
//! Note that many details in ICalendar are purposefully omitted, because they 
//! are less relevant to workload calculation.

use crate::{calendar::cal_event::Recurrence, time::MinInterval};

use super::{lexer::{IcsLexer, Token}, peekbuf::PeekBuffer};
use crate::error::ICSProcessError;

pub struct ICalendar{
  name: String,
  content: Vec<Vevent>
}

pub struct Vevent {
  repeat: Recurrence,
  summary: String,
}

pub struct ICSParser<'a> {
  name: String,
  peekbuf: PeekBuffer<'a>
}

impl<'a> ICSParser<'a> {

  // ---------------------------- Helper Functions ----------------------------

  pub fn from_ics_lexer(lex: IcsLexer<'a>) -> ICSParser<'a> {
    ICSParser { 
      name: lex.get_name(),
      peekbuf: PeekBuffer::from_lexer(lex) }
  }

  pub fn peek(&mut self, pos: usize) -> Result<&Token, &ICSProcessError> {
    self.peekbuf.peek(pos)
  }

  pub fn parse(&mut self) -> Result<ICalendar, ICSProcessError> {
    let vevents = Vec::<Vevent>::new();

    // do something

    return Ok(ICalendar{name: "dummy".to_string(), content: vevents});
  }
}

impl std::fmt::Display for Vevent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}\n{}", self.summary, self.repeat)
  }
}

impl std::fmt::Display for ICalendar {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "ICalendar `{}`: \n[\n", self.name)?;
    for item in &self.content {
      write!(f, "->\n{}", item)?;
    }
    write!(f, "\n]")
  }
}
