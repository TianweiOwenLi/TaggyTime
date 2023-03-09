

use super::{RefinementError, Result};


/// Ranged `i64` type, behaves exactly like `i64`, except that elements out 
/// of range can never be used to instantiate. 
/// 
/// ## Examples
/// ```
/// type weekdays = RangedI64<1, 7>;
/// let m = weekdays::new(1);
/// let n = weekdays::new(8);
/// assert!(m.is_ok());
/// assert!(n.is_err());
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct RangedI64<const MIN: i64, const MAX: i64> (i64);

impl<const MIN: i64, const MAX: i64> RangedI64<MIN, MAX> {

  /// Attempts to construct some ranged i64 using `n`. Returns underflow / 
  /// overflow error if bounds check failed. 
  pub fn new(n: i64) -> Result<Self> {
    if n < MIN {
      Err(RefinementError::RangedI64Underflow(n, MIN, MAX))
    } else if n > MAX {
      Err(RefinementError::RangedI64Overflow(n, MIN, MAX))
    } else {
      Ok(Self(n))
    }
  }

}

impl<const MIN: i64, const MAX: i64> std::fmt::Display for RangedI64<MIN, MAX> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0) // ranged nums shall just look like regular nums..
  }
}
