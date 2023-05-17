//! a module encapsulating the `Percent` type.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::time::{parse_f32, TimeError};

#[derive(Debug)]
pub enum PercentError {
  ComplementOutOfBound(u16),
  ArithmeticOutOfBound(u16, String, u16),
  PercentF32Overflow(f32),
}

/// A wrapper around u8, which represents a percentage (in integer), ranging
/// from 0 percent to 100 percent, inclusive.
///
/// # Example
/// ```
/// let p: Percent = Percent(56);
/// let q = Percent(44);
///
/// assert_eq!(p.complement().unwrap(), q);
/// ```
#[derive(
  PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Serialize, Deserialize,
)]
pub struct Percent(pub u16);

impl Percent {
  /// Returns a `Percent` instance that represents 100% minus oneself. If
  /// `Self` is an `Overflow` variant, returns `ComplementOutOfBound` error.
  pub fn complement(&self) -> Result<Self, PercentError> {
    match self.0 {
      0..=100 => Ok(Percent(100 - self.0)),
      _ => Err(PercentError::ComplementOutOfBound(self.0)),
    }
  }

  /// Gets the raw `u16` value of self.
  pub fn raw(&self) -> u16 {
    self.0
  }

  /// Checks whether this percent value is beyond `100%`.
  pub fn is_overflow(&self) -> bool {
    self.0 > 100
  }
}

impl TryFrom<f32> for Percent {
  type Error = PercentError;

  /// Converts some float-point to `Percent`, where 1.00 stands for 100%.
  fn try_from(value: f32) -> Result<Self, Self::Error> {
    let rounded = (100.0 * value).round();
    if rounded < 0.0 || rounded > (u16::MAX as f32) {
      return Err(PercentError::PercentF32Overflow(value));
    }
    Ok(Percent(rounded as u16))
  }
}

impl FromStr for Percent {
  type Err = TimeError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let n = s.len();
    Ok(if s.ends_with("%") {
      Percent::try_from(0.01 * parse_f32(&s[..(n - 1)])?)?
    } else {
      Percent::try_from(parse_f32(s)?)?
    })
  }
}

impl std::ops::Add for Percent {
  type Output = Result<Percent, PercentError>;
  fn add(self, rhs: Self) -> Self::Output {
    let add_raw = self.0.checked_add(rhs.0);
    match add_raw {
      Some(n) => Ok(Percent(n)),
      None => Err(PercentError::ArithmeticOutOfBound(
        self.0,
        "+".to_string(),
        rhs.0,
      )),
    }
  }
}

impl std::ops::Sub for Percent {
  type Output = Result<Percent, PercentError>;

  fn sub(self, rhs: Self) -> Self::Output {
    if self.0 >= rhs.0 {
      Ok(Percent(self.0 - rhs.0))
    } else {
      Err(PercentError::ArithmeticOutOfBound(
        self.0,
        "-".to_string(),
        rhs.0,
      ))
    }
  }
}

impl std::fmt::Display for Percent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}%", self.raw())
  }
}

impl std::fmt::Display for PercentError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::ComplementOutOfBound(n) => write!(
        f,
        "Cannot take the 
          complement of `{}`, which is an overflow variant of `Percent`.",
        n
      ),
      Self::ArithmeticOutOfBound(lhs, msg, rhs) => write!(
        f,
        "Percent arithmetic out of bound: {} {} {}.",
        lhs, msg, rhs
      ),
      Self::PercentF32Overflow(x) => write!(f, "Percent float overflow: {}", x),
    }
  }
}

#[allow(unused_imports)]
mod test {
  use super::*;

  #[test]
  fn instantiate_variant() {
    assert_eq!(Percent(3), Percent(3));
    assert_eq!(Percent(15251), Percent(15251));
    assert!(Percent(15251).is_overflow());
    assert!(!Percent(3).is_overflow());
  }

  #[test]
  fn cmp() {
    assert!(Percent(3) < Percent(5));
    assert!(Percent(101) >= Percent(6));
    assert_eq!(Percent(4), Percent(4));
    assert_eq!(Percent(233), Percent(233));
  }

  #[test]
  fn errors() {
    assert!(Percent(100).complement().is_ok());
    assert!(Percent(101).complement().is_err());
  }

  #[test]
  fn raw() {
    assert_eq!(Percent(15411).raw(), 15411 as u16);
  }

  #[test]
  fn sub() {
    let p = Percent(666) - Percent(233);
    assert_eq!(p.unwrap(), Percent(433));
    assert!((Percent(0) - Percent(1)).is_err());
  }

  #[test]
  fn cast_f32() {
    assert_eq!(Percent(23333), Percent::try_from(233.33333).unwrap())
  }
}
