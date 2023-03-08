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
  fn skip_till(&mut self, target_tok: Token) -> Result<(), ICSProcessError> {
    loop {
      match self.peek(0) {
        Ok(head_ok) => if &target_tok == head_ok { break Ok(()) }
        Err(e) => break Err(e.clone())
      }
      self.skip()?;
    }
  }

  // --------------------------- Main Functionality ---------------------------

  pub fn parse(&mut self) -> Result<ICalendar, ICSProcessError> {
    let mut vevents = Vec::<Vevent>::new();

    self.munch(Token::BEGIN)?;
    self.munch(Token::COLON)?;
    self.munch(Token::VCALENDAR)?;

    loop {
      self.skip_till(Token::BEGIN)?;
      if self.peek_copy(2)? == Token::VEVENT {

        // consume body of vevent
        vevents.push(self.vevent()?);

        // check for end of vcalendar
        if self.peek(0)? == &Token::END {
          self.munch(Token::END)?;
          self.munch(Token::COLON)?;
          self.munch(Token::VCALENDAR)?;

          // check for EOF
          match self.token() {
            Ok(_) => return Err(ICSProcessError::Other(
              "Unexpected tok after calendar".to_string()
            )),
            Err(ICSProcessError::EOF) => {},
            Err(e) => return Err(e),
          }
      
          return Ok(ICalendar{name: self.name.clone(), content: vevents});
        }
      } else {
        println!("skipped begin since it is {}", self.peek_copy(2)?);
        self.munch(Token::BEGIN)?;
        continue;
      }
    }
  }

  pub fn vevent(&mut self) -> Result<Vevent, ICSProcessError> {
    println!("-- start of vevent --");
    self.munch(Token::BEGIN)?;
    self.munch(Token::COLON)?;
    self.munch(Token::VEVENT)?;

    let mut dtstart: Option<MinInstant> = None;
    let mut dtend: Option<MinInstant> = None;
    let mut summary = String::new();

    loop {
      match self.peek(0)? {
        Token::DTSTART => {
          dtstart = Some(self.dtstart()?);
        }
        Token::DTEND => {
          dtend = Some(self.dtend()?);
        }
        Token::SUMMARY => {
          self.munch(Token::SUMMARY)?;
          self.munch(Token::COLON)?;
          summary = self.string()?;
          println!("Encountered summary: {}", summary);
        }
        Token::RRULE => {
          // todo when implementing recurrences.
        }
        Token::END => {
          self.munch(Token::END)?;
          self.munch(Token::COLON)?;
          let end_tag = self.token()?;
          if end_tag == Token::VEVENT {
            match (dtstart, dtend) {
              (Some(start), Some(end)) => {
                println!("-- end of vevent --");
                return Ok(Vevent { 
                  repeat: Recurrence::Once(MinInterval::new(start, end)), 
                  summary
                })
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
            return Err(ICSProcessError::Other(
              "VEVENT contains unexpected end".to_string()
            ));
          }
        }
        _ => {}
      }
    }

  }

  /// Parses the time associated with some `DTSTART`.
  pub fn dtstart(&mut self) -> Result<MinInstant, ICSProcessError> {
    self.munch(Token::DTSTART)?;
    self.dt_possible_timezone()
  }

  pub fn dtend(&mut self) -> Result<MinInstant, ICSProcessError> {
    self.munch(Token::DTEND)?;
    self.dt_possible_timezone()
  }

  fn dt_possible_timezone(&mut self) -> Result<MinInstant, ICSProcessError> {
    match self.token()? {

      // when timezone is specified
      Token::SEMICOLON => {
        self.munch(Token::TZID)?;
        self.munch(Token::EQ)?;
        println!("t1 {}", self.token()?);
        println!("t2 {}", self.token()?);
        println!("t3 {}", self.token()?);
        unimplemented!()
      }

      // when timezone is not specified
      Token::COLON => {
       unimplemented!()
      }

      x => Err(ICSProcessError::Other(
        format!("Expected : or ; after dt, found {}", x)
      ))
    }
  }

  /// Skips till the symbol `begin:after`.
  pub fn goto_colon_sandwich(&mut self, before: Token, after: Token) 
  -> Result<(), ICSProcessError> {
    self.skip_till(before.clone())?;
    
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

  /// Concats all subsequent `Other(..)` as string, until encounters some 
  /// token that is not `Other(..)`.
  pub fn string(&mut self) -> Result<String, ICSProcessError> {
    let mut ret = String::new();
    while let Ok(t) = self.peek(0) {
      let head = self.token()?;
      ret.push_str(head.cast_as_string());
    }
    Ok(ret)
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
