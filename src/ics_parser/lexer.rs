use std::iter::Peekable;
use std::str::Chars;
use crate::error::ICSProcessError;

pub fn char_after_keyword(c: char) -> bool {
  c.is_whitespace() || [';', ':', '='].contains(&c)
}

#[derive(Debug)]
pub enum Token {
  // structures
  BEGIN,
  COLON,
  SEMICOLON,
  END,
  PERIOD,
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

  // format
  NEXTLINE,

  // numeral
  Number(String),
}

pub struct IcsLexer<'a> {
  stream: Peekable<Chars<'a>>,
}

impl<'a> IcsLexer<'a> {
  // Creates an ics lexer from some string.
  pub fn new(content: &'a str) -> IcsLexer<'a> {
    IcsLexer {
      stream: content.chars().peekable(),
    }
  }

  /// Advances the lexer and returns a particular token.
  pub fn single(&mut self, tok: Token) -> Result<Token, ICSProcessError> {
    self.skip();
    Ok(tok)
  }

  /// Fetches the current character without advancing the lexer stream.
  pub fn current(&mut self) -> Result<char, ICSProcessError> {
    match self.stream.peek() {
      Some(c) => Ok(*c),
      None => Err(ICSProcessError::EOF),
    }
  }

  /// Fetches the current character while advancing the lexer stream.
  pub fn next(&mut self) -> Result<char, ICSProcessError> {
    match self.stream.next() {
      Some(c) => Ok(c),
      None => Err(ICSProcessError::EOF),
    }
  }

  /// Skips once.
  pub fn skip(&mut self) {
    self.stream.next();
  }

  /// Skips while some condition is true.
  pub fn skip_while<F>(&mut self, pred: F) -> Result<(), ICSProcessError>
  where
    F: Fn(char) -> bool,
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
  pub fn take_while<F>(&mut self, pred: F) -> Result<String, ICSProcessError>
  where
    F: Fn(char) -> bool,
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
  pub fn possible_keyword(&mut self) -> Result<Token, ICSProcessError> {
    let ident_str = self.take_while(|c| c.is_alphabetic())?;
    println!("{}", ident_str);

    // handles the case where something looks like a keyword appears as
    // part of normal ident
    if let Some(c) = self.stream.peek() {
      if !char_after_keyword(*c) {
        return Ok(Token::Other(ident_str));
      }
    }

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
      _ => Ok(Token::Other(ident_str)),
    }
  }

  /// Parses some sequence of number.
  pub fn number(&mut self) -> Result<Token, ICSProcessError> {
    let num_str = self.take_while(|c| c.is_digit(10))?;
    Ok(Token::Number(num_str))
  }

  pub fn token(&mut self) -> Result<Token, ICSProcessError> {
    let curr_char = self.current()?;
    if curr_char.is_whitespace() {
      self.skip_while(|c| c.is_whitespace())?;
      self.token()
    } else {
      match curr_char {
        ':' => self.single(Token::COLON),
        ';' => self.single(Token::SEMICOLON),
        '=' => self.single(Token::EQ),
        '/' => self.single(Token::SLASH),
        '_' => self.single(Token::UNDERSCORE),
        '-' => self.single(Token::DASH),
        '\n' => self.single(Token::NEXTLINE),
        '.' => self.single(Token::PERIOD),
        'A'..='Z' | 'a'..='z' => self.possible_keyword(),
        '0'..='9' => self.number(),
        c => self.single(Token::Other(c.to_string())),
      }
    }
  }
}

impl std::fmt::Display for Token {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Token::Other(s) => write!(f, "Other({})", s),
      tok => write!(f, "{:?}", tok),
    }
  }
}
