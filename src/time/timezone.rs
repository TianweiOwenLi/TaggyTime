use std::str::FromStr;

use super::{TimeError, fact::MIN_IN_HR};

pub const UTC_LB: i64 = -720;
pub const UTC_UB: i64 = 840;
const OFFSET_MIN_IN_HR: i64 = 60;

/// A representation of a timezone offset, in terms of difference in minutes
/// as compared to UTC.
#[derive(Debug, Clone, Copy)]
pub struct ZoneOffset(i64);

impl ZoneOffset {
  /// Creates a new ZoneOffset object according to the input offset in minutes.
  /// Only allows an offset between -12:00 and +14:00 inclusive (which is
  /// between -720 and +840 minutes).
  pub fn new(n: i64) -> Result<Self, String> {
    // Valid UTC offsets must be between -12:00 and +14:00, inclusive.
    if n < UTC_LB || n > UTC_UB {
      Err(
        "Timezone offset must be a value between -720min and +840min"
          .to_string(),
      )
    } else {
      Ok(ZoneOffset(n))
    }
  }

  pub fn utc() -> Self {
    ZoneOffset(0)
  }

  /// Returns the raw offset data.
  pub fn raw(&self) -> i64 {
    self.0
  }
}

/// Attempts to parse some expression as u32.
fn parse_u32(expr: &str) -> Result<u32, TimeError> {
  match expr.parse() {
    Ok(n) => Ok(n),
    _ => Err(TimeError::TimeZoneParseErr(expr.to_string()))
  }
}

impl FromStr for ZoneOffset {
  type Err = TimeError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let bad = Err(TimeError::TimeZoneParseErr(s.to_string()));

    let positive = if s.starts_with('+') { 
      true 
    } else if s.starts_with('-') { 
      false 
    } else {
      return bad;
    };

    let (hr_offset, min_offset) =  match &s[1..].split_once(':') {
      Some((hr_str, min_str)) => (parse_u32(hr_str)?, parse_u32(min_str)?),
      None => (parse_u32(&s[1..])?, 0) // no min field, only hours
    };

    let mut total_offset: i64 = i64::from(MIN_IN_HR * hr_offset + min_offset);
    if ! positive { total_offset *= -1; }

    match ZoneOffset::new(total_offset) {
      Ok(z) => Ok(z),
      _ => bad
    }
  }
}

impl std::fmt::Display for ZoneOffset {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let sign_char = if self.0 >= 0 { '+' } else { '-' };
    let abs = self.0.abs();
    let (hr, min) = (abs / OFFSET_MIN_IN_HR, abs % OFFSET_MIN_IN_HR);
    write!(f, "{}{:02}:{:02}", sign_char, hr, min)
  }
}

#[allow(unused_imports)]
mod test {

  use super::*;

  #[test]
  fn construction_constraint() {
    assert!(ZoneOffset::new(-23333).is_err())
  }
}
