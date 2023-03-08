pub const UTC_LB: i64 = -720;
pub const UTC_UB: i64 = 840;

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
      Err("Timezone offset must be a value between -720min and +840min".to_string())
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

#[allow(unused_imports)]
mod test {

  use super::*;

  #[test]
  fn construction_constraint() {
    assert!(ZoneOffset::new(-23333).is_err())
  }
}
