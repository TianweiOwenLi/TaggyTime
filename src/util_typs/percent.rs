//! a module encapsulating the `Percent` type.

use super::PercentError;

#[derive(Debug)]
pub enum Error {
  ComplementOutOfBound(u16),
  ArithmeticOutOfBound(u16, String, u16),
}

/// A wrapper around u8, which represents a percentage (in integer), ranging
/// from 0 percent to 100 percent, inclusive.
///
/// # Example
/// ```
/// let p: Percent = Percent::from_u8(56);
/// let q = Percent::from_u8(44);
///
/// assert_eq!(p.one_minus(), q);
/// ```
#[derive(PartialEq, Eq, PartialOrd, Debug)]
pub struct Percent(u16);

impl Percent {
  /// Constructs an instance of `Percent` from some `u16` argument.
  pub fn new(n: u16) -> Self {
    Percent(n)
  }

  /// Returns a `Percent` instance that represents 100% minus oneself. If
  /// `Self` is an `Overflow` variant, returns `ComplementOutOfBound` error.
  pub fn complement(&self) -> Result<Self, Error> {
    match self.0 {
      0..=100 => Ok(Percent(100 - self.0)),
      _ => Err(Error::ComplementOutOfBound(self.0)),
    }
  }

  /// Gets the raw `u16` value of self.
  pub fn raw(&self) -> u16 {
    self.0
  }

  /// A precentage representing `0%`
  pub fn zero() -> Self {
    Percent(0)
  }

  /// A precentage representing `100%`
  pub fn one() -> Self {
    Percent(100)
  }

  /// Checks whether this percent value is beyond `100%`.
  pub fn is_overflow(&self) -> bool {
    self.0 > 100
  }
}

impl TryFrom<f32> for Percent {
  type Error = PercentError;
  fn try_from(value: f32) -> Result<Self, Self::Error> {
    let rounded = value.round();
    if rounded < 0.0 || rounded > (u16::MAX as f32) {
      return Err(PercentError::PercentF32Overflow(value));
    }
    Ok(Percent(rounded as u16))
  }
}

impl std::ops::Sub for Percent {
  type Output = Result<Percent, Error>;

  fn sub(self, rhs: Self) -> Self::Output {
    if self.0 >= rhs.0 {
      Ok(Percent(self.0 - rhs.0))
    } else {
      Err(Error::ArithmeticOutOfBound(self.0, "-".to_string(), rhs.0))
    }
  }
}

impl std::fmt::Display for Percent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}%", self.raw())
  }
}

impl std::fmt::Display for Error {
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
    }
  }
}

#[allow(unused_imports)]
mod test {
  use super::*;

  #[test]
  fn instantiate_variant() {
    assert_eq!(Percent::new(3), Percent(3));
    assert_eq!(Percent::new(15251), Percent(15251));
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
    assert!(Percent::new(100).complement().is_ok());
    assert!(Percent::new(101).complement().is_err());
  }

  #[test]
  fn raw() {
    assert_eq!(Percent::new(15411).raw(), 15411 as u16);
  }

  #[test]
  fn sub() {
    let p = Percent(666) - Percent(233);
    assert_eq!(p.unwrap(), Percent(433));
    assert!((Percent(0) - Percent(1)).is_err());
  }

  #[test]
  fn cast_f32() {
    assert_eq!(Percent(233), Percent::try_from(233.33333).unwrap())
  }
}
