
#[derive(PartialEq, Eq, Debug)]

/// A wrapper around u8, which represents a percentage (in integer), ranging 
/// from 0 percent to 100 percent, inclusive.
/// 
/// # Example
/// ```
/// let p: Percent = Percent::from_u8(56);
/// let p = p.one_minus();
/// 
/// let q = Percent::from_str("44");
/// 
/// assert_eq!(p, q);
/// ```
pub struct Percent(u8);

impl Percent{

  /// Takes in a &str, trims it, and attempts to parse it 
  /// as a percentage, ie. an integer from 0 to 100 inclusive.
  /// Returns `Err()` if parsing fails.
  pub fn from_str(s: &str) -> Result<Self, String> {
    let ps: Result<u8, _> = s.trim().parse::<u8>();
    if let Ok(n) = s.parse() {
      Self::from_u8(n)
    } else {
      Err("Unable to parse percentage as u8".to_string())
    }
  }


  /// Constructs an instance of `Percent` from some `u8` argument. 
  /// If such argument is not between 0 an 100 (inclusive), returns 
  /// an `Err()`.
  pub fn from_u8(n: u8) -> Result<Self, String> {
    if n <= 100 {
      Ok(Percent(n))
    } else {
      Err("Percentage cannot exceed 100".to_string())
    }
  }


  /// Returns a `Percent` instance that represents 100% minus oneself.
  pub fn one_minus(&self) -> Self {
    Percent(100 - self.0)
  }

  /// Gets the raw `u8` value of self, which ranges between 0 an 100 inclusive.
  pub fn raw(&self) -> u8 { self.0 }

}


impl std::fmt::Display for Percent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{}%", self.0)
  }
}


#[allow(unused_imports)]
mod test {
  use super::*;

  #[test]
  fn test_leading_zero_in_str() {
    assert_eq!(Ok(Percent(3)), Percent::from_str("03"));
  }
}
