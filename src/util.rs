//! Utility functions.

use std::path::Path;

pub fn path2string<P: AsRef<Path>>(path: P) -> String {
  path.as_ref().display().to_string()
}