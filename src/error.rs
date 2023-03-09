use crate::ics_parser::lexer::Token;

#[allow(dead_code)]
#[derive(Clone)]
pub enum ICSProcessError {
  EOF,
  CannotCastTok(Token),
  Other(String),
}

impl std::fmt::Display for ICSProcessError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ICSProcessError::EOF => write!(f, "End of file error"),
      ICSProcessError::CannotCastTok(t) => {
        write!(f, "Cannot cast token `{}` to str", t)
      }
      ICSProcessError::Other(s) => write!(f, "ICS process error: {}", s),
    }
  }
}

impl<'a> From<&'a ICSProcessError> for ICSProcessError {
  fn from(value: &'a ICSProcessError) -> Self {
    value.clone()
  }
}
