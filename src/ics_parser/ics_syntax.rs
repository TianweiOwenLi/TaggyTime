//! ICalendar syntax and parsing.
//!
//! Note that many details in ICalendar are purposefully omitted, because they
//! are less relevant to workload calculation.

use crate::{
  ics_parser::lexer,
  time::{date::Date, MinInstant, MinInterval}, const_params::ICS_DEFAULT_TIME_IN_DAY,
};


use super::{
  lexer::{IcsLexer, Token},
  peekbuf::PeekBuffer,
  ICSProcessError,
};

pub struct ICalendar {
  name: String,
  content: Vec<Vevent>,
}

pub struct Vevent {
  repeat: Option<FreqAndRRules>, // corrsponds to `Pattern::Once | Many`.
  mi: MinInterval,
  summary: String,
}

/// Frequency of some `RRULE` line. 
#[derive(Debug)]
pub enum Freq {
  Daily,
  Weekly,
  Monthly,
  Yearly,
}

/// A single recurrence rule, in the form `BYXXX=item, item, item...`. 
/// Composed of tokens, and may not be valid. 
pub struct RRuleToks {
  tag: Token,
  content: Vec<String>
}

/// A frequency paired with a vec of `RRuleToks`. 
/// Corresponds to `Pattern::Many`. Specifically, `freq` indicates the specific 
/// variant of `Repeat`, `content` encodes the potential rules for such a 
/// variant, `interval` is self explanatory, and `count`, `until` are for 
/// `Term`.
pub struct FreqAndRRules {
  freq: Freq, 
  content: Vec<RRuleToks>,
  interval: usize,
  count: Option<usize>,
  until: Option<MinInstant>
}

pub struct ICSParser<'a> {
  name: String,
  peekbuf: PeekBuffer<'a>,
}

impl<'a> ICSParser<'a> {
  // ---------------------------- Helper Functions ----------------------------

  /// Constructs a parser instance from an ics lexer
  pub fn from_ics_lexer(lex: IcsLexer<'a>) -> ICSParser<'a> {
    ICSParser {
      name: lex.get_name(),
      peekbuf: PeekBuffer::from_lexer(lex),
    }
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
      Err(e) => Err(e.clone()),
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
      Err(ICSProcessError::Other(format!(
        "Expected {}, found {}",
        expected_tok, tok
      )))
    }
  }

  /// Simply skips a token.
  fn skip(&mut self) -> Result<(), ICSProcessError> {
    self.token()?;
    Ok(())
  }

  /// Skips till the given condition holds. If the condition is never met,
  /// returns EOF error.
  pub fn skip_until_lambda<F>(&mut self, cond: F) -> Result<(), ICSProcessError>
  where
    F: Fn(&Token) -> bool,
  {
    while let Ok(t) = self.peek(0) {
      if cond(t) {
        return Ok(());
      }
      self.skip()?;
    }
    Err(ICSProcessError::EOF)
  }

  /// Takes a token, verifies that it is of `Number(..)` variant, and
  /// returns the corresponding String literal.
  fn number(&mut self) -> Result<String, ICSProcessError> {
    let tok = self.token()?;
    match tok {
      Token::Number(s) => Ok(s),
      bad_tok => Err(ICSProcessError::NaN(bad_tok)),
    }
  }

  /// Keeps taking everything as a string, until the peeked token meets the
  /// given condition.
  pub fn string_until<F>(&mut self, cond: F) -> Result<String, ICSProcessError>
  where
    F: Fn(&Token) -> bool,
  {
    let mut ret = String::new();
    while let Ok(t) = self.peek(0) {
      if cond(t) {
        break;
      }
      let head = self.token()?;
      ret.push_str(&head.cast_as_string());
    }
    Ok(ret)
  }

  // --------------------------- Main Functionality ---------------------------

  pub fn parse(&mut self) -> Result<ICalendar, ICSProcessError> {
    let mut vevents = Vec::<Vevent>::new();

    self.munch(Token::BEGIN)?;
    self.munch(Token::COLON)?;
    self.munch(Token::VCALENDAR)?;

    loop {
      self.skip_until_lambda(|c| c == &Token::BEGIN || c == &Token::END)?;
      match (self.peek_copy(0)?, self.peek_copy(1)?, self.peek_copy(2)?) {
        (Token::BEGIN, Token::COLON, Token::VEVENT) => {
          vevents.push(self.vevent()?);
        }
        (Token::END, Token::COLON, Token::VCALENDAR) => {
          break self.end(vevents)
        }
        (Token::BEGIN | Token::END, Token::COLON, _) => {
          self.skip()?;
        }
        (Token::BEGIN | Token::END, x, _) => {
          break Err(ICSProcessError::Other(format!(
            "Expected COLON after BEGIN / END, found {}",
            x
          )))
        }
        _ => unreachable!(),
      }
    }
  }

