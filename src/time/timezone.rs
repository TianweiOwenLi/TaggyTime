use std::str::FromStr;

use super::{fact::MIN_IN_HR, time_parser::parse_hr_min, TimeError};

use serde::{Deserialize, Serialize};

pub const UTC_LB: i64 = -720;
pub const UTC_UB: i64 = 840;
const OFFSET_MIN_IN_HR: i64 = 60;

/// A representation of a timezone offset, in terms of difference in minutes
/// as compared to UTC.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ZoneOffset(i64);

impl ZoneOffset {
  /// Creates a new ZoneOffset object according to the input offset in minutes.
  /// Only allows an offset between -12:00 and +14:00 inclusive (which is
  /// between -720 and +840 minutes).
  pub fn new(n: i64) -> Result<Self, TimeError> {
    // Valid UTC offsets must be between -12:00 and +14:00, inclusive.
    if n < UTC_LB {
      Err(TimeError::ZoneOffsetConstructionUnderflow(n))
    } else if n > UTC_UB {
      Err(TimeError::ZoneOffsetConstructionOverflow(n))
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

impl FromStr for ZoneOffset {
  type Err = TimeError;
  fn from_str(s: &str) -> Result<Self, TimeError> {
    let bad = Err(TimeError::TimeZoneParseErr(s.to_string()));

    let positive = if s.starts_with('+') {
      true
    } else if s.starts_with('-') {
      false
    } else {
      return bad;
    };

    let (hr_offset, min_offset) = parse_hr_min(&s[1..])?;

    let mut total_offset: i64 = i64::from(MIN_IN_HR * hr_offset + min_offset);
    if !positive {
      total_offset *= -1;
    }

    ZoneOffset::new(total_offset)
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
