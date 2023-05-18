//! Utility functions.

use std::path::Path;

pub fn path2string<P: AsRef<Path>>(path: P) -> String {
  path.as_ref().display().to_string()
}

pub fn truncate(s: &str, maxlen: usize) -> String {
  if s.len() <= maxlen {
    s.to_string()
  } else {
    format!("{}...", &s[0..maxlen-3])
  }
}
