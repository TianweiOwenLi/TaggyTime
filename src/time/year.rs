use super::fact::*;

pub enum YearLength {
  Leap,
  Common,
}

pub trait Year {
  /// converts a Year to a CeYear
  fn to_ce(&self) -> CeYear;

  /// converts a Year to a UnixYear
  fn to_unix(&self) -> UnixYear;

  /// get the length of year, either Leap or Common.
  fn get_year_length(&self) -> YearLength {
    let CeYear(n) = self.to_ce();
    if (n % 400 == 0) || (n % 4 == 0 && n % 100 != 0) {
      YearLength::Leap
    } else {
      YearLength::Common
    }
  }

  /// get the number of days in a year, either 366 for a Leap year or
  /// 365 for a Common year.
  fn days_in_year(&self) -> u32 {
    match self.get_year_length() {
      YearLength::Leap => 366,
      YearLength::Common => 365,
    }
  }

  /// get the number of minutes in a year.
  #[inline(always)]
  fn num_min(&self) -> u32 {
    self.days_in_year() * MIN_IN_DAY
  }
}

/// A trait for various types of years, where a next_year function shall exist.
/// Not object-safe.
pub trait NextableYear: Year {
  /// returns the next year.
  fn next_year(&self) -> Self;
}

#[derive(Debug)]
pub struct UnixYear(u16);

#[derive(Debug)]
pub struct CeYear(u16);

impl UnixYear {
  pub fn new(n: u16) -> Self {
    UnixYear(n)
  }
}

impl CeYear {
  pub fn new(n: u16) -> Self {
    assert!(n >= 1970);
    CeYear(n)
  }

  pub fn raw(self) -> u16 {
    self.0
  }
}

impl Year for UnixYear {
  fn to_ce(&self) -> CeYear {
    CeYear(self.0 + 1970)
  }

  fn to_unix(&self) -> UnixYear {
    UnixYear(self.0)
  }
}

impl Year for CeYear {
  fn to_ce(&self) -> CeYear {
    CeYear(self.0)
  }

  fn to_unix(&self) -> UnixYear {
    UnixYear(self.0 - 1970)
  }
}

impl NextableYear for UnixYear {
  fn next_year(&self) -> Self {
    UnixYear(1 + self.0)
  }
}

impl NextableYear for CeYear {
  fn next_year(&self) -> Self {
    CeYear(1 + self.0)
  }
}