  /// Handles end of `VCALENDAR`.
  pub fn end(
    &mut self,
    vevents: Vec<Vevent>,
  ) -> Result<ICalendar, ICSProcessError> {
    self.munch(Token::END)?;
    self.munch(Token::COLON)?;
    self.munch(Token::VCALENDAR)?;

    return Ok(ICalendar {
      name: self.name.clone(),
      content: vevents,
    });
  }

  /// Parses some `VEVENT` from calendar. Note that only `DTSTART`, `DTEND`,
  /// `SUMMARY`, and `RRULE` will be processed; all other components are
  /// simply discarded.
  pub fn vevent(&mut self) -> Result<Vevent, ICSProcessError> {
    self.munch(Token::BEGIN)?;
    self.munch(Token::COLON)?;
    self.munch(Token::VEVENT)?;

    let mut dtstart: Option<MinInstant> = None;
    let mut dtend: Option<MinInstant> = None;
    let mut summary = String::new();
    let mut recur: Option<FreqAndRRules> = None;

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
          summary = self.string_until(lexer::not_in_summary)?;
        }
        Token::RRULE => {
          recur = Some(self.rrules()?);
        }
        Token::END => {
          self.munch(Token::END)?;
          self.munch(Token::COLON)?;
          let end_tag = self.token()?;
          if end_tag == Token::VEVENT {
            match (dtstart, dtend) {
              (Some(start), Some(end)) => {
                return Ok(Vevent {
                  repeat: recur,
                  mi: MinInterval::new(start, end),
                  summary,
                });
              }
              (None, _) => {
                return Err(ICSProcessError::Other(format!(
                  "VEVENT `{}` missing dtstart",
                  summary
                )));
              }
              _ => {
                return Err(ICSProcessError::Other(format!(
                  "VEVENT `{}` missing dtend",
                  summary
                )));
              }
            }
          } else {
            return Err(ICSProcessError::Other(format!(
              "VEVENT contains unexpected end: {}",
              end_tag
            )));
          }
        }
        _ => {
          self.skip()?;
        }
      }
    }
  }

  /// Parses the time associated with some `DTSTART`.
  pub fn dtstart(&mut self) -> Result<MinInstant, ICSProcessError> {
    self.munch(Token::DTSTART)?;
    self.dt_possible_timezone()
  }

  /// Parses the time associated with some `DTEND`.
  pub fn dtend(&mut self) -> Result<MinInstant, ICSProcessError> {
    self.munch(Token::DTEND)?;
    self.dt_possible_timezone()
  }

  /// Parses a datetime literal with an optional timezone prefix.
  ///
  /// ### Syntax
  /// `:[yyyymmdd]T[hhmmss]Z | ;TZID=..:[yyyymmdd]T[hhmmss]`
  fn dt_possible_timezone(&mut self) -> Result<MinInstant, ICSProcessError> {
    match self.token()? {
      // when timezone is specified
      Token::SEMICOLON => {
        self.munch(Token::TZID)?;
        self.munch(Token::EQ)?;
        let tz_string = self.string_until(|c| c == &Token::COLON)?;
        self.munch(Token::COLON)?;

        // TODO: implement zones.

        return self.dt_literal(true);
      }

      // when timezone is not specified
      Token::COLON => {
        return self.dt_literal(false);
      }

      x => Err(ICSProcessError::Other(format!(
        "Expected : or ; after dt, found {}",
        x
      ))),
    }
  }

  /// Parses recurrence rules.
  fn rrules(&mut self) -> Result<FreqAndRRules, ICSProcessError> {
    self.munch(Token::RRULE)?;
    self.munch(Token::COLON)?;

    self.munch(Token::FREQ)?;
    self.munch(Token::EQ)?;

    let freq = match self.token()? {
      Token::DAILY => Freq::Daily,
      Token::WEEKLY => Freq::Weekly,
      Token::MONTHLY => Freq::Monthly,
      Token::YEARLY => Freq::Yearly,
      x => return Err(ICSProcessError::InvalidFreq(x))
    };
    let mut content = Vec::<RRuleToks>::new();
    let mut interval: usize = 1; // default
    let mut count: Option<usize> = None;
    let mut until: Option<MinInstant> = None;

    let mut ready_to_rrule: bool = true;

    loop {
      match self.peek(0)? {
        Token::SEMICOLON => {
          self.skip()?;
          ready_to_rrule = true;
        }
        Token::NEXTLINE => {
          break Ok(FreqAndRRules { 
            freq, 
            content,
            count,
            interval,
            until,
          });
        }
        Token::INTERVAL => {
          self.skip()?;
          self.munch(Token::EQ)?;
          let num_string = self.number()?;
          let interval_opt: Result<usize, _> = num_string.parse();
          match interval_opt {
            Ok(explicit_interval) => interval = explicit_interval,
            Err(_) => return Err(ICSProcessError::Other(
              format!("{} is not valid interval usize", num_string)
            ))
          }
        }
        Token::COUNT => {
          self.skip()?;
          self.munch(Token::EQ)?;
          let num_string = self.number()?;
          let interval_opt: Result<usize, _> = num_string.parse();
          match interval_opt {
            Ok(x) => count = Some(x),
            Err(_) => return Err(ICSProcessError::Other(
              format!("{} is not count valid usize", num_string)
            ))
          }
        }
        Token::UNTIL => {
          self.skip()?;
          self.munch(Token::EQ)?;
          until = Some(self.dt_literal(false)?)
        }
        t => {
          if ready_to_rrule {
            content.push(self.rrule()?);
          } else {
            return Err(ICSProcessError::Other(
              format!("Cannot rrule yet but encountered `{}`", t)
            ))
          }
        }
      }
    }
    
  }

  /// Parses a single recur-rule.
  /// 
  /// ## Syntax
  /// `recur=tok_lst`
  fn rrule(&mut self) -> Result<RRuleToks, ICSProcessError> {
    let tag = self.token()?;
    self.munch(Token::EQ)?;
    let content = self.tok_lst(
      &Token::COMMA, 
      |t| {t == &Token::NEXTLINE || t == &Token::SEMICOLON}
    )?;
    Ok(RRuleToks{tag, content})
  }

  /// Parses a list of token (casted as string) lists with specified separator 
  /// and terminator. Does NOT munch terminator.
  /// 
  /// ## Syntax
  /// `vec<tok> end | vec<tok> sep tok_lst`
  fn tok_lst<F>(&mut self, sep: &Token, end: F) 
  -> Result<Vec<String>, ICSProcessError> 
  where 
    F: Fn(&Token) -> bool 
  {
    let mut ret = Vec::<String>::new();
    let mut entry = String::new();

    loop {
      let next_tok = self.peek(0)?;
      if end(next_tok) {
        ret.push(entry.clone());
        break Ok(ret);
      } else if next_tok == sep {
        self.skip()?;
        ret.push(entry.clone());
        entry.clear();
      } else {
        let tok = self.token()?;
        entry.push_str(&tok.cast_as_string());
      }
    }
  }

  /// Parses a datetime literal, in the form of `[yyyymmdd]T[hhmmss]Z`.
  fn dt_literal(
    &mut self,
    zone_specified: bool,
  ) -> Result<MinInstant, ICSProcessError> {
    let ymd = self.number()?;

    let dt = if self.peek(0)? == &Token::Other("T".to_string()) {
      self.skip()?;
      let hms = self.number()?;

      // deal with weird ICS format rules: if timezone is not directly 
      // specified, such a literal shall end with 'Z'.
      if !zone_specified {
        self.munch(Token::Other("Z".to_string()))?;
      }
      
      Date::from_ics_time_string(&ymd, &hms)?
    } else {
      // Handle the case where time of day is not specified. 
      let hms = ICS_DEFAULT_TIME_IN_DAY;
      Date::from_ics_time_string(&ymd, hms)?
    };

    match MinInstant::from_date(&dt) {
      Ok(mi) => Ok(mi),
      _ => unreachable!("Well-formatted ICS can never overflow MinInstant"),
    }
  }

}

impl std::fmt::Display for RRuleToks {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}={:?}", self.tag, self.content)
  }
}

impl std::fmt::Display for FreqAndRRules {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "  freq={:?}\n", self.freq)?;
    
    write!(f, "  interval={}\n", self.interval)?;

    if let Some(n) = self.count {
      write!(f, "  count={}\n", n)?;
    }

    if let Some(mi) = self.until {
      write!(f, "  until={}\n", Date::from_min_instant(mi))?;
    }

    write!(f, "  rrules=[\n")?;
    for rrt in &self.content {
      write!(f, "    {}\n", rrt)?;
    }
    write!(f, "  ]")
  }
}

impl std::fmt::Display for Vevent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let repeat_str = match &self.repeat {
      Some(rpt) => rpt.to_string(),
      None => "  No Repeat".to_string(),
    };
    write!(f, "  {}\n  {}\n{}\n", 
      self.summary.trim(), self.mi.as_date_string(), repeat_str)
  }
}

impl std::fmt::Display for ICalendar {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "ICalendar `{}`: \n[", self.name)?;
    for item in &self.content {
      write!(f, "\n{}", item)?;
    }
    write!(f, "\n]")
  }
}
