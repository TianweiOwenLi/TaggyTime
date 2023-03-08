

#[allow(dead_code)]
#[derive(Clone)]
pub enum ICSProcessError {
  EOF,
  Other(String),
}

impl std::fmt::Display for ICSProcessError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ICSProcessError::EOF => write!(f, "End of file error"),
      ICSProcessError::Other(s) => write!(f, "ICS process error: {}", s),
    }
  }
}
