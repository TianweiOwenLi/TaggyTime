
use std::iter::Peekable;
use std::str::Chars;

use std::vec::IntoIter;

/// Error during lexing stage, which can either be end of file, or some 
/// custom error.
pub enum LexerError {
  EOF,
  Other(String)
}

pub enum Token {
  // structures
  BEGIN,
  COLON,
  SEMICOLON,
  END,
  EQ,
  SLASH,
  UNDERSCORE,
  DASH,

  // times
  DTSTART,
  DTEND,
  TZID,

  // ignored strings
  Other(String),

  // reserved keywords for item types
  VCALENDAR,
  VEVENT,
  LOCATION,

  // repetitions
  RRULE,

  // other info
  SUMMARY,
}

pub struct IcsLexer<'a> {
  stream: Peekable<Chars<'a>>,
}

impl<'a> IcsLexer<'a> {

  // Creates an ics lexer from some string. 
  pub fn new(content: &'a str) -> IcsLexer<'a> {
    IcsLexer{
      stream: content.chars().peekable()
    }
  }

  /// Fetches the current character without advancing the lexer stream.
  pub fn current(&mut self) -> Result<char, LexerError> {
    match self.stream.peek() {
      Some(c) => Ok(*c),
      None => Err(LexerError::EOF),
    }
  }

  /// Fetches the current character while advancing the lexer stream. 
  pub fn next(&mut self) -> Result<char, LexerError> {
    match self.stream.next() {
      Some(c) => Ok(c),
      None => Err(LexerError::EOF),
    }
  }

  /// Skips once.
  pub fn skip(&mut self) {
    self.stream.next();
  }

  /// Skips while some condition is true.
  pub fn skip_while<F>(&mut self, pred: F) -> Result<(), LexerError> 
  where
    F: Fn(char) -> bool
  {
    let mut c = self.current()?;
    while pred(c) {
      self.skip();
      if let Some(&new_c) = self.stream.peek() {
        c = new_c;
      } else {
        break;
      }
    }
    Ok(())
  }

  /// Takes while some condition is true. 
  pub fn take_while<F>(&mut self, pred: F) -> Result<String, LexerError> 
  where
    F: Fn(char) -> bool
  {
    let mut ret = String::new();
    while let Some(&c) = self.stream.peek() {
      if pred(c) {
        ret.push(self.next()?);
      } else {
        break;
      }
    }
    Ok(ret)
  }

  /// Parses some possibly-keyword identifier
  pub fn possible_keyword(&mut self) -> Result<Token, LexerError> {
    let ident_str = self.take_while(|c| c.is_uppercase())?;
    match ident_str.as_str() {
      "BEGIN" => Ok(Token::BEGIN),
      "END" => Ok(Token::END),
      "DTSTART" => Ok(Token::DTSTART),
      "DTEND" => Ok(Token::DTEND),
      "TZID" => Ok(Token::TZID),
      "VCALENDAR" => Ok(Token::VCALENDAR),
      "VEVENT" => Ok(Token::VEVENT),
      "LOCATION" => Ok(Token::LOCATION),
      "RRULE" => Ok(Token::RRULE),
      "SUMMARY" => Ok(Token::SUMMARY),
      s => Ok(Token::Other(s.to_string())),
    }
  }

  pub fn token(&mut self) -> Result<Token, LexerError> {
    let curr_char = self.current()?;
    if curr_char.is_whitespace() {
      self.skip_while(|c| c.is_whitespace())?;
      self.token()
    } else {
      match curr_char {
        ':' => Ok(Token::COLON),
        ';' => Ok(Token::SEMICOLON),
        '=' => Ok(Token::EQ),
        '/' => Ok(Token::SLASH),
        '_' => Ok(Token::UNDERSCORE),
        '-' => Ok(Token::DASH),
        'A'..='Z' => self.possible_keyword(),
        _ => Ok(Token::Other(self.take_while(|c| c != '\n')?)),
      }
    }
  }


}

