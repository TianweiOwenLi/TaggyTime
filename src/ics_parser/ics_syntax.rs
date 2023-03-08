//! ICalendar syntax and parsing.
//! 
//! Note that many details in ICalendar are purposefully omitted, because they 
//! are less relevant to workload calculation.

use crate::{calendar::cal_event::Recurrence, time::{MinInstant, MinInterval}};

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

  /// Constructs a parser instance from an ics lexer
  pub fn from_ics_lexer(lex: IcsLexer<'a>) -> ICSParser<'a> {
    ICSParser { 
      name: lex.get_name(),
      peekbuf: PeekBuffer::from_lexer(lex) }
  }

  /// Peeks the `pos`th position ahead, with `pos=0` indicating the head. 
  fn peek(&mut self, pos: usize) -> Result<&Token, &ICSProcessError> {
    self.peekbuf.peek(pos)
  }

  /// Similar to peek but copies things 
  fn peek_copy(&mut self, pos: usize) -> Result<Token, ICSProcessError> {
    let ret = self.peekbuf.peek(pos);
    match ret {
      Ok(x) => Ok(x.clone()),
      Err(e) => Err(e.clone())
    }
  }

  /// Takes a token from lexer and advance the stream.
  fn token(&mut self) -> Result<Token, ICSProcessError> {
    self.peekbuf.token()
  }

  /// Takes a token from lexer, advances the stream, and checks that the token 
  /// is indeed expected. 
  fn munch(&mut self, expected_tok: Token) -> Result<(), ICSProcessError> {
    let tok = self.token()?;
    if tok == expected_tok {
      Ok(())
    } else {
      Err(ICSProcessError::Other(format!("Expected {}, found {}", expected_tok, tok)))
    }
  }

  /// Simply skips a token.
  fn skip(&mut self) -> Result<(), ICSProcessError> {
    self.token()?;
    Ok(())
  }

  /// Skips till we encounter the target token.
  fn skip_till(&mut self, target_tok: &Token) -> Result<(), ICSProcessError> {
    loop {
      match self.peek(0) {
        Ok(head_ok) => if target_tok == head_ok { break Ok(()) }
        Err(e) => break Err(e.clone())
      }
      self.skip()?;
    }
  }

  // --------------------------- Main Functionality ---------------------------

  pub fn parse(&mut self) -> Result<ICalendar, ICSProcessError> {
    let mut vevents = Vec::<Vevent>::new();

    // while true
    // vevents.push(self.vevent()?);

    return Ok(ICalendar{name: self.name.clone(), content: vevents});
  }

  pub fn vevent(&mut self) -> Result<Vevent, ICSProcessError> {
    self.munch(Token::BEGIN)?;
    self.munch(Token::COLON)?;
    self.munch(Token::VEVENT)?;

    let mut dtstart: Option<MinInstant> = None;
    let mut dtend: Option<MinInstant> = None;
    let mut summary = String::new();

    loop {
      match self.token()? {
        Token::DTSTART => {
          // todo
        }
        Token::DTEND => {
          // todo
        }
        Token::SUMMARY => {
          self.munch(Token::COLON)?;
          // todo
        }
        Token::RRULE => {
          // todo
        }
        Token::END => {
          self.munch(Token::COLON)?;
          let end_tag = self.token()?;
          if end_tag == Token::VEVENT {
            match (dtstart, dtend) {
              (Some(start), Some(end)) => {

              }
              (None, _) => {
                return Err(ICSProcessError::Other(
                  format!("VEVENT `{}` missing dtstart", summary)
                ));
              }
              _ => {
                return Err(ICSProcessError::Other(
                  format!("VEVENT `{}` missing dtend", summary)
                ));
              }
            }
          } else {

          }
        }
        _ => {}
      }
    }

  }

  /// Skips till the symbol `begin:after`.
  pub fn goto_colon_sandwich(&mut self, before: Token, after: Token) 
  -> Result<(), ICSProcessError> {
    self.skip_till(&before)?;
    
    let p0 = self.peek_copy(0);
    let p1 = self.peek_copy(1);
    let p2 = self.peek_copy(2);

    match (p0, p1, p2) {
      (Ok(tok1), Ok(Token::COLON), Ok(tok2)) => {
        if tok1 != before {
          panic!("skip_till() stops at {}, not {}", tok1, before)
        }
        if after == tok2 {
          Ok(())
        } else {
          self.munch(before.clone())?;
          self.munch(Token::COLON)?;
          self.skip()?;
          self.goto_colon_sandwich(before, after)
        }
      }
      _ => Err(ICSProcessError::Other("ICS colon sandwich malformed".to_string()))
    }
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
