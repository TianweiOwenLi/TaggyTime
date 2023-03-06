pub enum Token {
  // structures
  BEGIN,
  SEMICOLON,
  END,

  // times
  DTSTART,
  DTEND,

  // ignored keywords
  Kw(String),
}
