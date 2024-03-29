//! Refinement type wrappers for Rust primitives.

use std::fmt::Display;

use super::{RefineResult, RefinementError};
use serde::{Deserialize, Serialize};

pub const I64_MAX: i64 = i64::MAX;

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
#[derive(
  PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize,
)]
pub struct RangedI64<const MIN: i64, const MAX: i64>(i64);

impl<const MIN: i64, const MAX: i64> RangedI64<MIN, MAX> {
  /// Attempts to construct some ranged i64 using `n`. Returns underflow /
  /// overflow error if bounds check failed.
  pub fn new<T: Into<i64> + PartialOrd>(num: T) -> RefineResult<Self> {
    let n: i64 = num.into();
    if n < MIN {
      Err(RefinementError::RangedI64Underflow(n, MIN, MAX))
    } else if n > MAX {
      Err(RefinementError::RangedI64Overflow(n, MIN, MAX))
    } else {
      Ok(Self(n))
    }
  }

  /// Attempts to construct some ranged i64 using `n`. Returns underflow /
  /// overflow error if bounds check failed.
  pub fn try_new<T: TryInto<i64> + PartialOrd + Display + Copy>(
    num: T,
  ) -> RefineResult<Self> {
    let n_opt = num.try_into();
    match n_opt {
      Ok(n) => Self::new(n),
      Err(_) => Err(RefinementError::FailedConversionToI64(format!("{}", num))),
    }
  }

  /// Attempts to increment the ranged number; returns an error if fails.
  pub fn increment(&self) -> RefineResult<Self> {
    let new_raw = self.0.checked_add(1);
    if let Some(new_safe_raw) = new_raw {
      Self::new(new_safe_raw)
    } else {
      Err(RefinementError::RangedI64ArithmeticError(self.0, '+', 1))
    }
  }

  /// Attempts to increment the ranged number; panics if fails.
  pub fn increment_unwrap(&self) -> Self {
    self.increment().unwrap()
  }
}

pub type LowerBoundI64<const MIN: i64> = RangedI64<MIN, I64_MAX>;

impl<const MIN: i64, const MAX: i64> std::fmt::Display for RangedI64<MIN, MAX> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0) // ranged nums shall just look like regular nums..
  }
}
